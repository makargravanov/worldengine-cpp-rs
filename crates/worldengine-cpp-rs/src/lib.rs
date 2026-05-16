use std::ffi::CString;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::slice;
use std::sync::Mutex;

pub use worldengine_cpp_sys::{WE_STEP_FULL, WE_STEP_PLATES, WE_STEP_PRECIPITATIONS};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Step {
    Plates,
    Precipitations,
    Full,
}

impl Step {
    fn as_ffi(self) -> i32 {
        match self {
            Step::Plates => WE_STEP_PLATES,
            Step::Precipitations => WE_STEP_PRECIPITATIONS,
            Step::Full => WE_STEP_FULL,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Biome {
    Ocean = 0,
    Sea = 1,
    PolarDesert = 2,
    Ice = 3,
    SubpolarDryTundra = 4,
    SubpolarMoistTundra = 5,
    SubpolarWetTundra = 6,
    SubpolarRainTundra = 7,
    BorealDesert = 8,
    BorealDryScrub = 9,
    BorealMoistForest = 10,
    BorealWetForest = 11,
    BorealRainForest = 12,
    CoolTemperateDesert = 13,
    CoolTemperateDesertScrub = 14,
    CoolTemperateSteppe = 15,
    CoolTemperateMoistForest = 16,
    CoolTemperateWetForest = 17,
    CoolTemperateRainForest = 18,
    WarmTemperateDesert = 19,
    WarmTemperateDesertScrub = 20,
    WarmTemperateThornScrub = 21,
    WarmTemperateDryForest = 22,
    WarmTemperateMoistForest = 23,
    WarmTemperateWetForest = 24,
    WarmTemperateRainForest = 25,
    SubtropicalDesert = 26,
    SubtropicalDesertScrub = 27,
    SubtropicalThornWoodland = 28,
    SubtropicalDryForest = 29,
    SubtropicalMoistForest = 30,
    SubtropicalWetForest = 31,
    SubtropicalRainForest = 32,
    TropicalDesert = 33,
    TropicalDesertScrub = 34,
    TropicalThornWoodland = 35,
    TropicalVeryDryForest = 36,
    TropicalDryForest = 37,
    TropicalMoistForest = 38,
    TropicalWetForest = 39,
    TropicalRainForest = 40,
    BareRock = 41,
}

#[derive(Debug)]
pub enum Error {
    InteriorNul(std::ffi::NulError),
    GenerationFailed,
    NullLayer(&'static str),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InteriorNul(err) => write!(f, "world name contains an interior NUL byte: {err}"),
            Error::GenerationFailed => write!(f, "native world generation failed"),
            Error::NullLayer(layer) => write!(f, "native layer pointer was null: {layer}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::ffi::NulError> for Error {
    fn from(value: std::ffi::NulError) -> Self {
        Error::InteriorNul(value)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Grid<'a, T> {
    width: u32,
    height: u32,
    data: &'a [T],
}

impl<'a, T> Grid<'a, T> {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn as_slice(&self) -> &'a [T] {
        self.data
    }

    pub fn get(&self, x: u32, y: u32) -> Option<&'a T> {
        if x >= self.width || y >= self.height {
            return None;
        }
        self.data.get((y * self.width + x) as usize)
    }
}

#[derive(Clone, Debug)]
pub struct WorldBuilder {
    name: String,
    width: u32,
    height: u32,
    seed: u32,
    num_plates: u32,
    ocean_level: f32,
    step: Step,
    fade_borders: bool,
}

static GENERATE_LOCK: Mutex<()> = Mutex::new(());

impl Default for WorldBuilder {
    fn default() -> Self {
        Self {
            name: "world".to_owned(),
            width: 512,
            height: 512,
            seed: 0,
            num_plates: 10,
            ocean_level: 1.0,
            step: Step::Full,
            fade_borders: true,
        }
    }
}

impl WorldBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn seed(mut self, seed: u32) -> Self {
        self.seed = seed;
        self
    }

    pub fn plates(mut self, num_plates: u32) -> Self {
        self.num_plates = num_plates;
        self
    }

    pub fn ocean_level(mut self, ocean_level: f32) -> Self {
        self.ocean_level = ocean_level;
        self
    }

    pub fn step(mut self, step: Step) -> Self {
        self.step = step;
        self
    }

    pub fn fade_borders(mut self, fade_borders: bool) -> Self {
        self.fade_borders = fade_borders;
        self
    }

    pub fn generate(self) -> Result<World, Error> {
        let name = CString::new(self.name)?;
        let params = worldengine_cpp_sys::we_world_params {
            name: name.as_ptr(),
            width: self.width,
            height: self.height,
            seed: self.seed,
            num_plates: self.num_plates,
            ocean_level: self.ocean_level,
            step: self.step.as_ffi(),
            fade_borders: self.fade_borders,
        };
        let _guard = GENERATE_LOCK
            .lock()
            .expect("world generation lock poisoned");
        let ptr = unsafe { worldengine_cpp_sys::we_world_generate(&params) };
        let ptr = NonNull::new(ptr).ok_or(Error::GenerationFailed)?;
        Ok(World {
            ptr,
            _not_send_sync: PhantomData,
        })
    }
}

pub struct World {
    ptr: NonNull<worldengine_cpp_sys::we_world>,
    _not_send_sync: PhantomData<*mut ()>,
}

impl World {
    pub fn width(&self) -> u32 {
        unsafe { worldengine_cpp_sys::we_world_width(self.ptr.as_ptr()) }
    }

    pub fn height(&self) -> u32 {
        unsafe { worldengine_cpp_sys::we_world_height(self.ptr.as_ptr()) }
    }

    pub fn seed(&self) -> u32 {
        unsafe { worldengine_cpp_sys::we_world_seed(self.ptr.as_ptr()) }
    }

    pub fn elevation(&self) -> Result<Grid<'_, f32>, Error> {
        self.float_grid("elevation", |p| unsafe {
            worldengine_cpp_sys::we_world_elevation(p)
        })
    }

    pub fn ocean(&mut self) -> Result<Grid<'_, u8>, Error> {
        let ptr = unsafe { worldengine_cpp_sys::we_world_ocean(self.ptr.as_ptr()) };
        self.grid_from_ptr("ocean", ptr)
    }

    pub fn plates(&self) -> Result<Grid<'_, u16>, Error> {
        let ptr = unsafe { worldengine_cpp_sys::we_world_plates(self.ptr.as_ptr()) };
        self.grid_from_ptr("plates", ptr)
    }

    pub fn biome(&mut self) -> Result<Option<Grid<'_, Biome>>, Error> {
        if !unsafe { worldengine_cpp_sys::we_world_has_biome(self.ptr.as_ptr()) } {
            return Ok(None);
        }
        let ptr = unsafe { worldengine_cpp_sys::we_world_biome(self.ptr.as_ptr()) };
        self.grid_from_ptr("biome", ptr.cast::<Biome>()).map(Some)
    }

    pub fn humidity(&self) -> Result<Option<Grid<'_, f32>>, Error> {
        self.optional_float_grid(
            "humidity",
            worldengine_cpp_sys::we_world_has_humidity,
            worldengine_cpp_sys::we_world_humidity,
        )
    }

    pub fn icecap(&self) -> Result<Option<Grid<'_, f32>>, Error> {
        self.optional_float_grid(
            "icecap",
            worldengine_cpp_sys::we_world_has_icecap,
            worldengine_cpp_sys::we_world_icecap,
        )
    }

    pub fn irrigation(&self) -> Result<Option<Grid<'_, f32>>, Error> {
        self.optional_float_grid(
            "irrigation",
            worldengine_cpp_sys::we_world_has_irrigation,
            worldengine_cpp_sys::we_world_irrigation,
        )
    }

    pub fn lakemap(&self) -> Result<Option<Grid<'_, f32>>, Error> {
        self.optional_float_grid(
            "lakemap",
            worldengine_cpp_sys::we_world_has_lakemap,
            worldengine_cpp_sys::we_world_lakemap,
        )
    }

    pub fn permeability(&self) -> Result<Option<Grid<'_, f32>>, Error> {
        self.optional_float_grid(
            "permeability",
            worldengine_cpp_sys::we_world_has_permeability,
            worldengine_cpp_sys::we_world_permeability,
        )
    }

    pub fn precipitation(&self) -> Result<Option<Grid<'_, f32>>, Error> {
        self.optional_float_grid(
            "precipitation",
            worldengine_cpp_sys::we_world_has_precipitation,
            worldengine_cpp_sys::we_world_precipitation,
        )
    }

    pub fn rivermap(&self) -> Result<Option<Grid<'_, f32>>, Error> {
        self.optional_float_grid(
            "rivermap",
            worldengine_cpp_sys::we_world_has_rivermap,
            worldengine_cpp_sys::we_world_rivermap,
        )
    }

    pub fn sea_depth(&self) -> Result<Grid<'_, f32>, Error> {
        self.float_grid("sea_depth", |p| unsafe {
            worldengine_cpp_sys::we_world_sea_depth(p)
        })
    }

    pub fn temperature(&self) -> Result<Option<Grid<'_, f32>>, Error> {
        self.optional_float_grid(
            "temperature",
            worldengine_cpp_sys::we_world_has_temperature,
            worldengine_cpp_sys::we_world_temperature,
        )
    }

    pub fn watermap(&self) -> Result<Option<Grid<'_, f32>>, Error> {
        self.optional_float_grid(
            "watermap",
            worldengine_cpp_sys::we_world_has_watermap,
            worldengine_cpp_sys::we_world_watermap,
        )
    }

    fn optional_float_grid(
        &self,
        name: &'static str,
        has: unsafe extern "C" fn(*const worldengine_cpp_sys::we_world) -> bool,
        get: unsafe extern "C" fn(*const worldengine_cpp_sys::we_world) -> *const f32,
    ) -> Result<Option<Grid<'_, f32>>, Error> {
        if !unsafe { has(self.ptr.as_ptr()) } {
            return Ok(None);
        }
        self.float_grid(name, |p| unsafe { get(p) }).map(Some)
    }

    fn float_grid(
        &self,
        name: &'static str,
        get: impl FnOnce(*const worldengine_cpp_sys::we_world) -> *const f32,
    ) -> Result<Grid<'_, f32>, Error> {
        self.grid_from_ptr(name, get(self.ptr.as_ptr()))
    }

    fn grid_from_ptr<T>(&self, name: &'static str, ptr: *const T) -> Result<Grid<'_, T>, Error> {
        if ptr.is_null() {
            return Err(Error::NullLayer(name));
        }
        let len = unsafe { worldengine_cpp_sys::we_world_len(self.ptr.as_ptr()) };
        Ok(Grid {
            width: self.width(),
            height: self.height(),
            data: unsafe { slice::from_raw_parts(ptr, len) },
        })
    }
}

impl Drop for World {
    fn drop(&mut self) {
        unsafe { worldengine_cpp_sys::we_world_free(self.ptr.as_ptr()) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_small_world() {
        let mut world = WorldBuilder::new()
            .name("safe-test")
            .size(32, 16)
            .seed(1)
            .step(Step::Full)
            .generate()
            .unwrap();

        assert_eq!(world.width(), 32);
        assert_eq!(world.height(), 16);
        assert_eq!(world.elevation().unwrap().as_slice().len(), 32 * 16);
        assert!(world.elevation().unwrap().get(31, 15).is_some());
        assert!(world.elevation().unwrap().get(32, 15).is_none());
        assert!(world.ocean().unwrap().get(0, 0).is_some());
        assert!(world.biome().unwrap().is_some());
    }

    #[test]
    fn partial_step_has_missing_layers() {
        let mut world = WorldBuilder::new()
            .name("plates-test")
            .size(32, 16)
            .seed(1)
            .step(Step::Plates)
            .generate()
            .unwrap();

        assert!(world.biome().unwrap().is_none());
        assert!(world.temperature().unwrap().is_none());
    }
}
