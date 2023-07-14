use rustc_hir::{def_id::DefId, BodyId};
use rustc_middle::mir::Operand;
use rustc_middle::ty::{Instance, ParamEnv, TyKind};
use rustc_span::{Span, DUMMY_SP};

use snafu::{Backtrace, Snafu};
use termcolor::Color;

use crate::ir;

pub BrokenLayoutChecker<'tcx> {
    rcx: RuMorphCtxt<'tcx>,
}

impl BrokenLayoutChecker<'tcx> {
    pub fn new(rcx: RudraCtxt<'tcx>) -> Self {
        BrokenLayoutChecker { rcx }
    }

    pub fn analyze(self) {
        let tcx = self.rcx.tcx();
        let hir_map = tcx.hir();

        // Iterates all (type, related function) pairs
        for (_ty_hir_id, (body_id, related_item_span)) in self.rcx.types_with_related_items() {
            if let Some(status) = inner::BrokenLayoutBodyAnalyzer::analyze_body(self.rcx, body_id)
            {
                let behavior_flag = status.behavior_flag();
                if !behavior_flag.is_empty()
                    && behavior_flag.report_level() >= self.rcx.report_level()
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

                    rudra_report(Report::with_color_span(
                        tcx,
                        behavior_flag.report_level(),
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
    pub struct BrokenLayoutStatus {
        strong_bypasses: Vec<Span>,
        weak_bypasses: Vec<Span>,
        unresolvable_generic_functions: Vec<Span>,
        behavior_flag: BehaviorFlag,
    }

    impl BrokenLayoutStatus {
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

    pub struct BrokenLayoutBodyAnalyzer<'a, 'tcx> {
        rcx: RuMorphCtxt<'tcx>,
        body: &'a ir::Body<'tcx>,
        param_env: ParamEnv<'tcx>,
        status: BrokenLayoutStatus,
    }
}