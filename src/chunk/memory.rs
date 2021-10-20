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

use super::Chunk;
use alloc::alloc::Layout;
use core::marker::PhantomData;
use core::mem;
use core::mem::MaybeUninit;
use core::ptr::NonNull;

// Invariants:
//
// * `self.0` points to memory allocated by the global allocator with the
//   layout `Self::layout()`.
//
// * The memory pointed to by `self.0` is not accessible except through a
//   single instance of this type. (In particular, there must not be two
//   instances that refer to the same memory.)
#[repr(transparent)]
pub struct ChunkMemory<Size, Align>(
    NonNull<u8>,
    PhantomData<fn() -> (Size, Align)>,
);

impl<Size, Align> ChunkMemory<Size, Align> {
    pub fn new() -> Option<Self> {
        let layout = Self::layout();
        assert!(layout.size() > 0);
        Some(Self(
            // SAFETY: We ensured that `layout.size()` is non-zero above. This
            // should always be true, because the layout is at least as large
            // as `Self::prev_size()`.
            NonNull::new(unsafe { alloc::alloc::alloc(layout) })?,
            PhantomData,
        ))
    }

    pub fn prev(&mut self) -> &mut MaybeUninit<Option<Chunk<Size, Align>>> {
        // SAFETY: `Self::prev_offset()` is never larger than the size of the
        // memory pointed to by `self.0`.
        let ptr = unsafe { self.0.as_ptr().add(Self::prev_offset()) };
        // SAFETY: `ptr` points to valid (but possibly uninitialized) memory
        // with proper alignment and enough space for an `Option<Chunk>`, so we
        // can cast to a `*mut MaybeUninit` and dereference.
        #[allow(clippy::cast_ptr_alignment)]
        unsafe {
            &mut *ptr.cast::<MaybeUninit<_>>()
        }
    }

    /// Returns a pointer to the start of the storage. It is guaranteed to be
    /// at least [`Self::storage_size()`] bytes in size. Note that the memory
    /// could be uninitialized.
    pub fn storage(&self) -> NonNull<u8> {
        // SAFETY: `Self::storage_offset()` is never larger than the size
        // of the memory pointed to by `self.0`.
        let ptr = unsafe { self.0.as_ptr().add(Self::storage_offset()) };
        // SAFETY: `self.0` is non-null, so `ptr` must also be non-null.
        unsafe { NonNull::new_unchecked(ptr) }
    }

    /// The size of the memory returned by `Self::storage`. This is simply
    /// equal to [`mem::size_of::<Size>()`](mem::size_of).
    pub fn storage_size() -> usize {
        mem::size_of::<Size>()
    }

    /// The alignment of the memory returned by `Self::storage`. This is
    /// guaranteed to be at least [`mem::align_of::<Align>()`](mem::align_of).
    pub fn storage_align() -> usize {
        Self::storage_min_align().max(Self::prev_min_align())
    }

    fn layout() -> Layout {
        Layout::from_size_align(
            Self::prev_size().checked_add(Self::storage_size()).unwrap(),
            // `Self::storage_align()` is always at least
            // `Self::prev_min_align()`.
            Self::storage_align(),
        )
        .unwrap()
    }

    fn prev_offset() -> usize {
        if Self::prev_min_align() >= Self::storage_min_align() {
            0
        } else {
            Self::storage_size()
        }
    }

    fn storage_offset() -> usize {
        if Self::prev_offset() == 0 {
            Self::prev_size()
        } else {
            0
        }
    }

    fn prev_size() -> usize {
        mem::size_of::<Option<Chunk<Size, Align>>>()
    }

    fn prev_min_align() -> usize {
        mem::align_of::<Option<Chunk<Size, Align>>>()
    }

    fn storage_min_align() -> usize {
        mem::align_of::<Align>()
    }
}

impl<Size, Align> Drop for ChunkMemory<Size, Align> {
    fn drop(&mut self) {
        // SAFETY: `self.0` always points to memory allocated by the global
        // allocator with the layout `Self::layout()`.
        unsafe {
            alloc::alloc::dealloc(self.0.as_ptr(), Self::layout());
        }
    }
}

// SAFETY: `ChunkMemory` represents an owned region of memory (in particular,
// no two instances of `ChunkMemory` will point to the same region of memory),
// so it can be sent to another thread.
unsafe impl<Size, Align> Send for ChunkMemory<Size, Align> {}
