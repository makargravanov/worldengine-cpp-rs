#![allow(non_camel_case_types)]

use std::os::raw::{c_char, c_float, c_int, c_uint};

#[repr(C)]
pub struct we_world {
    _private: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct we_world_params {
    pub name: *const c_char,
    pub width: c_uint,
    pub height: c_uint,
    pub seed: c_uint,
    pub num_plates: c_uint,
    pub ocean_level: c_float,
    pub step: c_int,
    pub fade_borders: bool,
}

pub const WE_STEP_PLATES: c_int = 0;
pub const WE_STEP_PRECIPITATIONS: c_int = 1;
pub const WE_STEP_FULL: c_int = 2;

extern "C" {
    pub fn we_world_generate(params: *const we_world_params) -> *mut we_world;
    pub fn we_world_free(world: *mut we_world);

    pub fn we_world_width(world: *const we_world) -> c_uint;
    pub fn we_world_height(world: *const we_world) -> c_uint;
    pub fn we_world_seed(world: *const we_world) -> c_uint;
    pub fn we_world_len(world: *const we_world) -> usize;

    pub fn we_world_has_biome(world: *const we_world) -> bool;
    pub fn we_world_has_humidity(world: *const we_world) -> bool;
    pub fn we_world_has_icecap(world: *const we_world) -> bool;
    pub fn we_world_has_irrigation(world: *const we_world) -> bool;
    pub fn we_world_has_lakemap(world: *const we_world) -> bool;
    pub fn we_world_has_permeability(world: *const we_world) -> bool;
    pub fn we_world_has_precipitation(world: *const we_world) -> bool;
    pub fn we_world_has_rivermap(world: *const we_world) -> bool;
    pub fn we_world_has_temperature(world: *const we_world) -> bool;
    pub fn we_world_has_watermap(world: *const we_world) -> bool;

    pub fn we_world_elevation(world: *const we_world) -> *const c_float;
    pub fn we_world_ocean(world: *mut we_world) -> *const u8;
    pub fn we_world_plates(world: *const we_world) -> *const u16;
    pub fn we_world_biome(world: *mut we_world) -> *const u32;
    pub fn we_world_humidity(world: *const we_world) -> *const c_float;
    pub fn we_world_icecap(world: *const we_world) -> *const c_float;
    pub fn we_world_irrigation(world: *const we_world) -> *const c_float;
    pub fn we_world_lakemap(world: *const we_world) -> *const c_float;
    pub fn we_world_permeability(world: *const we_world) -> *const c_float;
    pub fn we_world_precipitation(world: *const we_world) -> *const c_float;
    pub fn we_world_rivermap(world: *const we_world) -> *const c_float;
    pub fn we_world_sea_depth(world: *const we_world) -> *const c_float;
    pub fn we_world_temperature(world: *const we_world) -> *const c_float;
    pub fn we_world_watermap(world: *const we_world) -> *const c_float;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn generate_and_free_small_world() {
        let name = CString::new("sys-test").unwrap();
        let params = we_world_params {
            name: name.as_ptr(),
            width: 32,
            height: 16,
            seed: 1,
            num_plates: 10,
            ocean_level: 1.0,
            step: WE_STEP_FULL,
            fade_borders: true,
        };

        let world = unsafe { we_world_generate(&params) };
        assert!(!world.is_null());
        assert_eq!(unsafe { we_world_width(world) }, 32);
        assert_eq!(unsafe { we_world_height(world) }, 16);
        assert_eq!(unsafe { we_world_len(world) }, 32 * 16);
        assert!(!unsafe { we_world_elevation(world) }.is_null());
        unsafe { we_world_free(world) };
    }
}
