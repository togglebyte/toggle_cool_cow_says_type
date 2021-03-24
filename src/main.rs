use std::env::args;

use tinybit::events::{events, Event, EventModel, KeyCode, KeyEvent, KeyModifiers};
use tinybit::render::RenderTarget;
use tinybit::widgets::Text;
use tinybit::{term_size, Color, Pixel, Renderer, ScreenPos, ScreenSize, StdoutTarget, Viewport};

mod config;
mod error;
mod gamestate;
mod words;

use config::Config;
use gamestate::{Game, GameState};
use words::words;

// -----------------------------------------------------------------------------
//     - Render -
// -----------------------------------------------------------------------------
fn render<T: RenderTarget>(
    game: &Game,
    config: &Config,
    viewport: &mut Viewport,
    renderer: &mut Renderer<T>,
) {
    match game.state {
        GameState::Running(_) => {
            let input = game.input();
            let index = input.len();
            let text = &game.text_chars;

            let char_count = game.text.chars().count() as u16;
            let lines = char_count / viewport.size.width;

            // Find the starting x value.
            let mut x = if lines > 0 {
                1
            } else {
                (viewport.size.width - char_count) / 2
            };

            let mut y = viewport.size.height / 2 - lines / 2;

            for i in 0..char_count as usize {
                // An input character can either be:
                // 1. Correct,
                // 2. Incorrect space over non-space character
                // 3. Incorrect character over space
                // 4. Incorrect non-space character over non-space correct character
                match input.get(i) {
                    // Correct
                    Some((c, _)) if *c == text[i] => viewport.draw_pixel(Pixel::new(
                        text[i],
                        ScreenPos::new(x, y),
                        Some(Color::Blue),
                        None,
                    )),
                    // Incorrect space over non-space character
                    Some((' ', _)) if text[i] != ' ' => viewport.draw_pixel(Pixel::new(
                        text[i],
                        ScreenPos::new(x, y),
                        Some(Color::DarkGrey),
                        None,
                    )),
                    // Incorrect character over space
                    Some((c, _)) if text[i] == ' ' => viewport.draw_pixel(Pixel::new(
                        *c,
                        ScreenPos::new(x, y),
                        Some(Color::DarkYellow),
                        None,
                    )),
                    Some((_, _)) => viewport.draw_pixel(Pixel::new(
                        text[i],
                        ScreenPos::new(x, y),
                        Some(Color::Red),
                        None,
                    )),
                    None if i == index => viewport.draw_pixel(Pixel::new(
                        text[i],
                        ScreenPos::new(x, y),
                        Some(config.cursor_foreground_color),
                        Some(config.cursor_background_color),
                    )),
                    None => viewport.draw_pixel(Pixel::new(
                        text[i],
                        ScreenPos::new(x, y),
                        Some(Color::White),
                        None,
                    )),
                }

                x += 1;
                if x >= viewport.size.width {
                    x = 1;
                    y += 1;
                }
            }
        }
        GameState::Stopped => {
            let text = "Press any key to start";
            let x = (viewport.size.width - text.chars().count() as u16) / 2;
            let y = viewport.size.height / 2;

            let text = Text::new(text, None, None);
            viewport.draw_widget(&text, ScreenPos::new(x, y));
        }
        GameState::Finished {
            elapsed,
            wpm,
            cpm,
            mistakes,
            word_count,
            accuracy,
        } => {
            // Split the text if the text is too long to fit on one line,
            // and show the results as multiple lines.
            let text_chunks: Vec<String> = {
                let mut result_text = format!(
                    "time: {} seconds | wpm: {} (cpm: {}) | mistakes: {} | accuracy: {:.2}% | word count: {}",
                    elapsed.as_secs(),
                    wpm,
                    cpm,
                    mistakes,
                    accuracy,
                    word_count
                );

                // If the accuracy is given, and achieved accuracy
                // is less than the target, don't show the results.
                match config.min_accuracy {
                    Some(acc) if accuracy < acc => result_text =format!("Accuracy too low ({:.2}%)", accuracy),
                    _ => {}
                }

                // If the result text can't fit on screen we split it on
                // the pipe char.
                let mut chunks = if result_text.chars().count() as u16 > viewport.size.width {
                    result_text
                        .split('|')
                        .map(str::trim)
                        .map(String::from)
                        .collect()
                } else {
                    vec![result_text]
                };

                // Add one empt line between the result
                // and the try-again text.
                chunks.push(String::from(" "));

                let text = "Try again? Y(es) | N(no) | R(etry same words)".to_string();

                // Same as for the result text: we split it on the pipe
                // if it can't fit.
                if text.chars().count() as u16 > viewport.size.width {
                    let mut t = text.split('|').map(str::trim).map(String::from).collect();
                    chunks.append(&mut t);
                } else {
                    chunks.push(text);
                }

                chunks
            };

            // Get the length of the longest line.
            let max_width = text_chunks.iter().map(|t| t.chars().count()).max().unwrap() as u16;

            let x = (viewport.size.width - max_width) / 2;
            let mut y = viewport.size.height / 2 - text_chunks.len() as u16 / 2;

            for chunk in text_chunks {
                let text = Text::new(chunk, None, None);
                viewport.draw_widget(&text, ScreenPos::new(x, y));
                y += 1;
            }
        }
    }

    renderer.render(viewport);
}

// -----------------------------------------------------------------------------
//     - Game loop -
// -----------------------------------------------------------------------------
fn play() -> error::Result<()> {
    let config = Config::from_args(args())?;
    let (w, h) = term_size().expect("could not get terminal size");
    let mut selected_words = words(&config, (w * h) as usize)?;

    let mut game = Game::new(&selected_words, config.strict, config.skip_word_on_space);

    let mut viewport = Viewport::new(ScreenPos::zero(), ScreenSize::new(w, h));

    let stdout = StdoutTarget::new().expect("failed to enter raw mode");
    let mut renderer = Renderer::new(stdout);

    render(&game, &config, &mut viewport, &mut renderer);

    for event in events(EventModel::Blocking) {
        match event {
            Event::Tick => unreachable!(),
            Event::Resize(w, h) => {
                viewport.resize(w, h);
                renderer.clear();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('w'),
                modifiers: KeyModifiers::CONTROL,
            }) => game.pop_word(),
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
            }) => break,
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                ..
            }) => match game.state {
                GameState::Finished { .. } => match c {
                    'y' => {
                        selected_words = words(&config, (w * h) as usize)?;
                        game = Game::new(&selected_words, config.strict, config.skip_word_on_space);
                        game.start();
                    }
                    'r' => game = Game::new(&selected_words, config.strict, config.skip_word_on_space),
                    'n' => break,
                    _ => {}
                },
                GameState::Running(_) => {
                    game.push(c);
                }
                GameState::Stopped => game.start(),
            },
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) if game.state == GameState::Stopped => game.start(),
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                ..
            }) => game.pop(),
            Event::Key(_) => (),
        }

        render(&game, &config, &mut viewport, &mut renderer);
    }

    Ok(())
}

fn main() {
    match play() {
        Ok(()) => (),
        Err(e) if e == error::Error::NeedsHelp => println!("{}", e.to_string()),
        Err(e) if e == error::Error::Version => println!("{}", e.to_string()),
        Err(e) => {
            eprintln!(
                "{}\nError: {}",
                error::Error::NeedsHelp.to_string(),
                e.to_string()
            );
            std::process::exit(1);
        }
    }

    eprintln!("");
}
