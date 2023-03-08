// TODO limit recursion???
#![recursion_limit = "250"]

use clap::{Arg, ArgAction, Command};
use colored::*;
use flexi_logger::{detailed_format, Duplicate, FileSpec, Logger};
use indicatif::{HumanDuration, ProgressBar, ProgressStyle};
use log::{error, warn};

use std::{
    env,
    // ffi::OsStr,
    fs,
    io,
    path::{Path, PathBuf},
    process,
    time::{Duration, Instant},
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
    let performance_flag = matches.get_flag("performance");
    let stats_flag = matches.get_flag("stats");
    if let Some(args) = matches
        .get_many::<String>("args")
        .map(|a| a.collect::<Vec<_>>())
    {
        let pattern = args[0];
        let path = Path::new(&args[1]).to_path_buf();

        let mut ext = Vec::new();
        if let Some(mut extensions) = matches
            .get_many::<String>("extension")
            .map(|a| a.collect::<Vec<_>>())
        {
            ext.append(&mut extensions);
        }

        let mut exclude_patterns = Vec::new();
        match matches.subcommand() {
            Some(("exclude", sub_matches)) => {
                if let Some(mut args) = sub_matches
                    .get_many::<String>("exclude")
                    .map(|a| a.collect::<Vec<_>>())
                {
                    exclude_patterns.append(&mut args);

                    search(
                        pattern,
                        &path,
                        &exclude_patterns,
                        &ext,
                        file_flag,
                        dir_flag,
                        performance_flag,
                        stats_flag,
                    );
                } else {
                    error!("Error while trying to get patterns to exclude");
                    process::exit(1);
                }
            }
            _ => search(
                pattern,
                &path,
                &exclude_patterns,
                &ext,
                file_flag,
                dir_flag,
                performance_flag,
                stats_flag,
            ),
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
        // TODO add more
        .long_about(format!("{}", "Simple file search",))
        // TODO update version
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
            Arg::new("dir")
                .short('d')
                .long("dir")
                .help("Search only in directory names for the pattern")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("extension")
                .short('e')
                .long("extension")
                .help("Only search in files with the given extensions")
                .long_help(format!(
                    "{}\n{}",
                    "Only search in files with the given extensions",
                    "Must be provided after the pattern and the search path"
                ))
                .action(ArgAction::Set)
                .num_args(1..)
                .value_name("EXTENSIONS"),
        )
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .help("Search only in file names for the pattern")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("performance")
                .short('p')
                .long("performance")
                .help("Disable everything that slows down the search")
                .long_help(format!(
                    "{}\n{}\n{}",
                    "Focus on performance",
                    "Disable everything that slows down the search",
                    "Only significant with larger searches"
                ))
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("stats")
                .short('s')
                .long("stats")
                .help("Show the number of search results at the end")
                .long_help(format!(
                    "{}\n{}",
                    "Don`t show the number of search results at the end",
                    "Prints out search results immediately when found"
                ))
                .action(ArgAction::SetTrue),
        )
        .subcommand(
            Command::new("exclude")
                .short_flag('E')
                .long_flag("exclude")
                .about("Exclude patterns from the search")
                .long_about(format!(
                    "{}\n{}",
                    "Exclude patterns from the search",
                    "Must be provided after the pattern and the search path"
                ))
                .arg_required_else_help(true)
                .arg(
                    Arg::new("exclude")
                        .help("Enter patterns to exclude from the search")
                        .action(ArgAction::Set)
                        .num_args(1..)
                        .value_name("PATTERNS"),
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
    extensions: &Vec<&String>,
    file_flag: bool,
    dir_flag: bool,
    performace_flag: bool,
    stats_flag: bool,
) {
    let start = Instant::now();
    let mut search_hits = 0;

    if performace_flag {
        forwards_search_and_catch_errors(
            pattern,
            path,
            &exclude_patterns,
            &extensions,
            file_flag,
            dir_flag,
            performace_flag,
            &mut search_hits,
            None,
        );
    } else {
        let spinner_style = ProgressStyle::with_template("{spinner:.red} {msg}").unwrap();
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(120));
        pb.set_style(spinner_style);
        pb.set_message(format!("{}", "searching".truecolor(250, 0, 104)));
        forwards_search_and_catch_errors(
            pattern,
            path,
            &exclude_patterns,
            &extensions,
            file_flag,
            dir_flag,
            performace_flag,
            &mut search_hits,
            Some(pb.clone()),
        );
        pb.finish_and_clear();
    }

    if stats_flag {
        get_search_hits(search_hits);
        println!(
            "{}",
            HumanDuration(start.elapsed())
                .to_string()
                .truecolor(112, 110, 255)
        );
    }
}

fn forwards_search_and_catch_errors(
    pattern: &str,
    path: &PathBuf,
    exclude_patterns: &Vec<&String>,
    extensions: &Vec<&String>,
    file_flag: bool,
    dir_flag: bool,
    performance_flag: bool,
    search_hits: &mut u64,
    pb: Option<ProgressBar>,
) {
    if let Err(err) = forwards_search(
        pattern,
        path,
        &exclude_patterns,
        &extensions,
        file_flag,
        dir_flag,
        performance_flag,
        search_hits,
        pb.clone(),
    ) {
        match err.kind() {
            io::ErrorKind::NotFound => {
                warn!("\'{}\' not found: {}", path.display(), err);
            }
            io::ErrorKind::PermissionDenied => {
                warn!(
                    "You don`t have access to a source in \'{}\': {}",
                    path.display(),
                    err
                );
            }
            _ => {
                error!(
                    "Error while scanning entries for {} in \'{}\': {}",
                    pattern,
                    path.display(),
                    err
                );
            }
        }
    };
}

fn forwards_search(
    pattern: &str,
    path: &PathBuf,
    exclude_patterns: &Vec<&String>,
    extensions: &Vec<&String>,
    file_flag: bool,
    dir_flag: bool,
    performance_flag: bool,
    search_hits: &mut u64,
    pb: Option<ProgressBar>,
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

        if entry.path().is_symlink() {
            continue;
        }

        if entry.path().is_dir() && fs::read_dir(entry.path())?.count() != 0 {
            let mut entry_path = entry.path().as_path().to_string_lossy().to_string();
            entry_path.push_str("\\");
            let path = Path::new(&entry_path);

            if let Err(err) = forwards_search(
                pattern,
                &path.to_path_buf(),
                &exclude_patterns,
                &extensions,
                file_flag,
                dir_flag,
                performance_flag,
                search_hits,
                pb.clone(),
            ) {
                match err.kind() {
                    io::ErrorKind::NotFound => {
                        warn!("\'{}\' not found: {}", path.display(), err);
                    }
                    io::ErrorKind::PermissionDenied => {
                        warn!(
                            "You don`t have access to a source in \'{}\': {}",
                            path.display(),
                            err
                        );
                    }
                    _ => {
                        error!(
                            "Error while scanning entries for {} in \'{}\': {}",
                            pattern,
                            path.display(),
                            err
                        );
                    }
                }
            };
        }

        // FIXME what`s wrong here??
        // why does it print out dirs?

        // let entry_extension = entry
        //     .path()
        //     .extension()
        //     .unwrap_or_else(|| OsStr::new(""))
        //     .to_string_lossy()
        //     .to_string();
        // let mut entry_extension = String::new();
        // if let Some(extension) = entry.path().extension() {
        //     entry_extension.push_str(&extension.to_string_lossy().to_string());
        // }

        // if !extensions.is_empty()
        //     && !entry_extension.is_empty()
        //     && entry.path().is_file()
        //     && !extensions.iter().any(|&it| &entry_extension == it)
        // {
        // println!("EXT: {entry_extension}");
        //     continue;
        // }

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

        let parent = entry
            .path()
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .to_string_lossy()
            .to_string();

        if exclude_patterns.is_empty() {
            if name.contains(pattern) || name.to_lowercase().contains(pattern) {
                *search_hits += 1;

                if performance_flag {
                    println!("{}", format!("{}\\{}", parent, name));
                } else {
                    match pb.clone() {
                        Some(pb) => {
                            pb.println(format!("{}\\{}", parent, name.truecolor(59, 179, 140)))
                        }
                        None => {}
                    }
                }
            }
        } else {
            if name.contains(pattern) && exclude_patterns.iter().all(|&it| !name.contains(it))
                || name.to_lowercase().contains(pattern)
                    && exclude_patterns
                        .iter()
                        .all(|&it| !name.to_lowercase().contains(it.to_lowercase().as_str()))
            {
                *search_hits += 1;

                if performance_flag {
                    println!("{}", format!("{}\\{}", parent, name));
                } else {
                    match pb.clone() {
                        Some(pb) => {
                            pb.println(format!("{}\\{}", parent, name.truecolor(59, 179, 140)))
                        }
                        None => {}
                    }
                }
            }
        }
    }

    Ok(())
}

fn get_search_hits(search_hits: u64) {
    // for hit in &search_hits {
    //     let parent = hit
    //         .parent()
    //         .unwrap_or_else(|| Path::new(""))
    //         .to_string_lossy()
    //         .to_string();

    //     let mut name = String::new();
    //     if let Some(filename) = hit.file_name() {
    //         name.push_str(&filename.to_string_lossy().to_string());
    //         println!("{}\\{}", parent, name.truecolor(59, 179, 140));
    //     } else {
    //         // TODO remove? how to handle this error?
    //         // error!("Unable to get the filename of {}", hit.display());
    //         println!("{}", hit.display());
    //     }
    // }

    if search_hits == 0 {
        println!(
            "found {} matches",
            search_hits.to_string().truecolor(250, 0, 104).bold()
        );
    } else if search_hits == 1 {
        println!(
            "\nfound {} match",
            search_hits.to_string().truecolor(59, 179, 140).bold()
        );
    } else {
        println!(
            "\nfound {} matches",
            search_hits.to_string().truecolor(59, 179, 140).bold()
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
                "No log file found:"
                    .truecolor(250, 0, 104)
                    .bold()
                    .to_string(),
                log_path.display()
            ))
        }
    }
}

// TODO
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
