use std::cell::RefCell;
use traits::{ThreadId, TraceId};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ThreadAndLocalId(pub ThreadId, pub u32);

thread_local!(static LOCAL_TRACE_ID_COUNTER: RefCell<u32> = RefCell::new(0));

impl TraceId for ThreadAndLocalId {
    fn new_id() -> Self {
        let local_id = LOCAL_TRACE_ID_COUNTER.with(|c| {
            let mut c = c.borrow_mut();
            let local_id = *c;
            *c = c.wrapping_add(1);
            local_id
        });

        ThreadAndLocalId(ThreadId::get(), local_id)
    }

    fn u32(&self) -> u32 {
        self.1
    }

    fn thread(&self) -> Option<ThreadId> {
        Some(self.0)
    }
}
