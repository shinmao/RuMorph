use rustc_hir::{def_id::DefId, BodyId, Unsafety};
use rustc_middle::mir::{Operand, StatementKind, Rvalue, CastKind, Place, HasLocalDecls, AggregateKind, BinOp};
use rustc_middle::mir::RETURN_PLACE;
use rustc_middle::ty::{self, Ty, Instance, ParamEnv, TyKind};
use rustc_span::{Span, DUMMY_SP};

use snafu::{Backtrace, Snafu};
use termcolor::Color;

use crate::graph::GraphTaint;
use crate::prelude::*;
use crate::{
    analysis::{AnalysisKind, IntoReportLevel, LayoutChecker, Comparison},
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
pub enum OverflowError {
    PushPopBlock { backtrace: Backtrace },
    ResolveError { backtrace: Backtrace },
    InvalidSpan { backtrace: Backtrace },
}

impl AnalysisError for OverflowError {
    fn kind(&self) -> AnalysisErrorKind {
        use OverflowError::*;
        match self {
            PushPopBlock { .. } => AnalysisErrorKind::Unreachable,
            ResolveError { .. } => AnalysisErrorKind::OutOfScope,
            InvalidSpan { .. } => AnalysisErrorKind::Unreachable,
        }
    }
}

pub struct OverflowChecker<'tcx> {
    rcx: RuMorphCtxt<'tcx>,
}

impl<'tcx> OverflowChecker<'tcx> {
    pub fn new(rcx: RuMorphCtxt<'tcx>) -> Self {
        OverflowChecker { rcx }
    }

    pub fn analyze(self) {
        let tcx = self.rcx.tcx();
        let hir_map = tcx.hir();

        // Iterates all (type, related function) pairs
        for (_ty_hir_id, (body_id, related_item_span)) in self.rcx.types_with_related_items() {
            
            // print the funciton name of current body
            progress_info!("OverflowChecker::analyze({})", 
                        tcx.def_path_str(hir_map.body_owner_def_id(body_id).to_def_id())
            );


            if let Some(status) = inner::OverflowBodyAnalyzer::analyze_body(self.rcx, body_id)
            {
                let behavior_flag = status.behavior_flag();
                if !behavior_flag.is_empty() {
                    progress_info!("find the bug with behavior_flag: {:?}", behavior_flag);
                    let mut color_span = unwrap_or!(
                        utils::ColorSpan::new(tcx, related_item_span).context(InvalidSpan) => continue
                    );

                    for &span in status.strong_bypass_spans() {
                        color_span.add_sub_span(Color::Red, span);
                    }

                    for &span in status.weak_bypass_spans() {
                        color_span.add_sub_span(Color::Yellow, span);
                    }

                    for &span in status.unresolvable_generic_function_spans() {
                        color_span.add_sub_span(Color::Cyan, span);
                    }

                    for &span in status.plain_deref_spans() {
                        color_span.add_sub_span(Color::Blue, span);
                    }
                    
                    for &span in status.ty_conv_spans() {
                        color_span.add_sub_span(Color::Green, span);
                    }

                    rumorph_report(Report::with_color_span(
                        tcx,
                        behavior_flag.report_level(true),
                        AnalysisKind::Overflow(behavior_flag),
                        format!(
                            "Potential overflow issue in `{}`",
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
    pub struct OverflowStatus {
        strong_bypasses: Vec<Span>,
        weak_bypasses: Vec<Span>,
        plain_deref: Vec<Span>,
        unresolvable_generic_functions: Vec<Span>,
        ty_convs: Vec<Span>,
        behavior_flag: BehaviorFlag,
    }

    impl OverflowStatus {
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

        // used as overflow operation here
        pub fn ty_conv_spans(&self) -> &Vec<Span> {
            &self.ty_convs
        }
    }

    pub struct OverflowBodyAnalyzer<'a, 'tcx> {
        rcx: RuMorphCtxt<'tcx>,
        body: &'a ir::Body<'tcx>,
        param_env: ParamEnv<'tcx>,
        status: OverflowStatus,
    }

    impl<'a, 'tcx> OverflowBodyAnalyzer<'a, 'tcx> {
        fn new(rcx: RuMorphCtxt<'tcx>, param_env: ParamEnv<'tcx>, body: &'a ir::Body<'tcx>) -> Self {
            OverflowBodyAnalyzer {
                rcx,
                body,
                param_env,
                status: Default::default(),
            }
        }

        pub fn analyze_body(rcx: RuMorphCtxt<'tcx>, body_id: BodyId) -> Option<OverflowStatus> {
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
            } else {
                match rcx.translate_body(body_did).as_ref() {
                    Err(e) => {
                        // MIR is not available for def - log it and continue
                        e.log();
                        None
                    }
                    Ok(body) => {
                        let param_env = rcx.tcx().param_env(body_did);
                        let body_analyzer = OverflowBodyAnalyzer::new(rcx, param_env, body);
                        Some(body_analyzer.analyze())
                    }
                }
            }
        }

        fn analyze(mut self) -> OverflowStatus {
            let mut taint_analyzer = TaintAnalyzer::new(self.body);
            // use `tainted_source` to maintain tainted external function args
            let mut tainted_source = Vec::new();

            // mark all the arguments as taint source
            for arg_idx in 1usize..self.body.original.arg_count + 1 {
                // progress_info!("external as source: {:?}", arg_idx);
                tainted_source.push(arg_idx);
                taint_analyzer.mark_source(arg_idx, &BehaviorFlag::EXTERNAL);
            }

            for statement in self.body.statements() {
                // statement here is mir::Statement without translation
                // while iterating statements, we plan to mark ty conv as source / plain deref as sink
                // progress_info!("{:?}", statement);
                match statement.kind {
                    StatementKind::Assign(box (lplace, rval)) => {
                        match rval {
                            Rvalue::BinaryOp(op, box (op1, op2))
                            | Rvalue::CheckedBinaryOp(op, box (op1, op2)) => {
                                match op {
                                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                                        if ( op1.ty(self.body, self.rcx.tcx()).is_numeric() ) {
                                            // check whether op1 belongs to func args
                                            let id1 = op1.place();
                                            if let Some(idx) = id1 {
                                                let idx1 = idx.local.index();
                                                if tainted_source.contains(&(idx1 as usize)) {
                                                    if let Operand::Constant(_) = op1 {
                                                        // if external function arg is constant, then clear the source
                                                        taint_analyzer.clear_source(idx1);
                                                    }
                                                }
                                            }
                                        } 
                                        if ( op2.ty(self.body, self.rcx.tcx()).is_numeric() ) {
                                            let id2 = op2.place();
                                            if let Some(idx) = id2 {
                                                let idx2 = idx.local.index();
                                                if tainted_source.contains(&(idx2 as usize)) {
                                                    if let Operand::Constant(_) = op2 {
                                                        taint_analyzer.clear_source(idx2);
                                                    }
                                                }
                                            }
                                        }
                                        taint_analyzer.mark_sink(lplace.local.index());
                                        self.status
                                            .ty_convs
                                            .push(statement.source_info.span);
                                    },
                                    _ => {},
                                }
                            },
                            _ => {},
                        }
                    },
                    _ => {},
                }
            }

            let prog_flag = taint_analyzer.propagate();
            // println!("{:?}", prog_flag);
            self.status.behavior_flag = prog_flag;
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
// Used to associate each broken layout bug report with its cause.
bitflags! {
    #[derive(Default)]
    pub struct BehaviorFlag: u16 {
        const EXTERNAL = 0b00000001;
        const TRANSMUTE = 0b00000010;
    }
}

impl IntoReportLevel for BehaviorFlag {
    fn report_level(&self, visibility: bool) -> ReportLevel {
        use BehaviorFlag as Flag;

        let high = Flag::EXTERNAL | Flag::TRANSMUTE;
        //let med = Flag::READ_FLOW | Flag::COPY_FLOW | Flag::WRITE_FLOW;

        // if !(*self & high).is_empty() {
        //     ReportLevel::Error
        // } else if !(*self & med).is_empty() {
        //     ReportLevel::Warning
        // } else {
        //     ReportLevel::Info
        // }
        
        ReportLevel::Error
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
