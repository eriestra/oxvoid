//! ox∅ — agent-native WASM apps. No framework.
//!
//! ```rust
//! use oxvoid::*;
//! ```

pub mod signals;
pub mod dom;
pub mod convex;

// Re-export everything at crate root for `use oxvoid::*;`
pub use signals::{signal, memo, effect, batch, ReadSignal, WriteSignal};
pub use dom::{document, el, text_el, on, append, attr, show_when, reactive_attr, reactive_class, input_value, log};
pub use convex::{convex_query, convex_mutate, convex_action, convex_subscribe, ConvexError};
