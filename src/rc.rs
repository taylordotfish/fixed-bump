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

use super::Bump;
use alloc::rc::Rc;
use core::ops::Deref;
#[cfg(any(feature = "allocator_api", feature = "allocator-fallback"))]
use {
    super::{AllocError, Allocator},
    alloc::alloc::Layout,
    core::ptr::NonNull,
};

#[allow(clippy::doc_markdown)]
/// A wrapper around <code>[Rc]<[Bump]<T>></code>.
///
/// This type exists mainly so that [`Allocator`](alloc::alloc::Allocator)
/// can be implemented for it.
pub struct RcBump<Size, Align = Size>(pub Rc<Bump<Size, Align>>);

impl<Size, Align> RcBump<Size, Align> {
    #[allow(clippy::doc_markdown)]
    /// Creates a new [`RcBump`]. This simply returns
    /// <code>[RcBump]\([Rc::new]\([Bump::new]\())</code>.
    pub fn new() -> Self {
        Self(Rc::new(Bump::new()))
    }
}

impl<Size, Align> Default for RcBump<Size, Align> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Size, Align> Clone for RcBump<Size, Align> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<Size, Align> Deref for RcBump<Size, Align> {
    type Target = Bump<Size, Align>;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

#[cfg(any(feature = "allocator_api", feature = "allocator-fallback"))]
// SAFETY: This impl simply forwards to `Bump`'s `Allocator` impl. See that
// impl for more safety documentation.
//
// `RcBump` is a wrapper around `Rc<Bump>`, so clones of `RcBump` will behave
// like the same allocator, and moving an `RcBump` will not invalidate any
// returned memory.
unsafe impl<Size, Align> Allocator for RcBump<Size, Align> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        Allocator::allocate(&*self.0, layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        // SAFETY: We simply forward to `&Bump`'s `Allocator` impl, which has
        // the same safety requirements as this method. The caller of this
        // method is responsible for ensuring those requirements are met.
        unsafe { Allocator::deallocate(&*self.0, ptr, layout) };
    }
}
