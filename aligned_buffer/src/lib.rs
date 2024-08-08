//! TODO

#[doc(inline)]
pub use aligned_buffer_internals::{assert_layout, AlignedBuffer};
#[doc(inline)]
pub use aligned_buffer_macro::aligned_buffer;

/// TODO
pub mod prelude {
    #[doc(inline)]
    pub use super::{aligned_buffer, AlignedBuffer};
}
