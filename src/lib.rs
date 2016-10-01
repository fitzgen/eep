#![deny(missing_debug_implementations)]

extern crate leb128;

pub mod simple_trace;
pub mod traits;
pub mod trace_ring_buffer;

// pub struct NsSinceEpoch(pub u64);
// pub struct ThreadId(pub usize);
// pub struct ThreadLocalTraceId(pub u32);

// thread_local!(static TRACE_ID_COUNTER: RefCell<u32> = RefCell::new(0));

// pub fn new_trace_id() -> ThreadLocalTraceId {
//     TRACE_ID_COUNTER.with(|c| {
//         let mut c = c.borrow_mut();
//         let id = *c;
//         *c = c.wrapping_add(1);
//         ThreadLocalTraceId(id)
//     })
// }
