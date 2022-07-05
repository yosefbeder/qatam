use std::{convert::TryFrom, env::Args, ffi::OsStr, path::PathBuf};

pub enum Mode {
    Help,
    Version,
    File { path: PathBuf, untrusted: bool },
    Repl,
}

impl Mode {
    fn new_file(path: String) -> Result<Self, String> {
        let path = PathBuf::from(path);

        if Some(OsStr::new("قتام")) != path.extension() {
            return Err("يجب أن يكون امتداد الملف 'قتام'".to_string());
        }

        Ok(File {
            path: PathBuf::from(path),
            untrusted: false,
        })
    }
}

use Mode::*;

impl TryFrom<Args> for Mode {
    type Error = String;

    fn try_from(mut args: Args) -> Result<Self, Self::Error> {
        let mut mode = Repl;
        args.next();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--الإصدار" => {
                    mode = Version;
                    break;
                }
                "--ساعد" => {
                    mode = Help;
                    break;
                }
                "--ملف" => {
                    if let Some(path) = args.next() {
                        mode = Mode::new_file(path)?;
                    } else {
                        return Err("توقعت مسار الملف بعد --ملف".to_string());
                    }
                }
                "--غير-موثوق" => match &mut mode {
                    File { untrusted, .. } => *untrusted = true,
                    _ => return Err("استخدام خاطئ ل'--غير موثوق'".to_string()),
                },
                _ => {}
            }
        }
        Ok(mode)
    }
}
