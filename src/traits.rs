#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TraceKind {
    Signpost = 0x0,
    Start = 0x1,
    Stop = 0x2,
}

/// TODO FITZGEN
///
/// We require `Copy` because we can't run `Drop` implementations, and we don't
/// want to leak all over.
pub trait Trace: Copy {
    fn label(u32) -> &'static str;
    fn tag(&self) -> u32;
    fn kind(&self) -> TraceKind;
}

pub trait TraceSink<T> where T: Trace {
    fn trace(&mut self, trace: T);
}
