extern crate signpost;

use std::marker::PhantomData;
use traits::{Trace, TraceId, TraceSink};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Signpost<T>(PhantomData<T>);

impl<T> Signpost<T> {
    pub fn get() -> Signpost<T> {
        Signpost(PhantomData)
    }
}

static EMPTY_ARGS: [usize; 4] = [0, 0, 0, 0];

impl<T> TraceSink<T> for Signpost<T>
    where T: Trace
{
    fn trace_event(&mut self, trace: T, _why: Option<T::Id>) -> T::Id {
        signpost::trace(trace.tag(), &EMPTY_ARGS);
        T::Id::new_id()
    }

    fn trace_start(&mut self, trace: T, _why: Option<T::Id>) -> T::Id {
        signpost::start(trace.tag(), &EMPTY_ARGS);
        T::Id::new_id()
    }

    fn trace_stop(&mut self, trace: T) {
        signpost::end(trace.tag(), &EMPTY_ARGS);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_trace::SimpleTrace;
    use traits::TraceSink;

    #[test]
    fn signpost_sanity_check() {
        Signpost::get().trace_event(SimpleTrace::FooEvent, None);
        Signpost::get().trace_start(SimpleTrace::OperationThing, None);
        Signpost::get().trace_start(SimpleTrace::OperationAnother, None);
        Signpost::get().trace_event(SimpleTrace::FooEvent, None);
        Signpost::get().trace_stop(SimpleTrace::OperationAnother);
        Signpost::get().trace_stop(SimpleTrace::OperationThing);
    }
}
