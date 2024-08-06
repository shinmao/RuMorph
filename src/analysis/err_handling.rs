use rustc_hir::{def_id::DefId, BodyId, Unsafety};
use rustc_middle::mir::{Operand, StatementKind, Rvalue, CastKind, Place, HasLocalDecls, AggregateKind};
use rustc_middle::mir::RETURN_PLACE;
use rustc_middle::ty::{self, Ty, Instance, ParamEnv, TyKind};
use rustc_span::{Span, DUMMY_SP};

use std::collections::HashMap;
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
pub enum ErrHandleError {
    PushPopBlock { backtrace: Backtrace },
    ResolveError { backtrace: Backtrace },
    InvalidSpan { backtrace: Backtrace },
}

impl AnalysisError for ErrHandleError {
    fn kind(&self) -> AnalysisErrorKind {
        use ErrHandleError::*;
        match self {
            PushPopBlock { .. } => AnalysisErrorKind::Unreachable,
            ResolveError { .. } => AnalysisErrorKind::OutOfScope,
            InvalidSpan { .. } => AnalysisErrorKind::Unreachable,
        }
    }
}

pub struct ErrHandleChecker<'tcx> {
    rcx: RuMorphCtxt<'tcx>,
}

impl<'tcx> ErrHandleChecker<'tcx> {
    pub fn new(rcx: RuMorphCtxt<'tcx>) -> Self {
        ErrHandleChecker { rcx }
    }

    pub fn analyze(self) {
        let tcx = self.rcx.tcx();
        let hir_map = tcx.hir();

        // Iterates all (type, related function) pairs
        for (_ty_hir_id, (body_id, related_item_span)) in self.rcx.types_with_related_items() {
            
            // print the funciton name of current body
            progress_info!("ErrHandleChecker::analyze({})", 
                        tcx.def_path_str(hir_map.body_owner_def_id(body_id).to_def_id())
            );


            if let Some(status) = inner::ErrHandleBodyAnalyzer::analyze_body(self.rcx, body_id)
            {
                let behavior_flag = status.behavior_flag();
                let err = status.error_kind();
                let lc = status.get_error_loc();
                if !behavior_flag.is_empty()
                    //&& behavior_flag.report_level() >= self.rcx.report_level()
                {
                    progress_info!("find the bug with behavior_flag: {:?}", behavior_flag);
                    let mut color_span = unwrap_or!(
                        utils::ColorSpan::new(tcx, related_item_span).context(InvalidSpan) => continue
                    );

                    for &span in status.plain_deref_spans() {
                        color_span.add_sub_span(Color::Blue, span);
                    }
                    
                    for &span in status.ty_conv_spans() {
                        color_span.add_sub_span(Color::Green, span);
                    }

                    for &span in status.branch_handle_spans() {
                        color_span.add_sub_span(Color::Red, span);
                    }

                    rumorph_report(Report::with_color_span(
                        tcx,
                        behavior_flag.report_level(true),
                        AnalysisKind::ErrHandle(behavior_flag),
                        format!(
                            "Potential Err Handling issue in `{}` with Pattern `{}` at line `{}`",
                            tcx.def_path_str(hir_map.body_owner_def_id(body_id).to_def_id()),
                            err,
                            lc
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
    pub struct ErrHandleStatus {
        strong_bypasses: Vec<Span>,
        weak_bypasses: Vec<Span>,
        plain_deref: Vec<Span>,
        unresolvable_generic_functions: Vec<Span>,
        ty_convs: Vec<Span>,
        branch_handles: Vec<Span>,
        behavior_flag: BehaviorFlag,
        error: usize,
        loc: usize,
    }

    impl ErrHandleStatus {
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

        pub fn branch_handle_spans(&self) -> &Vec<Span> {
            &self.branch_handles
        }

        pub fn error_kind(&self) -> usize {
            self.error
        }

        pub fn get_error_loc(&self) -> usize {
            self.loc
        }
    }

    pub struct ErrHandleBodyAnalyzer<'a, 'tcx> {
        rcx: RuMorphCtxt<'tcx>,
        body: &'a ir::Body<'tcx>,
        param_env: ParamEnv<'tcx>,
        status: ErrHandleStatus,
    }

    impl<'a, 'tcx> ErrHandleBodyAnalyzer<'a, 'tcx> {
        fn new(rcx: RuMorphCtxt<'tcx>, param_env: ParamEnv<'tcx>, body: &'a ir::Body<'tcx>) -> Self {
            ErrHandleBodyAnalyzer {
                rcx,
                body,
                param_env,
                status: Default::default(),
            }
        }

        pub fn analyze_body(rcx: RuMorphCtxt<'tcx>, body_id: BodyId) -> Option<ErrHandleStatus> {
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
                        let body_analyzer = ErrHandleBodyAnalyzer::new(rcx, param_env, body);
                        Some(body_analyzer.analyze())
                    }
                }
            }
        }

        fn analyze(mut self) -> ErrHandleStatus {
            let mut taint_analyzer = TaintAnalyzer::new(self.body);

            let mut error_kind_map = HashMap::new();
            let mut sink_loc_map = HashMap::new();

            let mut checked_idx = usize::MAX;

            // used to store immediate function call for tainted status
            let mut immediate_status = Vec::new();

            let mut checked_source = Vec::new();

            let mut cleanup_bb = Vec::new();
            let mut idx = 0;
            for bb in &self.body.basic_blocks {
                match &bb.terminator.kind {
                    ir::TerminatorKind::Return => {
                        cleanup_bb.push(idx);
                    },
                    _ => {},
                }
                idx = idx + 1;
            }
            progress_info!("cleanup_bb: {:?}", cleanup_bb);
            
            idx = 0;
            for bb in &self.body.basic_blocks {
                progress_info!("basic block index: {:?}", idx);
                for s in &bb.statements {
                    progress_info!("{:?}", s);
                }
                progress_info!("{:?}", bb.terminator);
                idx = idx + 1;
            }

            // lookup_char_pos

            for (bb_idx, terminator) in self.body.terminators().enumerate() {
                let sp = terminator.original.source_info.span;
                let sm = self.rcx.tcx().sess.source_map();
                let loc = sm.lookup_char_pos(sp.lo()).line;
                match &terminator.kind {
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
                        let symbol_vec = ext.get_def_path(*callee_did);
                        let sym = symbol_vec[ symbol_vec.len() - 1 ].as_str();
                        // checked_* return Option
                        // expect, ok_or, ok_or_else, map, map_or, map_or_else, unwrap, unwrap_or..
                        
                        let id = dest.local.index();
                        if sym.contains("checked_") {
                            taint_analyzer.mark_source(id, &BehaviorFlag::CHECKEDCALL);
                            checked_idx = bb_idx;
                            checked_source.push(id);
                        } 

                        // unwrap_or -> error handling -> clear source -> not alarm
                        // wnwrap_or -> error handling -> panic -> alarm
                        
                        // at the end, we will clear the place id in immediate_status since it is handled
                        if sym.contains("map") || sym.contains("unwrap_or") || sym.contains("ok_or") {
                            immediate_status.push(id);
                        }

                        if !checked_source.is_empty() {
                            if sym.contains("panic") || sym.contains("expect") {
                                // only if panic/expect after checked* doesn't handle error correctly
                                // use mark_at_once since there is no dataflow relationship, but only control flow
                                taint_analyzer.mark_at_once(id, &BehaviorFlag::CHECKEDCALL);
                                error_kind_map.insert(id, "panic");
                                sink_loc_map.insert(id, loc);
                                self.status
                                    .branch_handles
                                    .push(terminator.original.source_info.span);
                                immediate_status.clear();
                            }
                        }
                    },
                    ir::TerminatorKind::SwitchInt {
                        discr,
                        targets
                    } => {
                        // check whether SwitchInt is the direct successor of checked_* functions
                        if (checked_idx != usize::MAX) && (self.body.is_direct_successor(checked_idx, bb_idx)) {
                            // check whether the targets contain return statement
                            let mut not_return: bool = false;
                            let mut return_arrs = Vec::new();
                            for tg in targets.all_targets() {
                                // breadth first search
                                for cb in &cleanup_bb {
                                    progress_info!("{:?} -> {:?}", tg.index(), *cb);
                                    return_arrs.push(self.body.arr_return(tg.index(), *cb));
                                }
                            }

                            progress_info!("return arrays: {:?}", return_arrs);

                            for (i, sarr1) in return_arrs.iter().enumerate() {
                                for (j, sarr2) in return_arrs.iter().enumerate() {
                                    if let Some(path1) = sarr1 {
                                        if let Some(path2) = sarr2 {
                                            if i != j && is_subarray(&path1[1..], &path2[1..]) {
                                                not_return |= true; 
                                            }
                                        }
                                    }
                                }
                            }

                            progress_info!("no return? :{:?}", not_return);
                            
                            if not_return {
                                match discr {
                                    Operand::Copy(pl) | Operand::Move(pl) => {
                                        let id = pl.local.index();
                                        taint_analyzer.mark_sink(id);
                                        error_kind_map.insert(id, "ignore");
                                        sink_loc_map.insert(id, loc);
                                        self.status
                                            .branch_handles
                                            .push(terminator.original.source_info.span);
                                    },
                                    _ => {},
                                }
                            }
                        }
                    },
                    _ => {},
                }
            }

            // if already visiting all terminators
            // there is unwrap/ok/map, but not find panic/expect to handle it
            // then error is handled correctly and should not alarm
            if !immediate_status.is_empty() {
                progress_info!("clear source!! not alarm!!");
                for src_id in immediate_status {
                    taint_analyzer.clear_source(src_id);
                }
            }

            self.status.behavior_flag = taint_analyzer.propagate();
            
            // there are two kinds of error stored in error_kind_map: ignore and panic
            for sink in taint_analyzer.get_reachable_sinks() {
                self.status.error = match error_kind_map.get(sink) {
                    Some(err) => {
                        match *err {
                            "panic" => 3,
                            "ignore" => 1,
                            _ => 4,
                        }
                    },
                    _ => 4,
                };
                // 0 represent not able to get line number
                self.status.loc = match sink_loc_map.get(sink) {
                    Some(lc) => *lc,
                    _ => 0,
                };
            }

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

fn is_subarray(sub: &[usize], arr: &[usize]) -> bool {
    if sub.is_empty() {
        return false;
    }

    if sub.len() > arr.len() {
        return false;
    }

    for i in 0..=(arr.len() - sub.len()) {
        if (&arr[i..i + sub.len()] == sub) && (sub.len() > 3) {
            return true;
        }
    }
    false
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
        const CHECKEDCALL = 0b00000001;
        const TRANSMUTE = 0b00000010;
    }
}

impl IntoReportLevel for BehaviorFlag {
    fn report_level(&self, visibility: bool) -> ReportLevel {
        use BehaviorFlag as Flag;

        let high = Flag::CHECKEDCALL | Flag::TRANSMUTE;
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
