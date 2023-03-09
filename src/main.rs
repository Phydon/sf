// TODO limit recursion???
#![recursion_limit = "250"]

use clap::{Arg, ArgAction, Command};
use colored::*;
use flexi_logger::{detailed_format, Duplicate, FileSpec, Logger};
use indicatif::{HumanDuration, ProgressBar, ProgressStyle};
use log::{error, warn};

use std::{
    env, fs, io,
    os::windows::prelude::MetadataExt,
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
    let hidden_flag = matches.get_flag("hidden");
    let performance_flag = matches.get_flag("performance");
    let stats_flag = matches.get_flag("stats");
    if let Some(args) = matches
        .get_many::<String>("args")
        .map(|a| a.collect::<Vec<_>>())
    {
        let pattern = args[0];
        let path = Path::new(&args[1]).to_path_buf();

        let mut extensions = Vec::new();
        if let Some(mut ext) = matches
            .get_many::<String>("extension")
            .map(|a| a.collect::<Vec<_>>())
        {
            extensions.append(&mut ext);
        }

        let mut exclude_patterns = Vec::new();
        if let Some(mut excl) = matches
            .get_many::<String>("exclude")
            .map(|a| a.collect::<Vec<_>>())
        {
            exclude_patterns.append(&mut excl);
        }

        search(
            pattern,
            &path,
            &exclude_patterns,
            &extensions,
            file_flag,
            dir_flag,
            hidden_flag,
            performance_flag,
            stats_flag,
        );
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
        .version("1.0.2")
        .author("Leann Phydon <leann.phydon@gmail.com>")
        .arg_required_else_help(true)
        .arg(
            Arg::new("args")
                .help("Add a search pattern and a path")
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
            Arg::new("exclude")
                .short('E')
                .long("exclude")
                .help("Enter patterns to exclude from the search")
                .long_help(format!(
                    "{}\n{}",
                    "Enter patterns to exclude from the search",
                    "Must be provided after the pattern and the search path"
                ))
                .action(ArgAction::Set)
                .num_args(1..)
                .value_name("PATTERNS"),
        )
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .help("Search only in file names for the pattern")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("hidden")
                .short('H')
                .long("hidden")
                // .visible_alias("all")
                .help("Include hidden files in search")
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
    hidden_flag: bool,
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
            hidden_flag,
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
            hidden_flag,
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
    hidden_flag: bool,
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
        hidden_flag,
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
    hidden_flag: bool,
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
                hidden_flag,
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

        if !hidden_flag && is_hidden(&entry.path())? {
            continue;
        }

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

        if !extensions.is_empty() {
            if entry.path().is_file() {
                let mut entry_extension = String::new();
                if let Some(extension) = entry.path().extension() {
                    entry_extension.push_str(&extension.to_string_lossy().to_string());

                    if extensions.iter().any(|&it| &entry_extension == it) {
                        match_pattern_and_print(
                            name,
                            parent,
                            pattern,
                            pb.clone(),
                            exclude_patterns,
                            search_hits,
                            performance_flag,
                        );
                    }
                }
            } else {
                continue;
            }
        } else {
            match_pattern_and_print(
                name,
                parent,
                pattern,
                pb.clone(),
                exclude_patterns,
                search_hits,
                performance_flag,
            );
        }
    }

    Ok(())
}

fn match_pattern_and_print(
    name: String,
    parent: String,
    pattern: &str,
    pb: Option<ProgressBar>,
    exclude_patterns: &Vec<&String>,
    search_hits: &mut u64,
    performance_flag: bool,
) {
    if exclude_patterns.is_empty() {
        if name.contains(pattern) || name.to_lowercase().contains(pattern) {
            *search_hits += 1;

            print_search_hit(name, parent, pattern, pb.clone(), performance_flag);
        }
    } else {
        if name.contains(pattern) && exclude_patterns.iter().all(|&it| !name.contains(it))
            || name.to_lowercase().contains(pattern)
                && exclude_patterns
                    .iter()
                    .all(|&it| !name.to_lowercase().contains(it.to_lowercase().as_str()))
        {
            *search_hits += 1;

            print_search_hit(name, parent, pattern, pb.clone(), performance_flag);
        }
    }
}

fn print_search_hit(
    name: String,
    parent: String,
    pattern: &str,
    pb: Option<ProgressBar>,
    performance_flag: bool,
) {
    if performance_flag {
        println!("{}", format!("{}\\{}", parent, name));
    } else {
        match pb.clone() {
            Some(pb) => {
                let name_with_hi_pattern = highlight_pattern_in_name(&name, pattern);
                pb.println(format!(
                    "{}\\{}",
                    parent,
                    name_with_hi_pattern.truecolor(59, 179, 140)
                ))
            }
            None => {}
        }
    }
}

fn get_search_hits(search_hits: u64) {
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

fn highlight_pattern_in_name(name: &str, pattern: &str) -> String {
    let pat_in_name = name.find(pattern).unwrap_or_else(|| 9999999999);

    if pat_in_name == 9999999999 {
        return name.to_string();
    } else {
        let first_from_name = &name[..pat_in_name];
        let last_from_name = &name[(pat_in_name + pattern.len())..];
        let highlighted_pattern = pattern.truecolor(112, 110, 255).to_string();

        let mut result = String::from(first_from_name);
        result.push_str(&highlighted_pattern);
        result.push_str(last_from_name);

        result.to_string()
    }
}

fn is_hidden(file_path: &PathBuf) -> std::io::Result<bool> {
    let metadata = fs::metadata(file_path)?;
    let attributes = metadata.file_attributes();

    if (attributes & 0x2) > 0 {
        Ok(true)
    } else {
        Ok(false)
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
