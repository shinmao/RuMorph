use rustc_middle::ty::{Instance, InstanceDef, TyCtxt};
use rustc_span::{CharPos, Span};

use termcolor::{Buffer, Color, ColorSpec, WriteColor};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct ColorEventId(usize);

#[derive(Clone)]
struct ColorEvent {
    // Some(color) for start, None for clear
    color: Option<Color>,
    line: usize,
    col: CharPos,
    id: ColorEventId,
}


pub struct ColorSpan<'tcx> {
    tcx: TyCtxt<'tcx>,
    main_span: Span,
    main_span_start: rustc_span::Loc,
    main_span_end: rustc_span::Loc,
    id_counter: usize,
    sub_span_events: Vec<ColorEvent>,
}