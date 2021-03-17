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

use core::mem::MaybeUninit;
use core::ptr::NonNull;

mod memory;
use memory::ChunkMemory;

// Invariants:
//
// * `self.0.prev()` is initialized.
#[repr(transparent)]
pub struct Chunk<Size, Align>(ChunkMemory<Size, Align>);

impl<Size, Align> Chunk<Size, Align> {
    pub fn new(prev: Option<Self>) -> Option<Self> {
        let mut memory = ChunkMemory::new()?;
        *memory.prev() = MaybeUninit::new(prev);
        Some(Self(memory))
    }

    /// Returns a pointer to the start of the storage. It is guaranteed to be
    /// at least [`Self::size()`] bytes in size. Note that the memory could be
    /// uninitialized.
    pub fn storage(&mut self) -> NonNull<u8> {
        self.0.storage()
    }

    pub fn size() -> usize {
        ChunkMemory::<Size, Align>::storage_size()
    }

    pub fn align() -> usize {
        ChunkMemory::<Size, Align>::storage_align()
    }

    pub fn into_prev(self) -> Option<Self> {
        let mut memory = self.0;
        // SAFETY: `memory.prev()` must be initialized due to this type's
        // invariants. (In particular, `Self::new` initializes it.)
        unsafe { memory.prev().as_ptr().read() }
    }
}
