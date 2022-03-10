# SimpleFind
### command line tool
**simple recursive file search**
- starting from your current path / folder
- if file with given parameter exists, it outputs the complete path
- if not, it continues searching in the parent folder until it reaches root
	
## Installation
below are some easy ways to get it running (probably not the most efficient way but it works):

**Linux** (should work on all distributions)
- create a folder '~/Aliases'
- download the file 'target/release/sf' and put it in this folder
- add an alias to your terminal config:
	- for example with fish terminal:
		- find or create '~/.config/fish/config.fish'
		- add:	
			- 'alias sf="~/Aliases/sf"'
- restart terminal

**Windows Cmd**
- create a folder 'C:/Aliases'
- download the file 'target/release/sf.exe' and put it in this folder
- create a file 'sf.bat'
	- add:	
		- '@echo off'
		- 'echo.'
		- 'C:/Aliases/sf.exe %*'
- add the folder to your systems PATH variable
- restart cmd
 
**Windows PowerShell**
- create a folder 'C:/Aliases'
- download the file 'target/release/sf.exe' and put it in this folder
- find or create a file 'profile.ps1'
	- add:	
		- 'New-Alias sf C:/Aliases/sf.exe'
- restart powershell
	
## Usage
> sf [ Filename ]

> sf [ search_term ]

## Bugs / Errors / Criticism / Advise
=> leann.phydon@gmail.com

### Work In Progress
- [ ] TODO: search forwards
- [ ] TODO: sort output alphabetically

