#![feature(test)]

#[cfg(test)]
mod benches {
    mod ring_buffer {
        extern crate test;
        extern crate z;

        use self::z::simple_trace::{SimpleTrace, SimpleTraceBuffer};
        use self::z::traits::TraceSink;

        #[bench]
        fn small_capacity(b: &mut test::Bencher) {
            let mut buffer = SimpleTraceBuffer::new(100);
            b.iter(|| buffer.trace_event(SimpleTrace::FooEvent, None));
            test::black_box(buffer);
        }

        #[bench]
        fn large_capacity(b: &mut test::Bencher) {
            let mut buffer = SimpleTraceBuffer::new(2 * 1024 * 1024);
            b.iter(|| buffer.trace_event(SimpleTrace::FooEvent, None));
            test::black_box(buffer);
        }

        #[bench]
        fn in_mutex(b: &mut test::Bencher) {
            use std::sync::Mutex;

            let buffer = Mutex::new(SimpleTraceBuffer::new(2 * 1024 * 1024));
            b.iter(|| {
                let mut buffer = buffer.lock().unwrap();
                buffer.trace_event(SimpleTrace::FooEvent, None);
            });
            test::black_box(buffer);
        }
    }

    mod thread_and_local_id {
        extern crate z;
        extern crate test;

        use self::z::thread_and_local_id::ThreadAndLocalId;
        use self::z::traits::TraceId;

        #[bench]
        fn new_id(b: &mut test::Bencher) {
            ThreadAndLocalId::new_id();
            b.iter(|| test::black_box(ThreadAndLocalId::new_id()));
        }
    }

    mod simple_trace_id {
        extern crate z;
        extern crate test;

        use self::z::simple_trace::SimpleTraceId;
        use self::z::traits::TraceId;

        #[bench]
        fn new_id(b: &mut test::Bencher) {
            SimpleTraceId::new_id();
            b.iter(|| test::black_box(SimpleTraceId::new_id()));
        }
    }

    #[cfg(feature = "signpost")]
    mod signpost {
        extern crate z;
        extern crate test;

        use self::z::signpost::Signpost;
        use self::z::simple_trace::SimpleTrace;
        use self::z::traits::TraceSink;

        #[bench]
        fn signpost_event(b: &mut test::Bencher) {
            b.iter(|| {
                Signpost::get().trace_event(SimpleTrace::FooEvent, None);
            });
        }

        #[bench]
        fn signpost_start_stop(b: &mut test::Bencher) {
            b.iter(|| {
                let id = Signpost::get().trace_start(SimpleTrace::OperationThing, None);
                Signpost::get().trace_stop(id, SimpleTrace::OperationThing);
            });
        }
    }
}
