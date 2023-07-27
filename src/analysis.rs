mod broken_layout;

use rustc_middle::ty::{self, Ty, ParamEnv, TypeAndMut};

use snafu::{Error, ErrorCompat};

use crate::report::ReportLevel;
use crate::context::RuMorphCtxt;

pub use broken_layout::{BehaviorFlag as BrokenLayoutBehaviorFlag, BrokenLayoutChecker};

pub type AnalysisResult<'tcx, T> = Result<T, Box<dyn AnalysisError + 'tcx>>;

use std::borrow::Cow;

pub trait AnalysisError: Error + ErrorCompat {
    fn kind(&self) -> AnalysisErrorKind;
    fn log(&self) {
        match self.kind() {
            AnalysisErrorKind::Unreachable => {
                error!("[{:?}] {}", self.kind(), self);
                if cfg!(feature = "backtraces") {
                    if let Some(backtrace) = ErrorCompat::backtrace(self) {
                        error!("Backtrace:\n{:?}", backtrace);
                    }
                }
            }
            AnalysisErrorKind::Unimplemented => {
                info!("[{:?}] {}", self.kind(), self);
                if cfg!(feature = "backtraces") {
                    if let Some(backtrace) = ErrorCompat::backtrace(self) {
                        info!("Backtrace:\n{:?}", backtrace);
                    }
                }
            }
            AnalysisErrorKind::OutOfScope => {
                debug!("[{:?}] {}", self.kind(), self);
                if cfg!(feature = "backtraces") {
                    if let Some(backtrace) = ErrorCompat::backtrace(self) {
                        debug!("Backtrace:\n{:?}", backtrace);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum AnalysisErrorKind {
    /// An error that should never happen;
    /// If this happens, that means some of our assumption / invariant is broken.
    /// Normal programs would panic for it, but we want to avoid panic at all cost,
    /// so this error exists.
    Unreachable,
    /// A pattern that is not handled by our algorithm yet.
    Unimplemented,
    /// An expected failure, something like "we don't handle this by design",
    /// that worth recording.
    OutOfScope,
}

#[derive(Debug, Copy, Clone)]
pub enum AnalysisKind {
    BrokenLayout(BrokenLayoutBehaviorFlag),
}

trait IntoReportLevel {
    fn report_level(&self) -> ReportLevel;
}

impl Into<Cow<'static, str>> for AnalysisKind {
    fn into(self) -> Cow<'static, str> {
        match &self {
            AnalysisKind::BrokenLayout(bypass_kinds) => {
                let mut v = vec!["BrokenLayout:"];
                // if bypass_kinds.contains(BrokenLayoutBehaviorFlag::READ_FLOW) {
                //     v.push("ReadFlow")
                // }
                // if bypass_kinds.contains(BrokenLayoutBehaviorFlag::COPY_FLOW) {
                //     v.push("CopyFlow")
                // }
                // if bypass_kinds.contains(BrokenLayoutBehaviorFlag::VEC_FROM_RAW) {
                //     v.push("VecFromRaw")
                // }
                // if bypass_kinds.contains(BrokenLayoutBehaviorFlag::TRANSMUTE) {
                //     v.push("Transmute")
                // }
                // if bypass_kinds.contains(BrokenLayoutBehaviorFlag::WRITE_FLOW) {
                //     v.push("WriteFlow")
                // }
                // if bypass_kinds.contains(BrokenLayoutBehaviorFlag::PTR_AS_REF) {
                //     v.push("PtrAsRef")
                // }
                // if bypass_kinds.contains(BrokenLayoutBehaviorFlag::SLICE_UNCHECKED) {
                //     v.push("SliceUnchecked")
                // }
                // if bypass_kinds.contains(BrokenLayoutBehaviorFlag::SLICE_FROM_RAW) {
                //     v.push("SliceFromRaw")
                // }
                // if bypass_kinds.contains(BrokenLayoutBehaviorFlag::VEC_SET_LEN) {
                //     v.push("VecSetLen")
                // }
                v.join("/").into()
            }
        }
    }
}

// e.g., A is less than B
// A is equal to B
// A is greater than B
#[derive(Debug, Copy, Clone)]
pub enum Comparison {
    Less,
    Equal,
    Greater,
    Noidea,
}

pub struct LayoutChecker<'tcx> {
    rcx: RuMorphCtxt<'tcx>,
    from_ty: Ty<'tcx>,
    to_ty: Ty<'tcx>,
    align_status: Comparison,
    size_status: Comparison,
}

impl<'tcx> LayoutChecker<'tcx> {
    pub fn new(&self, rc: RuMorphCtxt<'tcx>, p_env: ParamEnv<'tcx>, f_ty: Ty<'tcx>, t_ty: Ty<'tcx>) -> Self {
        // rustc_middle::ty::TyCtxt
        let tcx = self.rcx.tcx();
        let (f_ty_, t_ty_) = (Self::get_pointee(f_ty), Self::get_pointee(t_ty));
        // from_ty_and_layout = rustc_target::abi::TyAndLayout
        // (align_status, size_status)
        let layout_res = if let Ok(from_ty_and_layout) = tcx.layout_of(p_env.and(f_ty_))
            && let Ok(to_ty_and_layout) = tcx.layout_of(p_env.and(t_ty_))
        {
            let (from_layout, to_layout) = (from_ty_and_layout.layout, to_ty_and_layout.layout);
            let (from_align, to_align) = (from_layout.align(), to_layout.align());
            let (from_size, to_size) = (from_layout.size(), to_layout.size());
            // for align_status
            let ag_status = if from_align.abi.bytes() < to_align.abi.bytes() {
                Comparison::Less
            } else if from_align.abi.bytes() == to_align.abi.bytes() {
                Comparison::Equal
            } else if from_align.abi.bytes() > to_align.abi.bytes() {
                Comparison::Greater
            } else {
                Comparison::Noidea
            };
            // for size_status
            let sz_status = if from_size.bytes() < to_size.bytes() {
                Comparison::Less
            } else if from_size.bytes() == to_size.bytes() {
                Comparison::Equal
            } else if from_size.bytes() > to_size.bytes() {
                Comparison::Greater
            } else {
                Comparison::Noidea
            };

            (ag_status, sz_status)
        } else {
            (Comparison::Noidea, Comparison::Noidea)
        };
        LayoutChecker { rcx: rc, 
            from_ty: f_ty, 
            to_ty: t_ty,
            align_status: layout_res.0,
            size_status: layout_res.1,
        }
    }

    pub fn get_pointee(matched_ty: Ty) -> Ty {
        let pointee = if let ty::RawPtr(ty_mut) = matched_ty.kind() {
            Self::get_pointee(ty_mut.ty)
        } else if let ty::Ref(_, referred_ty, _) = matched_ty.kind() {
            Self::get_pointee(referred_ty)
        } else {
            matched_ty
        };
        pointee
    }

    pub fn get_align_status(&self) -> Comparison {
        self.align_status
    }

    pub fn get_size_status(&self) -> Comparison {
        self.size_status
    }
}