use failure;
use sdl2::filesystem::*;
use std::path::{Path, PathBuf};

static ASSET_PATH: Option<&'static str> = option_env!("SLSENGINE_ASSET_PATH");

/// if the build system defines SLSENGINE_ASSET_PATH as a compile-time environment variable,
/// this function will return a PathBuff to this path. Otherwise, it will use the path
/// relative to project root
pub fn asset_path() -> PathBuf {
    match ASSET_PATH {
        Some(p) => PathBuf::from(p),
        None => {
            let p = base_path()
                .expect("fatal error: could not retrieve app base path");
            PathBuf::from(&p)
        }
    }
}

#[test]
fn test_asset_path() {
    use std::fs::*;
    let p = asset_path();
    let children: Vec<_> = read_dir(&p)
        .expect(&format!("directory {:?} should have children", p))
        .collect();
    assert!(
        children.iter().any(|ref x| match x {
            Ok(x) => x.file_name() == "assets",
            Err(_) => false,
        }),
        "directory {:?} should have a folder titled 'assets'",
    );
}

pub fn from_base_path<P: AsRef<Path>>(
    subpath: P,
) -> Result<PathBuf, failure::Error> {
    let path = base_path().map_err(&failure::err_msg)?;
    Ok(Path::new(&path).join(subpath))
}
