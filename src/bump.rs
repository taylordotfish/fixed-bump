/*
 * Copyright (C) 2021-2022 taylor.fish <contact@taylor.fish>
 *
 * This file is part of fixed-bump.
 *
 * fixed-bump is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * fixed-bump is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with fixed-bump. If not, see <https://www.gnu.org/licenses/>.
 */

use super::generic::{GenericBump, IntoLayout};
#[cfg(any(feature = "allocator_api", feature = "allocator-fallback"))]
use super::{AllocError, Allocator};
use alloc::alloc::Layout;
use core::marker::PhantomData;
use core::mem;
use core::ptr::NonNull;

struct ConstLayout<Size, Align>(PhantomData<fn() -> (Size, Align)>);

impl<Size, Align> Clone for ConstLayout<Size, Align> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Size, Align> Copy for ConstLayout<Size, Align> {}

impl<Size, Align> From<ConstLayout<Size, Align>> for Layout {
    fn from(_: ConstLayout<Size, Align>) -> Self {
        Self::from_size_align(mem::size_of::<Size>(), mem::align_of::<Align>())
            .unwrap()
    }
}

// SAFETY: `<Self as Into<Layout>>::into` forwards to
// `<Layout as From<Self>>:from`, which does not run any code that could call
// methods of any [`GenericBump`].
unsafe impl<Size, Align> IntoLayout for ConstLayout<Size, Align> {}

/// A bump allocator that allocates memory in non-amortized O(1) (constant)
/// time.
///
/// The allocator internally uses fixed-size chunks of memory. The size and
/// alignment of each chunk of memory is determined by the type parameters
/// `Size` and `Align`: the size is [`mem::size_of::<Size>()`] and the
/// alignment is [`mem::align_of::<Align>()`]. The default value of `Align` is
/// `Size`, so you can specify both the size and alignment with a single type
/// parameter.
///
/// A common use of this type, and the most space-efficient way to use it, is
/// to allocate many values of the same type (or at least the same size and
/// alignment). In this case, it may be convenient to specify the chunk size
/// using an array type: to use properly aligned chunks large enough to
/// allocate `n` values of type `T`, pass `[T; n]` as the `Size` parameter,
/// which will also be the `Align` parameter by default.
pub struct Bump<Size, Align = Size>(GenericBump<ConstLayout<Size, Align>>);

impl<Size, Align> Bump<Size, Align> {
    /// Creates a new [`Bump`].
    pub fn new() -> Self {
        Self(GenericBump::new(ConstLayout(PhantomData)))
    }

    /// Tries to allocate memory with a size and alignment matching `layout`.
    ///
    /// Returns a pointer to the memory on success, or [`None`] on failure.
    /// The memory is valid until the [`Bump`] is dropped. Note that the
    /// returned memory could be larger than [`layout.size()`].
    ///
    /// This method is similar to [`Allocator::allocate`], except it returns an
    /// [`Option`] instead of a [`Result`].
    ///
    /// Allocation is guaranteed to succeed, assuming the global allocator
    /// succeeds, if [`layout.size()`] is less than or equal to
    /// [`mem::size_of::<Size>()`] and [`layout.align()`] is less than or equal
    /// to [`mem::align_of::<Align>()`]. See [`Self::can_allocate`].
    ///
    /// [`layout.size()`]: Layout::size
    /// [`layout.align()`]: Layout::align
    /// [`Allocator::allocate`]: alloc::alloc::Allocator::allocate
    pub fn allocate(&self, layout: Layout) -> Option<NonNull<[u8]>> {
        self.0.allocate(layout)
    }

    /// Allocates a value of type `T`.
    ///
    /// The memory is initialized with `value` and a reference to the value is
    /// returned. Note that the value's destructor will not be called
    /// automatically.
    ///
    /// # Panics
    ///
    /// Panics if this allocator cannot allocate memory matching
    /// [`Layout::new::<T>()`] (see [`Self::can_allocate`]). Note that if the
    /// global allocator fails, [`handle_alloc_error`] is called instead of
    /// panicking.
    ///
    /// For an equivalent that doesn't panic or call [`handle_alloc_error`],
    /// see [`Self::try_alloc_value`].
    ///
    /// [`handle_alloc_error`]: alloc::alloc::handle_alloc_error
    #[allow(clippy::mut_from_ref)]
    #[must_use]
    pub fn alloc_value<T>(&self, value: T) -> &mut T {
        self.0.alloc_value(value)
    }

    /// Tries to allocate a value of type `T`.
    ///
    /// If the allocation succeeds, the memory is initialized with `value` and
    /// a reference to the value is returned. Note that the value's destructor
    /// will not be called automatically.
    ///
    /// Allocation succeeds if and only if [`Self::allocate`] is able to
    /// allocate memory matching [`Layout::new::<T>()`]. See [`Self::allocate`]
    /// for details regarding the circumstances in which allocation can fail.
    ///
    /// # Errors
    ///
    /// If allocation fails, <code>[Err]\(value)</code> is returned.
    #[allow(clippy::mut_from_ref)]
    pub fn try_alloc_value<T>(&self, value: T) -> Result<&mut T, T> {
        self.0.try_alloc_value(value)
    }

    /// Returns whether this allocator can allocate memory matching `layout`.
    ///
    /// This is guaranteed to return true if [`layout.size()`] is less than or
    /// equal to [`mem::size_of::<Size>()`] and [`layout.align()`] is less than
    /// or equal to [`mem::align_of::<Align>()`]. It *may* return true if the
    /// alignment is bigger, but never if the size is.
    ///
    /// [`layout.size()`]: Layout::size
    /// [`layout.align()`]: Layout::align
    pub fn can_allocate(&self, layout: Layout) -> bool {
        self.0.can_allocate(layout)
    }
}

impl<Size, Align> Default for Bump<Size, Align> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(any(feature = "allocator_api", feature = "allocator-fallback"))]
#[cfg_attr(
    feature = "doc_cfg",
    doc(cfg(any(
        feature = "allocator_api",
        feature = "allocator-fallback",
    )))
)]
// SAFETY: `Bump::allocate` (when not returning `None`) returns pointers to
// valid memory that matches the provided `Layout`.
//
// `Bump` cannot be cloned, as it does not implement `Clone`. Moving it will
// not invalidate any returned memory, as all returned memory is allocated on
// the heap via the global allocator.
unsafe impl<Size, Align> Allocator for Bump<Size, Align> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.allocate(layout).ok_or(AllocError)
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {
        // No-op: `Bump` deallocates all its memory when dropped.
    }
}

#[cfg(any(doc, doctest))]
/// [`Bump`] cannot implement [`Clone`], as this would make it unsound to
/// implement [`Allocator`](alloc::alloc::Allocator).
///
/// ```
/// use fixed_bump::Bump;
/// struct Test<T = Bump<u8>>(T);
/// ```
///
/// ```compile_fail
/// use fixed_bump::Bump;
/// struct Test<T: Clone = Bump<u8>>(T);
/// ```
mod bump_does_not_impl_clone {}
