use std::rc::Rc;

use rustc_hir::{
    def_id::{DefId, LocalDefId},
    BodyId, ConstContext, HirId,
};
use rustc_middle::mir::{self, TerminatorKind};
use rustc_middle::ty::{Ty, TyCtxt, TyKind};
use rustc_span::Span;

use dashmap::DashMap;
use snafu::Snafu;

// use crate::ir;
// use crate::prelude::*;
// use crate::report::ReportLevel;
// use crate::visitor::{create_adt_impl_map, AdtImplMap, RelatedFnCollector, RelatedItemMap};

// /// Maps Instance to MIR and cache the result.
// pub struct RuMorphCtxtOwner<'tcx> {
//     tcx: TyCtxt<'tcx>,
//     translation_cache: DashMap<DefId, Rc<TranslationResult<'tcx, ir::Body<'tcx>>>>,
//     related_item_cache: RelatedItemMap,
//     adt_impl_cache: AdtImplMap<'tcx>,
//     report_level: ReportLevel,
// }