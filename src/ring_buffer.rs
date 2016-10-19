extern crate serde;
extern crate time;

use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::mem;

use traits::{ThreadId, Trace, TraceId, TraceSink};

#[derive(Clone, Debug)]
pub struct RingBuffer<T> {
    // The data itself.
    data: Vec<u8>,

    // Where valid data begins.
    begin: usize,

    // The number of bytes in the ring buffer that are valid.
    length: usize,

    phantom: PhantomData<T>,
}

impl<T> Default for RingBuffer<T> {
    fn default() -> RingBuffer<T> {
        Self::new(4096)
    }
}

impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> RingBuffer<T> {
        assert!(capacity > Entry::<T>::size());
        RingBuffer {
            data: vec![0; capacity],
            begin: 0,
            length: 0,
            phantom: PhantomData,
        }
    }

    pub fn iter(&self) -> RingBufferIter<T> {
        RingBufferIter(if self.length == 0 {
            RingBufferIterState::Empty
        } else {
            RingBufferIterState::NonEmpty {
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

        if capacity - self.length < Entry::<T>::size() {
            self.begin = (self.begin + Entry::<T>::size()) % capacity;
            self.length -= Entry::<T>::size();
        }

        if end + new_data_len > capacity {
            let middle = capacity - end;
            self.data[end..capacity].copy_from_slice(&data[..middle]);
            self.data[0..new_data_len - middle].copy_from_slice(&data[middle..]);
        } else {
            self.data[end..end + new_data_len].copy_from_slice(data);
        }

        self.length += Entry::<T>::size();
        debug_assert!(self.length <= capacity);
    }
}

impl<T> TraceSink<T> for RingBuffer<T>
    where T: Trace
{
    fn trace_event(&mut self, trace: T, why: Option<T::Id>) -> T::Id {
        let id = T::Id::new_id();

        let entry: Entry<T> = Entry {
            why: why.map(|id| (id.thread(), id.u32())),
            thread: id.thread(),
            timestamp: NsSinceEpoch::now(),
            id: id.u32(),
            tag: trace.tag(),
            kind: TraceKind::Event,
            phantom: PhantomData,
        };

        let entry: [u8; 65] = unsafe { mem::transmute(entry) };
        self.write(&entry);

        id
    }

    fn trace_start(&mut self, trace: T, why: Option<T::Id>) -> T::Id {
        let id = T::Id::new_id();

        let entry: Entry<T> = Entry {
            why: why.map(|id| (id.thread(), id.u32())),
            thread: id.thread(),
            timestamp: NsSinceEpoch::now(),
            id: id.u32(),
            tag: trace.tag(),
            kind: TraceKind::Start,
            phantom: PhantomData,
        };

        let entry: [u8; 65] = unsafe { mem::transmute(entry) };
        self.write(&entry);

        id
    }

    fn trace_stop(&mut self, id: T::Id, trace: T) {
        let entry: Entry<T> = Entry {
            why: None,
            thread: id.thread(),
            timestamp: NsSinceEpoch::now(),
            id: id.u32(),
            tag: trace.tag(),
            kind: TraceKind::Stop,
            phantom: PhantomData,
        };

        let entry: [u8; 65] = unsafe { mem::transmute(entry) };
        self.write(&entry);
    }
}

impl<T> serde::Serialize for RingBuffer<T>
    where T: Trace
{
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
    {
        // Build up all the entries' labels in a map keyed by T's tag, serialize
        // that, and then serialize the individual entries.

        struct Entries<'a, T>(&'a RingBuffer<T>) where T: 'a + Trace;

        impl<'a, T> serde::Serialize for Entries<'a, T>
            where T: Trace
        {
            fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
                where S: serde::Serializer
            {
                let mut state = try!(serializer.serialize_seq(None));
                for entry in self.0.iter() {
                    try!(serializer.serialize_seq_elt(&mut state, entry));
                }
                serializer.serialize_seq_end(state)
            }
        }

        let mut labels = HashMap::new();
        for entry in self.iter() {
            let tag = entry.tag();
            // Turn the key into a string to support JSON.
            labels.insert(format!("{}", tag), T::label(tag));
        }

        let mut state = try!(serializer.serialize_struct("RingBuffer", 2));
        try!(serializer.serialize_struct_elt(&mut state, "labels", labels));
        try!(serializer.serialize_struct_elt(&mut state, "entries", Entries(self)));
        serializer.serialize_struct_end(state)
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

impl serde::Serialize for NsSinceEpoch {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
    {
        serializer.serialize_newtype_struct("NsSinceEpoch", self.0)
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TraceKind {
    Event = 0x0,
    Start = 0x1,
    Stop = 0x2,
}

impl serde::Serialize for TraceKind {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
    {
        match *self {
            TraceKind::Event => serializer.serialize_unit_variant("TraceKind", 0, "Event"),
            TraceKind::Start => serializer.serialize_unit_variant("TraceKind", 1, "Start"),
            TraceKind::Stop => serializer.serialize_unit_variant("TraceKind", 2, "Stop"),
        }
    }
}

#[repr(packed)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Entry<T> {
    why: Option<(Option<ThreadId>, u32)>,
    thread: Option<ThreadId>,
    id: u32,
    tag: u32,
    timestamp: NsSinceEpoch,
    kind: TraceKind,
    phantom: PhantomData<T>,
}

impl<T> Entry<T>
    where T: Trace
{
    pub fn label(&self) -> &'static str {
        T::label(self.tag)
    }
}

impl<T> Entry<T> {
    pub fn tag(&self) -> u32 {
        self.tag
    }

    pub fn kind(&self) -> TraceKind {
        self.kind
    }

    pub fn timestamp(&self) -> NsSinceEpoch {
        self.timestamp
    }

    pub fn thread(&self) -> Option<ThreadId> {
        self.thread
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn why(&self) -> Option<(Option<ThreadId>, u32)> {
        self.why
    }

    fn size() -> usize {
        mem::size_of::<Self>()
    }
}

impl<T> serde::Serialize for Entry<T> {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
    {
        let mut state = try!(serializer.serialize_struct("Entry", 6));
        try!(serializer.serialize_struct_elt(&mut state, "why", &self.why));
        try!(serializer.serialize_struct_elt(&mut state, "thread", &self.thread));
        try!(serializer.serialize_struct_elt(&mut state, "id", self.id));
        try!(serializer.serialize_struct_elt(&mut state, "tag", self.tag));
        try!(serializer.serialize_struct_elt(&mut state, "timestamp", self.timestamp));
        try!(serializer.serialize_struct_elt(&mut state, "kind", self.kind));
        serializer.serialize_struct_end(state)
    }
}

#[derive(Clone, Debug)]
enum RingBufferIterState<'a, T>
    where T: 'a
{
    Empty,
    NonEmpty {
        buffer: &'a RingBuffer<T>,
        idx: usize,
    },
}

#[derive(Clone, Debug)]
pub struct RingBufferIter<'a, T>(RingBufferIterState<'a, T>) where T: 'a;

impl<'a, T> Iterator for RingBufferIter<'a, T> {
    type Item = Entry<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let (next_state, result) = match self.0 {
            RingBufferIterState::Empty => return None,
            RingBufferIterState::NonEmpty { ref buffer, idx } => {
                let entry: Entry<T> = unsafe {
                    if idx + Entry::<T>::size() > buffer.data.len() {
                        // The entry is split across the end of the buffer and
                        // wraps around to the front of it again.
                        let entry: UnsafeCell<Entry<T>> = mem::uninitialized();
                        let mut entry: [u8; 65] = mem::transmute(entry);
                        let middle = buffer.data.len() - idx;
                        entry[..middle].copy_from_slice(&buffer.data[idx..]);
                        entry[middle..]
                            .copy_from_slice(&buffer.data[..Entry::<T>::size() - middle]);
                        mem::transmute(entry)
                    } else {
                        // The entry is in one contiguous block in the buffer.
                        let entry: UnsafeCell<Entry<T>> = mem::uninitialized();
                        let mut entry: [u8; 65] = mem::transmute(entry);
                        entry.copy_from_slice(&buffer.data[idx..idx + Entry::<T>::size()]);
                        mem::transmute(entry)
                    }
                };

                let next_idx = (idx + Entry::<T>::size()) % buffer.data.len();
                let next_state = if next_idx == buffer.end() {
                    RingBufferIterState::Empty
                } else {
                    RingBufferIterState::NonEmpty {
                        buffer: buffer,
                        idx: next_idx,
                    }
                };

                (next_state, Some(entry))
            }
        };

        mem::replace(&mut self.0, next_state);
        result
    }
}

#[cfg(test)]
mod tests {
    extern crate serde_json;

    use super::*;
    use simple_trace::{SimpleTrace, SimpleTraceBuffer};
    use traits::{Trace, TraceSink};

    type SimpleEntry = Entry<SimpleTrace>;

    #[test]
    fn trace_entry_has_right_size() {
        assert_eq!(SimpleEntry::size(), 65);
    }

    #[test]
    fn no_roll_over() {
        let mut buffer = SimpleTraceBuffer::new(100 * SimpleEntry::size());
        buffer.trace_event(SimpleTrace::FooEvent, None);
        let thing_id = buffer.trace_start(SimpleTrace::OperationThing, None);
        let another_id = buffer.trace_start(SimpleTrace::OperationAnother, None);
        buffer.trace_event(SimpleTrace::FooEvent, None);
        buffer.trace_stop(thing_id, SimpleTrace::OperationThing);
        buffer.trace_stop(another_id, SimpleTrace::OperationAnother);

        let mut iter = buffer.iter();

        let entry = iter.next().unwrap();
        assert_eq!(entry.tag(), SimpleTrace::FooEvent.tag());
        assert_eq!(entry.kind(), TraceKind::Event);
        assert_eq!(entry.label(), "Foo");

        let entry = iter.next().unwrap();
        assert_eq!(entry.tag(), SimpleTrace::OperationThing.tag());
        assert_eq!(entry.kind(), TraceKind::Start);
        assert_eq!(entry.label(), "Thing");

        let entry = iter.next().unwrap();
        assert_eq!(entry.tag(), SimpleTrace::OperationAnother.tag());
        assert_eq!(entry.kind(), TraceKind::Start);
        assert_eq!(entry.label(), "Another");

        let entry = iter.next().unwrap();
        assert_eq!(entry.tag(), SimpleTrace::FooEvent.tag());
        assert_eq!(entry.kind(), TraceKind::Event);
        assert_eq!(entry.label(), "Foo");

        let entry = iter.next().unwrap();
        assert_eq!(entry.tag(), SimpleTrace::OperationThing.tag());
        assert_eq!(entry.kind(), TraceKind::Stop);
        assert_eq!(entry.label(), "Thing");

        let entry = iter.next().unwrap();
        assert_eq!(entry.tag(), SimpleTrace::OperationAnother.tag());
        assert_eq!(entry.kind(), TraceKind::Stop);
        assert_eq!(entry.label(), "Another");

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn with_roll_over() {
        let mut buffer = SimpleTraceBuffer::new(5 * SimpleEntry::size());
        buffer.trace_event(SimpleTrace::FooEvent, None);
        let thing_id = buffer.trace_start(SimpleTrace::OperationThing, None);
        let another_id = buffer.trace_start(SimpleTrace::OperationAnother, None);
        buffer.trace_event(SimpleTrace::FooEvent, None);
        buffer.trace_stop(thing_id, SimpleTrace::OperationThing);
        buffer.trace_stop(another_id, SimpleTrace::OperationAnother);

        println!("buffer = {:#?}", buffer);

        let mut iter = buffer.iter();

        let entry = iter.next().unwrap();
        println!("entry = {:#?}", entry);
        assert_eq!(entry.tag(), SimpleTrace::OperationThing.tag());
        assert_eq!(entry.kind(), TraceKind::Start);
        assert_eq!(entry.label(), "Thing");

        let entry = iter.next().unwrap();
        println!("entry = {:#?}", entry);
        assert_eq!(entry.tag(), SimpleTrace::OperationAnother.tag());
        assert_eq!(entry.kind(), TraceKind::Start);
        assert_eq!(entry.label(), "Another");

        let entry = iter.next().unwrap();
        println!("entry = {:#?}", entry);
        assert_eq!(entry.tag(), SimpleTrace::FooEvent.tag());
        assert_eq!(entry.kind(), TraceKind::Event);
        assert_eq!(entry.label(), "Foo");

        let entry = iter.next().unwrap();
        println!("entry = {:#?}", entry);
        assert_eq!(entry.tag(), SimpleTrace::OperationThing.tag());
        assert_eq!(entry.kind(), TraceKind::Stop);
        assert_eq!(entry.label(), "Thing");

        let entry = iter.next().unwrap();
        println!("entry = {:#?}", entry);
        assert_eq!(entry.tag(), SimpleTrace::OperationAnother.tag());
        assert_eq!(entry.kind(), TraceKind::Stop);
        assert_eq!(entry.label(), "Another");

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn with_roll_over_and_does_not_divide_evenly() {
        let mut buffer = SimpleTraceBuffer::new(3 * SimpleEntry::size() + 1);
        buffer.trace_event(SimpleTrace::FooEvent, None);
        let thing_id = buffer.trace_start(SimpleTrace::OperationThing, None);
        let another_id = buffer.trace_start(SimpleTrace::OperationAnother, None);
        buffer.trace_event(SimpleTrace::FooEvent, None);
        buffer.trace_stop(thing_id, SimpleTrace::OperationThing);
        buffer.trace_stop(another_id, SimpleTrace::OperationAnother);

        println!("buffer = {:#?}", buffer);

        let mut iter = buffer.iter();

        let entry = iter.next().unwrap();
        println!("entry = {:#?}", entry);
        assert_eq!(entry.tag(), SimpleTrace::FooEvent.tag());
        assert_eq!(entry.kind(), TraceKind::Event);
        assert_eq!(entry.label(), "Foo");

        let entry = iter.next().unwrap();
        println!("entry = {:#?}", entry);
        assert_eq!(entry.tag(), SimpleTrace::OperationThing.tag());
        assert_eq!(entry.kind(), TraceKind::Stop);
        assert_eq!(entry.label(), "Thing");

        let entry = iter.next().unwrap();
        println!("entry = {:#?}", entry);
        assert_eq!(entry.tag(), SimpleTrace::OperationAnother.tag());
        assert_eq!(entry.kind(), TraceKind::Stop);
        assert_eq!(entry.label(), "Another");

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn why() {
        let mut buffer = SimpleTraceBuffer::default();

        let parent = buffer.trace_event(SimpleTrace::FooEvent, None);
        let child1 = buffer.trace_start(SimpleTrace::OperationThing, Some(parent));
        buffer.trace_stop(child1, SimpleTrace::OperationThing);
        let child2 = buffer.trace_start(SimpleTrace::OperationThing, None);
        buffer.trace_stop(child2, SimpleTrace::OperationThing);

        let mut iter = buffer.iter();

        let entry = iter.next().unwrap();
        println!("entry = {:#?}", entry);
        assert_eq!(entry.tag(), SimpleTrace::FooEvent.tag());
        assert_eq!(entry.kind(), TraceKind::Event);
        assert_eq!(entry.label(), "Foo");
        let parent = (entry.thread(), entry.id());

        let entry = iter.next().unwrap();
        println!("entry = {:#?}", entry);
        assert_eq!(entry.tag(), SimpleTrace::OperationThing.tag());
        assert_eq!(entry.kind(), TraceKind::Start);
        assert_eq!(entry.label(), "Thing");
        assert_eq!(entry.why(), Some(parent));

        let entry = iter.next().unwrap();
        println!("entry = {:#?}", entry);
        assert_eq!(entry.tag(), SimpleTrace::OperationThing.tag());
        assert_eq!(entry.kind(), TraceKind::Stop);
        assert_eq!(entry.label(), "Thing");
        assert_eq!(entry.why(), None);

        let entry = iter.next().unwrap();
        println!("entry = {:#?}", entry);
        assert_eq!(entry.tag(), SimpleTrace::OperationThing.tag());
        assert_eq!(entry.kind(), TraceKind::Start);
        assert_eq!(entry.label(), "Thing");
        assert_eq!(entry.why(), None);

        let entry = iter.next().unwrap();
        println!("entry = {:#?}", entry);
        assert_eq!(entry.tag(), SimpleTrace::OperationThing.tag());
        assert_eq!(entry.kind(), TraceKind::Stop);
        assert_eq!(entry.label(), "Thing");
        assert_eq!(entry.why(), None);
    }

    #[test]
    fn serialize_entry() {
        let mut buffer = SimpleTraceBuffer::new(2 * SimpleEntry::size());
        buffer.trace_event(SimpleTrace::FooEvent, None);
        let entry = buffer.iter().next().unwrap();

        let serialized = serde_json::to_string_pretty(&entry).expect("should serialize OK");

        println!("");
        println!("serialized = {}", serialized);
    }

    #[test]
    fn serialize_ring_buffer() {
        let mut buffer = SimpleTraceBuffer::new(10 * SimpleEntry::size());

        for _ in 0..10 {
            let event = buffer.trace_event(SimpleTrace::FooEvent, None);
            let child1 = buffer.trace_start(SimpleTrace::OperationThing, Some(event));
            let child2 = buffer.trace_start(SimpleTrace::OperationAnother, None);
            buffer.trace_stop(child2, SimpleTrace::OperationThing);
            buffer.trace_stop(child1, SimpleTrace::OperationAnother);
        }

        let serialized = serde_json::to_string_pretty(&buffer).expect("should serialize OK");

        println!("");
        println!("serialized = {}", serialized);
    }
}
