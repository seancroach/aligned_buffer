#![doc = include_str!("README.md")]

use core::mem;

/// A buffer of `SIZE` bytes that is aligned to `ALIGN` bytes.
///
/// # Safety
///
/// This trait is unsafe because any type that implements [`AlignedBuffer`]
/// **MUST** ensure that the buffer is properly aligned to `ALIGN` bytes.
/// Additionally, the pointer returned by [`AlignedBuffer::as_ptr`] must be
/// valid for reads of `SIZE` bytes; the pointer returned by
/// [`AlignedBuffer::as_mut_ptr`] must be valid for reads and writes of `SIZE`
/// bytes. Another guarantee is that `SIZE` is a multiple of `ALIGN` and that
/// `ALIGN` is a power of two. Naturally, this means that `SIZE` cannot be zero.
pub unsafe trait AlignedBuffer<const SIZE: usize, const ALIGN: usize> {
    // TODO: When it becomes stable, make the constant generic parameters into
    // associated constants so a type can only implement `AlignedBuffer` once.

    /// Creates a new buffer with every byte initialized to zero.
    #[must_use]
    fn new() -> Self;

    /// Creates a new buffer with every byte initialized to `byte`.
    #[must_use]
    fn splat(byte: u8) -> Self;

    /// Creates a new buffer from an array of bytes.
    #[must_use]
    fn from_bytes(bytes: [u8; SIZE]) -> Self;

    /// Returns an unsafe pointer to the buffer.
    ///
    /// The caller must ensure that the buffer outlives the pointer this
    /// function returns, or else it will end up pointing to garbage.
    #[must_use]
    fn as_ptr(&self) -> *const u8;

    /// Returns an unsafe mutable pointer to the buffer.
    ///
    /// The caller must ensure that the buffer outlives the pointer this
    /// function returns, or else it will end up pointing to garbage.
    #[must_use]
    fn as_mut_ptr(&mut self) -> *mut u8;

    /// Returns a slice of the buffer.
    #[must_use]
    #[inline]
    fn as_slice(&self) -> &[u8; SIZE] {
        let ptr = self.as_ptr().cast::<[u8; SIZE]>();
        // SAFETY: The pointer is valid for reads up to `SIZE` bytes.
        unsafe { &*ptr }
    }

    /// Returns a mutable slice of the buffer.
    #[must_use]
    #[inline]
    fn as_mut_slice(&mut self) -> &mut [u8; SIZE] {
        let ptr = self.as_mut_ptr().cast::<[u8; SIZE]>();
        // SAFETY: The pointer is valid for reads and writes up to `SIZE` bytes.
        unsafe { &mut *ptr }
    }

    /// Copies the contents of `slice` into the buffer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the length of `slice` is in the interval
    /// `[0, SIZE]`. Also, the caller must ensure that the buffer and the slice
    /// do not overlap.
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    unsafe fn fill_from_start(&mut self, slice: &[u8]) {
        debug_assert!(slice.len() <= SIZE, "slice length exceeds buffer size");

        let src = slice.as_ptr();
        let dst = self.as_mut_ptr();

        // SAFETY: For this trait to be implemented, `as_mut_ptr` must be valid
        // for writes up to `SIZE` bytes. The pointers are not allowed to
        // overlap.
        unsafe { dst.copy_from_nonoverlapping(src, slice.len()) };
    }

    /// Copies the contents of `slice` into buffer, such that the last byte of
    /// `slice` is at the last byte of the buffer.
    ///
    /// Note that this doesn't mean that slice is copied to the buffer reversed.
    /// Instead, only the last `slice.len()` bytes of the buffer are modified.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the length of `slice` is in the interval
    /// `[0, SIZE]`. Also, the caller must ensure that the buffer and the slice
    /// do not overlap.
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    unsafe fn fill_from_end(&mut self, slice: &[u8]) {
        debug_assert!(slice.len() <= SIZE, "slice length exceeds buffer size");

        let src = slice.as_ptr();
        // SAFETY: The caller must ensure that `slice.len()` is less than or
        // equal to `SIZE`.
        let dst_offset = unsafe { SIZE.unchecked_sub(slice.len()) };
        // SAFETY: `dst_offset` is less than or equal to `SIZE`. The pointer is
        // valid for writes up to `SIZE` bytes.
        let dst = unsafe { self.as_mut_ptr().add(dst_offset) };

        // SAFETY: For this trait to be implemented, `as_mut_ptr` must be valid
        // for writes up to `SIZE` bytes. The pointers are not allowed to
        // overlap.
        unsafe { dst.copy_from_nonoverlapping(src, slice.len()) };
    }
}

/// This function asserts that the layout of `T` is `SIZE` bytes and aligned to
/// `ALIGN` bytes.
///
/// # Panics
///
/// This function panics if the size of `T` is not `SIZE` bytes, if the
/// alignment of `T` is not `ALIGN` bytes, if `SIZE` is not a multiple of
/// `ALIGN`, if `ALIGN` is not a power of two, or if `SIZE` is zero.
#[inline]
#[track_caller]
pub fn assert_layout<const SIZE: usize, const ALIGN: usize, T>()
where
    T: Sized,
{
    assert_eq!(mem::size_of::<T>(), SIZE, "invalid size");
    assert_eq!(mem::align_of::<T>(), ALIGN, "invalid alignment");
    assert_eq!(SIZE % ALIGN, 0, "size is not a multiple of alignment");
    assert!(ALIGN.is_power_of_two(), "alignment is not a power of two");
    assert_ne!(SIZE, 0, "size cannot be zero");
}
