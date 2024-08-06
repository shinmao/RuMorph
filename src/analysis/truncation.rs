use rustc_hir::{def_id::DefId, BodyId, Unsafety};
use rustc_middle::mir::{Operand, StatementKind, Rvalue, CastKind, Place, HasLocalDecls, AggregateKind, BinOp};
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
pub enum TruncationError {
    PushPopBlock { backtrace: Backtrace },
    ResolveError { backtrace: Backtrace },
    InvalidSpan { backtrace: Backtrace },
}

impl AnalysisError for TruncationError {
    fn kind(&self) -> AnalysisErrorKind {
        use TruncationError::*;
        match self {
            PushPopBlock { .. } => AnalysisErrorKind::Unreachable,
            ResolveError { .. } => AnalysisErrorKind::OutOfScope,
            InvalidSpan { .. } => AnalysisErrorKind::Unreachable,
        }
    }
}

pub struct TruncationChecker<'tcx> {
    rcx: RuMorphCtxt<'tcx>,
}

impl<'tcx> TruncationChecker<'tcx> {
    pub fn new(rcx: RuMorphCtxt<'tcx>) -> Self {
        TruncationChecker { rcx }
    }

    pub fn analyze(self) {
        let tcx = self.rcx.tcx();
        let hir_map = tcx.hir();

        // Iterates all (type, related function) pairs
        for (_ty_hir_id, (body_id, related_item_span)) in self.rcx.types_with_related_items() {
            
            // print the funciton name of current body
            progress_info!("TruncationChecker::analyze({})", 
                        tcx.def_path_str(hir_map.body_owner_def_id(body_id).to_def_id())
            );


            if let Some(status) = inner::TruncationBodyAnalyzer::analyze_body(self.rcx, body_id)
            {
                let behavior_flag = status.behavior_flag();
                let err = status.error_kind();
                let lc = status.get_error_loc();
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
                        AnalysisKind::Truncation(behavior_flag),
                        format!(
                            "Potential Truncation issue in `{}` with Pattern `{}` at line `{}`",
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
    pub struct TruncationStatus {
        strong_bypasses: Vec<Span>,
        weak_bypasses: Vec<Span>,
        plain_deref: Vec<Span>,
        unresolvable_generic_functions: Vec<Span>,
        ty_convs: Vec<Span>,
        behavior_flag: BehaviorFlag,
        error: usize,
        loc: usize,
    }

    impl TruncationStatus {
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

        // used as truncation operation here
        pub fn ty_conv_spans(&self) -> &Vec<Span> {
            &self.ty_convs
        }

        pub fn error_kind(&self) -> usize {
            self.error
        }

        pub fn get_error_loc(&self) -> usize {
            self.loc
        }
    }

    pub struct TruncationBodyAnalyzer<'a, 'tcx> {
        rcx: RuMorphCtxt<'tcx>,
        body: &'a ir::Body<'tcx>,
        param_env: ParamEnv<'tcx>,
        status: TruncationStatus,
    }

    impl<'a, 'tcx> TruncationBodyAnalyzer<'a, 'tcx> {
        fn new(rcx: RuMorphCtxt<'tcx>, param_env: ParamEnv<'tcx>, body: &'a ir::Body<'tcx>) -> Self {
            TruncationBodyAnalyzer {
                rcx,
                body,
                param_env,
                status: Default::default(),
            }
        }

        pub fn analyze_body(rcx: RuMorphCtxt<'tcx>, body_id: BodyId) -> Option<TruncationStatus> {
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
                        let body_analyzer = TruncationBodyAnalyzer::new(rcx, param_env, body);
                        Some(body_analyzer.analyze())
                    }
                }
            }
        }

        fn analyze(mut self) -> TruncationStatus {
            let mut taint_analyzer = TaintAnalyzer::new(self.body);
            // use `tainted_source` to maintain tainted external function args
            let mut tainted_source = Vec::new();
            let mut place_size = HashMap::new();
            let mut sliced_array_place = 0 as usize;
            let mut rangefull_place = 0 as usize;

            let mut error_kind_map = HashMap::new();
            let mut sink_loc_map = HashMap::new();

            // mark all the arguments as taint source
            for arg_idx in 1usize..self.body.original.arg_count + 1 {
                // progress_info!("external as source: {:?}", arg_idx);
                tainted_source.push(arg_idx);
            }

            for local in self.body.original.args_iter() {
                let local_decl = &self.body.original.local_decls[local];
                let ty = get_pointee(local_decl.ty);

                if let Ok(layout) = self.rcx.tcx().layout_of(self.param_env.and(ty)) {
                    let size = layout.size.bytes();
                    println!("Parameter {:?} with idx {:?} size: {} bytes", local, local.index(), size);
                    place_size.insert(local.index(), size);
                } else {
                    println!("Failed to get layout for parameter {:?}", local);
                }
            }

            for statement in self.body.statements() {
                progress_info!("kind: {:?} -> {:?}", statement.kind, statement);
                match statement.kind {
                    StatementKind::Assign(box (lplace, rval)) => {
                        match rval {
                            Rvalue::Use(op) => {
                                progress_info!("use {:?}", op);
                            },
                            Rvalue::Repeat(op, _) => {
                                progress_info!("use {:?}", op);
                            },
                            Rvalue::Aggregate(box aggregate_kind, operands) => {
                                if let AggregateKind::Adt(def_id, variant_idx, _, _, _) = aggregate_kind {
                                    if is_range_full(self.rcx, def_id) {
                                        rangefull_place = lplace.local.index();
                                    }
                                }
                            },
                            _ => {},
                        }
                    },
                    _ => {},
                }
            }

            for (_id, terminator) in self.body.terminators().enumerate() {
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
                        let symbol_vec = ext.get_def_path(*callee_did);
                        progress_info!("here is symbol: {:?}", symbol_vec);
                        let sym = symbol_vec[ symbol_vec.len() - 1 ].as_str();
                        
                        // e.g., copy_from_slice
                        // requires the source and dest buffer have the same size
                        // to make sure whether the original size of source buffer is larger than dest buffer
                        // we need to detect whether source buffer is sliced
                        if sym.contains("copy_") {
                            if let [dst, src] = &args[..] {
                                if let Some(src_place) = src.place() {
                                    let src_idx = src_place.local.index();                                        
                                    if sliced_array_place != 0 && taint_analyzer.is_reachable(sliced_array_place, src_idx) {
                                        progress_info!("ts idx: {:?} -> src idx: {:?}", sliced_array_place, src_idx);
                                        // if let (src_ty, dst_ty) = (src.ty(&self.body.original, self.rcx.tcx()), dst.ty(&self.body.original, self.rcx.tcx())) {
                                            
                                        //     let src_layout = self.rcx.tcx().layout_of(self.param_env.and(src_ty)).unwrap();
                                        //     let dst_layout = self.rcx.tcx().layout_of(self.param_env.and(dst_ty)).unwrap();

                                        //     let src_size = src_layout.size.bytes();
                                        //     let dst_size = dst_layout.size.bytes();

                                        //     if let Some(src_decl_size) = place_size.get(ts) {
                                        //         if src_decl_size > &src_size {
                                        //             progress_info!("{:?} -> {:?}", src_decl_size, src_size);
                                        //             taint_analyzer.mark_at_once(src_idx, &BehaviorFlag::EXTERNAL);
                                        //             error_kind_map.insert(src_idx, "copycall");
                                        //             sink_loc_map.insert(src_idx, loc);
                                        //         }
                                        //     }
                                        // }
                                        // Here, if we found that the source buffer is sliced, we consider it copy from larger-sized buffer
                                        taint_analyzer.mark_at_once(src_idx, &BehaviorFlag::EXTERNAL);
                                        error_kind_map.insert(src_idx, "copycall");
                                        sink_loc_map.insert(src_idx, loc);
                                    }
                                }
                            }                            
                        } else if sym.contains("index") {
                            if let [buf, idx] = &args[..] {
                                for ts in &tainted_source {
                                    if let Some(buf_place) = buf.place() {
                                        let buf_idx = buf_place.local.index();
                                        if let Some(idx_place) = idx.place() {
                                            let idx = idx_place.local.index();
                                            if taint_analyzer.is_reachable(*ts, buf_idx) {
                                                if !taint_analyzer.is_reachable(rangefull_place, idx) && rangefull_place != 0 {
                                                    progress_info!("ts idx: {:?} -> buf idx: {:?}", *ts, buf_idx);
                                                    sliced_array_place = buf_idx;   
                                                }                                  
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    _ => {},
                }
            }

            let prog_flag = taint_analyzer.propagate();
            // println!("{:?}", prog_flag);
            self.status.behavior_flag = prog_flag;

            // there are two kinds of error stored in error_kind_map: downcast, unsafeop, unsafeopcall
            for sink in taint_analyzer.get_reachable_sinks() {
                self.status.error = match error_kind_map.get(sink) {
                    Some(err) => {
                        match *err {
                            "copycall" => 1,
                            _ => 2,
                        }
                    },
                    _ => 2,
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

fn is_range_full<'tcx>(rcx: RuMorphCtxt<'tcx>, def_id: DefId) -> bool {
    let range_full = rcx.tcx().lang_items().range_full_struct();
    if let Some(range_full_id) = range_full {
        def_id == range_full_id
    } else {
        false
    }
}

// get the pointee or wrapped type
fn get_pointee(matched_ty: Ty<'_>) -> Ty<'_> {
    // progress_info!("get_pointee: > {:?} as type: {:?}", matched_ty, matched_ty.kind());
    let pointee = if let ty::RawPtr(ty_mut) = matched_ty.kind() {
        get_pointee(ty_mut.ty)
    } else if let ty::Ref(_, referred_ty, _) = matched_ty.kind() {
        get_pointee(*referred_ty)
    } else {
        matched_ty
    };
    pointee
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
