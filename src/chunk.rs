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
use alloc::boxed::Box;
use core::mem::{self, MaybeUninit};
use core::ptr::{addr_of_mut, NonNull};

#[repr(C)]
struct ChunkMemory<Size, Align> {
    _align: [Align; 0],
    item: MaybeUninit<Size>,
    prev: Option<Chunk<Size, Align>>,
}

// Invariant: `self.0` always points to a valid, initialized, properly aligned
// `ChunkMemory`.
#[repr(transparent)]
pub struct Chunk<Size, Align>(NonNull<ChunkMemory<Size, Align>>);

impl<Size, Align> Chunk<Size, Align> {
    pub const SIZE: usize = mem::size_of::<Size>();
    pub const ALIGN: usize = mem::align_of::<ChunkMemory<Size, Align>>();
    pub const LAYOUT: Layout = Layout::new::<ChunkMemory<Size, Align>>();

    pub fn new(prev: Option<Self>) -> Option<Self> {
        assert!(Self::LAYOUT.size() > 0);

        // SAFETY: We ensured `Self::LAYOUT` has non-zero size above.
        let ptr: NonNull<ChunkMemory<Size, Align>> =
            NonNull::new(unsafe { alloc::alloc::alloc(Self::LAYOUT) })?.cast();

        // SAFETY: `alloc::alloc::alloc` returns valid, properly aligned
        // memory.
        unsafe {
            addr_of_mut!((*ptr.as_ptr()).prev).write(prev);
        }
        Some(Self(ptr))
    }

    /// Returns a pointer to the start of the storage. It is guaranteed to be
    /// at least [`Self::SIZE`] bytes in size. Note that the memory could be
    /// uninitialized.
    pub fn storage(&mut self) -> NonNull<u8> {
        self.0.cast()
    }

    pub fn into_prev(self) -> Option<Self> {
        // SAFETY: `self.0` is always initialized and properly aligned.
        unsafe { &mut (*self.0.as_ptr()).prev }.take()
    }
}

impl<Size, Align> Drop for Chunk<Size, Align> {
    fn drop(&mut self) {
        // SAFETY: `self.0` was allocated by `alloc::alloc::alloc` and can thus
        // be turned into a `Box`.
        drop(unsafe { Box::from_raw(self.0.as_ptr()) });
    }
}
