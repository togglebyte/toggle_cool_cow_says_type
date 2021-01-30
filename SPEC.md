# Simple typing game

Deps: tinybit, rand

Game state:
* input <- user input
* text  <- text to type

Should work with a term height of one.
In the event of three or more show a border.

## Start up

Config has a path pointing to a Rust project, and a word count.
Select a random Rust file from the project.
`read_to_string` the contents of the file.
Split the file on white space and select N words.
Filter words that are N chars long.

## Main run loop

Iterate over input events. Push chars to the input of the game state (remember
to handle backspace).

Create an `Instant` once the first key is pressed.

For every key press compare `input` with `text` in the game state.
If it's a match, then end the loop and check the `elapsed` time of the `Instant`.

## End screen

Show:
* # of words
* # of mistakes
* wpm
* restart: y/n

## Layout

src/
    - main.rs
    - gamestate.rs
    - words.rs

## Questions 

Q: Care about `\n`?
A: No, since it's just a list of words.

