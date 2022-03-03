use std::path::PathBuf;
use std::{env, fs};

fn main() {
    let mut args = Vec::new();

    for arg in env::args().skip(1) {
        args.push(arg);
    }

    if args.len() == 0 {
        eprintln!("Usage: sf [ FILENAME ]");
        std::process::exit(1);
    } else if args.len() > 1 {
        eprintln!("Too many arguments");
        std::process::exit(1);
    }

    let current_path = env::current_dir().unwrap();
    if !file_in_dir(&current_path, &args) {
        eprintln!("Your file doesn`t exist in the current directory");
    }
}

fn file_in_dir(dir: &PathBuf, parameters: &Vec<String>) -> bool {
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
        return true;
    } else {
        return false;
    }
}
