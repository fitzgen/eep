extern crate signpost;

use std::marker::PhantomData;
use traits::{Trace, TraceKind, TraceSink};

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
    fn trace(&mut self, trace: T) {
        let kind = trace.kind();
        let tag = trace.tag();

        match kind {
            TraceKind::Signpost => signpost::trace(tag, &EMPTY_ARGS),
            TraceKind::Start => signpost::start(tag, &EMPTY_ARGS),
            TraceKind::Stop => signpost::end(tag, &EMPTY_ARGS),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_trace::SimpleTrace;
    use traits::TraceSink;

    #[test]
    fn signpost_sanity_check() {
        Signpost::get().trace(SimpleTrace::FooEvent);
        Signpost::get().trace(SimpleTrace::StartThing);
        Signpost::get().trace(SimpleTrace::StartAnother);
        Signpost::get().trace(SimpleTrace::FooEvent);
        Signpost::get().trace(SimpleTrace::StopAnother);
        Signpost::get().trace(SimpleTrace::StopThing);
    }
}
