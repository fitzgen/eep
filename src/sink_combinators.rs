//! Combinators for building up complex `TraceSink` implementations from simple
//! parts.

use std::sync::atomic::{AtomicBool, Ordering};
use traits::{Trace, TraceId, TraceSink};

/// A wrapper around another `TraceSink` that adds dynamically enabling or
/// disabling tracing.
///
/// When a `ToggleSink` is disabled, its `TraceSink` implementation methods are
/// no-ops. When it is enabled, it passes the traces through to the underlying
/// sink.
#[derive(Debug)]
pub struct ToggleSink<S> {
    enabled: AtomicBool,
    sink: S,
}

impl<S> ToggleSink<S> {
    /// Construct a new `ToggleSink` with the given `sink` that is initially
    /// enabled.
    pub fn new_enabled(sink: S) -> ToggleSink<S> {
        ToggleSink {
            enabled: AtomicBool::new(true),
            sink: sink,
        }
    }

    /// Construct a new `ToggleSink` with the given `sink` that is initially
    /// disabled.
    pub fn new_disabled(sink: S) -> ToggleSink<S> {
        ToggleSink {
            enabled: AtomicBool::new(false),
            sink: sink,
        }
    }

    /// Enable this `ToggleSink`.
    pub fn enable(&self) {
        self.enabled.store(true, Ordering::Release);
    }

    /// Disable this `ToggleSink`.
    pub fn disable(&self) {
        self.enabled.store(false, Ordering::Release);
    }

    /// Return `true` if this `ToggleSink` is enabled, `false` otherwise.
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
    }
}

impl<S> AsRef<S> for ToggleSink<S> {
    fn as_ref(&self) -> &S {
        &self.sink
    }
}

impl<S> AsMut<S> for ToggleSink<S> {
    fn as_mut(&mut self) -> &mut S {
        &mut self.sink
    }
}

impl<S, T> TraceSink<T> for ToggleSink<S>
    where S: TraceSink<T>,
          T: Trace
{
    fn trace_event(&mut self, trace: T, why: Option<T::Id>) -> T::Id {
        if self.is_enabled() {
            self.sink.trace_event(trace, why)
        } else {
            T::Id::new_id()
        }
    }

    fn trace_start(&mut self, trace: T, why: Option<T::Id>) -> T::Id {
        if self.is_enabled() {
            self.sink.trace_start(trace, why)
        } else {
            T::Id::new_id()
        }
    }

    fn trace_stop(&mut self, id: T::Id, trace: T) {
        if self.is_enabled() {
            self.sink.trace_stop(id, trace);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_trace::{SimpleTrace, SimpleTraceBuffer};
    use traits::TraceSink;

    #[test]
    fn does_not_trace_when_disabled() {
        let mut sink = ToggleSink::new_enabled(SimpleTraceBuffer::default());
        assert!(sink.is_enabled());

        sink.disable();
        assert!(!sink.is_enabled());

        sink.trace_event(SimpleTrace::FooEvent, None);

        assert_eq!(sink.as_ref().iter().next(), None);
    }

    #[test]
    fn does_trace_when_enabled() {
        let mut sink = ToggleSink::new_disabled(SimpleTraceBuffer::default());
        assert!(!sink.is_enabled());

        sink.enable();
        assert!(sink.is_enabled());

        sink.trace_event(SimpleTrace::FooEvent, None);

        assert!(sink.as_ref().iter().next().is_some());
    }
}
