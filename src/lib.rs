//! # EEP: EEP Embeddable Profiler
//!
//! TODO FITZGEN: more docs here

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]

// extern crate leb128;

pub mod ring_buffer;

#[cfg(feature = "signpost")]
pub mod signpost;

pub mod simple_trace;

pub mod sink_combinators;

mod threaded_trace_id;
pub use threaded_trace_id::ThreadedTraceId;

pub mod traits;
