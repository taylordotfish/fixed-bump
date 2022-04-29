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

#![no_std]
#![cfg_attr(feature = "allocator_api", feature(allocator_api))]
#![cfg_attr(feature = "unstable", deny(unsafe_op_in_unsafe_fn))]
#![cfg_attr(not(feature = "unstable"), allow(unused_unsafe))]
#![warn(clippy::pedantic)]
#![allow(clippy::default_trait_access)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]

//! A bump allocator (like [bumpalo]) that internally uses fixed-size chunks
//! of memory.
//!
//! Other bump allocators, like [bumpalo], are optimized for throughput: they
//! allocate chunks of memory with exponentially increasing sizes, which
//! results in *amortized* constant-time allocations.
//!
//! [bumpalo]: https://docs.rs/bumpalo
//!
//! **fixed-bump** is optimized for latency: it internally allocates chunks of
//! memory with a fixed, configurable size, and individual value allocations
//! are performed in non-amortized constant time. However, a trade-off with
//! using this crate is that it may not be able to allocate certain types or
//! memory layouts if the specified chunk size or alignment is too small. See
//! [`Bump::allocate`] for the conditions under which allocation may fail.
//!
//! This crate depends only on [`core`] and [`alloc`], so it can be used in
//! `no_std` environments that support [`alloc`].
//!
//! [`core`]: https://doc.rust-lang.org/core/
//! [`alloc`]: https://doc.rust-lang.org/alloc/
//!
//! Example
//! -------
//!
//! ```rust
//! # #![cfg_attr(feature = "allocator_api", feature(allocator_api))]
//! use fixed_bump::Bump;
//! struct Item(u64);
//!
//! // Use chunks large and aligned enough to hold 128 `Item`s.
//! let bump = Bump::<[Item; 128]>::new();
//! let item1: &mut Item = bump.alloc_value(Item(1));
//! let item2: &mut Item = bump.alloc_value(Item(2));
//! item1.0 += item2.0;
//!
//! assert_eq!(item1.0, 3);
//! assert_eq!(item2.0, 2);
//!
//! // Can also allocate different types:
//! let array: &mut [u8; 8] = bump.alloc_value([0, 1, 2, 3, 4, 5, 6, 7]);
//! assert_eq!(array.iter().sum::<u8>(), 28);
//!
//! // Can also use `&Bump` as an `Allocator` (requires "allocator_api"):
//! # #[cfg(feature = "allocator_api")]
//! # {
//! # extern crate alloc;
//! # use alloc::vec::Vec;
//! // To avoid resizing, we create these `Vec`s with the maximum capacity
//! // we want them ever to have. Resizing would waste memory, since bump
//! // allocators don't reclaim or reuse memory until the entire allocator
//! // is dropped.
//! let mut vec1: Vec<u32, _> = Vec::with_capacity_in(8, &bump);
//! let mut vec2: Vec<u32, _> = Vec::with_capacity_in(4, &bump);
//! for i in 0..4 {
//!     vec1.push(i * 2);
//!     vec1.push(i * 2 + 1);
//!     vec2.push(i * 2);
//! }
//!
//! assert_eq!(vec1, [0, 1, 2, 3, 4, 5, 6, 7]);
//! assert_eq!(vec2, [0, 2, 4, 6]);
//! # }
//! ```
//!
//! Dropping
//! --------
//!
//! [`Bump`] can either return raw memory (see [`Bump::allocate`]) or allocate
//! a value of a specific type and return a reference (see
//! [`Bump::alloc_value`] and [`Bump::try_alloc_value`]). In the latter case
//! where references are returned, note that destructors will not be
//! automatically run. If this is an issue, you can do one of the following:
//!
//! * Drop those values manually with [`ptr::drop_in_place`].
//! * Enable the `allocator_api` feature, which lets you use [`Bump`], `&Bump`,
//!   and [`RcBump`] as allocators for various data structures like [`Box`] and
//!   [`Vec`]. Note that this requires Rust nightly.
//!
//! Note that, as with other bump allocators, the memory used by an allocated
//! object will not be reclaimed or reused until the entire bump allocator
//! is dropped.
//!
//! Crate features
//! --------------
//!
//! If the crate feature `allocator_api` is enabled, [`Bump`], `&Bump` (due to
//! the impl of [`Allocator`] for all `&A` where `A: Allocator`), and
//! [`RcBump`] will implement the unstable [`Allocator`] trait. This lets you
//! use those types as allocators for various data structures like [`Box`] and
//! [`Vec`]. Note that this feature requires Rust nightly.
//!
//! [`ptr::drop_in_place`]: core::ptr::drop_in_place
//! [`Box`]: alloc::boxed::Box
//! [`Vec`]: alloc::vec::Vec
//! [`Allocator`]: alloc::alloc::Allocator

extern crate alloc;

mod bump;
mod chunk;
mod inner;
mod rc;
#[cfg(test)]
mod tests;

pub use bump::Bump;
pub use rc::RcBump;
