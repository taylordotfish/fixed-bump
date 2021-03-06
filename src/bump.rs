/*
 * Copyright (C) 2021 taylor.fish <contact@taylor.fish>
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

use super::inner::BumpInner;
use alloc::alloc::Layout;
#[cfg(feature = "allocator_api")]
use alloc::alloc::{AllocError, Allocator};
use core::cell::UnsafeCell;
use core::ptr::NonNull;

/// A bump allocator that allocates memory in non-amortized O(1) (constant)
/// time.
///
/// The allocator internally uses fixed-size chunks of memory. The size and
/// alignment of each chunk of memory is determined by the type parameters
/// `Size` and `Align`: the size is [`mem::size_of::<Size>()`][size_of] and the
/// alignment is [`mem::align_of::<Align>()`][align_of]. The default value of
/// `Align` is `Size`, so you can specify both the size and alignment with a
/// single type parameter.
///
/// A common use of this type, and the most space-efficient way to use it, is
/// to allocate many values of the same type (or at least the same size and
/// alignment). In this case, it may be convenient to specify the chunk size
/// using an array type: to use properly aligned chunks large enough to
/// allocate `n` values of type `T`, pass `[T; n]` as the `Size` parameter,
/// which will also be the `Align` parameter by default.
///
/// [size_of]: core::mem::size_of
/// [align_of]: core::mem::align_of
pub struct Bump<Size, Align = Size>(UnsafeCell<BumpInner<Size, Align>>);

impl<Size, Align> Bump<Size, Align> {
    /// Creates a new [`Bump`].
    pub fn new() -> Self {
        Self(UnsafeCell::new(BumpInner::new()))
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
    /// [`mem::size_of::<Size>()`][size_of] and [`layout.align()`] is less than
    /// or equal to [`mem::align_of::<Align>()`][align_of].
    ///
    /// Allocation may fail, but is not guaranteed to fail, if
    /// [`layout.align()`] is greater than
    /// [`mem::align_of::<Align>()`][align_of]. Allocation *is* guaranteed to
    /// fail if [`layout.size()`] is greater than
    /// [`mem::size_of::<Size>()`][size_of].
    ///
    /// [`layout.size()`]: Layout::size
    /// [`layout.align()`]: Layout::align
    /// [size_of]: core::mem::size_of
    /// [align_of]: core::mem::align_of
    /// [`Allocator::allocate`]: alloc::alloc::Allocator::allocate
    pub fn allocate(&self, layout: Layout) -> Option<NonNull<[u8]>> {
        // SAFETY: `BumpInner::alloc` does not run any code that could possibly
        // call any methods of `Self`, which ensures that we do not borrow the
        // data in the `UnsafeCell` multiple times concurrently.
        unsafe { &mut *self.0.get() }.allocate(layout)
    }

    /// Allocates a value of type `T`.
    ///
    /// The memory is initialized with `value` and a reference to the value is
    /// returned. Note that the value's destructor will not be called
    /// automatically.
    ///
    /// # Panics
    ///
    /// Panics if [`Self::allocate`] is not able to allocate memory matching
    /// [`Layout::new::<T>()`](Layout::new). See [`Self::allocate`] for
    /// details regarding the circumstances in which allocation can fail.
    ///
    /// For a non-panicking equivalent, see [`Self::try_alloc_value`].
    #[allow(clippy::mut_from_ref)]
    pub fn alloc_value<T>(&self, value: T) -> &mut T {
        self.try_alloc_value(value).ok().expect("allocation failed")
    }

    #[allow(clippy::doc_markdown)]
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
        let memory = if let Some(memory) = self.allocate(Layout::new::<T>()) {
            memory.cast::<T>()
        } else {
            return Err(value);
        };
        // SAFETY: `Self::allocate`, when not returning `None`, is guaranteed
        // to return valid memory that matches the provided layout. Thus, we
        // can store a value of type `T` in it.
        unsafe {
            memory.as_ptr().write(value);
        }
        // SAFETY: We just initialized `memory` with `value`.
        Ok(unsafe { &mut *memory.as_ptr() })
    }
}

impl<Size, Align> Default for Bump<Size, Align> {
    fn default() -> Self {
        Self::new()
    }
}

// SAFETY: `Bump::allocate` (when not returning `None`) returns pointers to
// valid memory that matches the provided `Layout`.
//
// `Bump` cannot be cloned, as it does not implement `Clone`. Moving it will
// not invalidate any returned memory, as all returned memory is allocated on
// the heap via the global allocator.
#[cfg(feature = "allocator_api")]
unsafe impl<Size, Align> Allocator for Bump<Size, Align> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        (*self).allocate(layout).ok_or(AllocError)
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {
        // No-op: `Bump` deallocates all its memory when dropped.
    }
}

#[cfg(doctest)]
/// [`Bump`] cannot implement [`Clone`], as this would make it unsound to
/// implement [`Allocator`](core::alloc::alloc::Allocator).
///
/// ```compile_fail
/// use fixed_bump::Bump;
/// struct Test<T: Clone = Bump<u8>>(T);
/// ```
mod bump_does_not_impl_clone {}
