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

use super::chunk::Chunk;
use alloc::alloc::Layout;
use core::ptr;
use core::ptr::NonNull;

// Invariants:
//
// * `offset` is less than or equal to `Self::chunk_size()`.
pub struct BumpInner<Size, Align> {
    chunk: Option<Chunk<Size, Align>>,
    offset: usize,
}

impl<Size, Align> BumpInner<Size, Align> {
    pub fn new() -> Self {
        Self {
            chunk: None,
            offset: 0,
        }
    }

    fn chunk_size() -> usize {
        Chunk::<Size, Align>::size()
    }

    fn chunk_align() -> usize {
        Chunk::<Size, Align>::align()
    }

    /// Returns a pointer to memory matching `layout`, or `None` if the
    /// allocation fails.
    pub fn allocate(&mut self, layout: Layout) -> Option<NonNull<[u8]>> {
        if layout.align() > Self::chunk_align() {
            return None;
        }

        let (chunk, offset) = match (
            self.chunk.as_mut(),
            self.offset.checked_sub(layout.size()),
        ) {
            (Some(chunk), Some(offset)) => (chunk, offset),
            _ => return self.allocate_in_new_chunk(layout),
        };

        // Round down to a multiple of `layout.align()`.
        let offset = offset & !(layout.align() - 1);
        let storage: NonNull<u8> = chunk.storage();

        // SAFETY: `offset` must be less than or equal to `self.offset`
        // (specifically, at least `layout.size()` less, due to the checked
        // subtraction we performed), and `self.offset` is always less than or
        // equal to `Self::chunk_size()`, which is less than or equal to the
        // size of the memory pointed to by `storage`.
        let start = unsafe { storage.as_ptr().add(offset) };
        let len = self.offset - offset;
        self.offset = offset;

        // Note: Although not required by `slice_from_raw_parts_mut`, the
        // returned slice points to valid (but possibly uninitialized) memory:
        // there must be at least `len` bytes after `start` within the same
        // allocated object due to the subtraction of `layout.size()` we
        // performed earlier.
        let ptr = ptr::slice_from_raw_parts_mut(start, len);
        // SAFETY: `storage` is non-null, so `ptr` must also be non-null.
        Some(unsafe { NonNull::new_unchecked(ptr) })
    }

    /// If `layout.align()` is less than or equal to `Self::chunk_align()`,
    /// returns a pointer matching `layout`, or `None` if the allocation fails.
    ///
    /// If `layout.align()` is greater than `Self::chunk_align()`, the
    /// allocation may succeed, but the pointer will *not* be aligned to
    /// `layout.align()`.
    fn allocate_in_new_chunk(
        &mut self,
        layout: Layout,
    ) -> Option<NonNull<[u8]>> {
        let offset = Self::chunk_size().checked_sub(layout.size())?;
        let mut chunk = Chunk::new(self.chunk.take())?;

        // Round down to a multiple of `layout.align()`.
        let offset = offset & !(layout.align() - 1);
        let storage: NonNull<u8> = chunk.storage();

        // SAFETY: `offset` must be less than or equal to `Self::chunk_size()`
        // (specifically, at least `layout.size()` less, due to the checked
        // subtraction we performed).
        let start = unsafe { storage.as_ptr().add(offset) };
        let len = Self::chunk_size() - offset;
        self.offset = offset;
        self.chunk = Some(chunk);

        // Note: Although not required by `slice_from_raw_parts_mut`, the
        // returned slice points to valid (but possibly uninitialized) memory:
        // there must be at least `len` bytes after `start` within the same
        // allocated object due to the subtraction of `layout.size()` we
        // performed earlier.
        let ptr = ptr::slice_from_raw_parts_mut(start, len);
        // SAFETY: `storage` is non-null, so `ptr` must also be non-null.
        Some(unsafe { NonNull::new_unchecked(ptr) })
    }
}

impl<Size, Align> Drop for BumpInner<Size, Align> {
    fn drop(&mut self) {
        let mut prev = self.chunk.take();
        while let Some(chunk) = prev {
            prev = chunk.into_prev();
        }
    }
}
