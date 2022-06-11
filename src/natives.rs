use super::{
    value::{Arity, Native, Object, Value},
    vm::{Frame, Vm},
};
use rand::prelude::*;
use std::{
    cell::RefCell,
    collections::HashMap,
    fs::{self, OpenOptions},
    io::{self, prelude::*},
    process,
    rc::Rc,
    time::SystemTime,
};

pub fn as_string(frame: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let arg = frame.nth(1);
    Ok(Value::new_string(arg.to_string()))
}

pub fn as_char(frame: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let arg = frame.nth_u32(1)?;
    match std::char::from_u32(arg as u32) {
        Some(c) => Ok(Value::new_string(c.to_string())),
        None => Err(Value::new_string(format!(
            "لا يوجد حرف مقترن مع الرقم {arg}"
        ))),
    }
}

pub fn as_number(frame: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let arg = frame.nth_char(1)?;
    let n: u32 = arg.into();
    Ok(Value::Number(n as f64))
}

pub fn parse_number(frame: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let arg = frame.nth_string(1)?;
    let n: f64 = arg
        .parse()
        .map_err(|_| Value::new_string(format!("{} ليس رقم", arg)))?;
    Ok(Value::Number(n))
}

pub fn typ(frame: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let arg = frame.nth(1);
    Ok(Value::new_string(arg.get_type().to_string()))
}

pub fn size(frame: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let arg = frame.nth(1);
    Ok(Value::Number(match arg {
        Value::Object(Object::List(items)) => items.borrow().len(),
        Value::Object(Object::String(string)) => string.chars().count(),
        Value::Object(Object::Object(items)) => items.borrow().len(),
        _ => {
            return Err(Value::new_string(
                "يجب أن يكون المدخل قائمةً أو كائناً أو نصاً".to_string(),
            ))
        }
    } as f64))
}

pub fn props(frame: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let arg = frame.nth_object(1)?.borrow();
    let mut res = Vec::new();
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

pub fn push(frame: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Variadic(2), argc)?;
    let list = frame.nth_list(1)?;
    for idx in 2..=argc {
        list.borrow_mut().push(frame.nth(idx).clone());
    }
    Ok(Value::Nil)
}

pub fn pop(frame: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let list = frame.nth_list(1)?;
    let item = list.borrow_mut().pop();
    match item {
        Some(item) => Ok(item),
        None => Err(Value::new_string("لا يوجد عنصر لإزالته".to_string())),
    }
}

pub fn rand(_: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Fixed(0), argc)?;
    let mut rng = rand::thread_rng();
    Ok(Value::Number(rng.gen::<f64>()))
}

pub fn sin(frame: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let n = frame.nth_f64(1)?;
    Ok(Value::Number(n.sin()))
}

pub fn cos(frame: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let n = frame.nth_f64(1)?;
    Ok(Value::Number(n.cos()))
}

pub fn tan(frame: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let n = frame.nth_f64(1)?;
    Ok(Value::Number(n.tan()))
}

pub fn csc(frame: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let n = frame.nth_f64(1)?;
    Ok(Value::Number(1.0 / n.sin()))
}

pub fn sec(frame: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let n = frame.nth_f64(1)?;
    Ok(Value::Number(1.0 / n.cos()))
}

pub fn cot(frame: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let n = frame.nth_f64(1)?;
    Ok(Value::Number(1.0 / n.tan()))
}

pub fn pow(frame: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Fixed(2), argc)?;
    let n = frame.nth_f64(1)?;
    let power = frame.nth_f64(2)?;
    Ok(Value::Number(n.powf(power)))
}

pub fn log(frame: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Fixed(2), argc)?;
    let n = frame.nth_f64(1)?;
    let base = frame.nth_f64(2)?;
    Ok(Value::Number(n.log(base)))
}

pub fn args(_: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Fixed(0), argc)?;
    let res: Vec<_> = std::env::args().collect();
    Ok(Value::new_list(
        res.iter()
            .map(|arg| Value::new_string(arg.to_string()))
            .collect(),
    ))
}

pub fn env(_: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Fixed(0), argc)?;
    let mut res = HashMap::new();

    for (key, value) in std::env::vars() {
        res.insert(key, Value::new_string(value));
    }

    Ok(Value::new_object(res))
}

pub fn time(frame: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Fixed(0), argc)?;
    let now = SystemTime::now();
    let duration = now.duration_since(*frame.get_creation_time()).unwrap();
    Ok(Value::Number(duration.as_millis() as f64))
}

pub fn exit(frame: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let code = frame.nth_i32(1)?;
    process::exit(code as i32);
}

pub fn print(frame: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Variadic(1), argc)?;
    for idx in 1..=argc {
        let arg = frame.nth(idx);
        println!("{}", arg);
    }
    Ok(Value::Nil)
}

pub fn input(_: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Fixed(0), argc)?;
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();
    buffer.pop();
    Ok(Value::new_string(buffer))
}

pub fn create(frame: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let path = frame.nth_path(1)?;
    let mut open_options = OpenOptions::new();
    open_options.write(true).create_new(true);

    match open_options.open(&path) {
        Ok(file) => Ok(Value::Object(Object::File(Rc::new(RefCell::new(file))))),
        Err(_) => Err(Value::new_string("لا يمكن إنشاء الملف".to_string())),
    }
}

pub fn create_folder(frame: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let path = frame.nth_path(1)?;
    match fs::create_dir(&path) {
        Ok(_) => Ok(Value::Nil),
        Err(_) => Err(Value::new_string("لا يمكن إنشاء المجلد".to_string())),
    }
}

pub fn open(frame: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Fixed(2), argc)?;
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
        Err(_) => Err(Value::new_string("لا يمكن فتح الملف".to_string())),
    }
}

pub fn read(frame: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let file = frame.nth_file(1)?;
    let mut buffer = String::new();
    file.borrow_mut().read_to_string(&mut buffer).unwrap();
    Ok(Value::new_string(buffer))
}

pub fn read_folder(frame: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let path = frame.nth_path(1)?;

    match path.read_dir() {
        Ok(dir) => {
            let mut res = Vec::new();
            for entry in dir {
                res.push(Value::new_string(
                    entry.unwrap().path().to_str().unwrap().to_string(),
                ));
            }
            Ok(Value::new_list(res))
        }
        Err(_) => Err(Value::new_string("لا يمكن قراءة المجلد".to_string())),
    }
}

pub fn write(frame: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Fixed(2), argc)?;
    let file = frame.nth_file(1)?;
    let content = frame.nth_string(2)?;
    file.borrow_mut().write_all(content.as_bytes()).unwrap();
    Ok(Value::Nil)
}

pub fn move_(frame: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Fixed(2), argc)?;
    let old_path = frame.nth_string(1)?;
    let new_path = frame.nth_string(2)?;
    match fs::rename(&old_path, &new_path) {
        Ok(_) => Ok(Value::Nil),
        Err(_) => Err(Value::new_string("لا يمكن تغيير اسم الملف".to_string())),
    }
}

pub fn delete(frame: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let path = frame.nth_path(1)?;
    match fs::remove_file(&path) {
        Ok(_) => Ok(Value::Nil),
        Err(_) => Err(Value::new_string("لا يمكن حذف الملف".to_string())),
    }
}

pub fn delete_folder(frame: &Frame, argc: usize) -> Result<Value, Value> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let path = frame.nth_path(1)?;
    match fs::remove_dir(&path) {
        Ok(_) => Ok(Value::Nil),
        Err(_) => Err(Value::new_string("لا يمكن حذف المجلد".to_string())),
    }
}

pub const NATIVES: [(&'static str, Native); 33] = [
    ("كنص", as_string),
    ("كحرف", as_char),
    ("كعدد", as_number),
    ("حلل_عدد", parse_number),
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
