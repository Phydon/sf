use std::path::Path;
use std::{env, fs};

fn main() {
    println!("Hello");
}

fn backwards_search() {
    let mut args = Vec::new();

    for arg in env::args().skip(1) {
        args.push(arg);
    }

    if args.is_empty() {
        eprintln!("Usage: sf [PATTERN] <FLAGS>");
        eprintln!("type \"sf -h\" or \"sf --help\" to show the help menu");
        std::process::exit(1);
    } else if args.len() > 2 {
        eprintln!("Too many arguments");
        std::process::exit(1);
    }

    let current_path = env::current_dir().unwrap();

    if args.len() == 1 && args.contains(&String::from("--help"))
        || args.contains(&String::from("-h"))
    {
        todo!();
    } else if args.len() == 1 && args.contains(&String::from("--version"))
        || args.contains(&String::from("-V"))
    {
        todo!();
    } else if args.len() == 1 {
        let result = file_in_dir(&current_path, &args);
        if !result {
            let mut parent_iterator = Path::new(&current_path).ancestors();
            loop {
                let parent = parent_iterator.next();
                if parent == None {
                    eprintln!("File {:?} not found", &args.get(0).unwrap());
                    break;
                }

                let target = file_in_dir(&parent.unwrap(), &args);
                if target {
                    break;
                }
            }
        }
    } else if args.len() > 1 && args.contains(&String::from("-a"))
        || args.contains(&String::from("--all"))
    {
        let mut parent_iterator = Path::new(&current_path).ancestors();
        let mut file_storage: Vec<u8> = Vec::new();
        loop {
            let parent = parent_iterator.next();
            if parent != None {
                let target = file_in_dir(&parent.unwrap(), &args);
                if target {
                    file_storage.push(1);
                }
            } else {
                if file_storage.is_empty() {
                    eprintln!("File {:?} not found", &args.get(0).unwrap());
                }
                break;
            }
        }
    } else {
        eprintln!("Invalid argument given");
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

        // convert to string and lowercase
        let filename = file.to_str().unwrap();
        let filename_lowercase = filename.to_lowercase();

        // if pattern in current filename, print file path
        if entry.is_file() && filename.contains(&parameters[0])
            || entry.is_file() && filename_lowercase.contains(&parameters[0])
        {
            let path_str = entry.to_str().unwrap();
            file_container.push(path_str.to_string());
        }
    }

    if file_container.is_empty() {
        false
    } else {
        file_container.sort();
        for f in &file_container {
            println!("{:}", f);
        }
        true
    }
}
