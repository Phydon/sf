use aho_corasick::{AhoCorasick, AhoCorasickBuilder};
use clap::{Arg, ArgAction, Command};
use colored::*;
use flexi_logger::{detailed_format, Duplicate, FileSpec, Logger};
use indicatif::{HumanDuration, ProgressBar, ProgressStyle};
use log::{error, warn};
use walkdir::{DirEntry, WalkDir};

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
    // TODO don`t pass the flags around in every function call
    let mut file_flag = matches.get_flag("file");
    let mut dir_flag = matches.get_flag("dir");
    let mut hidden_flag = matches.get_flag("hidden");
    let mut performance_flag = matches.get_flag("performance");
    let mut stats_flag = matches.get_flag("stats");
    let mut count_flag = matches.get_flag("count");
    let mut case_insensitive_flag = matches.get_flag("case-insensitive");
    let override_flag = matches.get_flag("override");

    // set default search depth
    let mut depth_flag = 250;
    if let Some(d) = matches.get_one::<String>("depth") {
        match d.parse() {
            Ok(depth) => depth_flag = depth,
            Err(err) => {
                error!("Expected an integer for the search depth: {err}");
                process::exit(1);
            }
        }
    }

    // if override flag is set -> reset everything to default values
    if override_flag {
        file_flag = false;
        dir_flag = false;
        hidden_flag = false;
        performance_flag = false;
        stats_flag = false;
        count_flag = false;
        depth_flag = 250;
        case_insensitive_flag = false;
    }

    if let Some(args) = matches
        .get_many::<String>("args")
        .map(|a| a.collect::<Vec<_>>())
    {
        // get search pattern from arguments
        let pattern = vec![args[0].as_str()];
        // store search pattern in aho-corasick builder
        // handle case-insensitive flag
        let ac = AhoCorasickBuilder::new()
            .ascii_case_insensitive(case_insensitive_flag)
            .build(&pattern);

        // get search path from arguments
        let path = Path::new(&args[1]).to_path_buf();

        // get possible file extensions for filtering
        let mut extensions = Vec::new();
        if let Some(mut ext) = matches
            .get_many::<String>("extension")
            .map(|a| a.collect::<Vec<_>>())
        {
            extensions.append(&mut ext);
        }

        // get exclude patterns
        let mut exclude_patterns = Vec::new();
        if let Some(mut excl) = matches
            .get_many::<String>("exclude")
            .map(|a| a.collect::<Vec<_>>())
        {
            exclude_patterns.append(&mut excl);
        }

        // store exclude patterns in aho-corasick builder
        // handle case-insensitive flag for exclude patterns
        let ex_ac = AhoCorasickBuilder::new()
            .ascii_case_insensitive(case_insensitive_flag)
            .build(&exclude_patterns);

        // start search
        search(
            &pattern,
            ac,
            &path,
            ex_ac,
            &extensions,
            file_flag,
            dir_flag,
            hidden_flag,
            performance_flag,
            stats_flag,
            count_flag,
            depth_flag,
        );
    } else {
        // handle commands
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

// build cli
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
        .long_about(format!(
            "{}\n  {}\n  {}\n  {}\n  {}\n  {}\n  {}\n  {}\n  {}\n  {}",
            "Simple file search",
            "- smart-case by default",
            "- no regex search",
            "- colourful output and search indicating spinner by default ",
            "- filter by file, directory and file-extension",
            "- ignores symlinks",
            "- exclude patterns from the search ",
            "- include hidden files",
            "- show number of search results and search time",
            "- accepts \'.\' as current directory"
        ))
        // TODO update version
        .version("1.3.1")
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
            Arg::new("case-insensitive")
                .short('i')
                .long("case-insensitive")
                .help("Search case insensitivly")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("count")
                .short('c')
                .long("count")
                .help("Only print the number of search results")
                .long_help(format!(
                    "{}\n{}",
                    "Only print the number of search results",
                    "Can be combined with the --stats flag to only show stats and no paths",
                ))
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("depth")
                .short('D')
                .long("depth")
                .help("Set max search depth")
                .long_help(format!(
                    "{}\n{}",
                    "Set max search depth",
                    "Default is 250",
                ))
                .default_value("250")
                .action(ArgAction::Set)
                .num_args(1)
                .value_name("NUMBER"),
        )
        .arg(
            Arg::new("dir")
                .short('d')
                .long("dir")
                .help("Search only in directory names for the pattern")
                .action(ArgAction::SetTrue)
                .conflicts_with("file"),
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
                .conflicts_with("dir")
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
                .help("Include hidden files and directories in search")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("override")
                .short('o')
                .long("override")
                .help("Override all previously set flags")
                .long_help(format!(
                    "{}\n{}\n{}",
                    "Override all previously set flags",
                    "This is usually used when a custom alias for this command is set together with regularly used flags",
                    "This flag allows to disable these flags and specify new ones"
                ))
                // TODO if new args -> add here to this list to override if needed
                .overrides_with_all(["stats", "file", "dir", "extension", "exclude", "hidden", "performance", "count"])
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
                .help("Show search statistics at the end")
                .long_help(format!(
                    "{}\n{}",
                    "Show search statistics at the end",
                    "Can be combined with the --count flag to only show stats and no paths",
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
    pattern: &Vec<&str>,
    ac: AhoCorasick,
    path: &PathBuf,
    ex_ac: AhoCorasick,
    extensions: &Vec<&String>,
    file_flag: bool,
    dir_flag: bool,
    hidden_flag: bool,
    performace_flag: bool,
    stats_flag: bool,
    count_flag: bool,
    depth_flag: u32,
) {
    let start = Instant::now();
    let mut entry_count = 0;
    let mut search_hits = 0;

    // disable the search indicating spinner and colourful output
    if performace_flag {
        forwards_search_and_catch_errors(
            pattern,
            ac,
            path,
            ex_ac,
            &extensions,
            file_flag,
            dir_flag,
            hidden_flag,
            performace_flag,
            count_flag,
            depth_flag,
            &mut search_hits,
            &mut entry_count,
            None,
        );
    } else {
        // spinner
        let spinner_style = ProgressStyle::with_template("{spinner:.red} {msg}").unwrap();
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(120));
        pb.set_style(spinner_style);
        pb.set_message(format!("{}", "searching".truecolor(250, 0, 104)));

        forwards_search_and_catch_errors(
            pattern,
            ac,
            path,
            ex_ac,
            &extensions,
            file_flag,
            dir_flag,
            hidden_flag,
            performace_flag,
            count_flag,
            depth_flag,
            &mut search_hits,
            &mut entry_count,
            Some(pb.clone()),
        );

        pb.finish_and_clear();
    }

    // print output
    if count_flag && !stats_flag {
        println!("{}", search_hits.to_string());
    } else if stats_flag {
        get_search_hits(search_hits, entry_count, start);
    }
}

fn forwards_search_and_catch_errors(
    pattern: &Vec<&str>,
    ac: AhoCorasick,
    path: &PathBuf,
    ex_ac: AhoCorasick,
    extensions: &Vec<&String>,
    file_flag: bool,
    dir_flag: bool,
    hidden_flag: bool,
    performance_flag: bool,
    count_flag: bool,
    depth_flag: u32,
    search_hits: &mut u64,
    entry_count: &mut u64,
    pb: Option<ProgressBar>,
) {
    if let Err(err) = forwards_search(
        pattern,
        ac,
        path,
        ex_ac,
        &extensions,
        file_flag,
        dir_flag,
        hidden_flag,
        performance_flag,
        count_flag,
        depth_flag,
        search_hits,
        entry_count,
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
                    pattern[0].italic(),
                    path.display(),
                    err
                );
            }
        }
    };
}

fn forwards_search(
    pattern: &Vec<&str>,
    ac: AhoCorasick,
    path: &PathBuf,
    ex_ac: AhoCorasick,
    extensions: &Vec<&String>,
    file_flag: bool,
    dir_flag: bool,
    hidden_flag: bool,
    performance_flag: bool,
    count_flag: bool,
    depth_flag: u32,
    search_hits: &mut u64,
    entry_count: &mut u64,
    pb: Option<ProgressBar>,
) -> io::Result<()> {
    let mut search_path = Path::new(&path).to_path_buf();

    // accept "." as current directory
    if path.as_path().to_string_lossy().to_string() == "." {
        let current_dir = env::current_dir().unwrap_or_else(|err| {
            error!("Unable to get current directory: {err}");
            process::exit(1);
        });
        search_path.push(current_dir);
    }

    // filter files
    let valid_entries = WalkDir::new(search_path)
        .max_depth(depth_flag as usize) // set maximum search depth
        .into_iter()
        .filter_entry(|e| file_check(e, hidden_flag, dir_flag)); // handle hidden flag and dir flag

    for entry in valid_entries {
        let entry = entry?;

        // handle file flag
        // must be outside of function file_check()
        // else no file will be searched with WalkDir...filter_entry()
        if file_flag && !entry.file_type().is_file() {
            continue;
        }

        // count seached entries
        *entry_count += 1;

        // get filename
        let mut name = String::new();
        name.push_str(&entry.file_name().to_string_lossy().to_string());

        // get parent path
        let parent = entry
            .path()
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .to_string_lossy()
            .to_string();

        // handle possible file extensions
        if !extensions.is_empty() {
            if entry.path().is_file() {
                // get entry extension
                let mut entry_extension = String::new();
                if let Some(extension) = entry.path().extension() {
                    entry_extension.push_str(&extension.to_string_lossy().to_string());

                    // check if entry_extension matches any given extensions via extensions flag
                    if extensions.iter().any(|&it| &entry_extension == it) {
                        match_pattern_and_print(
                            name,
                            parent,
                            pattern,
                            &ac,
                            pb.clone(),
                            &ex_ac,
                            search_hits,
                            performance_flag,
                            count_flag,
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
                &ac,
                pb.clone(),
                &ex_ac,
                search_hits,
                performance_flag,
                count_flag,
            );
        }
    }

    Ok(())
}

fn match_pattern_and_print(
    name: String,
    parent: String,
    pattern: &Vec<&str>,
    ac: &AhoCorasick,
    pb: Option<ProgressBar>,
    ex_ac: &AhoCorasick,
    search_hits: &mut u64,
    performance_flag: bool,
    count_flag: bool,
) {
    // check for pattern match in filename via aho-corasick algorithm
    if ac.is_match(&name) && !ex_ac.is_match(&name) {
        *search_hits += 1;

        if !count_flag {
            print_search_hit(name, parent, pattern, pb.clone(), performance_flag);
        }
    }
}

fn print_search_hit(
    name: String,
    parent: String,
    pattern: &Vec<&str>,
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

fn get_search_hits(search_hits: u64, entry_count: u64, start: Instant) {
    println!(
        "\n{} {}",
        entry_count.to_string().dimmed(),
        "entries searched".dimmed()
    );

    if search_hits == 0 {
        println!(
            "found {} matches",
            search_hits.to_string().truecolor(250, 0, 104).bold()
        );
    } else if search_hits == 1 {
        println!(
            "found {} match",
            search_hits.to_string().truecolor(59, 179, 140).bold()
        );
    } else {
        println!(
            "found {} matches",
            search_hits.to_string().truecolor(59, 179, 140).bold()
        );
    }

    println!(
        "{}",
        HumanDuration(start.elapsed())
            .to_string()
            .truecolor(112, 110, 255)
    );
}

fn highlight_pattern_in_name(name: &str, pattern: &Vec<&str>) -> String {
    // find first byte of pattern in filename
    let pat_in_name = name.find(pattern[0]).unwrap_or_else(|| 9999999999);

    if pat_in_name == 9999999999 {
        // if no pattern found return just the filename
        return name.to_string();
    } else {
        let first_from_name = &name[..pat_in_name];
        let last_from_name = &name[(pat_in_name + pattern[0].len())..];
        // colourize the pattern in the filename
        let highlighted_pattern = pattern[0].truecolor(112, 110, 255).to_string();

        let mut result = String::from(first_from_name);
        result.push_str(&highlighted_pattern);
        result.push_str(last_from_name);

        result.to_string()
    }
}

// check entries if hidden, dir or symlink
// and compare to hidden flag and dir flag
fn file_check(entry: &DirEntry, hidden_flag: bool, dir_flag: bool) -> bool {
    // always skip symlinks
    if entry.file_type().is_symlink() {
        return false;
    }

    if !hidden_flag && is_hidden(&entry.path().to_path_buf()).unwrap_or(false) {
        return false;
    }

    if dir_flag && !entry.file_type().is_dir() {
        return false;
    }

    return true;
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
