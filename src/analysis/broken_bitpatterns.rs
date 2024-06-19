use rustc_hir::{def_id::DefId, BodyId, Unsafety};
use rustc_middle::mir::{Operand, StatementKind, Rvalue, CastKind, Place, HasLocalDecls, AggregateKind};
use rustc_middle::mir::RETURN_PLACE;
use rustc_middle::ty::{self, Ty, Instance, ParamEnv, TyKind};
use rustc_span::{Span, DUMMY_SP};

use snafu::{Backtrace, Snafu};
use termcolor::Color;

use crate::graph::GraphTaint;
use crate::prelude::*;
use crate::{
    analysis::{AnalysisKind, IntoReportLevel, ValueChecker, Comparison},
    graph::TaintAnalyzer,
    ir,
    paths::{self, *},
    report::{Report, ReportLevel},
    utils,
    visitor::ContainsUnsafe,
    context::RuMorphCtxt,
    progress_info,
};

#[derive(Debug, Snafu)]
pub enum BrokenBitPatternsError {
    PushPopBlock { backtrace: Backtrace },
    ResolveError { backtrace: Backtrace },
    InvalidSpan { backtrace: Backtrace },
}

impl AnalysisError for BrokenBitPatternsError {
    fn kind(&self) -> AnalysisErrorKind {
        use BrokenBitPatternsError::*;
        match self {
            PushPopBlock { .. } => AnalysisErrorKind::Unreachable,
            ResolveError { .. } => AnalysisErrorKind::OutOfScope,
            InvalidSpan { .. } => AnalysisErrorKind::Unreachable,
        }
    }
}



pub struct BrokenBitPatternsChecker<'tcx> {
    rcx: RuMorphCtxt<'tcx>,
}

impl<'tcx> BrokenBitPatternsChecker<'tcx> {
    pub fn new(rcx: RuMorphCtxt<'tcx>) -> Self {
        BrokenBitPatternsChecker { rcx }
    }

    pub fn analyze(self) {
        let tcx = self.rcx.tcx();
        let hir_map = tcx.hir();

        // Iterates all (type, related function) pairs
        for (_ty_hir_id, (body_id, related_item_span)) in self.rcx.types_with_related_items() {
            
            let f_def_id = hir_map.body_owner_def_id(body_id).to_def_id();
            // print the funciton name of current body
            progress_info!("BrokenBitPatternsChecker::analyze({})", 
                        tcx.def_path_str(f_def_id)
            );

            let visible = utils::check_visibility(tcx, f_def_id);
            
            if let Some(status) = inner::BrokenBitPatternsBodyAnalyzer::analyze_body(self.rcx, body_id)
            {
                let behavior_flag = status.behavior_flag();
                if !behavior_flag.is_empty()
                    && behavior_flag.report_level(visible) >= self.rcx.report_level()
                {
                    progress_info!("find the bug with behavior_flag: {:?}", behavior_flag);
                    let mut color_span = unwrap_or!(
                        utils::ColorSpan::new(tcx, related_item_span).context(InvalidSpan) => continue
                    );

                    // for &span in status.strong_bypass_spans() {
                    //     color_span.add_sub_span(Color::Red, span);
                    // }

                    // for &span in status.weak_bypass_spans() {
                    //     color_span.add_sub_span(Color::Yellow, span);
                    // }

                    // for &span in status.unresolvable_generic_function_spans() {
                    //     color_span.add_sub_span(Color::Cyan, span);
                    // }

                    // for &span in status.plain_deref_spans() {
                    //     color_span.add_sub_span(Color::Blue, span);
                    // }
                    
                    for &span in status.ty_conv_spans() {
                        color_span.add_sub_span(Color::Green, span);
                    }

                    rumorph_report(Report::with_color_span(
                        tcx,
                        behavior_flag.report_level(visible),
                        AnalysisKind::BrokenBitPatterns(behavior_flag),
                        format!(
                            "Potential broken bit patterns issue in `{}`",
                            tcx.def_path_str(hir_map.body_owner_def_id(body_id).to_def_id())
                        ),
                        &color_span,
                    ))
                } else {
                    progress_info!("bug not found");
                }
            }
        }
    }
}

mod inner {
    use super::*;

    #[derive(Debug, Default)]
    pub struct BrokenBitPatternsStatus {
        strong_bypasses: Vec<Span>,
        weak_bypasses: Vec<Span>,
        plain_deref: Vec<Span>,
        unresolvable_generic_functions: Vec<Span>,
        ty_convs: Vec<Span>,
        creation: Vec<Span>,
        behavior_flag: BehaviorFlag,
    }

    impl BrokenBitPatternsStatus {
        pub fn behavior_flag(&self) -> BehaviorFlag {
            self.behavior_flag
        }

        pub fn strong_bypass_spans(&self) -> &Vec<Span> {
            &self.strong_bypasses
        }

        pub fn weak_bypass_spans(&self) -> &Vec<Span> {
            &self.weak_bypasses
        }

        pub fn plain_deref_spans(&self) -> &Vec<Span> {
            &self.plain_deref
        }

        pub fn unresolvable_generic_function_spans(&self) -> &Vec<Span> {
            &self.unresolvable_generic_functions
        }

        pub fn ty_conv_spans(&self) -> &Vec<Span> {
            &self.ty_convs
        }
    }

    pub struct BrokenBitPatternsBodyAnalyzer<'a, 'tcx> {
        rcx: RuMorphCtxt<'tcx>,
        body: &'a ir::Body<'tcx>,
        param_env: ParamEnv<'tcx>,
        status: BrokenBitPatternsStatus,
    }

    impl<'a, 'tcx> BrokenBitPatternsBodyAnalyzer<'a, 'tcx> {
        fn new(rcx: RuMorphCtxt<'tcx>, param_env: ParamEnv<'tcx>, body: &'a ir::Body<'tcx>) -> Self {
            BrokenBitPatternsBodyAnalyzer {
                rcx,
                body,
                param_env,
                status: Default::default(),
            }
        }

        pub fn analyze_body(rcx: RuMorphCtxt<'tcx>, body_id: BodyId) -> Option<BrokenBitPatternsStatus> {
            let hir_map = rcx.tcx().hir();
            let body_did = hir_map.body_owner_def_id(body_id).to_def_id();

            if rcx.tcx().ext().match_def_path(
                body_did,
                &["rumorph_paths_discovery", "PathsDiscovery", "discover"],
            ) {
                progress_info!("special case required");
                // Special case for paths discovery
                trace_calls_in_body(rcx, body_did);
                None
            } else if ContainsUnsafe::contains_unsafe(rcx.tcx(), body_id) {
                let fn_sig = rcx.tcx().fn_sig(body_did).skip_binder();
                if let Unsafety::Unsafe = fn_sig.unsafety() {
                    progress_info!("The function is unsafe");
                    None
                } else {
                    progress_info!("This function contains unsafe block");
                    match rcx.translate_body(body_did).as_ref() {
                        Err(e) => {
                            // MIR is not available for def - log it and continue
                            e.log();
                            None
                        }
                        Ok(body) => {
                            let param_env = rcx.tcx().param_env(body_did);
                            let body_analyzer = BrokenBitPatternsBodyAnalyzer::new(rcx, param_env, body);

                            let mut tconv_source: Vec<usize> = Vec::new();

                            Some(body_analyzer.analyze(tconv_source))
                        }
                    }
                }
            } else {
                Some(Default::default())
            }
        }

        fn analyze(mut self, mut tconv_source: Vec<usize>) -> BrokenBitPatternsStatus {
            let loc_decls = self.body.local_decls();
            let mut taint_analyzer = TaintAnalyzer::new(self.body);

            for statement in self.body.statements() {
                // statement here is mir::Statement without translation
                // while iterating statements, we plan to mark ty conv as source / plain deref as sink
                // progress_info!("statement: {:?}", statement);
                match statement.kind {
                    StatementKind::Assign(box (lplace, rval)) => {
                        // rhs
                        match rval {
                            Rvalue::Cast(cast_kind, op, to_ty) => {
                                match cast_kind {
                                    CastKind::PtrToPtr => {
                                        progress_info!("cast::ptr-ptr");
                                        let f_ty = get_ty_from_op(self.body, self.rcx, &op);
                                        match f_ty {
                                            Ok(from_ty) => {
                                                let vc = ValueChecker::new(self.rcx, self.param_env, from_ty, to_ty);
                                                let value_status = vc.get_val_status();

                                                let pl = get_place_from_op(&op);
                                                match pl {
                                                    Ok(place) => {
                                                        let id = place.local.index();

                                                        // if A could be generic type or composite type, and B is primitive type, taint as source
                                                        match value_status {
                                                            Comparison::Less => {
                                                                let mut pre_conv: bool = false;
                                                                if !tconv_source.is_empty() {
                                                                    for conv_id in tconv_source.iter() {
                                                                        if taint_analyzer.is_reachable(*conv_id, id) {
                                                                            pre_conv = true;
                                                                            break;
                                                                        }
                                                                    }
                                                                }
                                                                if !pre_conv {
                                                                    taint_analyzer.mark_at_once(id, &BehaviorFlag::CAST);
                                                                    self.status
                                                                        .ty_convs
                                                                        .push(statement.source_info.span);

                                                                    self.status
                                                                        .creation
                                                                        .push(statement.source_info.span);
                                                                    // progress_info!("cast leads to ub in this statement");
                                                                }
                                                            },
                                                            _ => {
                                                                tconv_source.push(id);
                                                            },
                                                        }
                                                    },
                                                    Err(_e) => {
                                                        // progress_info!("Can't get place from the cast operand");
                                                    },
                                                }
                                            },
                                            Err(_e) => {
                                                // progress_info!("Can't get ty from the cast place");
                                            },
                                        }
                                    },
                                    CastKind::Transmute => {
                                        let f_ty = get_ty_from_op(self.body, self.rcx, &op);
                                        match f_ty {
                                            Ok(from_ty) => {
                                                if !is_ptr_ty(from_ty, to_ty) {
                                                    continue;
                                                }
                                                progress_info!("transmute::ptr-ptr");
                                                let vc = ValueChecker::new(self.rcx, self.param_env, from_ty, to_ty);
                                                let value_status = vc.get_val_status();

                                                let pl = get_place_from_op(&op);
                                                match pl {
                                                    Ok(place) => {
                                                        let id = place.local.index();

                                                        match value_status {
                                                            // make sure it is not kind of bug 1
                                                            Comparison::Less => {
                                                                let mut pre_conv: bool = false;
                                                                if !tconv_source.is_empty() {
                                                                    for conv_id in tconv_source.iter() {
                                                                        if taint_analyzer.is_reachable(*conv_id, id) {
                                                                            pre_conv = true;
                                                                            break;
                                                                        }
                                                                    }
                                                                }
                                                                if !pre_conv {
                                                                    taint_analyzer.mark_at_once(id, &BehaviorFlag::TRANSMUTE);
                                                                    self.status
                                                                        .ty_convs
                                                                        .push(statement.source_info.span);

                                                                    self.status
                                                                        .creation
                                                                        .push(statement.source_info.span);
                                                                    // progress_info!("cast leads to ub in this statement");
                                                                }
                                                            },
                                                            _ => {
                                                                tconv_source.push(id);
                                                            },
                                                        }
                                                    },
                                                    Err(_e) => {
                                                        // progress_info!("Can't get place from the transmute operand");
                                                    },
                                                }
                                            },
                                            Err(_e) => {
                                                // progress_info!("Can't get ty from the transmute place");
                                            },
                                        }
                                    },
                                    _ => (),
                                }
                            },
                            _ => {},
                        }
                    },
                    _ => {},
                }
            }

            for (_id, terminator) in self.body.terminators().enumerate() {
                // progress_info!("terminator: {:?}", terminator);
                match terminator.kind {
                    ir::TerminatorKind::StaticCall {
                        callee_did,
                        callee_substs,
                        ref args,
                        dest,
                        ..
                    } => {
                        let tcx = self.rcx.tcx();
                        // TyCtxtExtension
                        let ext = tcx.ext();
                        // Check for lifetime bypass
                        let symbol_vec = ext.get_def_path(callee_did);
                        progress_info!("terminator with symbol: {:?}", symbol_vec);
                        let sym = symbol_vec[ symbol_vec.len() - 1 ].as_str();
                        if sym.contains("alloc") {
                            let id = dest.local.index();
                            taint_analyzer
                                .clear_source(id);
                        } else if paths::STR_UNCHECKED_LIST.contains(&symbol_vec) {
                            let id = dest.local.index();
                            // for conv_id in tconv_source.iter() {
                            // if taint_analyzer.is_reachable(*conv_id, id) {
                            taint_analyzer.mark_at_once(id, &BehaviorFlag::TRANSMUTE);
                            self.status.ty_convs.push(terminator.original.source_info.span);

                            self.status.creation.push(terminator.original.source_info.span);
                            progress_info!("leads to ub in this terminator");
                            break;
                            // }
                            // }
                        } else if paths::STRONG_LIFETIME_BYPASS_LIST.contains(&symbol_vec) {
                            // taint_analyzer
                            //     .mark_source(id, STRONG_BYPASS_MAP.get(&symbol_vec).unwrap());
                            // self.status
                            //     .strong_bypasses
                            //     .push(terminator.original.source_info.span);
                        } else if paths::WEAK_LIFETIME_BYPASS_LIST.contains(&symbol_vec) {
                            // if self.fn_called_on_copy(
                            //     (callee_did, args),
                            //     &[&PTR_WRITE[..], &PTR_DIRECT_WRITE[..]],
                            // ) {
                            //     // writing Copy types is not a lifetime bypass.
                            //     continue;
                            // }

                            // taint_analyzer
                            //     .mark_source(id, WEAK_BYPASS_MAP.get(&symbol_vec).unwrap());
                            // self.status
                            //     .weak_bypasses
                            //     .push(terminator.original.source_info.span);
                        }
                    },
                    ir::TerminatorKind::Return => {
                        // _0 is always considered as return value
                        let return_pl0 = self.body.local_decls().get(RETURN_PLACE);
                        if return_pl0.is_some() {
                            match return_pl0.unwrap().ty.kind() {
                                TyKind::Ref(..) => {
                                    // taint_analyzer.mark_sink(0);
                                    // self.status
                                    //     .plain_deref
                                    //     .push(terminator.original.source_info.span);
                                },
                                _ => {},
                            }
                        }
                    },
                    _ => {},
                }
            }

            self.status.behavior_flag = taint_analyzer.propagate();
            self.status
        }
    }

    fn trace_calls_in_body<'tcx>(rcx: RuMorphCtxt<'tcx>, body_def_id: DefId) {
        warn!("Paths discovery function has been detected");
        if let Ok(body) = rcx.translate_body(body_def_id).as_ref() {
            for terminator in body.terminators() {
                match terminator.kind {
                    ir::TerminatorKind::StaticCall { callee_did, .. } => {
                        let ext = rcx.tcx().ext();
                        println!(
                            "{}",
                            ext.get_def_path(callee_did)
                                .iter()
                                .fold(String::new(), |a, b| a + " :: " + &*b.as_str())
                        );
                    }
                    _ => (),
                }
            }
        }
    }
}

// check whether both from_ty and to_ty are pointer types
fn is_ptr_ty<'tcx>(from_ty: Ty<'tcx>, to_ty: Ty<'tcx>) -> bool {
    // (from_ty|to_ty) needs to be raw pointer or reference
    let is_fty_ptr = if let ty::RawPtr(_) = from_ty.kind() {
        true
    } else if let ty::Ref(..) = from_ty.kind() {
        true
    } else {
        false
    };
    let is_tty_ptr = if let ty::RawPtr(_) = to_ty.kind() {
        true
    } else if let ty::Ref(..) = to_ty.kind() {
        true
    } else {
        false
    };
    (is_fty_ptr & is_tty_ptr)
}

fn get_place_from_op<'tcx>(op: &Operand<'tcx>) -> Result<Place<'tcx>, &'static str> {
    match op {
        Operand::Copy(place) | Operand::Move(place) => {
            Ok(*place)
        },
        _ => { Err("Can't get place from operand") },
    }
}

fn get_ty_from_op<'tcx>(bd: &ir::Body<'tcx>, rcx: RuMorphCtxt<'tcx>, op: &Operand<'tcx>) -> Result<Ty<'tcx>, &'static str> {
    match op {
        Operand::Copy(place) | Operand::Move(place) => {
            Ok(place.ty(bd, rcx.tcx()).ty)
        },
        Operand::Constant(box cnst) => {
            Ok(cnst.ty())
        },
        _ => { Err("Can't get ty from place") },
    }
}

// Type Conversion Kind.
// Used to associate each broken bit patterns bug report with its cause.
bitflags! {
    #[derive(Default)]
    pub struct BehaviorFlag: u16 {
        const CAST = 0b00000001;
        const TRANSMUTE = 0b00000010;
    }
}

impl IntoReportLevel for BehaviorFlag {
    fn report_level(&self, visibility: bool) -> ReportLevel {
        use BehaviorFlag as Flag;

        let high = Flag::CAST | Flag::TRANSMUTE;
        //let med = Flag::READ_FLOW | Flag::COPY_FLOW | Flag::WRITE_FLOW;

        // if !(*self & high).is_empty() {
        //     ReportLevel::Error
        // } else if !(*self & med).is_empty() {
        //     ReportLevel::Warning
        // } else {
        //     ReportLevel::Info
        // }

        if visibility == false {
            // if the function is private
            ReportLevel::Warning
        } else {
            ReportLevel::Error
        }
    }
}

impl GraphTaint for BehaviorFlag {
    fn is_empty(&self) -> bool {
        self.is_all()
    }

    fn contains(&self, taint: &Self) -> bool {
        self.contains(*taint)
    }

    fn join(&mut self, taint: &Self) {
        *self |= *taint;
    }
}
