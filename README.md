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
 - [X] Upload work
 - [X] Download work
 - [X] Download template
 - [X] Reorder file structure
 - [X] Grade work
 - [X] Login
 - [X] Open work dir
 - [ ] Handle setup
 
### Extra features
 - [X] Change default editor
 - [X] Open work offline 
 - [X] Download all assignments 
 - [X] Check for todo's in your code before upload for "sql", "rs", "py", "js", "css", "html", "svelte"
 - [ ] Add set command for config changes 
 - [ ] Add get command to verify config changes 

Available for `arm` and `x86` systems

### Settings example

in ~/.config/lms.ini

```ini
[auth]
token=123

[setup]
move_node_directories=false
enabled=false

[custom]
editor=custom_script
check_todo=true
```
