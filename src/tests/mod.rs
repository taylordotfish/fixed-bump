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

use crate::Bump;

mod rc;

#[test]
fn empty() {
    Bump::<[u8; 16]>::new();
}

#[test]
fn basic() {
    let bump = Bump::<[u8; 16]>::new();
    let item1 = bump.alloc_value(1_u8);
    let item2 = bump.alloc_value(2_u8);
    let item3 = bump.alloc_value(3_u8);
    assert_eq!(*item1, 1_u8);
    assert_eq!(*item2, 2_u8);
    assert_eq!(*item3, 3_u8);
}

#[test]
fn multiple_chunks() {
    let bump = Bump::<[u8; 2]>::new();
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

#[test]
fn multiple_types() {
    let bump = Bump::<[u64; 2]>::new();
    let item1 = bump.alloc_value(1_u8);
    let item2 = bump.alloc_value(2_u32);
    let item3 = bump.alloc_value(3_u64);
    let item4 = bump.alloc_value(4_u64);
    let item5 = bump.alloc_value(5_u16);
    let item6 = bump.alloc_value(6_u8);
    let item7 = bump.alloc_value(7_u32);
    let item8 = bump.alloc_value(8_u32);
    let item9 = bump.alloc_value(9_u8);
    let item10 = bump.alloc_value(10_u64);
    let item11 = bump.alloc_value(11_u64);
    assert_eq!(*item1, 1);
    assert_eq!(*item2, 2);
    assert_eq!(*item3, 3);
    assert_eq!(*item4, 4);
    assert_eq!(*item5, 5);
    assert_eq!(*item6, 6);
    assert_eq!(*item7, 7);
    assert_eq!(*item8, 8);
    assert_eq!(*item9, 9);
    assert_eq!(*item10, 10);
    assert_eq!(*item11, 11);
}

#[cfg(feature = "allocator_api")]
#[test]
fn allocator() {
    use alloc::vec::Vec;
    let bump = Bump::<[u32; 32]>::new();
    let mut vec1: Vec<u16, _> = Vec::with_capacity_in(32, &bump);
    for i in 0..32 {
        vec1.push(i);
    }
    let mut vec2: Vec<u32, _> = Vec::with_capacity_in(32, &bump);
    for i in 0..32 {
        vec2.push(i);
    }
    for i in 0_u16..32 {
        assert_eq!(vec1[i as usize], i);
        assert_eq!(vec2[i as usize], i.into());
    }

    core::mem::drop(vec1);
    core::mem::drop(vec2);
    let mut vec3: Vec<u32, _> = Vec::with_capacity_in(32, bump);
    for i in 0..32 {
        vec3.push(i);
    }
    for i in 0..32 {
        assert_eq!(vec3[i as usize], i);
    }
}

#[test]
#[should_panic]
fn zero_chunk_size() {
    let bump = Bump::<[u8; 0]>::new();
    for i in 0..4 {
        let _ = bump.alloc_value(i);
    }
}
