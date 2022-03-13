use std::path::Path;
use std::{env, fs};

fn main() {
    let mut args = Vec::new();

    for arg in env::args().skip(1) {
        args.push(arg);
    }

    if args.is_empty() {
        eprintln!("Usage: sf [FILENAME] <FLAGS>");
        std::process::exit(1);
    } else if args.len() > 2 {
        eprintln!("Too many arguments");
        std::process::exit(1);
    }

    let current_path = env::current_dir().unwrap();

    if args.len() == 1 && args.contains(&String::from("--help")) {
        // TODO add helpful information
        help_flag();
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
    } else if args.len() > 1 && args.contains(&String::from("-a")) {
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
        if entry.is_file() && filename.contains(&parameters[0]) || entry.is_file() && filename_lowercase.contains(&parameters[0]) {
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

fn help_flag() {
    println!("\nSimpleFind => fast and simple recursive file search");
    println!("---------------------------------------------------\n");
    println!("      USAGE:      sf [PATTERN] <FLAGS>\n");
    println!("DESCRIPTION\n");
    println!("Searches for the given PATTERN in filenames. If there`s a match, it stops and returns all files with the PATTERN from that directory. If there is no match, it searches in the parent directory and so on until it reaches root.");
    println!("You can change this behavior with FLAGS.\n");
    println!("PATTERN:      the filename (or parts of it) you want to search for\n");
    println!("FLAGS: ");
    println!("              -a          =>  recursive search in all directories till root");
    println!("              --help      =>  get help");
}
