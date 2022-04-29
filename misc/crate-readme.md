fixed-bump
==========

A bump allocator (like [bumpalo]) that internally uses fixed-size chunks
of memory.

Other bump allocators, like [bumpalo], are optimized for throughput: they
allocate chunks of memory with exponentially increasing sizes, which
results in *amortized* constant-time allocations.

[bumpalo]: https://docs.rs/bumpalo

**fixed-bump** is optimized for latency: it internally allocates chunks of
memory with a fixed, configurable size, and individual value allocations
are performed in non-amortized constant time. However, a trade-off with
using this crate is that it may not be able to allocate certain types or
memory layouts if the specified chunk size or alignment is too small. See
[`Bump::allocate`] for the conditions under which allocation may fail.

This crate depends only on [`core`] and [`alloc`], so it can be used in
`no_std` environments that support [`alloc`].

[`core`]: https://doc.rust-lang.org/core/
[`alloc`]: https://doc.rust-lang.org/alloc/

Example
-------

```rust
use fixed_bump::Bump;
struct Item(u64);

// Use chunks large and aligned enough to hold 128 `Item`s.
let bump = Bump::<[Item; 128]>::new();
let item1: &mut Item = bump.alloc_value(Item(1));
let item2: &mut Item = bump.alloc_value(Item(2));
item1.0 += item2.0;

assert_eq!(item1.0, 3);
assert_eq!(item2.0, 2);

// Can also allocate different types:
let array: &mut [u8; 8] = bump.alloc_value([0, 1, 2, 3, 4, 5, 6, 7]);
assert_eq!(array.iter().sum::<u8>(), 28);

// Can also use `&Bump` as an `Allocator` (requires "allocator_api"):
// To avoid resizing, we create these `Vec`s with the maximum capacity
// we want them ever to have. Resizing would waste memory, since bump
// allocators don't reclaim or reuse memory until the entire allocator
// is dropped.
let mut vec1: Vec<u32, _> = Vec::with_capacity_in(8, &bump);
let mut vec2: Vec<u32, _> = Vec::with_capacity_in(4, &bump);
for i in 0..4 {
    vec1.push(i * 2);
    vec1.push(i * 2 + 1);
    vec2.push(i * 2);
}

assert_eq!(vec1, [0, 1, 2, 3, 4, 5, 6, 7]);
assert_eq!(vec2, [0, 2, 4, 6]);
```

Dropping
--------

[`Bump`] can either return raw memory (see [`Bump::allocate`]) or allocate
a value of a specific type and return a reference (see
[`Bump::alloc_value`] and [`Bump::try_alloc_value`]). In the latter case
where references are returned, note that destructors will not be
automatically run. If this is an issue, you can do one of the following:

* Drop those values manually with [`ptr::drop_in_place`].
* Enable the `allocator_api` feature, which lets you use [`Bump`], `&Bump`,
  and [`RcBump`] as allocators for various data structures like [`Box`] and
  [`Vec`]. Note that this requires Rust nightly.

Note that, as with other bump allocators, the memory used by an allocated
object will not be reclaimed or reused until the entire bump allocator
is dropped.

Crate features
--------------

If the crate feature `allocator_api` is enabled, [`Bump`], `&Bump` (due to
the impl of [`Allocator`] for all `&A` where `A: Allocator`), and
[`RcBump`] will implement the unstable [`Allocator`] trait. This lets you
use those types as allocators for various data structures like [`Box`] and
[`Vec`]. Note that this feature requires Rust nightly. Alternatively, if
the feature `allocator-fallback` is enabled, this crate will use the
allocator API provided by [allocator-fallback] instead of the standard
library's.

[allocator-fallback]: https://docs.rs/allocator-fallback

[`Bump`]: https://docs.rs/fixed-bump/latest/fixed_bump/struct.Bump.html
[`Bump::allocate`]: https://docs.rs/fixed-bump/latest/fixed_bump/struct.Bump.html#method.allocate
[`Bump::alloc_value`]: https://docs.rs/fixed-bump/latest/fixed_bump/struct.Bump.html#method.alloc_value
[`Bump::try_alloc_value`]: https://docs.rs/fixed-bump/latest/fixed_bump/struct.Bump.html#method.try_alloc_value
[`ptr::drop_in_place`]: https://doc.rust-lang.org/core/ptr/fn.drop_in_place.html
[`RcBump`]: https://docs.rs/fixed-bump/latest/fixed_bump/struct.RcBump.html
[`Box`]: https://doc.rust-lang.org/stable/alloc/boxed/struct.Box.html
[`Vec`]: https://doc.rust-lang.org/stable/alloc/vec/struct.Vec.html
[`Allocator`]: https://doc.rust-lang.org/stable/alloc/alloc/trait.Allocator.html
