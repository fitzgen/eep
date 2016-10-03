extern crate time;

use std::marker::PhantomData;
use std::mem;
use std::ptr;

use traits::{Trace, TraceKind, TraceSink};

#[derive(Clone, Debug)]
pub struct TraceRingBuffer<T> {
    // The data itself.
    data: Vec<u8>,

    // Where valid data begins.
    begin: usize,

    // The number of bytes in the ring buffer that are valid.
    length: usize,

    phantom: PhantomData<T>,
}

impl<T> Default for TraceRingBuffer<T> {
    fn default() -> TraceRingBuffer<T> {
        Self::new(4096)
    }
}

impl<T> TraceRingBuffer<T> {
    pub fn new(capacity: usize) -> TraceRingBuffer<T> {
        assert!(capacity > TraceEntry::<T>::size());
        TraceRingBuffer {
            data: vec![0; capacity],
            begin: 0,
            length: 0,
            phantom: PhantomData,
        }
    }

    pub fn iter(&self) -> TraceRingBufferIter<T> {
        TraceRingBufferIter(if self.length == 0 {
            TraceRingBufferIterState::Empty
        } else {
            TraceRingBufferIterState::NonEmpty {
                buffer: self,
                idx: self.begin,
            }
        })
    }

    #[inline(always)]
    fn end(&self) -> usize {
        (self.begin + self.length) % self.data.len()
    }

    fn write(&mut self, data: &[u8]) {
        let end = self.end();
        let new_data_len = data.len();
        let capacity = self.data.len();

        if capacity - self.length < TraceEntry::<T>::size() {
            self.begin = (self.begin + TraceEntry::<T>::size()) % capacity;
            self.length -= TraceEntry::<T>::size();
        }

        if end + new_data_len > capacity {
            let middle = capacity - end;
            self.data[end..capacity].copy_from_slice(&data[..middle]);
            self.data[0..new_data_len - middle].copy_from_slice(&data[middle..]);
        } else {
            self.data[end..end + new_data_len].copy_from_slice(data);
        }

        self.length += TraceEntry::<T>::size();
        debug_assert!(self.length <= capacity);
    }
}

impl<T> TraceSink<T> for TraceRingBuffer<T>
    where T: Trace
{
    fn trace(&mut self, trace: T) {
        let entry: TraceEntry<T> = TraceEntry {
            timestamp: NsSinceEpoch::now(),
            tag: trace.tag(),
            kind: trace.kind(),
            phantom: PhantomData,
        };
        let entry: [u8; 13] = unsafe { mem::transmute(entry) };
        self.write(&entry);
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct NsSinceEpoch(pub u64);

impl NsSinceEpoch {
    #[inline(always)]
    pub fn now() -> NsSinceEpoch {
        let timespec = time::get_time();
        let sec = timespec.sec as u64;
        let nsec = timespec.nsec as u64;
        NsSinceEpoch(sec * 1_000_000_000 + nsec)
    }
}

#[repr(packed)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TraceEntry<T> {
    timestamp: NsSinceEpoch,
    tag: u32,
    kind: TraceKind,
    phantom: PhantomData<T>,
}

impl<T> TraceEntry<T>
    where T: Trace
{
    pub fn label(&self) -> &'static str {
        T::label(self.tag)
    }
}

impl<T> TraceEntry<T> {
    pub fn tag(&self) -> u32 {
        self.tag
    }

    pub fn kind(&self) -> TraceKind {
        self.kind
    }

    fn size() -> usize {
        mem::size_of::<Self>()
    }
}

#[derive(Clone, Debug)]
enum TraceRingBufferIterState<'a, T>
    where T: 'a
{
    Empty,
    NonEmpty {
        buffer: &'a TraceRingBuffer<T>,
        idx: usize,
    }
}

#[derive(Clone, Debug)]
pub struct TraceRingBufferIter<'a, T>(TraceRingBufferIterState<'a, T>)
    where T: 'a;

impl<'a, T> Iterator for TraceRingBufferIter<'a, T> {
    type Item = TraceEntry<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let (next_state, result) = match self.0 {
            TraceRingBufferIterState::Empty => return None,
            TraceRingBufferIterState::NonEmpty { ref buffer, idx } => {
                let result = unsafe {
                    if idx + TraceEntry::<T>::size() > buffer.data.len() {
                        let mut temp = [0; 13];
                        let middle = buffer.data.len() - idx;
                        temp[..middle].copy_from_slice(&buffer.data[idx..]);
                        temp[middle..].copy_from_slice(&buffer.data[..TraceEntry::<T>::size() - middle]);
                        Some(mem::transmute(temp))
                    } else {
                        let entry_ptr = buffer.data[idx..].as_ptr() as *const TraceEntry<T>;
                        Some(ptr::read(entry_ptr))
                    }
                };

                let next_idx = (idx + TraceEntry::<T>::size()) % buffer.data.len();
                let next_state = if next_idx == buffer.end() {
                    TraceRingBufferIterState::Empty
                } else {
                    TraceRingBufferIterState::NonEmpty {
                        buffer: buffer,
                        idx: next_idx,
                    }
                };

                (next_state, result)
            }
        };

        mem::replace(&mut self.0, next_state);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_trace::{SimpleTrace, SimpleTraceBuffer};
    use traits::{Trace, TraceKind, TraceSink};

    type SimpleTraceEntry = TraceEntry<SimpleTrace>;

    #[test]
    fn trace_entry_has_right_size() {
        assert_eq!(SimpleTraceEntry::size(), 13);
    }

    #[test]
    fn trace_ring_buffer_no_roll_over() {
        let mut buffer = SimpleTraceBuffer::new(100 * SimpleTraceEntry::size());
        buffer.trace(SimpleTrace::FooEvent);
        buffer.trace(SimpleTrace::StartThing);
        buffer.trace(SimpleTrace::StartAnother);
        buffer.trace(SimpleTrace::FooEvent);
        buffer.trace(SimpleTrace::StopThing);
        buffer.trace(SimpleTrace::StopAnother);

        let mut iter = buffer.iter();

        let entry = iter.next().unwrap();
        assert_eq!(entry.tag(), SimpleTrace::FooEvent.tag());
        assert_eq!(entry.kind(), TraceKind::Signpost);
        assert_eq!(entry.label(), "Foo");

        let entry = iter.next().unwrap();
        assert_eq!(entry.tag(), SimpleTrace::StartThing.tag());
        assert_eq!(entry.kind(), TraceKind::Start);
        assert_eq!(entry.label(), "Thing");

        let entry = iter.next().unwrap();
        assert_eq!(entry.tag(), SimpleTrace::StartAnother.tag());
        assert_eq!(entry.kind(), TraceKind::Start);
        assert_eq!(entry.label(), "Another");

        let entry = iter.next().unwrap();
        assert_eq!(entry.tag(), SimpleTrace::FooEvent.tag());
        assert_eq!(entry.kind(), TraceKind::Signpost);
        assert_eq!(entry.label(), "Foo");

        let entry = iter.next().unwrap();
        assert_eq!(entry.tag(), SimpleTrace::StopThing.tag());
        assert_eq!(entry.kind(), TraceKind::Stop);
        assert_eq!(entry.label(), "Thing");

        let entry = iter.next().unwrap();
        assert_eq!(entry.tag(), SimpleTrace::StopAnother.tag());
        assert_eq!(entry.kind(), TraceKind::Stop);
        assert_eq!(entry.label(), "Another");

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn trace_ring_buffer_with_roll_over() {
        let mut buffer = SimpleTraceBuffer::new(5 * SimpleTraceEntry::size());
        buffer.trace(SimpleTrace::FooEvent);
        buffer.trace(SimpleTrace::StartThing);
        buffer.trace(SimpleTrace::StartAnother);
        buffer.trace(SimpleTrace::FooEvent);
        buffer.trace(SimpleTrace::StopThing);
        buffer.trace(SimpleTrace::StopAnother);

        println!("buffer = {:#?}", buffer);

        let mut iter = buffer.iter();

        let entry = iter.next().unwrap();
        println!("entry = {:#?}", entry);
        assert_eq!(entry.tag(), SimpleTrace::StartThing.tag());
        assert_eq!(entry.kind(), TraceKind::Start);
        assert_eq!(entry.label(), "Thing");

        let entry = iter.next().unwrap();
        println!("entry = {:#?}", entry);
        assert_eq!(entry.tag(), SimpleTrace::StartAnother.tag());
        assert_eq!(entry.kind(), TraceKind::Start);
        assert_eq!(entry.label(), "Another");

        let entry = iter.next().unwrap();
        println!("entry = {:#?}", entry);
        assert_eq!(entry.tag(), SimpleTrace::FooEvent.tag());
        assert_eq!(entry.kind(), TraceKind::Signpost);
        assert_eq!(entry.label(), "Foo");

        let entry = iter.next().unwrap();
        println!("entry = {:#?}", entry);
        assert_eq!(entry.tag(), SimpleTrace::StopThing.tag());
        assert_eq!(entry.kind(), TraceKind::Stop);
        assert_eq!(entry.label(), "Thing");

        let entry = iter.next().unwrap();
        println!("entry = {:#?}", entry);
        assert_eq!(entry.tag(), SimpleTrace::StopAnother.tag());
        assert_eq!(entry.kind(), TraceKind::Stop);
        assert_eq!(entry.label(), "Another");

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn trace_ring_buffer_with_roll_over_and_does_not_divide_evenly() {
        let mut buffer = SimpleTraceBuffer::new(3 * SimpleTraceEntry::size() + 1);
        buffer.trace(SimpleTrace::FooEvent);
        buffer.trace(SimpleTrace::StartThing);
        buffer.trace(SimpleTrace::StartAnother);
        buffer.trace(SimpleTrace::FooEvent);
        buffer.trace(SimpleTrace::StopThing);
        buffer.trace(SimpleTrace::StopAnother);

        println!("buffer = {:#?}", buffer);

        let mut iter = buffer.iter();

        let entry = iter.next().unwrap();
        println!("entry = {:#?}", entry);
        assert_eq!(entry.tag(), SimpleTrace::FooEvent.tag());
        assert_eq!(entry.kind(), TraceKind::Signpost);
        assert_eq!(entry.label(), "Foo");

        let entry = iter.next().unwrap();
        println!("entry = {:#?}", entry);
        assert_eq!(entry.tag(), SimpleTrace::StopThing.tag());
        assert_eq!(entry.kind(), TraceKind::Stop);
        assert_eq!(entry.label(), "Thing");

        let entry = iter.next().unwrap();
        println!("entry = {:#?}", entry);
        assert_eq!(entry.tag(), SimpleTrace::StopAnother.tag());
        assert_eq!(entry.kind(), TraceKind::Stop);
        assert_eq!(entry.label(), "Another");

        assert_eq!(iter.next(), None);
    }
}
