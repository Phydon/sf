# sf

__Simple Find__

*simple file search*

	
## Usage

```
sf [OPTIONS] [PATTERN] [PATH] [COMMAND]

Commands:
  exclude, -E, --exclude  Exclude patterns from the search
  log, -L, --log          Show content of the log file
  help                    Print this message or the help of the given subcommand(s)

Arguments:
  [PATTERN] [PATH]  add a search pattern and a path

Options:
  -d, --dir                        Search only in directory names for the pattern
  -e, --extension <EXTENSIONS>...  Only search in files with the given extensions
  -f, --file                       Search only in file names for the pattern
  -p, --performance                Disable everything that slows down the search
  -s, --stats                      Show the number of search results at the end
  -h, --help                       Print help (see more with '--help')
  -V, --version                    Print version```

## Installation

### Windows

via Cargo or get the ![binary](https://github.com/Phydon/sf/releases)

## TODO

- speed up
