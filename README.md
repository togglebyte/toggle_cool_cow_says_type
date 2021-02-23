# ToggleCoolCowSaysType

A terminal based typing game.

Usage:
```bash
toggle_cool_cow_says_type [-t {word_count}] [-t {file_extension}] [-s] {project_path}
```

* `-s` : flag to tell the game to run in strict mode
* `-t` : file extension (defaults to "rs")
* `-w` : word count (defaults to 10)
* -ss: skip word on space, unless it's the first character.".into(),
* `-cf`: foreground colour of the cursor (defaults to `green`)
* `-cb`: background colour of the cursor (defaults to `dark_grey`)
* `-ma`: minimum accuracy required to see the actual result.

Example:
```bash
toggle_cool_cow_says_type -p path_to_project -t c -w 5
```

The name provided by BareCoolCowSaysMoomah won the naming poll.

Colour Names:
```
black
dark_grey
red
dark_red
green
dark_green
yellow
dark_yellow
blue
dark_blue
magenta
dark_magenta
cyan
dark_cyan
white
grey
```
