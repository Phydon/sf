# 🔎📄 sf

__Simple Find__

*simple file search*

* colourful output, clickable filepaths and search indicating spinner by default 
  * disable via ```--performance``` flag
* filter by file, directory and file-extension
  * via: 
    * ```--file``` flag
    * ```--dir``` flag
    * ```--extension``` flag
* exclude patterns from the search 
  * via ```--exclude``` flag
* exclude hidden files
  * via ```--no-hidden``` flag
* show number of searched entries, search results and search time
  * via ```--stats``` flag
* only show number of search results 
  * via ```--count``` flag
* search case-insensitivly
  * via ```--case-insensitive``` flag
* set maximum search depth
  * via ```--depth``` flag
* accepts ```.``` as current directory
* ignores filesystem errors (e.g. no permission to access file) by default
  * show errors via ```--show-errors``` flag
* no regex search (for now)

## Example

- search for every file and directory that contains the word 'ron', excluding hidden files but including ![*.ron files](https://github.com/ron-rs/ron)

```sf ron . -s```

![screenshot](https://github.com/Phydon/sf/blob/master/assets/sf_ron_current_s_spinner.png)

![screenshot](https://github.com/Phydon/sf/blob/master/assets/sf_ron_current_s_done.png)

- search all *python* files in a specified directory but only show stats at the end

```sf "" ~\main\ -e py -sc```

![screenshot](https://github.com/Phydon/sf/blob/master/assets/sf___path_e_py_sc.png)

- search only files containing the word *helix*, exclude results containing the words *test* or *json* or *bin*

```sf helix . -fs -E test json bin```

![screenshot](https://github.com/Phydon/sf/blob/master/assets/sf_helix_current_fs_E_test_json_bin.png)

- count all entries in the current directory and disable the search indicating spinner 

```sf "" . -cp```

![screenshot](https://github.com/Phydon/sf/blob/master/assets/sf_current_count_all.png)

- you can use ```sf``` to list all files and sub-directories recursively via ```""``` as an empty search pattern 

```sf "" .```

	
## Usage

### Short Usage

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
  -H, --no-hidden                  Exclude hidden files and directories from search
  -o, --override                   Override all previously set flags
  -p, --performance                Disable spinner, don`t colourize the search output and speed up the output printing
      --show-errors                Show possible filesystem errors
  -s, --stats                      Show short search statistics at the end
      --stats-long                 Show search statistics at the end
  -h, --help                       Print help (see more with '--help')
  -V, --version                    Print version
```

### Long Usage

```
sf [OPTIONS] [PATTERN] [PATH] [COMMAND]

Commands:
  log, -L, --log  Show content of the log file
  help            Print this message or the help of the given subcommand(s)

Arguments:
  [PATTERN] [PATH]
          Add a search pattern and a path

Options:
  -i, --case-insensitive
          Search case insensitivly

  -c, --count
          Only print the number of search results
          Can be combined with the --stats flag to only show stats

  -D, --depth <NUMBER>
          Set max search depth

          [default: 250]

  -d, --dir
          Search only in directory names for the pattern

  -e, --extension <EXTENSIONS>...
          Only search in files with the given extensions
          Must be provided after the pattern and the search path

  -E, --exclude <PATTERNS>...
          Enter patterns to exclude from the search
          Must be provided after the pattern and the search path

  -f, --file
          Search only in file names for the pattern

  -H, --no-hidden
          Exclude hidden files and directories from search
          If a directory is hidden, all its content will be skiped as well

  -o, --override
          Override all previously set flags
          This can be used when a custom alias for this command is set together with regularly used flags
          This flag allows to disable these flags and specify new ones

  -p, --performance
          Focus on performance
          Disable search indicating spinner and don`t colourize the search output
          Write the output via BufWriter
          Cannot be set together with the --stats flag

      --show-errors
          Show possible filesystem errors
          For example for situations such as insufficient permissions

  -s, --stats
          Show short search statistics at the end
          Can be combined with the --count flag to only show stats
          Cannot be set together with the --performance flag

      --stats-long
          Show search statistics at the end
          Can be combined with the --count flag to only show stats
          Cannot be set together with the --performance flag

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
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
