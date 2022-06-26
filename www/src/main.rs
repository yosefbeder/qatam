#[macro_use]
extern crate rocket;
use rocket::{
    fs::FileServer,
    serde::{json::Json, Deserialize, Serialize},
};
use std::{fs, io::prelude::*, time::Duration};
use subprocess::{Popen, PopenConfig, Redirection};

#[derive(Deserialize, Debug)]
struct Req {
    code: String,
}

#[derive(Serialize, Debug)]
struct Res {
    success: bool,
    stdout: String,
    stderr: String,
}

#[post("/execute", format = "json", data = "<req>")]
fn execute(req: Json<Req>) -> Json<Res> {
    let path = "تجربة.قتام";
    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(path)
        .unwrap();
    file.write_all(req.code.as_bytes()).unwrap();
    let mut p = Popen::create(
        &["target/release/قتام", "--ملف", path, "--غير-موثوق"],
        PopenConfig {
            stdout: Redirection::Pipe,
            stderr: Redirection::Pipe,
            ..PopenConfig::default()
        },
    )
    .unwrap();

    let success;

    if let Some(status) = p.wait_timeout(Duration::from_secs(1)).unwrap() {
        success = status.success();
    } else {
        p.kill().unwrap();
        success = false;
    }

    let mut stdout = String::new();
    let mut stderr = String::new();

    p.stdout
        .as_ref()
        .unwrap()
        .read_to_string(&mut stdout)
        .unwrap();
    p.stderr
        .as_ref()
        .unwrap()
        .read_to_string(&mut stderr)
        .unwrap();

    fs::remove_file(path).unwrap();

    Json::from(Res {
        success,
        stdout,
        stderr,
    })
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![execute])
        .mount("/", FileServer::from("www/public"))
}
