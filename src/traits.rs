extern crate thread_id;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ThreadId(pub usize);

impl ThreadId {
    pub fn get() -> ThreadId {
        ThreadId(thread_id::get())
    }
}

/// The pair of `(id.into<u32>, id.into<Option<ThreadId>>())` must be unique
/// across all IDs of a particalur `TraceId` type. In other words, either:
///
///   * `id.into::<u32>()` is unique and the `Option<ThreadId>` can always be
///     `None`, or
///
///   * `id.into::<u32>()` is unique within a thread, and `Option<ThreadId>` is
///     `Some` to disambiguate IDs across threads.
pub trait TraceId: Copy + Into<u32> + Into<Option<ThreadId>> {
    fn new_id() -> Self;
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

pub trait TraceSink<T>
    where T: Trace
{
    fn trace_event(&mut self, trace: T, why: Option<T::Id>) -> T::Id;

    fn trace_start(&mut self, trace: T, why: Option<T::Id>) -> T::Id;
    fn trace_stop(&mut self, trace: T);
}
