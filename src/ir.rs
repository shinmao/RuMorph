//! Reduced MIR intended to cover many common use cases while keeping the analysis pipeline manageable.
//! Note that this is a translation of non-monomorphized, generic MIR.

use std::borrow::Cow;

use rustc_hir::def_id::DefId;
use rustc_index::{IndexVec, IndexSlice};
use rustc_middle::{
    mir,
    ty::{subst::SubstsRef, Ty},
};

#[derive(Debug)]
pub struct Terminator<'tcx> {
    pub kind: TerminatorKind<'tcx>,
    pub original: mir::Terminator<'tcx>,
}

// https://doc.rust-lang.org/stable/nightly-rustc/rustc_middle/mir/syntax/enum.TerminatorKind.html
#[derive(Debug)]
pub enum TerminatorKind<'tcx> {
    Goto(usize),
    Return,
    StaticCall {
        callee_did: DefId,
        callee_substs: SubstsRef<'tcx>,
        args: Vec<mir::Operand<'tcx>>,
    },
    FnPtr {
        value: mir::ConstantKind<'tcx>,
    },
    Unimplemented(Cow<'static, str>),
}

#[derive(Debug)]
pub struct BasicBlock<'tcx> {
    pub statements: Vec<mir::Statement<'tcx>>,
    pub terminator: Terminator<'tcx>,
    pub is_cleanup: bool,
}

#[derive(Debug)]
pub struct LocalDecl<'tcx> {
    pub ty: Ty<'tcx>,
}

#[derive(Debug)]
pub struct Body<'tcx> {
    pub local_decls: Vec<LocalDecl<'tcx>>,
    pub original_decls: IndexVec<mir::Local, mir::LocalDecl<'tcx>>,
    pub basic_blocks: Vec<BasicBlock<'tcx>>,
    pub original: mir::Body<'tcx>,
    pub place_neighbor_list: Vec<Vec<usize>>,
}

impl<'tcx> mir::HasLocalDecls<'tcx> for Body<'tcx> {
    fn local_decls(&self) -> &IndexSlice<mir::Local, mir::LocalDecl<'tcx>> {
        &self.original_decls.as_slice()
    }
}

impl<'tcx> Body<'tcx> {
    pub fn statements(&self) -> Vec<mir::Statement<'tcx>> {
        let mut statement_list: Vec<mir::Statement<'tcx>> = Vec::new();
        for block in &self.basic_blocks {
            for st in &block.statements {
                statement_list.push(st.clone());
            }
        }
        statement_list
    }

    pub fn terminators(&self) -> impl Iterator<Item = &Terminator<'tcx>> {
        self.basic_blocks.iter().map(|block| &block.terminator)
    }
}
