#![feature(test)]

#[cfg(test)]
mod benches {
    extern crate test;
    extern crate hydra;

    use self::hydra::traits::TraceSink;
    use self::hydra::simple_trace::{SimpleTrace, SimpleTraceBuffer};

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
}
