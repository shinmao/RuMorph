use std::rc::Rc;

use rustc_hir::{
    def_id::{DefId, LocalDefId},
    BodyId, ConstContext, HirId,
};
use rustc_middle::mir::{self, TerminatorKind, StatementKind, Rvalue, Operand};
use rustc_middle::ty::{Ty, TyCtxt, TyKind};
use rustc_span::Span;
use crate::progress_info;

use dashmap::DashMap;
use snafu::Snafu;

use crate::ir;
use crate::prelude::*;
use crate::report::ReportLevel;
use crate::visitor::{create_adt_impl_map, AdtImplMap, RelatedFnCollector, RelatedItemMap};

#[derive(Debug, Snafu, Clone)]
pub enum MirInstantiationError {
    NotAvailable { def_id: DefId },
}

impl AnalysisError for MirInstantiationError {
    fn kind(&self) -> AnalysisErrorKind {
        use MirInstantiationError::*;
        match self {
            NotAvailable { .. } => AnalysisErrorKind::OutOfScope,
        }
    }
}

pub type RuMorphCtxt<'tcx> = &'tcx RuMorphCtxtOwner<'tcx>;
pub type TranslationResult<'tcx, T> = Result<T, MirInstantiationError>;

/// Maps Instance to MIR and cache the result.
pub struct RuMorphCtxtOwner<'tcx> {
    tcx: TyCtxt<'tcx>,
    translation_cache: DashMap<DefId, Rc<TranslationResult<'tcx, ir::Body<'tcx>>>>,
    related_item_cache: RelatedItemMap,
    adt_impl_cache: AdtImplMap<'tcx>,
    report_level: ReportLevel,
    optimize_option: bool,
}

/// Visit MIR body and returns a RuMorph IR function
/// Check rustc::mir::visit::Visitor for possible visit targets
/// https://doc.rust-lang.org/nightly/nightly-rustc/rustc/mir/visit/trait.Visitor.html
impl<'tcx> RuMorphCtxtOwner<'tcx> {
    pub fn new(tcx: TyCtxt<'tcx>, report_level: ReportLevel, optimize_option: bool) -> Self {
        RuMorphCtxtOwner {
            tcx,
            translation_cache: DashMap::new(),
            related_item_cache: RelatedFnCollector::collect(tcx),
            adt_impl_cache: create_adt_impl_map(tcx),
            report_level,
            optimize_option,
        }
    }

    pub fn tcx(&self) -> TyCtxt<'tcx> {
        self.tcx
    }

    pub fn opt_option(&self) -> bool {
        self.optimize_option
    }

    pub fn types_with_related_items(
        &self,
    ) -> impl Iterator<Item = (Option<HirId>, (BodyId, Span))> + '_ {
        (&self.related_item_cache)
            .into_iter()
            .flat_map(|(&k, v)| v.iter().map(move |&body_id| (k, body_id)))
    }

    pub fn translate_body(&self, def_id: DefId) -> Rc<TranslationResult<'tcx, ir::Body<'tcx>>> {
        let tcx = self.tcx();
        //let codegen_list = tcx.collect_and_partition_mono_items(()).1;
        //for cg in codegen_list {
        //    progress_info!("The name of codegen: {:?}", cg.name());
        //    progress_info!("The items: {:?}", cg.items());
        //}
        let result = self.translation_cache.entry(def_id).or_insert_with(|| {
            Rc::new(
                try {
                    let mir_body = Self::find_fn(tcx, def_id)?;
                    self.translate_body_impl(mir_body)?
                },
            )
        });

        result.clone()
    }

    fn translate_body_impl(
        &self,
        body: &mir::Body<'tcx>,
    ) -> TranslationResult<'tcx, ir::Body<'tcx>> {
        let local_decls = body
            .local_decls
            .iter()
            .map(|local_decl| self.translate_local_decl(local_decl))
            .collect::<Vec<_>>();

        let basic_blocks: Vec<_> = body
            .basic_blocks
            .iter()
            .map(|basic_block| self.translate_basic_block(basic_block))
            .collect::<Result<Vec<_>, _>>()?;

        // we only locate local rather than place
        // e.g., (*_3).field: we would only locate _3
        let mut v = Vec::new();
        for _ in 0..local_decls.len() {
            let mut vv = Vec::new();
            v.push(vv);
        }

        for bb in &basic_blocks {
            for statement in &bb.statements {
                // statement: mir::Statement
                match &statement.kind {
                    StatementKind::Assign(box (lplace, rval)) => {
                        match rval {
                            Rvalue::Cast(_, op, _)
                            | Rvalue::Use(op)
                            | Rvalue::Repeat(op, _)
                            | Rvalue::ShallowInitBox(op, _) => {
                                match op {
                                    Operand::Copy(rplace) | Operand::Move(rplace) => {
                                        let id = rplace.local.index();
                                        v[id].push(lplace.local.index());
                                    },
                                    _ => {},
                                }
                            },
                            Rvalue::Ref(_, _, rplace)
                            | Rvalue::AddressOf(_, rplace)
                            | Rvalue::Len(rplace)
                            | Rvalue::Discriminant(rplace)
                            | Rvalue::CopyForDeref(rplace) => {
                                let id = rplace.local.index();
                                v[id].push(lplace.local.index());
                            },
                            Rvalue::BinaryOp(_, box (op1, op2))
                            | Rvalue::CheckedBinaryOp(_, box (op1, op2)) => {
                                let id1 = op1.place().unwrap().local.index();
                                let id2 = op2.place().unwrap().local.index();
                                v[id1].push(lplace.local.index());
                                v[id2].push(lplace.local.index());
                            },
                            _ => {},
                        }
                    },
                    _ => {},
                }
            }

            // we also need to handle terminator case
            match &bb.terminator.kind {
                // ir::Terminator
                ir::TerminatorKind::StaticCall {
                    callee_did,
                    callee_substs,
                    ref args,
                    dest,
                } => {
                    for arg in args {
                        // arg: mir::Operand
                        match arg {
                            Operand::Copy(pl) | Operand::Move(pl) => {
                                let id = pl.local.index();
                                v[id].push(dest.local.index());
                            },
                            _ => {},
                        }
                    }
                },
                _ => {},
            }
        }

        Ok(ir::Body {
            local_decls,
            original_decls: body.local_decls.to_owned(),
            basic_blocks,
            original: body.to_owned(),
            place_neighbor_list: v,
        })
    }

    fn translate_basic_block(
        &self,
        basic_block: &mir::BasicBlockData<'tcx>,
    ) -> TranslationResult<'tcx, ir::BasicBlock<'tcx>> {
        let statements = basic_block
            .statements
            .iter()
            .map(|statement| statement.clone())
            .collect::<Vec<_>>();

        let terminator = self.translate_terminator(
            basic_block
                .terminator
                .as_ref()
                .expect("Terminator should not be empty at this point"),
        )?;

        Ok(ir::BasicBlock {
            statements,
            terminator,
            is_cleanup: basic_block.is_cleanup,
        })
    }

    fn translate_terminator(
        &self,
        terminator: &mir::Terminator<'tcx>,
    ) -> TranslationResult<'tcx, ir::Terminator<'tcx>> {
        Ok(ir::Terminator {
            kind: match &terminator.kind {
                TerminatorKind::Goto { target } => ir::TerminatorKind::Goto(target.index()),
                TerminatorKind::Return => ir::TerminatorKind::Return,
                TerminatorKind::Call {
                    func: func_operand,
                    args,
                    destination: dest,
                    ..
                } => {

                    if let mir::Operand::Constant(box func) = func_operand {
                        let func_ty = func.literal.ty();
                        match func_ty.kind() {
                            TyKind::FnDef(def_id, callee_substs) => {
                                ir::TerminatorKind::StaticCall {
                                    callee_did: *def_id,
                                    callee_substs,
                                    args: args.clone(),
                                    dest: *dest,
                                }
                            }
                            TyKind::FnPtr(_) => ir::TerminatorKind::FnPtr {
                                value: func.literal.clone(),
                            },
                            _ => panic!("invalid callee of type {:?}", func_ty),
                        }
                    } else {
                        ir::TerminatorKind::Unimplemented("non-constant function call".into())
                    }
                }
                TerminatorKind::Drop { .. } => {
                    // TODO: implement Drop and DropAndReplace terminators
                    ir::TerminatorKind::Unimplemented(
                        format!("TODO terminator: {:?}", terminator).into(),
                    )
                }
                _ => ir::TerminatorKind::Unimplemented(
                    format!("Unknown terminator: {:?}", terminator).into(),
                ),
            },
            original: terminator.clone(),
        })
    }

    fn translate_local_decl(&self, local_decl: &mir::LocalDecl<'tcx>) -> ir::LocalDecl<'tcx> {
        ir::LocalDecl { ty: local_decl.ty }
    }

    /// Try to find MIR function body with def_id.
    fn find_fn(
        tcx: TyCtxt<'tcx>,
        def_id: DefId,
    ) -> Result<&'tcx mir::Body<'tcx>, MirInstantiationError> {
        if tcx.is_mir_available(def_id)
            && matches!(
                tcx.hir().body_const_context(def_id.expect_local()),
                None | Some(ConstContext::ConstFn)
            )
        {
            Ok(tcx.optimized_mir(def_id))
        } else {
            debug!(
                "Skipping an item {:?}, no MIR available for this item",
                def_id
            );
            NotAvailable { def_id }.fail()
        }
    }

    // pub fn index_adt_cache(&self, adt_did: &DefId) -> Option<&Vec<(LocalDefId, Ty)>> {
    //     self.adt_impl_cache.get(adt_did)
    // }

    pub fn report_level(&self) -> ReportLevel {
        self.report_level
    }
}
