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

use crate::RcBump;

#[test]
fn empty() {
    RcBump::<[u8; 16]>::new();
}

#[test]
fn basic() {
    let bump = RcBump::<[u8; 16]>::new();
    let item1 = bump.alloc_value(1_u8);
    let item2 = bump.alloc_value(2_u8);
    let item3 = bump.alloc_value(3_u8);
    assert_eq!(*item1, 1_u8);
    assert_eq!(*item2, 2_u8);
    assert_eq!(*item3, 3_u8);
}

#[test]
fn multiple_chunks() {
    let bump = RcBump::<[u8; 2]>::new();
    let item1 = bump.alloc_value(1_u8);
    let item2 = bump.alloc_value(2_u8);
    let item3 = bump.alloc_value(3_u8);
    let item4 = bump.alloc_value(4_u8);
    let item5 = bump.alloc_value(5_u8);
    assert_eq!(*item1, 1_u8);
    assert_eq!(*item2, 2_u8);
    assert_eq!(*item3, 3_u8);
    assert_eq!(*item4, 4_u8);
    assert_eq!(*item5, 5_u8);
}

#[cfg(feature = "allocator_api")]
#[test]
fn allocator() {
    use alloc::vec::Vec;
    let bump = RcBump::<[u8; 16]>::new();
    let mut vec1: Vec<u8, _> = Vec::with_capacity_in(8, bump.clone());
    let mut vec2: Vec<u8, _> = Vec::with_capacity_in(8, bump);
    for i in 0..8 {
        vec1.push(i);
        vec2.push(i * 2);
    }
    assert_eq!(vec1, [0, 1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(vec2, [0, 2, 4, 6, 8, 10, 12, 14]);
}
