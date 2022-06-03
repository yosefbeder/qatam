use super::{
    value::{Arity, Native, Value},
    vm::Vm,
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

pub fn as_string(vm: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let arg = vm.get_any(1, argc);
    Ok(Value::String(arg.to_string()))
}

pub fn as_char(vm: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let arg = vm.get_pos_int(1, argc)?;
    match std::char::from_u32(arg as u32) {
        Some(c) => Ok(Value::String(c.to_string())),
        None => Err(format!("لا يوجد حرف مقترن مع الرقم {arg}")),
    }
}

pub fn as_number(vm: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let arg = vm.get_char(1, argc)?;
    let n: u32 = arg.into();
    Ok(Value::Number(n as f64))
}

pub fn parse_number(vm: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let arg = vm.get_string(1, argc)?;
    let n: f64 = arg.parse().map_err(|_| format!("{} ليس رقم", arg))?;
    Ok(Value::Number(n))
}

pub fn typ(vm: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let arg = vm.get_any(1, argc);
    Ok(Value::String(arg.get_type().to_string()))
}

pub fn size(vm: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let arg = vm.get_any(1, argc);
    Ok(Value::Number(match arg {
        Value::List(items) => items.borrow().len(),
        Value::String(string) => string.chars().count(),
        Value::Object(items) => items.borrow().len(),
        _ => return Err("يجب أن يكون المدخل قائمةً أو كائناً أو نصاً".to_string()),
    } as f64))
}

pub fn props(vm: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let arg = vm.get_object(1, argc)?;
    let mut props = Vec::new();

    for (key, value) in arg.borrow().iter() {
        let mut prop = Vec::with_capacity(2);
        prop.push(Value::String(key.to_string()));
        prop.push(value.clone());
        props.push(Value::List(Rc::new(RefCell::new(prop))));
    }

    Ok(Value::List(Rc::new(RefCell::new(props))))
}

pub fn push(vm: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Variadic(2), argc)?;
    let list = vm.get_list(1, argc)?;
    for idx in 2..=argc {
        list.borrow_mut().push(vm.get_any(idx, argc));
    }
    Ok(Value::Nil)
}

pub fn pop(vm: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let list = vm.get_list(1, argc)?;
    let item = list.borrow_mut().pop();
    match item {
        Some(item) => Ok(item),
        None => Err("لا يوجد عنصر لإزالته".to_string()),
    }
}

pub fn rand(_: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Fixed(0), argc)?;
    let mut rng = rand::thread_rng();
    Ok(Value::Number(rng.gen::<f64>()))
}

pub fn sin(vm: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let n = vm.get_number(1, argc)?;
    Ok(Value::Number(n.sin()))
}

pub fn cos(vm: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let n = vm.get_number(1, argc)?;
    Ok(Value::Number(n.cos()))
}

pub fn tan(vm: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let n = vm.get_number(1, argc)?;
    Ok(Value::Number(n.tan()))
}

pub fn csc(vm: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let n = vm.get_number(1, argc)?;
    Ok(Value::Number(1.0 / n.sin()))
}

pub fn sec(vm: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let n = vm.get_number(1, argc)?;
    Ok(Value::Number(1.0 / n.cos()))
}

pub fn cot(vm: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let n = vm.get_number(1, argc)?;
    Ok(Value::Number(1.0 / n.tan()))
}

pub fn pow(vm: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Fixed(2), argc)?;
    let n = vm.get_number(1, argc)?;
    let power = vm.get_number(2, argc)?;
    Ok(Value::Number(n.powf(power)))
}

pub fn log(vm: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Fixed(2), argc)?;
    let n = vm.get_number(1, argc)?;
    let base = vm.get_number(2, argc)?;
    Ok(Value::Number(n.log(base)))
}

pub fn args(_: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Fixed(0), argc)?;
    let res: Vec<_> = std::env::args().collect();
    Ok(Value::List(Rc::new(RefCell::new(
        res.iter()
            .map(|arg| Value::String(arg.to_string()))
            .collect(),
    ))))
}

pub fn env(_: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Fixed(0), argc)?;
    let mut res = HashMap::new();

    for (key, value) in std::env::vars() {
        res.insert(key, Value::String(value));
    }

    Ok(Value::Object(Rc::new(RefCell::new(res))))
}

pub fn time(vm: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Fixed(0), argc)?;
    let now = SystemTime::now();
    let duration = now.duration_since(vm.created_at).unwrap();
    Ok(Value::Number(duration.as_millis() as f64))
}

pub fn exit(vm: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let code = vm.get_int(1, argc)?;
    process::exit(code as i32);
}

pub fn print(vm: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Variadic(1), argc)?;
    for i in 1..=argc {
        let arg = vm.get_any(i, argc);
        println!("{}", arg);
    }
    Ok(Value::Nil)
}

pub fn input(_: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Fixed(0), argc)?;
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();
    buffer.pop();
    Ok(Value::String(buffer))
}

pub fn create(vm: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let path = vm.get_path(1, argc)?;
    let mut open_options = OpenOptions::new();
    open_options.write(true).create_new(true);

    match open_options.open(&path) {
        Ok(file) => Ok(Value::File(Rc::new(RefCell::new(file)))),
        Err(_) => Err("لا يمكن إنشاء الملف".to_string()),
    }
}

pub fn create_folder(vm: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let path = vm.get_path(1, argc)?;
    match fs::create_dir(&path) {
        Ok(_) => Ok(Value::Nil),
        Err(_) => Err("لا يمكن إنشاء المجلد".to_string()),
    }
}

pub fn open(vm: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Fixed(2), argc)?;
    let path = vm.get_path(1, argc)?;
    let mode = vm.get_string(2, argc)?;
    let mut open_options = OpenOptions::new();

    match mode.as_str() {
        "قراءة" => open_options.read(true),
        "كتابة" => open_options.write(true),
        "أي شئ" => open_options.read(true).write(true),
        _ => return Err("يمكن أن يكون الوضع \"قراءة\" أو \"كتابة\" أو \"أي شئ\"".to_string()),
    };

    match open_options.open(&path) {
        Ok(file) => Ok(Value::File(Rc::new(RefCell::new(file)))),
        Err(_) => Err("لا يمكن فتح الملف".to_string()),
    }
}

pub fn read(vm: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let file = vm.get_file(1, argc)?;
    let mut buffer = String::new();
    file.borrow_mut().read_to_string(&mut buffer).unwrap();
    Ok(Value::String(buffer))
}

pub fn read_folder(vm: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let path = vm.get_path(1, argc)?;

    match path.read_dir() {
        Ok(dir) => {
            let mut res = Vec::new();
            for entry in dir {
                res.push(Value::String(
                    entry.unwrap().path().to_str().unwrap().to_string(),
                ));
            }
            Ok(Value::List(Rc::new(RefCell::new(res))))
        }
        Err(_) => Err("لا يمكن قراءة المجلد".to_string()),
    }
}

pub fn write(vm: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Fixed(2), argc)?;
    let file = vm.get_file(1, argc)?;
    let content = vm.get_string(2, argc)?;
    file.borrow_mut().write_all(content.as_bytes()).unwrap();
    Ok(Value::Nil)
}

pub fn move_(vm: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Fixed(2), argc)?;
    let old_path = vm.get_string(1, argc)?;
    let new_path = vm.get_string(2, argc)?;
    match fs::rename(&old_path, &new_path) {
        Ok(_) => Ok(Value::Nil),
        Err(_) => Err("لا يمكن تغيير اسم الملف".to_string()),
    }
}

pub fn delete(vm: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let path = vm.get_path(1, argc)?;
    match fs::remove_file(&path) {
        Ok(_) => Ok(Value::Nil),
        Err(_) => Err("لا يمكن حذف الملف".to_string()),
    }
}

pub fn delete_folder(vm: &Vm, argc: usize) -> Result<Value, String> {
    Vm::check_arity(Arity::Fixed(1), argc)?;
    let path = vm.get_path(1, argc)?;
    match fs::remove_dir(&path) {
        Ok(_) => Ok(Value::Nil),
        Err(_) => Err("لا يمكن حذف المجلد".to_string()),
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
