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
        let mut buffer = SimpleTraceBuffer::new(8);
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
}
