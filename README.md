# sf

__Simple Find__

*simple file search*

* smart-case by default
* no regex search (for now)
* colourful output and search indicating spinner by default 
  * disable via ```--performance``` flag
* filter by file, directory and file-extension
* ignores symlinks
* exclude patterns from the search 
  * via ```--exclude``` flag
* include hidden files
  * via ```--hidden``` flag
* show number of searched entries, search results and search time
  * via ```--stats``` flag
* only show number of search results 
  * via ```--count``` flag
* accepts ```.``` as current directory

## Example

- search for every file and directory that contains the word 'ron', including ![*.ron files](https://github.com/ron-rs/ron)

```sf ron . -s```

![screenshot](https://github.com/Phydon/sf/blob/master/assets/sf_ron_current_s_spinner.png)

![screenshot](https://github.com/Phydon/sf/blob/master/assets/sf_ron_current_s_done.png)

- search all *rust* files in a specified directory, include hidden files and show stats at the end

```sf "" ~\main\Rust\up\ -e rs -sH```

![screenshot](https://github.com/Phydon/sf/blob/master/assets/sf_path_ers_sH_done.png)

- search only files containing the word *test*, exclude results containing the words *json* or *py* or *bin*

```sf test ~\main\Rust\sf -fs -E json py bin```

![screenshot](https://github.com/Phydon/sf/blob/master/assets/sf_test_path_fs_Ejsonpybin_done.png)

- you can use ```sf``` to list all files and sub-directories recursively via ```""``` as an empty search pattern 

```sf "" .```

- count all entries in the current directory and disable the search indicating spinner

```sf "" . -cp```

	
## Usage

```
sf [OPTIONS] [PATTERN] [PATH] [COMMAND]

Commands:
  log, -L, --log  Show content of the log file
  help            Print this message or the help of the given subcommand(s)

Arguments:
  [PATTERN] [PATH]  Add a search pattern and a path

Options:
  -i, --case-insensitive           Search case insensitivly
  -c, --count                      Only print the number of search results
  -D, --depth <NUMBER>             Set max search depth [default: 250]
  -d, --dir                        Search only in directory names for the pattern
  -e, --extension <EXTENSIONS>...  Only search in files with the given extensions
  -E, --exclude <PATTERNS>...      Enter patterns to exclude from the search
  -f, --file                       Search only in file names for the pattern
  -H, --hidden                     Include hidden files and directories in search
  -o, --override                   Override all previously set flags
  -p, --performance                Disable everything that slows down the search
  -s, --stats                      Show the number of search results at the end
  -h, --help                       Print help (see more with '--help')
  -V, --version                    Print version
```

## Installation

### Windows

via Cargo or get the ![binary](https://github.com/Phydon/sf/releases)

## Known Issues

### PowerShell

- ```sf "" . -e rs``` to find all Rust files in the current directory doesn`t work in PowerShell
  solution => you have to escape the quotes: 

```

sf `"`" . -e rs

```

## Todo

- speed up
