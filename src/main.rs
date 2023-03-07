// TODO limit recursion???
#![recursion_limit = "250"]

use clap::{Arg, ArgAction, Command};
use colored::*;
use flexi_logger::{detailed_format, Duplicate, FileSpec, Logger};
use log::error;

use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process,
};

fn main() {
    // handle Ctrl+C
    ctrlc::set_handler(move || {
        println!(
            "{} {} {}",
            "🤬",
            "Received Ctrl-C! => Exit program!".bold().yellow(),
            "☠",
        );
        process::exit(0)
    })
    .expect("Error setting Ctrl-C handler");

    // get config dir
    let config_dir = check_create_config_dir().unwrap_or_else(|err| {
        error!("Unable to find or create a config directory: {err}");
        process::exit(1);
    });

    // initialize the logger
    let _logger = Logger::try_with_str("info") // log warn and error
        .unwrap()
        .format_for_files(detailed_format) // use timestamp for every log
        .log_to_file(
            FileSpec::default()
                .directory(&config_dir)
                .suppress_timestamp(),
        ) // change directory for logs, no timestamps in the filename
        .append() // use only one logfile
        .duplicate_to_stderr(Duplicate::Info) // print infos, warnings and errors also to the console
        .start()
        .unwrap();

    // handle arguments
    let matches = sf().get_matches();
    let file_flag = matches.get_flag("file");
    let dir_flag = matches.get_flag("dir");
    if let Some(args) = matches
        .get_many::<String>("args")
        .map(|a| a.collect::<Vec<_>>())
    {
        let pattern = args[0];
        let path = Path::new(&args[1]).to_path_buf();

        let mut exclude_patterns = Vec::new();
        match matches.subcommand() {
            Some(("exclude", sub_matches)) => {
                if let Some(mut args) = sub_matches
                    .get_many::<String>("exclude")
                    .map(|a| a.collect::<Vec<_>>())
                {
                    exclude_patterns.append(&mut args);

                    if let Err(err) = search(pattern, &path, &exclude_patterns, file_flag, dir_flag)
                    {
                        error!(
                            "Unable to find anything with {} in {}: {}",
                            pattern,
                            path.display(),
                            err
                        );
                        process::exit(1);
                    }
                } else {
                    error!("Error while trying to get patterns to exclude");
                    process::exit(1);
                }
            }
            _ => {
                if let Err(err) = search(pattern, &path, &exclude_patterns, file_flag, dir_flag) {
                    error!(
                        "Unable to find anything with {} in {}: {}",
                        pattern,
                        path.display(),
                        err
                    );
                    process::exit(1);
                }
            }
        }
    } else {
        match matches.subcommand() {
            Some(("log", _)) => {
                if let Ok(logs) = show_log_file(&config_dir) {
                    println!("{}", "Available logs:".bold().yellow());
                    println!("{}", logs);
                } else {
                    error!("Unable to read logs");
                    process::exit(1);
                }
            }
            _ => {
                unreachable!();
            }
        }
    }
}

fn sf() -> Command {
    Command::new("sf")
        .bin_name("sf")
        .before_help(format!(
            "{}\n{}",
            "SIMPLE FIND".bold().truecolor(250, 0, 104),
            "Leann Phydon <leann.phydon@gmail.com>".italic().dimmed()
        ))
        .about("Simple file search")
        .before_long_help(format!(
            "{}\n{}",
            "SIMPLE FIND".bold().truecolor(250, 0, 104),
            "Leann Phydon <leann.phydon@gmail.com>".italic().dimmed()
        ))
        .long_about(format!("{}", "Simple file search",))
        .version("1.0.0")
        .author("Leann Phydon <leann.phydon@gmail.com>")
        .arg_required_else_help(true)
        .arg(
            Arg::new("args")
                .help("add a search pattern and a path")
                .action(ArgAction::Set)
                .num_args(2)
                .value_names(["PATTERN", "PATH"]),
        )
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .help("Search only in file names for the pattern")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("dir")
                .short('d')
                .long("dir")
                .help("Search only in directory names for the pattern")
                .action(ArgAction::SetTrue),
        )
        .subcommand(
            Command::new("exclude")
                .short_flag('e')
                .long_flag("exclude")
                .about("Exclude patterns from the search")
                // .next_line_help(true)
                .long_about(format!(
                    "{}\n{}",
                    "Exclude patterns from the search", "Must be provided as the last argument"
                ))
                .arg_required_else_help(true)
                .arg(
                    Arg::new("exclude")
                        .help("Enter patterns to exclude from the search")
                        .action(ArgAction::Set)
                        .num_args(1..)
                        .value_name("PATTERN"),
                ),
        )
        .subcommand(
            Command::new("log")
                .short_flag('L')
                .long_flag("log")
                .about("Show content of the log file"),
        )
}

fn search(
    pattern: &str,
    path: &PathBuf,
    exclude_patterns: &Vec<&String>,
    file_flag: bool,
    dir_flag: bool,
) -> io::Result<()> {
    let mut search_hits = Vec::new();
    forwards_search(
        pattern,
        path,
        &exclude_patterns,
        file_flag,
        dir_flag,
        &mut search_hits,
    )?;
    get_search_hits(search_hits);

    Ok(())
}

fn forwards_search(
    pattern: &str,
    path: &PathBuf,
    exclude_patterns: &Vec<&String>,
    file_flag: bool,
    dir_flag: bool,
    search_hits: &mut Vec<PathBuf>,
) -> io::Result<()> {
    let mut search_path = Path::new(&path).to_path_buf();

    if path.as_path().to_string_lossy().to_string() == "." {
        let current_dir = env::current_dir().unwrap_or_else(|err| {
            error!("Unable to get current directory: {err}");
            process::exit(1);
        });
        search_path.push(current_dir);
    }

    for entry in fs::read_dir(search_path)? {
        let entry = entry?;

        if file_flag && !entry.path().is_file() {
            continue;
        }

        if dir_flag && !entry.path().is_dir() {
            continue;
        }

        let mut name = String::new();
        if let Some(filename) = entry.path().file_name() {
            name.push_str(&filename.to_string_lossy().to_string());
        } else {
            error!("Unable to get the filename of {}", entry.path().display());
        }

        if exclude_patterns.is_empty() {
            if name.contains(pattern) {
                search_hits.push(entry.path());
            }
        } else {
            if name.contains(pattern) && exclude_patterns.iter().all(|&it| !name.contains(it)) {
                search_hits.push(entry.path());
            }
        }

        if entry.path().is_dir() && fs::read_dir(entry.path())?.count() != 0 {
            let mut entry_path = entry.path().as_path().to_string_lossy().to_string();
            entry_path.push_str("\\");
            let path = Path::new(&entry_path);
            forwards_search(
                pattern,
                &path.to_path_buf(),
                &exclude_patterns,
                file_flag,
                dir_flag,
                search_hits,
            )?;
        }
    }

    Ok(())
}

fn get_search_hits(search_hits: Vec<PathBuf>) {
    for hit in &search_hits {
        let parent = hit
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .to_string_lossy()
            .to_string();

        let mut name = String::new();
        if let Some(filename) = hit.file_name() {
            name.push_str(&filename.to_string_lossy().to_string());
            println!("{}\\{}", parent, name.bright_green());
        } else {
            // TODO remove? how to handle this error?
            // error!("Unable to get the filename of {}", hit.display());
            println!("{}", hit.display());
        }
    }

    if search_hits.len() == 0 {
        println!(
            "found {} matches",
            search_hits.len().to_string().red().bold()
        );
    } else if search_hits.len() == 1 {
        println!(
            "\nfound {} match",
            search_hits.len().to_string().bright_green().bold()
        );
    } else {
        println!(
            "\nfound {} matches",
            search_hits.len().to_string().bright_green().bold()
        );
    }
}

fn check_create_config_dir() -> io::Result<PathBuf> {
    let mut new_dir = PathBuf::new();
    match dirs::config_dir() {
        Some(config_dir) => {
            new_dir.push(config_dir);
            new_dir.push("sf");
            if !new_dir.as_path().exists() {
                fs::create_dir(&new_dir)?;
            }
        }
        None => {
            error!("Unable to find config directory");
        }
    }

    Ok(new_dir)
}

fn show_log_file(config_dir: &PathBuf) -> io::Result<String> {
    let log_path = Path::new(&config_dir).join("sf.log");
    match log_path.try_exists()? {
        true => {
            return Ok(format!(
                "{} {}\n{}",
                "Log location:".italic().dimmed(),
                &log_path.display(),
                fs::read_to_string(&log_path)?
            ));
        }
        false => {
            return Ok(format!(
                "{} {}",
                "No log file found:".red().bold().to_string(),
                log_path.display()
            ))
        }
    }
}

// fn backwards_search() {
//     let mut args = Vec::new();

//     for arg in env::args().skip(1) {
//         args.push(arg);
//     }

//     if args.is_empty() {
//         eprintln!("Usage: sf [PATTERN] <FLAGS>");
//         eprintln!("type \"sf -h\" or \"sf --help\" to show the help menu");
//         std::process::exit(1);
//     } else if args.len() > 2 {
//         eprintln!("Too many arguments");
//         std::process::exit(1);
//     }

//     let current_path = env::current_dir().unwrap();

//     if args.len() == 1 && args.contains(&String::from("--help"))
//         || args.contains(&String::from("-h"))
//     {
//         todo!();
//     } else if args.len() == 1 && args.contains(&String::from("--version"))
//         || args.contains(&String::from("-V"))
//     {
//         todo!();
//     } else if args.len() == 1 {
//         let result = file_in_dir(&current_path, &args);
//         if !result {
//             let mut parent_iterator = Path::new(&current_path).ancestors();
//             loop {
//                 let parent = parent_iterator.next();
//                 if parent == None {
//                     eprintln!("File {:?} not found", &args.get(0).unwrap());
//                     break;
//                 }

//                 let target = file_in_dir(&parent.unwrap(), &args);
//                 if target {
//                     break;
//                 }
//             }
//         }
//     } else if args.len() > 1 && args.contains(&String::from("-a"))
//         || args.contains(&String::from("--all"))
//     {
//         let mut parent_iterator = Path::new(&current_path).ancestors();
//         let mut file_storage: Vec<u8> = Vec::new();
//         loop {
//             let parent = parent_iterator.next();
//             if parent != None {
//                 let target = file_in_dir(&parent.unwrap(), &args);
//                 if target {
//                     file_storage.push(1);
//                 }
//             } else {
//                 if file_storage.is_empty() {
//                     eprintln!("File {:?} not found", &args.get(0).unwrap());
//                 }
//                 break;
//             }
//         }
//     } else {
//         eprintln!("Invalid argument given");
//     }
// }

// fn file_in_dir(dir: &Path, parameters: &[String]) -> bool {
//     let mut file_container: Vec<String> = Vec::new();

//     // list all filepaths in current directory
//     for entry in fs::read_dir(&dir).unwrap() {
//         let entry = entry.unwrap().path();
//         // println!("entry = {}", entry.display());

//         // get file name with extension
//         let file = entry.file_name().unwrap();

//         // convert to string and lowercase
//         let filename = file.to_str().unwrap();
//         let filename_lowercase = filename.to_lowercase();

//         // if pattern in current filename, print file path
//         if entry.is_file() && filename.contains(&parameters[0])
//             || entry.is_file() && filename_lowercase.contains(&parameters[0])
//         {
//             let path_str = entry.to_str().unwrap();
//             file_container.push(path_str.to_string());
//         }
//     }

//     if file_container.is_empty() {
//         false
//     } else {
//         file_container.sort();
//         for f in &file_container {
//             println!("{:}", f);
//         }
//         true
//     }
// }
