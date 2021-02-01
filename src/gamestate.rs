use std::time::{Duration, Instant};

#[derive(PartialEq)]
pub enum GameState {
    Stopped,
    Running(Instant),
    Finished {
        elapsed: Duration,
        wpm: usize,
        word_count: usize,
        mistakes: usize,
        accuracy: f32,
    },
}

pub struct Game {
    input: String,
    pub text: String,
    pub text_chars: Vec<char>,
    mistakes: usize,
    word_count: usize,
    pub state: GameState,
}

impl Game {
    pub fn new(words: Vec<String>) -> Self {
        let word_count = words.len();
        let text = words.join(" ");
        let text_chars = text.chars().collect::<Vec<_>>();

        Self {
            word_count,
            input: String::new(),
            text,
            text_chars,
            mistakes: 0,
            state: GameState::Stopped,
        }
    }

    fn wpm(&self, dur: Duration) -> usize {
        let ms = dur.as_millis() as f32 / 1000.0 / 60.0;
        (self.word_count as f32 / ms) as usize
    }

    pub fn input(&self) -> Vec<(char, bool)> {
        let input = self.input.chars().collect::<Vec<_>>();
        let text = self.text_chars.iter().take(input.len());

        input
            .into_iter()
            .zip(text)
            .map(|(i, t)| (i, i == *t))
            .collect()
    }

    pub fn push(&mut self, c: char) {
        let current_index = self.input.len();
        let next_index = current_index + 1;

        let mut skip = false;
        let mut ignore = false;

        // if space is pressed
        if c == ' ' {
            if let Some(n) = self.text.chars().skip(current_index).next() {
                // and we are currently not on a space char
                if n != ' ' {
                    // ignore the input
                    ignore = true;
                    // if we are not at index 0
                    if current_index > 0 {
                        if let Some(prev) = self.text.chars().skip(current_index - 1).next() {
                            // and the previous char was not space
                            if prev != ' ' {
                                // skip word
                                skip = true;
                            }
                        }
                    }
                }
            }
        }

        if skip {
            // skip until next space character
            for _ in self.text.chars().skip(current_index).take_while(|&n| n != ' ') {
                self.input.push(' ');
                self.mistakes += 1;
            }
            // including the space character itself
            self.input.push(' ');
            self.mistakes += 1;
            if self.input.len() >= self.text.len() {
                self.finish();
            }
            return;
        }

        if ignore {
            return;
        }

        self.input.push(c);

        let a = self.input.chars().last();
        let b = self.text.chars().take(next_index).last();

        // if we have mistyped and press space after the last word
        // quit the game
        let should_quit = self.input.len() > self.text.len() + 1 && a.unwrap_or('.') == ' ';

        if !should_quit && a != b {
            self.mistakes += 1;
        }

        // if we input the text correctly or we press space after the last word
        if self.input == self.text || should_quit {
            self.finish();
        }
    }

    pub fn pop(&mut self) {
        self.input.pop();
    }

    pub fn start(&mut self) {
        self.state = GameState::Running(Instant::now());
    }

    pub fn finish(&mut self) {
        match self.state {
            GameState::Stopped | GameState::Finished { .. } => (),
            GameState::Running(now) => {
                let elapsed = now.elapsed();
                let mistakes = self.mistakes as f32;
                let char_count = self.text_chars.len() as f32;

                let accuracy = {
                    let a = 100.0 - (mistakes / char_count) * 100.0;
                    if a < 0.0 {
                        0.0
                    } else { a }
                };
                self.state = GameState::Finished {
                    elapsed,
                    wpm: self.wpm(elapsed),
                    word_count: self.word_count,
                    mistakes: self.mistakes,
                    accuracy,
                };
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_wpm() {
        let words = vec!["one".to_string(), "two".into(), "three".into()];
        let gs = Game::new(words);
        let wpm = gs.wpm(Duration::from_secs(60));
        assert_eq!(wpm, 3);
    }

    #[test]
    fn test_word_count() {
        let words = vec!["one".to_string(), "two".into(), "three".into()];
        let gs = Game::new(words);
        assert_eq!(gs.word_count, 3);
    }

    #[test]
    fn test_mistakes() {
        let mut gs = Game::new(vec!["one".into()]);
        gs.push('o');
        assert_eq!(gs.mistakes, 0);
        gs.push('o');
        assert_eq!(gs.mistakes, 1);
        gs.pop();
        gs.push('n');
        assert_eq!(gs.mistakes, 1);
    }
}
