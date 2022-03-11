// TODO add deppsearch
// TODO add forward search
// TODO remove case sensitivity

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
        let mut parent = Path::new(&current_path).ancestors();
        loop {
            let checker = parent.next();

            if checker == None { 
                println!("File {:?} not found", &args.pop().unwrap());
                break;
            }

            let target = file_in_dir(&checker.unwrap(), &args);
            if target { break; }
        }
    }
}

fn file_in_dir(dir: &Path, parameters: &[String]) -> bool {
    let mut file_container: Vec<String> = Vec::new();
    
    // list all filepaths in current directory
    for entry in fs::read_dir(&dir).unwrap() {
        let entry = entry.unwrap().path();
        // println!("entry = {}", entry.display());

        // get file name with extension
        let file = entry.file_name().unwrap();

        // convert to string
        let filename = file.to_str().unwrap();

        // if argument in current filename, print file path
        if filename.contains(&parameters[0]) && !entry.is_dir() {
            let path_str = entry.to_str().unwrap();
            file_container.push(path_str.to_string());
        }
    }

    if file_container.is_empty() {
        false
    } else {
        file_container.sort();
        for f in file_container {
            println!("=> {:}", f);
        }
        true
    }
}
