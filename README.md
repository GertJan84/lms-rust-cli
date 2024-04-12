# LMS Rust client

This isn't the official client for LMS.

[LMS](https://gitlab.com/saxion.nl/42/lms42)

### How to install

Run this in terminal.

```bash
wget -qO- https://gitlab.com/gj-535479/lms-rust-cli/-/raw/main/install | python
```

### How to run

Just use `lms` as usual

### Features

- [x] Upload work
- [x] Download work
- [x] Download template
- [x] Reorder file structure
- [x] Grade work
- [x] Login
- [x] Open work dir
- [ ] Handle setup

### Extra features

- [x] Change default editor
- [x] Open work offline
- [x] Download all assignments
- [x] Check for todo's in your code before upload for "sql", "rs", "py", "js", "css", "html", "svelte"
- [ ] Add set command for config changes
- [ ] Add get command to verify config changes

Available for `arm` and `x86` systems

### Settings example

in ~/.config/lms.ini

```ini
[auth]
token=123

[setup]
move_node_directories=true
enabled=true
upload_open_browser=true
check_todo=true

[custom]
editor=custom_script
```

### Setups options

To update an setup use the toggle command with the subcommand you want to toggle.
If the option is not in your `lms.ini` file it will be set to true automatically.

`lms toggle <option>|<flag>` to toggle. The correct setup can be called by there full name or there flag.

`move_node_directories` or `-D`: Moves the directories if they are on the wrong place in your system.
`upload_open_browser` or `-B`: Opens an browser to the correct attempt what you want to upload.
`check_todo` or `-T`: Checks if there are any todo's in the current files and will ask it.
