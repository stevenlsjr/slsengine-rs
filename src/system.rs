use failure;
use sdl2::filesystem::*;
use std::path::{Path, PathBuf};

pub fn from_base_path<P: AsRef<Path>>(
    subpath: P,
) -> Result<PathBuf, failure::Error> {
    let mut path = base_path().map_err(&failure::err_msg)?;
    Ok(Path::new(&path).join(subpath))
}
