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

/// Returns a pointer matching `layout` if `layout.align()` is less than or
/// equal to `Chunk::<Size, Align>::align()`. Otherwise, the returned pointer
/// will *not* necessarily be aligned to `layout.align()`.
///
/// # Safety
///
/// * `offset` must be less than or equal to `Chunk::<Size, Align>::size()`.
/// * `offset` must be greater than or equal to `layout.size()`.
unsafe fn allocate_in_chunk<Size, Align>(
    layout: Layout,
    chunk: &mut Chunk<Size, Align>,
    offset: &mut usize,
) -> NonNull<[u8]> {
    // Round down to a multiple of `layout.align()`. Note that this subtraction
    // will not underflow due to this function's safety requirements.
    let new_offset = (*offset - layout.size()) & !(layout.align() - 1);
    let storage: NonNull<u8> = chunk.storage();

    // SAFETY: `new_offset` must be less than or equal to `offset`, and the
    // caller guarantees that `offset` is less than or equal to
    // `Chunk::<Size, Align>::size()`.
    let start = unsafe { storage.as_ptr().add(new_offset) };
    let len = *offset - new_offset;
    *offset = new_offset;

    // Note: Although not required by `slice_from_raw_parts_mut`, the
    // returned slice points to valid (but possibly uninitialized) memory:
    // there must be at least `len` bytes after `start` within the same
    // allocated object due to the subtraction of `layout.size()` we
    // performed earlier.
    let ptr = ptr::slice_from_raw_parts_mut(start, len);
    // SAFETY: `storage` is non-null, so `ptr` must also be non-null.
    unsafe { NonNull::new_unchecked(ptr) }
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

        if let Some(chunk) = self.chunk.as_mut() {
            if self.offset >= layout.size() {
                // SAFETY: `self.offset` is always less than or equal to
                // `Self::chunk_size()` due to this type's invariants, and we
                // just ensured that `self.offset` is at least `layout.size()`.
                return Some(unsafe {
                    allocate_in_chunk(layout, chunk, &mut self.offset)
                });
            }
        }

        if layout.size() > Self::chunk_size() {
            return None;
        }

        let chunk = self.chunk.take();
        let chunk = self.chunk.insert(Chunk::new(chunk)?);
        self.offset = Self::chunk_size();
        // SAFETY: `self.offset` is always less than or equal to
        // `Self::chunk_size()` due to this type's invariants, and we ensured
        // that `Self::chunk_size()` (the current value of `self.offset`) is
        // at least `layout.size()` above.
        Some(unsafe { allocate_in_chunk(layout, chunk, &mut self.offset) })
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
