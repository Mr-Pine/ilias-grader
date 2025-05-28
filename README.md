# ilias-grader
*The solution to more ilias is more tooling*

---
A tool to download assignment submissions and upload feedback files for them

## Installation
Clone this repository and either install it with `cargo install --path .` or simply run it with `cargo run -- [args]`

## Usage

```
ilias-grader [OPTIONS] --id <ID> --assignment <ASSIGNMENT> --username <USERNAME> <COMMAND>

Commands:
  download       Download submissions
  feedback       Upload feedback
  upload-points  Upload points 
  help      Print this message or the help of the given subcommand(s)

Options:
  -i, --id <ID>                  The id of the exercise you want to handle
  -a, --assignment <ASSIGNMENT>  The index of the assignment you want to handle
  -u, --username <USERNAME>
  -p, --password <PASSWORD>
  -h, --help                     Print help
```

### Downloading
```
ilias-grader --id <ID> --assignment <ASSIGNMENT> --username <USERNAME> download [OPTIONS] <TO>

Arguments:
  <TO>  The path to download the assignments to

Options:
      --extract  Whether to extract the zip file
      --flatten  Whether to flatten the extracted files into one directory
  -h, --help     Print help
```

### Uploading feedback
```
ilias-grader --id <ID> --assignment <ASSIGNMENT> --username <USERNAME> feedback [OPTIONS] <FEEDBACK_DIR>

Arguments:
  <FEEDBACK_DIR>  The directory where your feedback files are located

Options:
  -s, --suffix <SUFFIX>  A suffix to append to uploaded feedback files [default: ]
      --no-confim        Upload without confirmation
  -h, --help             Print help
```

## Uploading points 
```
ilias-grader --id <ID> --assignment <ASSIGNMENT> --username <USERNAME> upload-points 
```
You'll see a search screen where you can select a student and assign them points. You can repeat this process for as many students as needed. When you're done, press <ESC> to upload the points. If you need to cancel at any time, press <Ctrl-C>.

## Logging
By default, the log level is `info`, you can configure logging with the `RUST_LOG` environment variable: https://docs.rs/env_logger/latest/env_logger/#enabling-logging
