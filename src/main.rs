use std::env::args;

use tinybit::events::{events, Event, EventModel, KeyCode, KeyEvent};
use tinybit::widgets::Text;
use tinybit::{term_size, Color, Pixel, Renderer, ScreenPos, ScreenSize, StdoutTarget, Viewport};

mod config;
mod error;
mod gamestate;
mod words;

use config::Config;
use gamestate::{Game, GameState};
use words::words;

fn render(game: &Game, viewport: &mut Viewport, renderer: &mut Renderer<StdoutTarget>) {
    match game.state {
        GameState::Running(_) => {
            let input = game.input();
            let text = game.text_chars.iter().skip(input.len());

            let mut x = (viewport.size.width - game.text.chars().count() as u16) / 2;
            let y = viewport.size.height / 2;
            for (c, correct) in input {
                let color = match correct {
                    true => Color::Blue,
                    false => Color::Red,
                };
                viewport.draw_pixel(Pixel::new(c, ScreenPos::new(x, y), Some(color), None));
                x += 1;
            }

            for c in text {
                viewport.draw_pixel(Pixel::new(
                    *c,
                    ScreenPos::new(x, y),
                    Some(Color::White),
                    None,
                ));
                x += 1;
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
            mistakes,
            word_count,
            accuracy,
        } => {
            let text = format!(
                "time: {} seconds | wpm: {} | mistakes: {} | accuracy: {:.2}% | word count: {}",
                elapsed.as_secs(),
                wpm,
                mistakes,
                accuracy,
                word_count
            );
            let x = (viewport.size.width - text.chars().count() as u16) / 2;
            let y = viewport.size.height / 2;

            let text = Text::new(text, None, None);
            viewport.draw_widget(&text, ScreenPos::new(x, y));
        }
    }

    renderer.render(viewport);
}

fn play() -> error::Result<()> {
    let config = Config::from_args(args())?;
    let selected_words = words(&config)?;

    let mut game = Game::new(selected_words);

    let (w, h) = term_size().expect("could not get terminal size");
    let mut viewport = Viewport::new(ScreenPos::zero(), ScreenSize::new(w, h));

    let stdout = StdoutTarget::new().expect("failed to enter raw mode");
    let mut renderer = Renderer::new(stdout);

    render(&game, &mut viewport, &mut renderer);

    for event in events(EventModel::Blocking) {
        match event {
            Event::Tick => unreachable!(),
            Event::Resize(w, h) => {
                viewport.resize(w, h);
                renderer.clear();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                ..
            }) => match game.state {
                GameState::Finished { .. } => {
                    if c == 'y' {
                        let selected_words = words(&config)?;
                        game = Game::new(selected_words);
                    } else if c == 'n' {
                        break;
                    }
                }
                GameState::Running(_) => {
                    game.push(c);
                }
                GameState::Stopped => game.start(),
            },
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                ..
            }) => game.pop(),
            Event::Key(KeyEvent {
                code: KeyCode::Esc, ..
            }) => break,
            Event::Key(_) => (),
        }

        render(&game, &mut viewport, &mut renderer);
    }

    Ok(())
}

fn main() {
    match play() {
        Ok(()) => (),
        Err(e) if e == error::Error::NeedsHelp => {
            println!("{}", e.to_string());
        }
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
