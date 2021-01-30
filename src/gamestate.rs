use std::time::{Duration, Instant};

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
        self.input.push(c);

        let input_chars = self.input.chars().collect::<Vec<_>>();
        let text_chars = self.text.chars().take(input_chars.len());

        let a = input_chars.last().expect("this should always be some");
        let b = text_chars.last().expect("this should always be some");

        if *a != b {
            self.mistakes += 1;
        }

        if self.input == self.text {
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
