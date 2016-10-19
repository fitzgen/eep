//! Trait definitions for EEP.

extern crate serde;
extern crate thread_id;

/// A unique identifier for a thread.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ThreadId(pub usize);

impl ThreadId {
    /// Get the current thread's ID.
    pub fn get() -> ThreadId {
        ThreadId(thread_id::get())
    }
}

impl serde::Serialize for ThreadId {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
    {
        serializer.serialize_newtype_struct("ThreadId", self.0)
    }
}

/// A unique identifier for a traced event or start/stop pair.
///
/// The pair of `(id.u32(), id.thread())` must be unique across all IDs of a
/// particalur `TraceId` type. In other words, either:
///
///   * `id.u32()` is unique and `id.thread()` can always return `None`, or
///
///   * `id.u32()` is only unique within a thread, not globally, and
///     `id.thread()` always returns `Some` to disambiguate IDs across threads.
pub trait TraceId: Copy {
    /// Construct a fresh ID.
    fn new_id() -> Self;

    /// Turn this `TraceId` into a `u32`.
    fn u32(&self) -> u32;

    /// Get the ID of the thread upon which this trace was taken.
    fn thread(&self) -> Option<ThreadId>;
}

/// A `Trace` provides metadata about a traced event or operation.
///
/// Typically, this will be an enumeration of all the kinds of events you'd like
/// to trace. For example, a web browser engine might do something like this:
///
/// ```
/// use eep::traits::Trace;
/// use eep::ThreadedTraceId;
/// 
/// #[repr(u32)]
/// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// enum WebBrowserEngineTrace {
///     Compositing       = 0,
///     Painting          = 1,
///     Layout            = 2,
///     ImageDecoding     = 3,
///     DomEvent          = 4,
///     Timer             = 5,
///     GarbageCollection = 6,
///     HttpRequest       = 7,
/// }
/// 
/// impl Trace for WebBrowserEngineTrace {
///     type Id = ThreadedTraceId;
/// 
///     fn label(tag: u32) -> &'static str {
///         match tag {
///             0 => "Compositing",
///             1 => "Painting",
///             2 => "Layout",
///             3 => "ImageDecoding",
///             4 => "DomEvent",
///             5 => "Timer",
///             6 => "GarbageCollection",
///             7 => "HttpRequest",
///             _ => unreachable!(),
///         }
///     }
/// 
///     fn tag(&self) -> u32 {
///         *self as _
///     }
/// }
/// ```
///
/// We require `Copy` because we can't run `Drop` implementations, and we don't
/// want to leak all over.
pub trait Trace: Copy {
    /// The type of ID this `Trace` type uses to distinguish between different
    /// traces.
    type Id: TraceId;

    /// Get the label for the given `Trace::tag()` tag value.
    fn label(tag: u32) -> &'static str;

    /// Get the tag value for this trace instance.
    fn tag(&self) -> u32;
}

/// TODO FITZGEN
pub trait TraceSink<T>
    where T: Trace
{
    /// Trace a one-off event.
    fn trace_event(&mut self, trace: T, why: Option<T::Id>) -> T::Id;

    /// Trace the start of an operation.
    ///
    /// Finish the trace by calling `trace_stop` with the returned ID.
    fn trace_start(&mut self, trace: T, why: Option<T::Id>) -> T::Id;

    /// Trace the end of the operation with the given `id`.
    ///
    /// Start the trace by calling `trace_start` to obtain an ID.
    fn trace_stop(&mut self, id: T::Id, trace: T);
}
