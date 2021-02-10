use std::env::args;

use tinybit::events::{events, Event, EventModel, KeyCode, KeyEvent, KeyModifiers};
use tinybit::widgets::Text;
use tinybit::{term_size, Color, Pixel, Renderer, ScreenPos, ScreenSize, StdoutTarget, Viewport};
use tinybit::render::RenderTarget;

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
            let text = format!(
                "time: {} seconds | wpm: {} (cpm: {}) | mistakes: {} | accuracy: {:.2}% | word count: {}",
                elapsed.as_secs(),
                wpm,
                cpm,
                mistakes,
                accuracy,
                word_count
            );
            let x = (viewport.size.width - text.chars().count() as u16) / 2;
            let y = viewport.size.height / 2 - 1;

            let text = Text::new(text, None, None);
            viewport.draw_widget(&text, ScreenPos::new(x, y));

            let text = "Try again? Y(es) | N(no) | R(etry same words)".to_string();
            let text = Text::new(text, None, None);
            viewport.draw_widget(&text, ScreenPos::new(x, y + 2));
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

    let mut game = Game::new(&selected_words, config.strict);

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
                        game = Game::new(&selected_words, config.strict);
                    }
                    'r' => game = Game::new(&selected_words, config.strict),
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
}
