mod cli_reporter;
use cli_reporter::CliReporter;
use qatam::run;
use std::fs;

fn main() {
    let mut args = std::env::args().skip(1);
    let subcommand = args.next().expect("توقعت أمراً الفرعية");
    match subcommand.as_str() {
        "run" => {
            let path = args.next().expect("توقعت مسار الملف");
            if args.next().is_some() {
                panic!("عدد غير متوقع من المدخلات");
            }
            let source = fs::read_to_string(&path).expect("فشلت في قراءة الملف");
            let mut cli_reporter = CliReporter::new();
            run(&source, &mut cli_reporter);
        }
        "help" => {
            println!("{}", fs::read_to_string("help.txt").unwrap());
        }
        _ => panic!("أمر فرعي غير متوقع"),
    }
}
