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

use alloc::alloc::Layout;
use core::mem;
use core::ptr::{addr_of_mut, NonNull};

struct ChunkHeader {
    prev: Option<Chunk>,
}

// Invariant: `self.0` always points to a valid, initialized, properly aligned
// `ChunkHeader`.
#[repr(transparent)]
pub struct Chunk(NonNull<ChunkHeader>);

impl Chunk {
    pub fn new(layout: Layout, prev: Option<Self>) -> Option<Self> {
        let layout = Self::full_layout(layout);
        assert!(layout.size() > 0);

        // SAFETY: We ensured `layout` has non-zero size above.
        let ptr: NonNull<ChunkHeader> =
            NonNull::new(unsafe { alloc::alloc::alloc(layout) })?.cast();

        // SAFETY: `alloc::alloc::alloc` returns valid, properly aligned
        // memory.
        unsafe {
            addr_of_mut!((*ptr.as_ptr()).prev).write(prev);
        }
        Some(Self(ptr))
    }

    /// The layout of the memory returned by [`Self::storage`], assuming
    /// `layout` was passed to [`Self::new`]. The size and alignment are
    /// guaranteed to be greater than or equal to the size and alignment of
    /// `layout`, respectively.
    pub fn layout(layout: Layout) -> Layout {
        let size = layout.size();
        let align = layout.align().max(mem::align_of::<ChunkHeader>());
        Layout::from_size_align(size, align).unwrap()
    }

    /// The layout of the entire block of memory allocated by [`Self::new`],
    /// assuming `layout` was the layout provided to that function. This is
    /// useful mainly when calling [`handle_alloc_error`].
    ///
    /// [`handle_alloc_error`]: alloc::alloc::handle_alloc_error
    pub fn full_layout(layout: Layout) -> Layout {
        let size =
            layout.size().checked_add(mem::size_of::<ChunkHeader>()).unwrap();
        let align = layout.align().max(mem::align_of::<ChunkHeader>());
        Layout::from_size_align(size, align).unwrap()
    }

    /// Returns a pointer to the start of the storage. It is guaranteed to
    /// match [`Self::layout(cl)`], where `cl` is the layout that was passed to
    /// [`Self::new`]. Note that the memory could be uninitialized.
    pub fn storage(&self) -> NonNull<u8> {
        // SAFETY: `self.0` points to a valid `ChunkHeader`, so adding 1 must
        // result in a pointer within or one byte past the end of the same
        // allocated object.
        let end = unsafe { self.0.as_ptr().add(1) };

        // SAFETY: `self.0` is non-null and points to a valid object, so `end`
        // must also be non-null.
        unsafe { NonNull::new_unchecked(end) }.cast()
    }

    pub fn take_prev(&mut self) -> Option<Self> {
        // SAFETY: `self.0` is always initialized and properly aligned.
        unsafe { &mut (*self.0.as_ptr()).prev }.take()
    }

    /// # Safety
    ///
    /// `layout` must be equal to the layout passed to [`Self::new`].
    pub unsafe fn drop(self, layout: Layout) {
        // SAFETY: `self.0` is always allocated by the global allocator. Caller
        // ensures `layout` is correct.
        unsafe {
            alloc::alloc::dealloc(
                self.0.as_ptr().cast(),
                Self::full_layout(layout),
            );
        }
    }
}
