use std::{env, fs};
use std::path::{Path, PathBuf};

fn get_output_path() -> PathBuf {
    //<root or manifest path>/target/<profile>/
    let manifest_dir_string = env::var("CARGO_MANIFEST_DIR").unwrap();
    let build_type = env::var("PROFILE").unwrap();
    let path = Path::new(&manifest_dir_string).join("target").join(build_type);
    PathBuf::from(path)
}

fn main() {
    let input_dir = PathBuf::from(&env::var("CARGO_MANIFEST_DIR").unwrap());
    let output_dir = get_output_path();

    fs::copy(Path::new(&input_dir).join("open.flac"), Path::new(&output_dir).join("open.flac")).expect("copy open.flac");
}
