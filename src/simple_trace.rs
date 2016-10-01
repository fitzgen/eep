use traits::{Trace, TraceKind};
use trace_ring_buffer::TraceRingBuffer;

/// A simple `Trace` type for tests, benches, and to serve as an example.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SimpleTrace {
    FooEvent,

    StartThing,
    StopThing,

    StartAnother,
    StopAnother,
}

impl Trace for SimpleTrace {
    fn label(tag: u32) -> &'static str {
        match tag {
            0 => "Foo",
            1 => "Thing",
            2 => "Another",
            _ => unreachable!(),
        }
    }

    fn tag(&self) -> u32 {
        match *self {
            SimpleTrace::FooEvent => 0,
            SimpleTrace::StartThing | SimpleTrace::StopThing => 1,
            SimpleTrace::StartAnother | SimpleTrace::StopAnother => 2,
        }
    }

    fn kind(&self) -> TraceKind {
        match *self {
            SimpleTrace::FooEvent => TraceKind::Signpost,
            SimpleTrace::StartThing | SimpleTrace::StartAnother => TraceKind::Start,
            SimpleTrace::StopThing | SimpleTrace::StopAnother => TraceKind::Stop,
        }
    }
}

pub type SimpleTraceBuffer = TraceRingBuffer<SimpleTrace>;
