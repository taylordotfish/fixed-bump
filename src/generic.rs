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

use super::chunk::Chunk;
use super::inner::BumpInner;
use alloc::alloc::{Layout, handle_alloc_error};
use core::cell::UnsafeCell;
use core::ptr::NonNull;

/// # Safety
///
/// `<Self as Into<Layout>>::into` must not call any method of any
/// [`GenericBump`].
pub unsafe trait IntoLayout: Copy + Into<Layout> {}

pub struct GenericBump<L: IntoLayout>(UnsafeCell<BumpInner<L>>);

impl<L: IntoLayout> GenericBump<L> {
    pub fn new(layout: L) -> Self {
        Self(UnsafeCell::new(BumpInner::new(layout)))
    }

    fn inner(&self) -> &BumpInner<L> {
        // SAFETY: `BumpInner` has no `&self` methods that could possibly call
        // any methods of `Self`, which ensures we do not concurrently mutably
        // borrow the `UnsafeCell`.
        unsafe { &*self.0.get() }
    }

    pub fn layout(&self) -> Layout {
        self.inner().layout()
    }

    pub fn allocate(&self, layout: Layout) -> Option<NonNull<[u8]>> {
        // SAFETY: `BumpInner::alloc` does not run any code that could possibly
        // call any methods of `Self`, which ensures that we do not borrow the
        // data in the `UnsafeCell` multiple times concurrently.
        unsafe { &mut *self.0.get() }.allocate(layout)
    }

    #[allow(clippy::mut_from_ref)]
    #[must_use]
    pub fn alloc_value<T>(&self, value: T) -> &mut T {
        if let Ok(r) = self.try_alloc_value(value) {
            return r;
        }
        if self.can_allocate(Layout::new::<T>()) {
            handle_alloc_error(Chunk::full_layout(self.inner().layout()));
        }
        panic!("this allocator cannot allocate values of this type");
    }

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

    pub fn can_allocate(&self, layout: Layout) -> bool {
        let cl = Chunk::layout(self.inner().layout());
        layout.size() <= cl.size() && layout.align() <= cl.align()
    }
}
