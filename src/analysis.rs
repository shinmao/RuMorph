mod broken_layout;
mod uninit_exposure;
mod broken_bitpatterns;

use rustc_hir::{ItemKind, ImplPolarity, ItemId, hir_id::OwnerId, OwnerNode};
use rustc_middle::hir::Owner;
use rustc_middle::ty::{self, Ty, ParamEnv, TypeAndMut, TyKind, TyCtxt, IntTy, UintTy, FloatTy, TraitPredicate, Binder};

use snafu::{Error, ErrorCompat};

use crate::report::ReportLevel;
use crate::context::RuMorphCtxt;
use crate::progress_info;

use std::collections::HashSet;

pub use broken_layout::{BehaviorFlag as BrokenLayoutBehaviorFlag, BrokenLayoutChecker};
pub use uninit_exposure::{BehaviorFlag as UninitExposureBehaviorFlag, UninitExposureChecker};
pub use broken_bitpatterns::{BehaviorFlag as BrokenBitPatternsBehaviorFlag, BrokenBitPatternsChecker};

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
    UninitExposure(UninitExposureBehaviorFlag),
    BrokenBitPatterns(BrokenBitPatternsBehaviorFlag),
}

trait IntoReportLevel {
    fn report_level(&self, visibility: bool) -> ReportLevel;
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
            },
            AnalysisKind::UninitExposure(bypass_kinds) => {
                let mut v = vec!["UninitExposure:"];
                v.join("/").into()
            },
            AnalysisKind::BrokenBitPatterns(bypass_kinds) => {
                let mut v = vec!["BrokenBitPatterns:"];
                v.join("/").into()
            },
        }
    }
}

// e.g., A is less than B
// A is equal to B
// A is greater than B
// In the case of NoideaG, A >= B
// In the case of NoideaL, A <= B
#[derive(Debug, Copy, Clone)]
pub enum Comparison {
    Less,
    Equal,
    Greater,
    Noidea,
    NoideaG,
    NoideaL,
}

pub struct LayoutChecker<'tcx> {
    rcx: RuMorphCtxt<'tcx>,
    from_ty: Ty<'tcx>,
    to_ty: Ty<'tcx>,
    fty_layout: bool,
    tty_layout: bool,
    align_status: Comparison,
    size_status: Comparison,
    ty_bnd: HashSet<Ty<'tcx>>,
}

// LayoutChecker can help us get the align/size status of type conversion
// take one exception case for target into consideration
// on x86 u64 and f64 are only aligned to 4 bytes
impl<'tcx> LayoutChecker<'tcx> {
    pub fn new(rc: RuMorphCtxt<'tcx>, p_env: ParamEnv<'tcx>, f_ty: Ty<'tcx>, t_ty: Ty<'tcx>) -> Self {
        progress_info!("LayoutChecker- from_ty:{:?}, to_ty:{:?}", f_ty, t_ty);
        let (mut f_layout, mut t_layout) = (false, false);
        // rustc_middle::ty::TyCtxt
        let tcx = rc.tcx();
        let (f_ty_, t_ty_) = (get_pointee(f_ty), get_pointee(t_ty));

        let tc = TraitChecker::new(rc, p_env, f_ty_, t_ty_);
        let ty_bnd = tc.get_satisfied_ty();

        // try to handle external type if we can't get type information
        let mut ext_fty_info: u64 = 0;
        let mut ext_tty_info: u64 = 0;
        if let Err(_) = tcx.layout_of(p_env.and(f_ty_)) {
            match get_external(tcx, f_ty_) {
                Some(external_ty) => {
                    ext_fty_info = external_ty;
                },
                None => {},
            }
        }
        if let Err(_) = tcx.layout_of(p_env.and(t_ty_)) {
            match get_external(tcx, t_ty_) {
                Some(external_ty) => {
                    ext_tty_info = external_ty;
                },
                None => {},
            }
        }
        // from_ty_and_layout = rustc_target::abi::TyAndLayout
        // (align_status, size_status)
        let layout_res = if let Ok(from_ty_and_layout) = tcx.layout_of(p_env.and(f_ty_))
            && let Ok(to_ty_and_layout) = tcx.layout_of(p_env.and(t_ty_))
        {
            // in this case, only align_status == Comparison::Less will warn
            f_layout = true;
            t_layout = true;
            let (from_layout, to_layout) = (from_ty_and_layout.layout, to_ty_and_layout.layout);
            let (from_align, to_align) = (from_layout.align(), to_layout.align());
            let (from_size, to_size) = (from_layout.size(), to_layout.size());
            // for align_status
            progress_info!("LayoutChecker- from_align:{}, to_align:{}", from_align.abi.bytes(), to_align.abi.bytes());
            let mut ag_status = if from_align.abi.bytes() < to_align.abi.bytes() {
                Comparison::Less
            } else if from_align.abi.bytes() == to_align.abi.bytes() {
                Comparison::Equal
            } else if from_align.abi.bytes() > to_align.abi.bytes() {
                Comparison::Greater
            } else {
                Comparison::Noidea
            };
            // exception cases for u64 and f64
            match ag_status {
                Comparison::Less | Comparison::Equal => {},
                _ => {
                    // check type kind of from_ty
                    match f_ty_.kind() {
                        TyKind::Uint(uint_ty) => {
                            // u64
                            if (uint_ty.name_str() == "u64") && (4 < to_align.abi.bytes()) {
                                progress_info!("from_align could be :{} on x86", 4);
                                ag_status = Comparison::Less;
                            }
                        },
                        TyKind::Float(float_ty) => {
                            // f64
                            if (float_ty.name_str() == "f64") && (4 < to_align.abi.bytes()) {
                                progress_info!("from_align could be :{} on x86", 4);
                                ag_status = Comparison::Less;
                            }
                        },
                        _ => {},
                    }
                },
            }
            // if to_ty is usize, then take the case away
            // or from_ty is c_void, also take the case away
            if t_ty_.to_string() == "usize" || f_ty_.is_c_void(tcx) {
                ag_status = Comparison::Noidea;
            }
            progress_info!("LayoutChecker- from_size:{}, to_size:{}", from_size.bytes(), to_size.bytes());
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
        } else if let Ok(from_ty_and_layout) = tcx.layout_of(p_env.and(f_ty_)) {
            f_layout = true;
            // we can only identify from_ty's layout
            let from_layout = from_ty_and_layout.layout;
            let from_align = from_layout.align();
            let from_size = from_layout.size();
            let mut ag_status = if let TyKind::Param(_) = t_ty_.kind() {
                // in this case, we can't get layout because t_ty_ is generic type
                // call TraitChecker for help
                if tc.is_ty_arbitrary() {
                    // t_ty_ could be arbitrary types
                    Comparison::Less
                } else {
                    // t_ty_ is limited to trait bound
                    let mut res = Comparison::Noidea;
                    for satisfied_ty in &ty_bnd {
                        let sub_lc = LayoutChecker::new(rc, p_env, f_ty_, *satisfied_ty);
                        let sub_align_status = sub_lc.get_align_status();
                        match sub_align_status {
                            Comparison::Less => {
                                res = Comparison::Less;
                            },
                            _ => {},
                        }
                    }
                    res
                }
            } else {
                // we can't get layout because t_ty_ might be external type
                // apply heuristics
                let mut res = Comparison::Noidea;
                if ext_tty_info != 0 {
                    // we have some type info of to_ty
                    if from_align.abi.bytes() < ext_tty_info {
                        res = Comparison::Less;
                    } else if from_align.abi.bytes() == ext_tty_info {
                        res = Comparison::Equal;
                    } else {
                        res = Comparison::Greater;
                    }
                }
                res
            };

            if f_ty_.is_c_void(tcx) {
                ag_status = Comparison::Noidea;
            }
            
            let sz_status = if from_size.bytes() == 1 {
                Comparison::NoideaL
            } else {
                Comparison::Noidea
            };

            (ag_status, sz_status)
        } else if let Ok(to_ty_and_layout) = tcx.layout_of(p_env.and(t_ty_)) {
            t_layout = true;
            // we can only identify to_ty's layout
            let to_layout = to_ty_and_layout.layout;
            let to_align = to_layout.align();
            let to_size = to_layout.size();
            let mut ag_status = if let TyKind::Param(_) = f_ty_.kind() {
                // f_ty_ is generic type
                // call TraitChecker for help
                if tc.is_ty_arbitrary() {
                    // f_ty_ could be arbitrary types
                    if to_align.abi.bytes() == 1 {
                        Comparison::NoideaG
                    } else {
                        Comparison::Less
                    }
                } else {
                    // f_ty_ is limited to trait bound
                    let mut res = Comparison::Noidea;
                    for satisfied_ty in &ty_bnd {
                        let sub_lc = LayoutChecker::new(rc, p_env, *satisfied_ty, t_ty_);
                        let sub_align_status = sub_lc.get_align_status();
                        match sub_align_status {
                            Comparison::Less => {
                                res = Comparison::Less;
                            },
                            _ => {},
                        }
                    }
                    res
                }
            } else {
                // we can't identify f_ty layout not because genric type
                // try applying heuristics
                let mut res = Comparison::Noidea;
                if ext_fty_info != 0 {
                    if ext_fty_info < to_align.abi.bytes() {
                        res = Comparison::Less;
                    } else if ext_fty_info == to_align.abi.bytes() {
                        res = Comparison::Equal;
                    } else {
                        res = Comparison::Greater;
                    }
                }
                res
            };

            if t_ty_.to_string() == "usize" {
                ag_status = Comparison::Noidea;
            }

            let sz_status = if to_size.bytes() == 1 {
                Comparison::NoideaG
            } else {
                Comparison::Noidea
            };

            (ag_status, sz_status)
        } else {
            (Comparison::Noidea, Comparison::Noidea)
        };

        LayoutChecker { rcx: rc, 
            from_ty: f_ty_, 
            to_ty: t_ty_,
            fty_layout: f_layout,
            tty_layout: t_layout,
            align_status: layout_res.0,
            size_status: layout_res.1,
            ty_bnd: ty_bnd.clone(),
        }
    }

    pub fn get_align_status(&self) -> Comparison {
        self.align_status
    }

    pub fn get_size_status(&self) -> Comparison {
        self.size_status
    }

    pub fn get_ty_bnd(&self) -> HashSet<Ty<'tcx>> {
        self.ty_bnd.clone()
    }

    pub fn get_from_ty(&self) -> Ty<'tcx> {
        self.from_ty
    }

    pub fn get_to_ty(&self) -> Ty<'tcx> {
        self.to_ty
    }

    pub fn is_fty_layout_spec(&self) -> bool {
        self.fty_layout
    }

    pub fn is_tty_layout_spec(&self) -> bool {
        self.tty_layout
    }

    pub fn get_from_ty_name(&self) -> String {
        self.from_ty.to_string()
    }

    pub fn get_to_ty_name(&self) -> String {
        self.to_ty.to_string()
    }

    pub fn is_from_to_primitive(&self) -> (bool, bool) {
        (self.from_ty.is_primitive_ty(), self.to_ty.is_primitive_ty())
    }

    pub fn is_from_to_arr_slice(&self) -> (bool, bool) {
        (self.from_ty.is_array_slice(), self.to_ty.is_array_slice())
    }

    pub fn is_from_to_adt(&self) -> (bool, bool) {
        (self.from_ty.is_adt(), self.to_ty.is_adt())
    }

    pub fn is_from_to_transparent(&self) -> (bool, bool) {
        let is_from = if let TyKind::Adt(def, _) = self.from_ty.kind() {
            def.repr().transparent()
        } else {
            false
        };

        let is_to = if let TyKind::Adt(def, _) = self.to_ty.kind() {
            def.repr().transparent()
        } else {
            false
        };

        (is_from, is_to)
    }

    pub fn is_from_to_c(&self) -> (bool, bool) {
        let is_from = if let TyKind::Adt(def, _) = self.from_ty.kind() {
            def.repr().c()
        } else {
            false
        };

        let is_to = if let TyKind::Adt(def, _) = self.to_ty.kind() {
            def.repr().c()
        } else {
            false
        };

        (is_from, is_to)
    }

    pub fn is_from_to_generic(&self) -> (bool, bool) {
        let is_from_generic = match self.from_ty.kind() {
            TyKind::Param(_) => {
                true
            },
            _ => { false },
        };
        let is_to_generic = match self.to_ty.kind() {
            TyKind::Param(_) => {
                true
            },
            _ => { false },
        };
        (is_from_generic, is_to_generic)
    }

    pub fn is_from_to_foreign(&self) -> (bool, bool) {
        let is_from_foreign = match self.from_ty.kind() {
            TyKind::Foreign(_) => { true },
            _ => { false },
        };
        let is_to_foreign = match self.to_ty.kind() {
            TyKind::Foreign(_) => { true },
            _ => { false },
        };
        (is_from_foreign, is_to_foreign)
    }
}

// get the pointee or wrapped type
fn get_pointee(matched_ty: Ty<'_>) -> Ty<'_> {
    progress_info!("get_pointee: > {:?} as type: {:?}", matched_ty, matched_ty.kind());
    let pointee = if let ty::RawPtr(ty_mut) = matched_ty.kind() {
        get_pointee(ty_mut.ty)
    } else if let ty::Ref(_, referred_ty, _) = matched_ty.kind() {
        get_pointee(*referred_ty)
    } else {
        matched_ty
    };
    pointee
}

// try to get external type
// only try to handle primitive type
fn get_external<'tcx>(tcx: TyCtxt<'tcx>, matched_ty: Ty<'tcx>) -> Option<u64>{
    let ty_symbol = matched_ty.to_string();
    if ty_symbol.contains("bool") || ty_symbol.contains("i8") || ty_symbol.contains("u8") {
        Some(1)
    } else if ty_symbol.contains("i16") || ty_symbol.contains("u16") {
        Some(2)
    } else if ty_symbol.contains("i32") || ty_symbol.contains("u32") || ty_symbol.contains("f32") || ty_symbol.contains("char") {
        Some(4)
    } else if ty_symbol.contains("u64") || ty_symbol.contains("i64") || ty_symbol.contains("f64") {
        Some(8)
    } else if ty_symbol.contains("u128") || ty_symbol.contains("i128") {
        Some(16)
    } else {
        None
    }
}

pub struct ValueChecker<'tcx> {
    rcx: RuMorphCtxt<'tcx>,
    from_ty: Ty<'tcx>,
    to_ty: Ty<'tcx>,
    value_status: Comparison,
}

impl<'tcx> ValueChecker<'tcx> {
    pub fn new(rc: RuMorphCtxt<'tcx>, p_env: ParamEnv<'tcx>, f_ty: Ty<'tcx>, t_ty: Ty<'tcx>) -> Self {
        let tcx = rc.tcx();
        let lc = LayoutChecker::new(rc, p_env, f_ty, t_ty);
        let from_ty = lc.get_from_ty();
        let to_ty = lc.get_to_ty();
        let ty_bnd = lc.get_ty_bnd();
        
        let (from_gen, to_gen) = lc.is_from_to_generic();

        // we only focus on the type conversion between generic and concrete type
        // compare the range of value set to get the size status
        progress_info!("to_ty is c_void? {}", to_ty.is_c_void(tcx));
        let val_status = if (from_gen == true || from_ty.is_c_void(tcx)) && (to_gen == false && !to_ty.is_c_void(tcx)) {
            // generic > concrete
            if ty_bnd.len() == 0 {
                // from_ty could be arbitrary type
                if to_ty.is_bool() || to_ty.is_str() || to_ty.is_char() || to_ty.is_enum() {
                    Comparison::Less
                } else {
                    Comparison::Noidea
                }
            } else {
                let mut res = Comparison::Noidea;
                for satisfied_ty in ty_bnd {
                    if (satisfied_ty.is_numeric() || satisfied_ty.is_str() || satisfied_ty.is_char())
                        && (to_ty.is_bool() || to_ty.is_str() || to_ty.is_char() || to_ty.is_enum()) {
                        res = Comparison::Less;
                    }
                }
                res
            }
        } else if from_gen == false && to_gen == true {
            // concrete > generic
            if ty_bnd.len() == 0 {
                // to_ty could be arbitrary type
                if from_ty.is_numeric() || from_ty.is_str() || from_ty.is_char() {
                    Comparison::Less
                } else {
                    Comparison::Noidea
                }
            } else {
                let mut res = Comparison::Noidea;
                for satisfied_ty in ty_bnd {
                    if (from_ty.is_numeric() || from_ty.is_str() || from_ty.is_char())
                        && (satisfied_ty.is_bool() || satisfied_ty.is_str() || satisfied_ty.is_char() || satisfied_ty.is_enum()) {
                        res = Comparison::Less;
                    }
                }
                res
            }
        } else {
            Comparison::Noidea
        };

        ValueChecker { rcx: rc, 
            from_ty: from_ty, 
            to_ty: to_ty,
            value_status: val_status,
        }
    }

    pub fn get_val_status(&self) -> Comparison {
        self.value_status
    }
}

pub struct TraitChecker<'tcx> {
    rcx: RuMorphCtxt<'tcx>,
    trait_set: HashSet<Ty<'tcx>>,
}

impl<'tcx> TraitChecker<'tcx> {
    pub fn new(rc: RuMorphCtxt<'tcx>, p_env: ParamEnv<'tcx>, from_ty: Ty<'tcx>, to_ty: Ty<'tcx>) -> Self {
        let tcx = rc.tcx();
        let hir = tcx.hir();

        let mut satisfied_ty_set: HashSet<Ty<'tcx>> = HashSet::new();

        for cb in p_env.caller_bounds() {
            // cb: Binder(TraitPredicate(<Self as trait>, ..)
            // Focus on the trait bound applied to our generic parameter

            if let Some(trait_pred) = cb.to_opt_poly_trait_pred() {
                let trait_def_id = trait_pred.def_id();
                let trait_name = tcx.def_path_str(trait_def_id);
                progress_info!("current trait name: ({})", trait_name);

                // for each implementation
                for &impl_id in hir.trait_impls(trait_def_id) {
                    // impl_id: LocalDefId
                    let impl_owner_id = hir.expect_owner(impl_id).def_id();
                    let item = hir.item(ItemId { owner_id: impl_owner_id});
                    if_chain! {
                        if let ItemKind::Impl(impl_item) = item.kind;
                        if impl_item.polarity == ImplPolarity::Positive;
                        if let Some(binder) = tcx.impl_trait_ref(impl_id);
                        then {
                            let trait_ref = binder.skip_binder();
                            let impl_ty = trait_ref.self_ty();
                            match impl_ty.kind() {
                                TyKind::Adt(adt_def, impl_trait_substs) => {
                                    let adt_did = adt_def.did();
                                    let adt_ty = tcx.type_of(adt_did).skip_binder();
                                    for gen_arg in impl_trait_substs.iter() {
                                        if let Some(arg_ty) = gen_arg.as_type() {
                                            // if arg_ty.to_string() == from_ty.to_string() {
                                            //     progress_info!("{} is implemented on from_ty: {:?}", trait_name, from_ty);
                                            //     from_satisfied_ty_set.extend(&satisfied_ty_set);
                                            // }
                                            // if arg_ty.to_string() == to_ty.to_string() {
                                            //     progress_info!("{} is implemented on to_ty: {:?}", trait_name, to_ty);
                                            //     to_satisfied_ty_set.extend(&satisfied_ty_set);
                                            // }
                                            progress_info!("{} is implemented on adt({:?})", trait_name, arg_ty);
                                        }
                                    }
                                },
                                TyKind::Param(p_ty) => {
                                    let param_ty = p_ty.to_ty(tcx);
                                    // if param_ty.to_string() == from_ty.to_string() {
                                    //     progress_info!("{} is implemented on from_ty: {:?}", trait_name, from_ty);
                                    //     from_satisfied_ty_set.extend(&satisfied_ty_set);
                                    // }
                                    // if param_ty.to_string() == to_ty.to_string() {
                                    //     progress_info!("{} is implemented on to_ty: {:?}", trait_name, to_ty);
                                    //     to_satisfied_ty_set.extend(&satisfied_ty_set);
                                    // }
                                    progress_info!("{} is implemented on gen_param({:?})", trait_name, param_ty);
                                },
                                _ => {
                                    progress_info!("{} is implemented on {:?}", trait_name, impl_ty);
                                    satisfied_ty_set.insert(impl_ty);
                                },
                            }
                        }
                    }
                }
            }
        }

        progress_info!("trait bound type set: {:?}", satisfied_ty_set);

        TraitChecker {
            rcx: rc,
            trait_set: satisfied_ty_set.clone(),
        }
    }

    pub fn get_satisfied_ty(&self) -> HashSet<Ty<'tcx>> {
        self.trait_set.clone()
    }

    pub fn is_ty_arbitrary(&self) -> bool {
        self.trait_set.len() == 0
    }
}
