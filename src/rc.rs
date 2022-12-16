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
use alloc::rc;
use core::ops::Deref;
#[cfg(any(feature = "allocator_api", feature = "allocator-fallback"))]
use {
    super::{AllocError, Allocator},
    alloc::alloc::Layout,
    core::ptr::NonNull,
};

/// A wrapper around [`Rc`](rc::Rc).
///
/// This type exists mainly so that [`Allocator`](alloc::alloc::Allocator)
/// can be implemented for it:
///
/// ```
/// # #![cfg_attr(feature = "allocator_api", feature(allocator_api))]
/// use fixed_bump::Bump;
/// let rc = std::rc::Rc::new(Bump::<[u32; 16]>::new());
/// # #[cfg(feature = "allocator_api")]
/// # {
/// // Error: `std::rc::Rc<Bump<...>>` doesn't implement `Allocator`:
/// //let b = Box::new_in(1_u32, rc);
/// // Compiles: `fixed_bump::Rc<Bump<...>>` implements `Allocator`:
/// let b = Box::new_in(1_u32, fixed_bump::Rc(rc));
/// # }
/// ```
pub struct Rc<Bump>(pub rc::Rc<Bump>);

impl<Bump> Rc<Bump> {
    /// Creates a new [`Rc`]. This simply returns
    /// <code>[Rc]\([rc::Rc::new]\(bump))</code>.
    pub fn new(bump: Bump) -> Self {
        Self(rc::Rc::new(bump))
    }
}

impl<Bump: Default> Default for Rc<Bump> {
    fn default() -> Self {
        Self::new(Bump::default())
    }
}

impl<Bump> Clone for Rc<Bump> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<Bump> Deref for Rc<Bump> {
    type Target = Bump;

    fn deref(&self) -> &Self::Target {
        &self.0
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
// SAFETY: This impl simply forwards to `Bump`'s `Allocator` impl.
//
// `Rc` is a wrapper around `rc::Rc<Bump>`, so clones of `Rc` will behave
// like the same allocator, and moving an `Rc` will not invalidate any
// returned memory.
unsafe impl<Bump: Allocator> Allocator for Rc<Bump> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        Allocator::allocate(&*self.0, layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        // SAFETY: We simply forward to `Bump`'s `Allocator` impl, which has
        // the same safety requirements as this method. The caller of this
        // method is responsible for ensuring those requirements are met.
        unsafe { Allocator::deallocate(&*self.0, ptr, layout) };
    }
}

#[doc(hidden)]
#[deprecated = "use `fixed_bump::Rc<Bump<...>>` instead"]
/// A wrapper around <code>[Rc](rc::Rc)<Bump<...>></code>.
pub struct RcBump<Size, Align = Size>(pub rc::Rc<Bump<Size, Align>>);

#[allow(deprecated)]
impl<Size, Align> RcBump<Size, Align> {
    /// Creates a new [`RcBump`]. This simply returns
    /// <code>[RcBump]\([rc::Rc::new]\([Bump::new]\())</code>.
    pub fn new() -> Self {
        Self(rc::Rc::new(Bump::new()))
    }
}

#[allow(deprecated)]
impl<Size, Align> Default for RcBump<Size, Align> {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(deprecated)]
impl<Size, Align> Clone for RcBump<Size, Align> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[allow(deprecated)]
impl<Size, Align> Deref for RcBump<Size, Align> {
    type Target = Bump<Size, Align>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(any(feature = "allocator_api", feature = "allocator-fallback"))]
#[allow(deprecated)]
// SAFETY: This impl simply forwards to `Bump`'s `Allocator` impl.
//
// `RcBump` is a wrapper around `rc::Rc<Bump<...>>`, so clones of `RcBump` will
// behave like the same allocator, and moving an `RcBump` will not invalidate
// any returned memory.
unsafe impl<Size, Align> Allocator for RcBump<Size, Align> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        Allocator::allocate(&*self.0, layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        // SAFETY: We simply forward to `Bump`'s `Allocator` impl, which has
        // the same safety requirements as this method. The caller of this
        // method is responsible for ensuring those requirements are met.
        unsafe { Allocator::deallocate(&*self.0, ptr, layout) };
    }
}
