use path_absolutize::*;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

pub fn qatam_path(path: &Path) -> Result<(), String> {
    if path.extension() == Some(OsStr::new("قتام")) {
        return Ok(());
    }
    Err("يجب أن يكون امتداد الملف \"قتام\"".to_string())
}

/// note: the path is expected to be absolute
fn get_dir(path: &Path) -> PathBuf {
    match path.parent() {
        Some(dir) => dir.to_owned(),
        None => PathBuf::from("/"), // only for linux
    }
}

/// note: cur_file must be absolute
pub fn resolve_path(
    cur_file: Option<PathBuf>,
    path: &str,
    pred: fn(&Path) -> Result<(), String>,
) -> Result<PathBuf, String> {
    let mut path = PathBuf::from(path);

    if path.is_relative() {
        match cur_file {
            Some(cur_file) => {
                path = (*get_dir(&cur_file).join(path).absolutize().unwrap()).to_owned();
            }
            None => {
                path = (*path.absolutize().unwrap()).to_owned();
            }
        };
    }

    if !path.is_file() {
        return Err("لا يوجد ملف بهذا المسار".to_string());
    }

    pred(&path)?;

    Ok(path)
}
