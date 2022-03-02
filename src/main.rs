use std::{env, fs};
// extern crate walkdir;
// use walkdir::Walkdir;

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

    // let recursive_dir = Walkdir::new(current_path);
    // for entry in recursive_dir {
    //     println!("{}", entry.unwrap().path().display());
    // }

    // list all files in current directory
    for entry in fs::read_dir(&current_path).unwrap() {
        let entry = entry.unwrap().path();
        // println!("entry = {}", entry.display());

        // if argument in current directory, print path
        let path_str = entry.to_str().unwrap();
        if path_str.contains(&args[0]) {
            println!(
                "You can find the file {:?} in here: [ {:?} ]",
                args[0], path_str
            );
        }
    }
    // TODO 
    eprintln!("Your file doesn`t exists in the current directory");
}
