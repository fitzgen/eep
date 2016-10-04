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

    /// Get the ID of the thread upon which this ID's trace was taken.
    fn thread(&self) -> Option<ThreadId>;
}

/// TODO FITZGEN
///
/// We require `Copy` because we can't run `Drop` implementations, and we don't
/// want to leak all over.
pub trait Trace: Copy {
    type Id: TraceId;

    fn label(u32) -> &'static str;
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
