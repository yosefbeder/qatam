use super::Frame;
use compiler::value::{Arity, ArityType::*, Native, Object, Value};
use rand::prelude::*;
use std::fs::{self, OpenOptions};
use std::io::{prelude::*, stdin};
use std::{cell::RefCell, collections::HashMap, process, rc::Rc, time::SystemTime};

pub fn as_string(frame: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Fixed, 1, 0), argc)?;
    let arg = frame.nth(1);
    Ok(Value::new_string(arg.to_string()))
}

pub fn as_unicode(frame: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Fixed, 1, 0), argc)?;
    let arg = frame.nth_u32(1)?;
    match std::char::from_u32(arg as u32) {
        Some(c) => Ok(Value::new_string(c.to_string())),
        None => Err(Value::new_string(format!(
            "لا يوجد حرف مقترن مع الرقم {arg}"
        ))),
    }
}

pub fn as_unicode_number(frame: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Fixed, 1, 0), argc)?;
    let arg = frame.nth_char(1)?;
    let n: u32 = arg.into();
    Ok(Value::Number(n as f64))
}

pub fn as_number(frame: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Fixed, 1, 0), argc)?;
    let arg = frame.nth_string(1)?;
    let n: f64 = arg
        .parse()
        .map_err(|_| Value::new_string(format!("{} ليس عدداً", arg)))?;
    Ok(Value::Number(n))
}

pub fn as_int(frame: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Fixed, 1, 0), argc)?;
    match frame.nth(1) {
        Value::Object(Object::String(string)) => {
            let n: u32 = string
                .parse()
                .map_err(|_| Value::new_string(format!("{} ليس عدداً صحيحاً", string)))?;
            Ok(Value::Number(n as f64))
        }
        Value::Number(n) => Ok(Value::Number(n.trunc() as f64)),
        _ => Err(Value::new_string(
            "يجب أن يكون المدخل عدداً أو نصاً".to_string(),
        )),
    }
}

pub fn typ(frame: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Fixed, 1, 0), argc)?;
    let arg = frame.nth(1);
    Ok(Value::new_string(arg.get_type().to_string()))
}

pub fn size(frame: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Fixed, 1, 0), argc)?;
    let arg = frame.nth(1);
    Ok(Value::Number(match arg {
        Value::Object(Object::List(items)) => items.borrow().len(),
        Value::Object(Object::String(string)) => string.chars().count(),
        _ => {
            return Err(Value::new_string(
                "يجب أن يكون المدخل قائمةً أو كائناً أو نصاً".to_string(),
            ))
        }
    } as f64))
}

pub fn props(frame: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Fixed, 1, 0), argc)?;
    let arg = frame.nth_object(1)?.borrow();
    let mut res = vec![];
    let mut entries = arg.iter().collect::<Vec<_>>();
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    for (key, value) in entries.into_iter() {
        let mut prop = Vec::with_capacity(2);
        prop.push(Value::new_string(key.to_string()));
        prop.push(value.clone());
        res.push(Value::new_list(prop));
    }

    Ok(Value::new_list(res))
}

pub fn push(frame: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Variadic, 2, 0), argc)?;
    let list = frame.nth_list(1)?;
    for idx in 2..=argc {
        list.borrow_mut().push(frame.nth(idx).clone());
    }
    Ok(Value::Nil)
}

pub fn pop(frame: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Fixed, 1, 0), argc)?;
    let list = frame.nth_list(1)?;
    let item = list.borrow_mut().pop();
    match item {
        Some(item) => Ok(item),
        None => Err(Value::new_string("لا يوجد عنصر لإزالته".to_string())),
    }
}

pub fn rand(_: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Fixed, 0, 0), argc)?;
    let mut rng = rand::thread_rng();
    Ok(Value::Number(rng.gen::<f64>()))
}

pub fn sin(frame: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Fixed, 1, 0), argc)?;
    let n = frame.nth_f64(1)?;
    Ok(Value::Number(n.sin()))
}

pub fn cos(frame: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Fixed, 1, 0), argc)?;
    let n = frame.nth_f64(1)?;
    Ok(Value::Number(n.cos()))
}

pub fn tan(frame: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Fixed, 1, 0), argc)?;
    let n = frame.nth_f64(1)?;
    Ok(Value::Number(n.tan()))
}

pub fn csc(frame: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Fixed, 1, 0), argc)?;
    let n = frame.nth_f64(1)?;
    Ok(Value::Number(1.0 / n.sin()))
}

pub fn sec(frame: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Fixed, 1, 0), argc)?;
    let n = frame.nth_f64(1)?;
    Ok(Value::Number(1.0 / n.cos()))
}

pub fn cot(frame: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Fixed, 1, 0), argc)?;
    let n = frame.nth_f64(1)?;
    Ok(Value::Number(1.0 / n.tan()))
}

pub fn pow(frame: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Fixed, 2, 0), argc)?;
    let n = frame.nth_f64(1)?;
    let power = frame.nth_f64(2)?;
    Ok(Value::Number(n.powf(power)))
}

pub fn log(frame: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Fixed, 2, 0), argc)?;
    let n = frame.nth_f64(1)?;
    let base = frame.nth_f64(2)?;
    Ok(Value::Number(n.log(base)))
}

pub fn args(_: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Fixed, 0, 0), argc)?;
    let res: Vec<_> = std::env::args().collect();
    Ok(Value::new_list(
        res.iter()
            .map(|arg| Value::new_string(arg.to_string()))
            .collect(),
    ))
}

pub fn env(frame: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Fixed, 0, 0), argc)?;
    frame.check_trust()?;
    let mut res = HashMap::new();

    for (key, value) in std::env::vars() {
        res.insert(key, Value::new_string(value));
    }

    Ok(Value::new_object(res))
}

pub fn time(frame: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Fixed, 0, 0), argc)?;
    let now = SystemTime::now();
    let duration = now.duration_since(*frame.get_creation_time()).unwrap();
    Ok(Value::Number(duration.as_millis() as f64))
}

pub fn exit(frame: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Fixed, 1, 0), argc)?;
    let code = frame.nth_i32(1)?;
    process::exit(code as i32);
}

pub fn print(frame: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Variadic, 1, 0), argc)?;
    for idx in 1..=argc {
        let arg = frame.nth(idx);
        println!("{}", arg);
    }
    Ok(Value::Nil)
}

pub fn input(_: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Fixed, 0, 0), argc)?;
    let mut buffer = String::new();
    stdin().read_line(&mut buffer).unwrap();
    buffer.pop();
    Ok(Value::new_string(buffer))
}

pub fn create(frame: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Fixed, 1, 0), argc)?;
    frame.check_trust()?;
    let path = frame.nth_path(1)?;
    let mut open_options = OpenOptions::new();
    open_options.write(true).create_new(true);

    match open_options.open(&path) {
        Ok(file) => Ok(Value::Object(Object::File(Rc::new(RefCell::new(file))))),
        Err(err) => Err(Value::new_string(format!("{err}"))),
    }
}

pub fn create_folder(frame: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Fixed, 1, 0), argc)?;
    frame.check_trust()?;
    let path = frame.nth_path(1)?;
    match fs::create_dir(&path) {
        Ok(_) => Ok(Value::Nil),
        Err(err) => Err(Value::new_string(format!("{err}"))),
    }
}

pub fn open(frame: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Fixed, 2, 0), argc)?;
    frame.check_trust()?;
    let path = frame.nth_path(1)?;
    let mode = frame.nth_string(2)?;
    let mut open_options = OpenOptions::new();

    match mode {
        "قراءة" => open_options.read(true),
        "كتابة" => open_options.write(true),
        "أي شئ" => open_options.read(true).write(true),
        _ => {
            return Err(Value::new_string(
                "يمكن أن يكون الوضع \"قراءة\" أو \"كتابة\" أو \"أي شئ\"".to_string(),
            ))
        }
    };

    match open_options.open(&path) {
        Ok(file) => Ok(Value::new_file(file)),
        Err(err) => Err(Value::new_string(format!("{err}"))),
    }
}

pub fn read(frame: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Fixed, 1, 0), argc)?;
    frame.check_trust()?;
    let path = frame.nth_file(1)?;
    let mut buffer = String::new();
    path.borrow_mut().read_to_string(&mut buffer).unwrap();
    Ok(Value::new_string(buffer))
}

pub fn read_folder(frame: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Fixed, 1, 0), argc)?;
    frame.check_trust()?;
    let path = frame.nth_path(1)?;

    match path.read_dir() {
        Ok(dir) => {
            let mut res = vec![];
            for entry in dir {
                res.push(Value::new_string(
                    entry.unwrap().path().to_str().unwrap().to_string(),
                ));
            }
            Ok(Value::new_list(res))
        }
        Err(err) => Err(Value::new_string(format!("{err}"))),
    }
}

pub fn write(frame: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Fixed, 2, 0), argc)?;
    frame.check_trust()?;
    let path = frame.nth_file(1)?;
    let content = frame.nth_string(2)?;
    path.borrow_mut().write_all(content.as_bytes()).unwrap();
    Ok(Value::Nil)
}

pub fn move_(frame: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Fixed, 2, 0), argc)?;
    frame.check_trust()?;
    let old_path = frame.nth_string(1)?;
    let new_path = frame.nth_string(2)?;
    match fs::rename(&old_path, &new_path) {
        Ok(_) => Ok(Value::Nil),
        Err(err) => Err(Value::new_string(format!("{err}"))),
    }
}

pub fn delete(frame: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Fixed, 1, 0), argc)?;
    frame.check_trust()?;
    let path = frame.nth_path(1)?;
    match fs::remove_file(&path) {
        Ok(_) => Ok(Value::Nil),
        Err(err) => Err(Value::new_string(format!("{err}"))),
    }
}

pub fn delete_folder(frame: &dyn compiler::value::Frame, argc: usize) -> Result<Value, Value> {
    Frame::check_arity(Arity::new(Fixed, 1, 0), argc)?;
    frame.check_trust()?;
    let path = frame.nth_path(1)?;
    match fs::remove_dir(&path) {
        Ok(_) => Ok(Value::Nil),
        Err(err) => Err(Value::new_string(format!("{err}"))),
    }
}

pub const NATIVES: [(&'static str, Native); 34] = [
    ("كنص", as_string),
    ("كيونيكود", as_unicode),
    ("كعدد_يونيكود", as_unicode_number),
    ("كعدد", as_number),
    ("كصحيح", as_int),
    ("نوع", typ),
    ("حجم", size),
    ("خصائص", props),
    ("إدفع", push),
    ("إسحب", pop),
    ("عشوائي", rand),
    ("جا", sin),
    ("جتا", cos),
    ("ظا", tan),
    ("قتا", csc),
    ("قا", sec),
    ("ظتا", cot),
    ("رفع", pow),
    ("لوغاريتم", log),
    ("المدخلات", args),
    ("البيئة", env),
    ("الوقت", time),
    ("أغلق", exit),
    ("إطبع", print),
    ("أدخل", input),
    ("أنشئ", create),
    ("أنشئ_مجلد", create_folder),
    ("إفتح", open),
    ("إقرأ", read),
    ("إقرأ_مجلد", read_folder),
    ("إكتب", write),
    ("إنقل", move_),
    ("إحذف", delete),
    ("إحذف_مجلد", delete_folder),
];
