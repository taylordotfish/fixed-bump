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

mod bump;
mod rc;

/// The example from the crate documentation. It's duplicated here because Miri
/// currently doesn't run doctests.
#[test]
fn crate_example() {
    use crate::Bump;
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
    #[cfg(feature = "allocator_api")]
    {
        use alloc::vec::Vec;
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
    }
}
