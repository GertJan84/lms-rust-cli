# LMS Rust client

This isn't the official client for LMS of the Saxion software development associad degree.

You can find the offical client [here](https://gitlab.com/saxionnl/42/lms42)


## Installation

Install the client with `wget`

```sh
  wget -qO- https://raw.githubusercontent.com/GertJan84/lms-rust-cli/main/install | python
```
or `curl`
```sh
  curl -sSL https://raw.githubusercontent.com/GertJan84/lms-rust-cli/main/install | python
```
## Run Locally

```
LMS Command Line Interface

Usage: lms <COMMAND>

Commands:
  login     Connect to your sd42.nl account
  update    Upgrade lms
  upload    Upload your work for the current assignment
  open      Open the current assignment in the IDE
  verify    Verify the integrity of your lms directory
  template  Download the current assignment template
  download  Download submitted attempts or all attempts
  grade     Teachers only: download everything needed for grading
  show      Show info from the client
  toggle    Toggle settings true or false
  review    Send code to ai to review
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

```

## Running Tests

To run tests, run the following command

```sh
  cargo test
```


## Variables

For this project the following variables are avarible in the `lms.ini` file located in the `~/.config` folder.

```ini
[auth]
token=123

[setup]
move_node_directories=true
enabled=true
open_first_folder=false
upload_open_browser=true
check_todo=true

[ai]
endpoint=https://api.openai.com/v1/chat/completions 
key=123

[custom]
editor=helix
```

`lms toggle <option>|<flag>` to toggle. The correct setup can be called by there full name or there flag.

Options|Flags:

- `move_node_directories` or `-D`: Moves the directories if they are on the wrong place in your system.
- `upload_open_browser` or `-B`: Opens an browser to the current assignment LMS webpage.
- `check_todo` or `-T`: Checks if there are any todo's in the current files and will ask it.
- `open_first_folder` or `-O`: Opens the first and only unhidden folder in the attempt (useful for opening Android Studio)

## Contributing

Contributions are always welcome!

See `contributing.md` for ways to get started.

Please adhere to this project's `code of conduct`.



## Authors

- [@GertJan](https://github.com/GertJan84) 
- [@Tom](https://github.com/TomvanhetBolscher)
- [@Stan](https://github.com/StandUp2001)
