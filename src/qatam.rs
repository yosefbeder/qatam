use super::value::Value;
use rand::Rng;
use std::{
    cell::RefCell,
    fs, io, process,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn as_string(args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::String(args.get(1).unwrap().to_string()))
}

pub fn as_float(args: Vec<Value>) -> Result<Value, String> {
    match args.get(1).unwrap() {
        Value::String(string) => {
            let float = string.parse::<f64>();
            match float {
                Ok(float) => Ok(Value::Number(float)),
                Err(_) => Err(format!("\"{}\" لا يبدأ بعدد عشري", string)),
            }
        }
        _ => Err("يجب أن يكون المدخل نصاً".to_string()),
    }
}

pub fn as_int(args: Vec<Value>) -> Result<Value, String> {
    match args.get(1).unwrap() {
        Value::String(string) => {
            let int = string.parse::<i64>();
            match int {
                Ok(int) => Ok(Value::Number(int as f64)),
                Err(_) => Err(format!("\"{}\" لا يبدأ بعدد صحيح", string)),
            }
        }
        _ => Err("يجب أن يكون المدخل نصاً".to_string()),
    }
}

pub fn typ(args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::String(args.get(1).unwrap().get_type().to_string()))
}

pub fn size(args: Vec<Value>) -> Result<Value, String> {
    match args.get(1).unwrap() {
        Value::String(string) => Ok(Value::Number(string.chars().count() as f64)), //TODO optimize this!
        Value::List(items) => Ok(Value::Number(items.borrow().len() as f64)),
        Value::Object(items) => Ok(Value::Number(items.borrow().len() as f64)),
        _ => Err("يجب أن يكون المدخل نص أو قائمة أو كائن".to_string()),
    }
}

pub fn properties(args: Vec<Value>) -> Result<Value, String> {
    match args.get(1).unwrap() {
        Value::Object(items) => Ok(Value::List(Rc::new(RefCell::new(
            items
                .borrow()
                .iter()
                .map(|(key, value)| {
                    Value::List(Rc::new(RefCell::new(vec![
                        Value::String(key.to_string()),
                        value.clone(),
                    ])))
                })
                .collect::<Vec<Value>>(),
        )))),
        _ => Err("يجب أن يكون المدخل كائناً".to_string()),
    }
}

pub fn push(args: Vec<Value>) -> Result<Value, String> {
    match args.get(1).unwrap() {
        Value::List(items) => {
            let mut items = items.borrow_mut();
            items.push(args.get(2).unwrap().clone());
            Ok(Value::Nil)
        }
        _ => Err("يجب أن يكون المدخل الأول قائمة".to_string()),
    }
}

pub fn pop(args: Vec<Value>) -> Result<Value, String> {
    match args.get(1).unwrap() {
        Value::List(items) => {
            let mut items = items.borrow_mut();
            if items.len() > 0 {
                Ok(items.pop().unwrap())
            } else {
                Err("لا يمكن إزالة عنصر من قائمة فارغة".to_string())
            }
        }
        _ => Err("يجب أن يكون المدخل الأول قائمة".to_string()),
    }
}

pub fn time(_: Vec<Value>) -> Result<Value, String> {
    Ok(Value::Number(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as f64,
    ))
}

pub fn exit(args: Vec<Value>) -> Result<Value, String> {
    let code = match args.get(1).unwrap() {
        Value::Number(n) => {
            if *n == n.trunc() {
                *n as i32
            } else {
                return Err("يجب أن يكون المدخل عدداً صحيحاً".to_string());
            }
        }
        _ => return Err("يجب أن يكون المدخل عدداً صحيحاً".to_string()),
    };

    process::exit(code);
}

pub fn random(_: Vec<Value>) -> Result<Value, String> {
    let mut rng = rand::thread_rng();

    Ok(Value::Number(rng.gen_range(0.0..1.0)))
}

pub fn read(args: Vec<Value>) -> Result<Value, String> {
    let path = match args.get(1).unwrap() {
        Value::String(string) => string,
        _ => return Err("يجب أن يكون المدخل نصاً".to_string()),
    };

    match fs::read_to_string(&path) {
        Ok(string) => Ok(Value::String(string)),
        Err(err) => Err(format!("لا يمكن قراءة ملف \"{path}\"\n{err}")),
    }
}

pub fn write(args: Vec<Value>) -> Result<Value, String> {
    let path = match args.get(1).unwrap() {
        Value::String(string) => string,
        _ => return Err("يجب أن يكون المدخل الأول نصاً".to_string()),
    };

    let content = args.get(2).unwrap().to_string();

    match fs::write(&path, content) {
        Ok(_) => Ok(Value::Nil),
        Err(err) => Err(format!("لا يمكن كتابة ملف \"{path}\"\n{err}")),
    }
}

pub fn sin(args: Vec<Value>) -> Result<Value, String> {
    let number = match args.get(1).unwrap() {
        Value::Number(n) => n,
        _ => return Err("يجب أن يكون المدخل عدداً".to_string()),
    };

    Ok(Value::Number(number.sin()))
}

pub fn cos(args: Vec<Value>) -> Result<Value, String> {
    let number = match args.get(1).unwrap() {
        Value::Number(n) => n,
        _ => return Err("يجب أن يكون المدخل عدداً".to_string()),
    };

    Ok(Value::Number(number.cos()))
}

pub fn tan(args: Vec<Value>) -> Result<Value, String> {
    let number = match args.get(1).unwrap() {
        Value::Number(n) => n,
        _ => return Err("يجب أن يكون المدخل عدداً".to_string()),
    };

    Ok(Value::Number(number.tan()))
}

pub fn csc(args: Vec<Value>) -> Result<Value, String> {
    let number = match args.get(1).unwrap() {
        Value::Number(n) => n,
        _ => return Err("يجب أن يكون المدخل عدداً".to_string()),
    };

    Ok(Value::Number(1.0 / number.sin()))
}

pub fn sec(args: Vec<Value>) -> Result<Value, String> {
    let number = match args.get(1).unwrap() {
        Value::Number(n) => n,
        _ => return Err("يجب أن يكون المدخل عدداً".to_string()),
    };

    Ok(Value::Number(1.0 / number.cos()))
}

pub fn cot(args: Vec<Value>) -> Result<Value, String> {
    let number = match args.get(1).unwrap() {
        Value::Number(n) => n,
        _ => return Err("يجب أن يكون المدخل عدداً".to_string()),
    };

    Ok(Value::Number(1.0 / number.tan()))
}

pub fn print(args: Vec<Value>) -> Result<Value, String> {
    for arg in args.iter().skip(1) {
        println!("{arg}");
    }
    Ok(Value::Nil)
}

pub fn scan(_: Vec<Value>) -> Result<Value, String> {
    let mut buffer = String::new();
    match io::stdin().read_line(&mut buffer) {
        Ok(_) => {
            buffer.pop();
            Ok(Value::String(buffer))
        }
        Err(e) => Err(format!("حدث خطأ أثناء مسح المدخل من المستخدم\n{}", e)),
    }
}
