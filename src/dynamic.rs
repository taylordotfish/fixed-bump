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
use core::ptr::NonNull;

// SAFETY: Trivially, `<Layout as Into<Layout>>::into` cannot call any methods
// of any [`GenericBump`] as it is a no-op.
unsafe impl IntoLayout for Layout {}

/// Like [`Bump`], but uses chunk size and alignment values provided at runtime
/// rather than compile time.
///
/// Instead of passing `Size` and `Align` type parameters, [`Self::new`]
/// accepts a [`Layout`]. Otherwise, this type behaves identically to [`Bump`].
///
/// [`Bump`]: crate::Bump
pub struct DynamicBump(GenericBump<Layout>);

impl DynamicBump {
    /// Creates a new [`DynamicBump`]. `layout` specifies the size and
    /// alignment of the chunks allocated internally by the allocator.
    pub fn new(layout: Layout) -> Self {
        Self(GenericBump::new(layout))
    }

    /// The layout passed to [`Self::new`].
    pub fn layout(&self) -> Layout {
        self.0.layout()
    }

    /// Tries to allocate memory with a size and alignment matching `layout`.
    ///
    /// Returns a pointer to the memory on success, or [`None`] on failure.
    /// The memory is valid until the [`DynamicBump`] is dropped. Note that the
    /// returned memory could be larger than [`layout.size()`].
    ///
    /// This method is similar to [`Allocator::allocate`], except it returns an
    /// [`Option`] instead of a [`Result`].
    ///
    /// Allocation is guaranteed to succeed, assuming the global allocator
    /// succeeds, if [`layout.size()`] is less than or equal to
    /// <code>[self.layout()].[size()]</code> and [`layout.align()`] is less
    /// than or equal to <code>[self.layout()].[align()]</code>. See
    /// [`Self::can_allocate`].
    ///
    /// [`layout.size()`]: Layout::size
    /// [`layout.align()`]: Layout::align
    /// [`Allocator::allocate`]: alloc::alloc::Allocator::allocate
    /// [self.layout()]: Self::layout
    /// [size()]: Layout::size
    /// [align()]: Layout::align
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
    /// equal to <code>[self.layout()].[size()]</code> and [`layout.align()`]
    /// is less than or equal to <code>[self.layout()].[align()]</code>. It
    /// *may* return true if the alignment is bigger, but never if the size is.
    ///
    /// [`layout.size()`]: Layout::size
    /// [`layout.align()`]: Layout::align
    /// [self.layout()]: Self::layout
    /// [size()]: Layout::size
    /// [align()]: Layout::align
    pub fn can_allocate(&self, layout: Layout) -> bool {
        self.0.can_allocate(layout)
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
// SAFETY: `DynamicBump::allocate` (when not returning `None`) returns pointers
// to valid memory that matches the provided `Layout`.
//
// `DynamicBump` cannot be cloned, as it does not implement `Clone`. Moving it
// will not invalidate any returned memory, as all returned memory is allocated
// on the heap via the global allocator.
unsafe impl Allocator for DynamicBump {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.allocate(layout).ok_or(AllocError)
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {
        // No-op: `DynamicBump` deallocates all its memory when dropped.
    }
}

#[cfg(any(doc, doctest))]
/// [`DynamicBump`] cannot implement [`Clone`], as this would make it unsound
/// to implement [`Allocator`](alloc::alloc::Allocator).
///
/// ```
/// use fixed_bump::DynamicBump;
/// struct Test<T = DynamicBump>(T);
/// ```
///
/// ```compile_fail
/// use fixed_bump::DynamicBump;
/// struct Test<T: Clone = DynamicBump>(T);
/// ```
mod dynamic_bump_does_not_impl_clone {}
