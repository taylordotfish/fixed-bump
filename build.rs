/*
 * Copyright (C) 2022 taylor.fish <contact@taylor.fish>
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

fn main() {
    if cfg!(feature = "allocator_api") {
        println!("cargo:rustc-cfg=has_allocator_api");
        #[allow(clippy::needless_return)]
        return;
    }
    #[cfg(feature = "allocator-fallback")]
    if allocator_fallback::HAS_ALLOCATOR_API {
        println!("cargo:rustc-cfg=has_allocator_api");
    }
}
