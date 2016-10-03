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
        b.iter(|| buffer.trace_event(SimpleTrace::FooEvent, None));
        test::black_box(buffer);
    }

    #[bench]
    fn trace_ring_buffer_large_capacity(b: &mut test::Bencher) {
        let mut buffer = SimpleTraceBuffer::new(2 * 1024 * 1024);
        b.iter(|| buffer.trace_event(SimpleTrace::FooEvent, None));
        test::black_box(buffer);
    }

    #[bench]
    fn trace_ring_buffer_in_mutex(b: &mut test::Bencher) {
        let buffer = Mutex::new(SimpleTraceBuffer::new(2 * 1024 * 1024));
        b.iter(|| {
            let mut buffer = buffer.lock().unwrap();
            buffer.trace_event(SimpleTrace::FooEvent, None);
        });
        test::black_box(buffer);
    }

    mod thread_and_local_id {
        extern crate hydra;
        extern crate test;

        use self::hydra::thread_and_local_id::ThreadAndLocalId;
        use self::hydra::traits::TraceId;

        #[bench]
        fn new_id(b: &mut test::Bencher) {
            ThreadAndLocalId::new_id();
            b.iter(|| test::black_box(ThreadAndLocalId::new_id()));
        }
    }

    mod simple_trace_id {
        extern crate hydra;
        extern crate test;

        use self::hydra::simple_trace::SimpleTraceId;
        use self::hydra::traits::TraceId;

        #[bench]
        fn new_id(b: &mut test::Bencher) {
            SimpleTraceId::new_id();
            b.iter(|| test::black_box(SimpleTraceId::new_id()));
        }
    }

    #[cfg(feature = "signpost")]
    mod signpost {
        extern crate hydra;
        extern crate test;

        use self::hydra::signpost::Signpost;
        use self::hydra::simple_trace::SimpleTrace;
        use self::hydra::traits::TraceSink;

        #[bench]
        fn signpost_event(b: &mut test::Bencher) {
            b.iter(|| {
                Signpost::get().trace_event(SimpleTrace::FooEvent, None);
            });
        }

        #[bench]
        fn signpost_start_stop(b: &mut test::Bencher) {
            b.iter(|| {
                Signpost::get().trace_start(SimpleTrace::OperationThing, None);
                Signpost::get().trace_stop(SimpleTrace::OperationThing);
            });
        }
    }
}
