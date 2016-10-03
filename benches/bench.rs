#![feature(test)]

#[cfg(test)]
mod benches {
    extern crate test;
    extern crate hydra;

    use self::hydra::traits::TraceSink;
    use self::hydra::simple_trace::{SimpleTrace, SimpleTraceBuffer};
    use std::sync::Mutex;

    #[bench]
    fn trace_ring_buffer_small_capacity(b: &mut test::Bencher) {
        let mut buffer = SimpleTraceBuffer::new(27);
        b.iter(|| buffer.trace(SimpleTrace::FooEvent));
        test::black_box(buffer);
    }

    #[bench]
    fn trace_ring_buffer_large_capacity(b: &mut test::Bencher) {
        let mut buffer = SimpleTraceBuffer::new(2 * 1024 * 1024);
        b.iter(|| buffer.trace(SimpleTrace::FooEvent));
        test::black_box(buffer);
    }

    #[bench]
    fn trace_ring_buffer_in_mutex(b: &mut test::Bencher) {
        let buffer = Mutex::new(SimpleTraceBuffer::new(2 * 1024 * 1024));
        b.iter(|| {
            let mut buffer = buffer.lock().unwrap();
            buffer.trace(SimpleTrace::FooEvent)
        });
        test::black_box(buffer);
    }

    mod thread_local {
        extern crate test;

        use std::cell::RefCell;

        struct ThreadLocalTraceId(pub u32);

        thread_local!(static TRACE_ID_COUNTER: RefCell<u32> = RefCell::new(0));

        fn new_trace_id() -> ThreadLocalTraceId {
            TRACE_ID_COUNTER.with(|c| {
                let mut c = c.borrow_mut();
                let id = *c;
                *c = c.wrapping_add(1);
                ThreadLocalTraceId(id)
            })
        }

        #[bench]
        fn increment_id_counter(b: &mut test::Bencher) {
            new_trace_id();
            b.iter(|| test::black_box(new_trace_id()));
        }
    }

    mod atomics {
        extern crate test;

        use std::sync::atomic::{ATOMIC_USIZE_INIT, AtomicUsize, Ordering};

        struct GlobalTraceId(pub u32);

        static TRACE_ID_COUNTER: AtomicUsize = ATOMIC_USIZE_INIT;

        fn new_trace_id() -> GlobalTraceId {
            let id = TRACE_ID_COUNTER.fetch_add(1, Ordering::AcqRel);
            GlobalTraceId((id % (::std::u32::MAX as usize)) as u32)
        }

        #[bench]
        fn increment_id_counter(b: &mut test::Bencher) {
            new_trace_id();
            b.iter(|| test::black_box(new_trace_id()));
        }
    }
}
