use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

/// note: not tested for linux
pub fn get_path(cwd: &Path, arg: &str) -> Result<PathBuf, String> {
    let mut path = PathBuf::from(arg);

    if path.extension() != Some(OsStr::new("قتام")) {
        return Err("يجب أن يكون إمتداد الملف \"قتام\"".to_string());
    }

    if !path.is_absolute() {
        path = match cwd.join(&path).canonicalize() {
            Ok(path) => path,
            Err(_) => return Err("الملف غير موجود".to_string()),
        };
    } else {
        if !path.is_file() {
            return Err("الملف غير موجود".to_string());
        }
    }

    Ok(path)
}

/// note: this functions is intenteded to work with the path returned from the one above
pub fn get_dir(path: &Path) -> PathBuf {
    path.parent().unwrap().to_owned()
}
