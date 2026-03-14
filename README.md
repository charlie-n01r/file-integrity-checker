# Log Integrity Checker
A tool that verifies the integrity of files to detect any changes or tampering.

## Install
```sh
curl -sSf https://raw.githubusercontent.com/charlie-n01r/file-integrity-checker/main/install | sh
```

## Usage:
To store the hash value of a file future monitoring run:
```sh
./integrity-check init /path/to/file.log
```

This command can be used to recursively add every file within a directory like so:
```sh
./integrity-check init /path/to/folder
```

To check if there have been any modifications to a file or all the files contained in a folder:
```sh
./integrity-check check /path/to/check
```

And last but not least, you can update the stored hash value of any file or all the files contained in a folder (useful when approving any changes after inspecting any mismatching file detected by check)
```sh
./integrity-check update /path/to/changed.file
```