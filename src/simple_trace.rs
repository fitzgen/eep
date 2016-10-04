use std::sync::atomic::{ATOMIC_USIZE_INIT, AtomicUsize, Ordering};
use traits::{ThreadId, Trace, TraceId};
use ring_buffer::RingBuffer;

/// A simple `Trace` type for tests, benches, and to serve as an example.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SimpleTrace {
    FooEvent,
    OperationThing,
    OperationAnother,
}

impl Trace for SimpleTrace {
    type Id = SimpleTraceId;

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
            SimpleTrace::OperationThing => 1,
            SimpleTrace::OperationAnother => 2,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SimpleTraceId(pub u32);

static SIMPLE_TRACE_ID_COUNTER: AtomicUsize = ATOMIC_USIZE_INIT;

impl TraceId for SimpleTraceId {
    fn new_id() -> Self {
        let id = SIMPLE_TRACE_ID_COUNTER.fetch_add(1, Ordering::AcqRel);
        SimpleTraceId((id % (::std::u32::MAX as usize)) as u32)
    }

    fn u32(&self) -> u32 {
        self.0
    }

    fn thread(&self) -> Option<ThreadId> {
        None
    }
}

pub type SimpleTraceBuffer = RingBuffer<SimpleTrace>;

#[cfg(test)]
mod tests {
    use std::mem;

    #[test]
    fn usize_is_big_enough() {
        // Pretty safe assumption here.
        assert!(mem::size_of::<usize>() >= mem::size_of::<u32>());
    }
}
