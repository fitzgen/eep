use std::cell::RefCell;
use traits::{ThreadId, TraceId};

/// A `TraceId` implementation that is a pair of a thread ID and a thread-local
/// monotonically increasing ID counter.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ThreadedTraceId(pub ThreadId, pub u32);

thread_local!(static LOCAL_TRACE_ID_COUNTER: RefCell<u32> = RefCell::new(0));

impl TraceId for ThreadedTraceId {
    fn new_id() -> Self {
        let local_id = LOCAL_TRACE_ID_COUNTER.with(|c| {
            let mut c = c.borrow_mut();
            let local_id = *c;
            *c = c.wrapping_add(1);
            local_id
        });

        ThreadedTraceId(ThreadId::get(), local_id)
    }

    fn u32(&self) -> u32 {
        self.1
    }

    fn thread(&self) -> Option<ThreadId> {
        Some(self.0)
    }
}
