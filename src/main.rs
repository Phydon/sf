// TODO check loop -> still panics sometimes
// TODO add deppsearch
// TODO add forward search
use std::path::Path;
use std::{env, fs};

fn main() {
    let mut args = Vec::new();

    for arg in env::args().skip(1) {
        args.push(arg);
    }

    if args.is_empty() {
        eprintln!("Usage: sf [ FILENAME ]");
        std::process::exit(1);
    } else if args.len() > 1 {
        eprintln!("Too many arguments");
        std::process::exit(1);
    }

    let current_path = env::current_dir().unwrap();
    let result = file_in_dir(&current_path, &args);
    if !result {
        // TODO still panics sometimes
        let mut parent = Path::new(&current_path).ancestors();
        loop {
            let checker = parent.next();
            if checker == None { break; }

            let target = file_in_dir(&checker.unwrap(), &args);
            if target { break; }
        }
        println!("File {:?} not found", &args.pop().unwrap());
    }
}

fn file_in_dir(dir: &Path, parameters: &[String]) -> bool {
    let mut counter: u32 = 0;
    // list all files in current directory
    for entry in fs::read_dir(&dir).unwrap() {
        let entry = entry.unwrap().path();
        // println!("entry = {}", entry.display());

        // if argument in current directory, print path
        let path_str = entry.to_str().unwrap();
        if path_str.contains(&parameters[0]) {
            counter += 1;
            println!("=> {:?}", path_str);
        }
    }
    if counter != 0 {
        true
    } else {
        // eprintln!("Your file doesn`t exist in the current directory");
        false
    }
}
