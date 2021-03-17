use std::time::{Duration, Instant};

#[derive(PartialEq)]
pub enum GameState {
    Stopped,
    Running(Instant),
    Finished {
        elapsed: Duration,
        wpm: usize,
        cpm: usize,
        word_count: usize,
        mistakes: usize,
        accuracy: f32,
    },
}

pub struct Game {
    pub text: String,
    pub text_chars: Vec<char>,
    pub state: GameState,
    input: String,
    mistakes: usize,
    word_count: usize,
    strict: bool,
    skip_word_on_space: bool,
}

impl Game {
    pub fn new(words: &[String], strict: bool, skip_word_on_space: bool) -> Self {
        let word_count = words.len();
        let text = words.join(" ");
        let text_chars = text.chars().collect::<Vec<_>>();

        Self {
            word_count,
            input: String::new(),
            text,
            text_chars,
            mistakes: 0,
            state: GameState::Running(Instant::now()),
            strict,
            skip_word_on_space,
        }
    }

    fn wpm(&self, dur: Duration) -> f32 {
        // the average word length in English is 4.7 characters, so we are using 5
        // ideally we would also compare this to collected correct characters to provide additional normalize results
        // ((self.text.chars().count() as f32 * (60.0 / dur.as_secs_f32())) / 5.0) as usize
        self.cpm(dur) / 5.0
    }

    fn cpm(&self, dur: Duration) -> f32 {
        self.text.chars().count() as f32 * (60.0 / dur.as_secs_f32())
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
        if self.input.len() == 0 {
            self.state = GameState::Running(Instant::now());
        }
        let current_index = self.input.len();
        let next_index = current_index + 1;

        // If skip_word_on_space: Skip the entire word if space was pressed anywhere
        // but on the first character of the word, or as the absolute
        // first input.
        match (self.skip_word_on_space, c, self.text.chars().skip(current_index).next()) {
            (false, ..) => {}
            // If space is pressed and current char is not a space,
            // and there is some player input, we advance the cursor
            // to the next word and count skipped chars as mistakes.
            (true, ' ', Some(current)) if current != ' ' && current_index > 0 => {
                // Don't advance if the cursor is at the beginning of a word
                match self.text.chars().skip(current_index - 1).next() {
                    None | Some(' ') => return,
                    Some(_) => (),
                };

                let mistakes = self
                    .text
                    .chars()
                    .skip(current_index)
                    .take_while(|&n| n != ' ')
                    .count()
                    + 1; // + 1 for the initial space character.

                (0..mistakes).for_each(|_| self.input.push(' '));
                self.mistakes += mistakes;

                if !self.strict && self.input.len() >= self.text.len() {
                    self.finish();
                }

                return;
            }
            (true, ' ', Some(nc)) if nc != ' ' => return,
            _ => (),
        };

        self.input.push(c);

        let b = self.text.chars().take(next_index).last();

        // if we have mistyped and press space after the last word
        // quit the game
        let should_quit = !self.strict && next_index >= self.text.len() + 1 && c == ' ';

        if !should_quit && Some(c) != b {
            self.mistakes += 1;
        }

        // if we input the text correctly or we press space after the last word
        if self.input == self.text || should_quit {
            self.finish();
        }

        if self.input.len() > self.text.len() {
            self.input.pop();
        }
    }

    pub fn pop(&mut self) {
        match self.input.chars().last() {
            Some(' ') => {
                while let Some(' ') = self.input.chars().last() {
                    self.input.pop();
                }
            }
            _ => drop(self.input.pop()),
        }
    }

    pub fn pop_word(&mut self) {
        let to_remove = self.input.len() - self
            .input
            .chars()
            .rev()
            .skip_while(|&c| c == ' ') // remove until non-whitespace is found
            .skip_while(|&c| c != ' ') // remove until whitespace is found
            .count();
        (0..to_remove).for_each(|_| drop(self.input.pop()));
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
                    } else {
                        a
                    }
                };
                self.state = GameState::Finished {
                    elapsed,
                    wpm: self.wpm(elapsed) as usize,
                    cpm: self.cpm(elapsed) as usize,
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
        let words = vec!["fives".to_string(), "fives".into(), "fives".into()];
        let gs = Game::new(&words, false, false);
        let wpm = gs.wpm(Duration::from_secs(60));
        assert_eq!(wpm as usize, 3);
    }

    #[test]
    fn test_word_count() {
        let words = vec!["one".to_string(), "two".into(), "three".into()];
        let gs = Game::new(&words, false, false);
        assert_eq!(gs.word_count, 3);
    }

    #[test]
    fn test_mistakes() {
        let mut gs = Game::new(&vec!["one".into()], false, false);
        gs.push('o');
        assert_eq!(gs.mistakes, 0);
        gs.push('o');
        assert_eq!(gs.mistakes, 1);
        gs.pop();
        gs.push('n');
        assert_eq!(gs.mistakes, 1);
    }
}
