use rustc_hir::{def_id::DefId, BodyId};
use rustc_middle::mir::Operand;
use rustc_middle::ty::{Instance, ParamEnv, TyKind};
use rustc_span::{Span, DUMMY_SP};

use snafu::{Backtrace, Snafu};
use termcolor::Color;

use crate::graph::GraphTaint;
use crate::prelude::*;
use crate::{
    analysis::{AnalysisKind, IntoReportLevel},
    graph::TaintAnalyzer,
    ir,
    paths::{self, *},
    report::{Report, ReportLevel},
    utils,
    visitor::ContainsUnsafe,
};

#[derive(Debug, Snafu)]
pub enum UnsafeDataflowError {
    PushPopBlock { backtrace: Backtrace },
    ResolveError { backtrace: Backtrace },
    InvalidSpan { backtrace: Backtrace },
}

impl AnalysisError for UnsafeDataflowError {
    fn kind(&self) -> AnalysisErrorKind {
        use UnsafeDataflowError::*;
        match self {
            PushPopBlock { .. } => AnalysisErrorKind::Unreachable,
            ResolveError { .. } => AnalysisErrorKind::OutOfScope,
            InvalidSpan { .. } => AnalysisErrorKind::Unreachable,
        }
    }
}

pub struct UnsafeDataflowChecker<'tcx> {
    rcx: RuMorphCtxt<'tcx>,
}

impl<'tcx> UnsafeDataflowChecker<'tcx> {
    pub fn new(rcx: RuMorphCtxt<'tcx>) -> Self {
        UnsafeDataflowChecker { rcx }
    }

    pub fn analyze(self) {
        let tcx = self.rcx.tcx();
        let hir_map = tcx.hir();

        // Iterates all (type, related function) pairs
        for (_ty_hir_id, (body_id, related_item_span)) in self.rcx.types_with_related_items() {
            if let Some(status) = inner::UnsafeDataflowBodyAnalyzer::analyze_body(self.rcx, body_id)
            {
                let behavior_flag = status.behavior_flag();
                if !behavior_flag.is_empty()
                    && behavior_flag.report_level(true) >= self.rcx.report_level()
                {
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

                    rumorph_report(Report::with_color_span(
                        tcx,
                        behavior_flag.report_level(true),
                        AnalysisKind::UnsafeDataflow(behavior_flag),
                        format!(
                            "Potential unsafe dataflow issue in `{}`",
                            tcx.def_path_str(hir_map.body_owner_def_id(body_id).to_def_id())
                        ),
                        &color_span,
                    ))
                }
            }
        }
    }
}

mod inner {
    use super::*;

    #[derive(Debug, Default)]
    pub struct UnsafeDataflowStatus {
        strong_bypasses: Vec<Span>,
        weak_bypasses: Vec<Span>,
        unresolvable_generic_functions: Vec<Span>,
        behavior_flag: BehaviorFlag,
    }

    impl UnsafeDataflowStatus {
        pub fn behavior_flag(&self) -> BehaviorFlag {
            self.behavior_flag
        }

        pub fn strong_bypass_spans(&self) -> &Vec<Span> {
            &self.strong_bypasses
        }

        pub fn weak_bypass_spans(&self) -> &Vec<Span> {
            &self.weak_bypasses
        }

        pub fn unresolvable_generic_function_spans(&self) -> &Vec<Span> {
            &self.unresolvable_generic_functions
        }
    }

    pub struct UnsafeDataflowBodyAnalyzer<'a, 'tcx> {
        rcx: RuMorphCtxt<'tcx>,
        body: &'a ir::Body<'tcx>,
        param_env: ParamEnv<'tcx>,
        status: UnsafeDataflowStatus,
    }

    impl<'a, 'tcx> UnsafeDataflowBodyAnalyzer<'a, 'tcx> {
        fn new(rcx: RuMorphCtxt<'tcx>, param_env: ParamEnv<'tcx>, body: &'a ir::Body<'tcx>) -> Self {
            UnsafeDataflowBodyAnalyzer {
                rcx,
                body,
                param_env,
                status: Default::default(),
            }
        }

        pub fn analyze_body(rcx: RuMorphCtxt<'tcx>, body_id: BodyId) -> Option<UnsafeDataflowStatus> {
            let hir_map = rcx.tcx().hir();
            let body_did = hir_map.body_owner_def_id(body_id).to_def_id();

            if rcx.tcx().ext().match_def_path(
                body_did,
                &["rumorph_paths_discovery", "PathsDiscovery", "discover"],
            ) {
                // Special case for paths discovery
                trace_calls_in_body(rcx, body_did);
                None
            } else if ContainsUnsafe::contains_unsafe(rcx.tcx(), body_id) {
                match rcx.translate_body(body_did).as_ref() {
                    Err(e) => {
                        // MIR is not available for def - log it and continue
                        e.log();
                        None
                    }
                    Ok(body) => {
                        let param_env = rcx.tcx().param_env(body_did);
                        let body_analyzer = UnsafeDataflowBodyAnalyzer::new(rcx, param_env, body);
                        Some(body_analyzer.analyze())
                    }
                }
            } else {
                // We don't perform interprocedural analysis,
                // thus safe functions are considered safe
                Some(Default::default())
            }
        }

        fn analyze(mut self) -> UnsafeDataflowStatus {
            let mut taint_analyzer = TaintAnalyzer::new(self.body);

            for (id, terminator) in self.body.terminators().enumerate() {
                match terminator.kind {
                    ir::TerminatorKind::StaticCall {
                        callee_did,
                        callee_substs,
                        ref args,
                        ..
                    } => {
                        let tcx = self.rcx.tcx();
                        let ext = tcx.ext();
                        // Check for lifetime bypass
                        let symbol_vec = ext.get_def_path(callee_did);
                        if paths::STRONG_LIFETIME_BYPASS_LIST.contains(&symbol_vec) {
                            if self.fn_called_on_copy(
                                (callee_did, args),
                                &[&PTR_READ[..], &PTR_DIRECT_READ[..]],
                            ) {
                                // read on Copy types is not a lifetime bypass.
                                continue;
                            }

                            if ext.match_def_path(callee_did, &VEC_SET_LEN)
                                && vec_set_len_to_0(self.rcx, callee_did, args)
                            {
                                // Leaking data is safe (`vec.set_len(0);`)
                                continue;
                            }

                            taint_analyzer
                                .mark_source(id, STRONG_BYPASS_MAP.get(&symbol_vec).unwrap());
                            self.status
                                .strong_bypasses
                                .push(terminator.original.source_info.span);
                        } else if paths::WEAK_LIFETIME_BYPASS_LIST.contains(&symbol_vec) {
                            if self.fn_called_on_copy(
                                (callee_did, args),
                                &[&PTR_WRITE[..], &PTR_DIRECT_WRITE[..]],
                            ) {
                                // writing Copy types is not a lifetime bypass.
                                continue;
                            }

                            taint_analyzer
                                .mark_source(id, WEAK_BYPASS_MAP.get(&symbol_vec).unwrap());
                            self.status
                                .weak_bypasses
                                .push(terminator.original.source_info.span);
                        } else if paths::GENERIC_FN_LIST.contains(&symbol_vec) {
                            taint_analyzer.mark_sink(id);
                            self.status
                                .unresolvable_generic_functions
                                .push(terminator.original.source_info.span);
                        } else {
                            // Check for unresolvable generic function calls
                            match Instance::resolve(
                                self.rcx.tcx(),
                                self.param_env,
                                callee_did,
                                callee_substs,
                            ) {
                                Err(_e) => log_err!(ResolveError),
                                Ok(Some(_)) => {
                                    // Calls were successfully resolved
                                }
                                Ok(None) => {
                                    // Call contains unresolvable generic parts
                                    // Here, we are making a two step approximation:
                                    // 1. Unresolvable generic code is potentially user-provided
                                    // 2. User-provided code potentially panics
                                    taint_analyzer.mark_sink(id);
                                    self.status
                                        .unresolvable_generic_functions
                                        .push(terminator.original.source_info.span);
                                }
                            }
                        }
                    }
                    _ => (),
                }
            }

            self.status.behavior_flag = taint_analyzer.propagate();
            self.status
        }

        fn fn_called_on_copy(
            &self,
            (callee_did, callee_args): (DefId, &Vec<Operand<'tcx>>),
            paths: &[&[&str]],
        ) -> bool {
            let tcx = self.rcx.tcx();
            let ext = tcx.ext();
            for path in paths.iter() {
                if ext.match_def_path(callee_did, path) {
                    for arg in callee_args.iter() {
                        if_chain! {
                            if let Operand::Move(place) = arg;
                            let place_ty = place.ty(self.body, tcx);
                            if let TyKind::RawPtr(ty_and_mut) = place_ty.ty.kind();
                            let pointed_ty = ty_and_mut.ty;
                            if pointed_ty.is_copy_modulo_regions(*tcx.at(DUMMY_SP), self.param_env);
                            then {
                                return true;
                            }
                        }
                        // No need to inspect beyond first arg of the
                        // target bypass functions.
                        break;
                    }
                }
            }
            false
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

    // Check if the argument of `Vec::set_len()` is 0_usize.
    fn vec_set_len_to_0<'tcx>(
        rcx: RuMorphCtxt<'tcx>,
        callee_did: DefId,
        args: &Vec<Operand<'tcx>>,
    ) -> bool {
        let tcx = rcx.tcx();
        for arg in args.iter() {
            if_chain! {
                if let Operand::Constant(c) = arg;
                if let Some(c_val) = c.literal.try_eval_target_usize(
                    tcx,
                    tcx.param_env(callee_did),
                );
                if c_val == 0;
                then {
                    // Leaking(`vec.set_len(0);`) is safe.
                    return true;
                }
            }
        }
        false
    }
}

// Unsafe Dataflow BypassKind.
// Used to associate each Unsafe-Dataflow bug report with its cause.
bitflags! {
    #[derive(Default)]
    pub struct BehaviorFlag: u16 {
        const READ_FLOW = 0b00000001;
        const COPY_FLOW = 0b00000010;
        const VEC_FROM_RAW = 0b00000100;
        const TRANSMUTE = 0b00001000;
        const WRITE_FLOW = 0b00010000;
        const PTR_AS_REF = 0b00100000;
        const SLICE_UNCHECKED = 0b01000000;
        const SLICE_FROM_RAW = 0b10000000;
        const VEC_SET_LEN = 0b100000000;
    }
}

impl IntoReportLevel for BehaviorFlag {
    fn report_level(&self, visibility: bool) -> ReportLevel {
        use BehaviorFlag as Flag;

        let high = Flag::VEC_FROM_RAW | Flag::VEC_SET_LEN;
        let med = Flag::READ_FLOW | Flag::COPY_FLOW | Flag::WRITE_FLOW;

        if !(*self & high).is_empty() {
            ReportLevel::Error
        } else if !(*self & med).is_empty() {
            ReportLevel::Warning
        } else {
            ReportLevel::Info
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
