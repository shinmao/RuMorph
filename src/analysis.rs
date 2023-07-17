mod broken_layout;

use snafu::{Error, ErrorCompat};

use crate::report::ReportLevel;

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
                if bypass_kinds.contains(BrokenLayoutBehaviorFlag::READ_FLOW) {
                    v.push("ReadFlow")
                }
                if bypass_kinds.contains(BrokenLayoutBehaviorFlag::COPY_FLOW) {
                    v.push("CopyFlow")
                }
                if bypass_kinds.contains(BrokenLayoutBehaviorFlag::VEC_FROM_RAW) {
                    v.push("VecFromRaw")
                }
                if bypass_kinds.contains(BrokenLayoutBehaviorFlag::TRANSMUTE) {
                    v.push("Transmute")
                }
                if bypass_kinds.contains(BrokenLayoutBehaviorFlag::WRITE_FLOW) {
                    v.push("WriteFlow")
                }
                if bypass_kinds.contains(BrokenLayoutBehaviorFlag::PTR_AS_REF) {
                    v.push("PtrAsRef")
                }
                if bypass_kinds.contains(BrokenLayoutBehaviorFlag::SLICE_UNCHECKED) {
                    v.push("SliceUnchecked")
                }
                if bypass_kinds.contains(BrokenLayoutBehaviorFlag::SLICE_FROM_RAW) {
                    v.push("SliceFromRaw")
                }
                if bypass_kinds.contains(BrokenLayoutBehaviorFlag::VEC_SET_LEN) {
                    v.push("VecSetLen")
                }
                v.join("/").into()
            }
        }
    }
}