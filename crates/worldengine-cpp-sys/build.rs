use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf();

    println!("cargo:rerun-if-env-changed=WORLDENGINE_CPP_RS_BOOST_DIR");
    println!("cargo:rerun-if-env-changed=WORLDENGINE_CPP_RS_EIGEN_DIR");
    println!("cargo:rerun-if-env-changed=CXX");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let boost_dir = include_dir(
        "WORLDENGINE_CPP_RS_BOOST_DIR",
        &[
            root.join("vendor/boost"),
            PathBuf::from("C:/msys64/mingw64/include"),
            PathBuf::from("C:/msys64/usr/include"),
            PathBuf::from("/usr/include"),
            PathBuf::from("/usr/local/include"),
        ],
        "boost/multi_array.hpp",
        || {
            fetch_archive_include(
                &out_dir,
                "boost_1_81_0",
                "https://archives.boost.io/release/1.81.0/source/boost_1_81_0.tar.gz",
                "boost_1_81_0",
            )
        },
    );
    let eigen_dir = include_dir(
        "WORLDENGINE_CPP_RS_EIGEN_DIR",
        &[
            root.join("vendor/eigen"),
            PathBuf::from("C:/msys64/mingw64/include/eigen3"),
            PathBuf::from("C:/msys64/usr/include/eigen3"),
            PathBuf::from("/usr/include/eigen3"),
            PathBuf::from("/usr/local/include/eigen3"),
        ],
        "unsupported/Eigen/CXX11/Tensor",
        || {
            fetch_archive_include(
                &out_dir,
                "eigen-3.4.0",
                "https://gitlab.com/libeigen/eigen/-/archive/3.4.0/eigen-3.4.0.tar.gz",
                "eigen-3.4.0",
            )
        },
    );

    let target = env::var("TARGET").unwrap();

    let include_dirs = [
        root.join("crates/worldengine-cpp-sys/cpp/include"),
        root.join("crates/worldengine-cpp-sys/cpp"),
        root.join("worldengine/include"),
        root.join("worldengine/source"),
        root.join("worldengine/source/simulations"),
        root.join("external/plate-tectonics/src"),
        root.join("external/OpenSimplexNoise/OpenSimplexNoise"),
        boost_dir,
        eigen_dir,
    ];

    let mut build = cc::Build::new();
    configure_cpp_build(&mut build, &target, "c++17");
    build.define("WORLDENGINE_DATA_ONLY", None);
    build.define("EIGEN_MPL2_ONLY", None);
    force_include_cstdint(&mut build, &target);

    for dir in &include_dirs {
        build.include(dir);
    }

    for file in worldengine_sources(&root)
        .into_iter()
        .chain(open_simplex_sources(&root))
        .chain([root.join("crates/worldengine-cpp-sys/cpp/worldengine_c_api.cpp")])
    {
        println!("cargo:rerun-if-changed={}", file.display());
        build.file(file);
    }

    build.compile("worldengine_cpp_rs_native");

    let mut plate_build = cc::Build::new();
    configure_cpp_build(&mut plate_build, &target, "c++14");
    force_include_header(&mut plate_build, &target, "plate_compat.hpp");
    for dir in &include_dirs {
        plate_build.include(dir);
    }
    for file in plate_tectonics_sources(&root) {
        println!("cargo:rerun-if-changed={}", file.display());
        plate_build.file(file);
    }
    plate_build.compile("plate_tectonics_rs");
}

fn configure_cpp_build(build: &mut cc::Build, target: &str, standard: &str) {
    build.cpp(true);
    build.std(standard);
    build.warnings(false);

    if target.contains("windows-gnu") {
        prefer_compiler(build, &["g++", "clang++"]);
    } else if target.contains("windows-msvc") {
        prefer_compiler(build, &["clang-cl", "cl"]);
    }

    if target.contains("windows-msvc") {
        build.flag_if_supported("/EHsc");
    } else {
        build.flag_if_supported("-fexceptions");
        build.flag_if_supported("-frtti");
        build.flag_if_supported("-O3");
    }
}

fn force_include_cstdint(build: &mut cc::Build, target: &str) {
    force_include_header(build, target, "cstdint");
}

fn force_include_header(build: &mut cc::Build, target: &str, header: &str) {
    if target.contains("windows-msvc") {
        build.flag_if_supported(&format!("/FI{header}"));
    } else {
        build.flag("-include");
        build.flag(header);
    }
}

fn prefer_compiler(build: &mut cc::Build, candidates: &[&str]) {
    if env::var_os("CXX").is_some() {
        return;
    }
    for candidate in candidates {
        if command_exists(candidate) {
            build.compiler(candidate);
            return;
        }
    }
}

fn command_exists(command: &str) -> bool {
    let checker = if cfg!(windows) { "where" } else { "which" };
    std::process::Command::new(checker)
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn include_dir(
    env_name: &str,
    candidates: &[PathBuf],
    probe: &str,
    fetch: impl FnOnce() -> PathBuf,
) -> PathBuf {
    if let Some(value) = env::var_os(env_name) {
        let path = PathBuf::from(value);
        if path.join(probe).exists() {
            return path;
        }
        panic!(
            "{env_name} is set to {}, but {} was not found",
            path.display(),
            probe
        );
    }

    for candidate in candidates {
        if candidate.join(probe).exists() {
            return candidate.clone();
        }
    }

    let fetched = fetch();
    if fetched.join(probe).exists() {
        return fetched;
    }

    panic!(
        "Could not find required C++ headers for {probe}. Set {env_name} to an include directory."
    );
}

fn fetch_archive_include(out_dir: &Path, name: &str, url: &str, root_dir: &str) -> PathBuf {
    let deps_dir = out_dir.join("native-deps");
    let include_dir = deps_dir.join(root_dir);
    if include_dir.exists() {
        return include_dir;
    }

    std::fs::create_dir_all(&deps_dir).unwrap();
    let archive = deps_dir.join(format!("{name}.tar.gz"));
    if !archive.exists() {
        let status = std::process::Command::new("curl")
            .args(["-L", "--fail", "-o"])
            .arg(&archive)
            .arg(url)
            .status()
            .unwrap_or_else(|err| panic!("failed to run curl for {url}: {err}"));
        if !status.success() {
            panic!("failed to download {url}");
        }
    }

    let status = std::process::Command::new("tar")
        .arg("-xzf")
        .arg(&archive)
        .arg("-C")
        .arg(&deps_dir)
        .status()
        .unwrap_or_else(|err| panic!("failed to run tar for {}: {err}", archive.display()));
    if !status.success() {
        panic!("failed to unpack {}", archive.display());
    }

    include_dir
}

fn worldengine_sources(root: &Path) -> Vec<PathBuf> {
    [
        "worldengine/source/basic.cpp",
        "worldengine/source/common.cpp",
        "worldengine/source/generation.cpp",
        "worldengine/source/path.cpp",
        "worldengine/source/plates.cpp",
        "worldengine/source/world.cpp",
        "worldengine/source/simulations/biome.cpp",
        "worldengine/source/simulations/erosion.cpp",
        "worldengine/source/simulations/humidity.cpp",
        "worldengine/source/simulations/hydrology.cpp",
        "worldengine/source/simulations/icecap.cpp",
        "worldengine/source/simulations/irrigation.cpp",
        "worldengine/source/simulations/permeability.cpp",
        "worldengine/source/simulations/precipitation.cpp",
        "worldengine/source/simulations/temperature.cpp",
    ]
    .into_iter()
    .map(|path| root.join(path))
    .collect()
}

fn plate_tectonics_sources(root: &Path) -> Vec<PathBuf> {
    [
        "external/plate-tectonics/src/sqrdmd.cpp",
        "external/plate-tectonics/src/heightmap.cpp",
        "external/plate-tectonics/src/lithosphere.cpp",
        "external/plate-tectonics/src/plate.cpp",
        "external/plate-tectonics/src/rectangle.cpp",
        "external/plate-tectonics/src/platecapi.cpp",
        "external/plate-tectonics/src/simplexnoise.cpp",
        "external/plate-tectonics/src/noise.cpp",
        "external/plate-tectonics/src/utils.cpp",
        "external/plate-tectonics/src/simplerandom.cpp",
        "external/plate-tectonics/src/plate_functions.cpp",
        "external/plate-tectonics/src/bounds.cpp",
        "external/plate-tectonics/src/movement.cpp",
        "external/plate-tectonics/src/mass.cpp",
        "external/plate-tectonics/src/segments.cpp",
        "external/plate-tectonics/src/world_point.cpp",
        "external/plate-tectonics/src/geometry.cpp",
        "external/plate-tectonics/src/segment_creator.cpp",
        "external/plate-tectonics/src/segment_data.cpp",
    ]
    .into_iter()
    .map(|path| root.join(path))
    .collect()
}

fn open_simplex_sources(root: &Path) -> Vec<PathBuf> {
    vec![root.join("external/OpenSimplexNoise/OpenSimplexNoise/OpenSimplexNoise.cpp")]
}
