use std::fs::read_to_string;
use std::path::PathBuf;

use rand::prelude::*;
use walkdir::WalkDir;

use crate::config::Config;
use crate::error::{Error, Result};

fn find_files(path: PathBuf, required_ext: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    for entry in WalkDir::new(path) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.into_path();
        if let Some(file_ext) = path.extension() {
            match file_ext.to_str() {
                Some(s) if s == required_ext => paths.push(path),
                Some(_) | None => continue,
            }
        }
    }

    paths
}

fn code_to_words(code: String) -> Vec<String> {
    let words = code
        .split('\n')
        .map(|line| match line.find("//") {
            Some(pos) => &line[..pos],
            None => line,
        })
        .map(|line| line.split_whitespace())
        .flatten()
        .map(String::from)
        .collect::<Vec<_>>();

    words
}

fn choose_words(words: Vec<String>, word_count: usize, rng: &mut ThreadRng) -> Vec<String> {
    let max = words.len() - word_count;
    let to = rng.gen_range(0..=max);
    words[to..to + word_count].into()
}

pub fn words(config: &Config, max_len: usize) -> Result<Vec<String>> {
    let mut rng = thread_rng();

    let mut files = find_files(config.project_path.clone(), &config.file_extension);
    if files.len() == 0 {
        return Err(Error::NoFiles);
    }

    loop {
        match files.choose(&mut rng) {
            Some(file) => {
                let file_index = files.iter().position(|f| f == file).unwrap();
                let file = files.remove(file_index);
                let mut code = read_to_string(file)
                    .expect("file was deleted during execution!")
                    .trim()
                    .to_string();
                if code.chars().count() > max_len {
                    code = code[..max_len].to_string();
                }
                let words = code_to_words(code);

                if words.len() < config.word_count {
                    continue;
                }

                let words = choose_words(words, config.word_count, &mut rng);

                return Ok(words);
            }
            None => return Err(Error::InsufficientWords),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_words() {
        let words = code_to_words("a b c".into());
        let expected = vec!["a".to_string(), "b".into(), "c".into()];
        assert_eq!(words, expected);

        let words = code_to_words("a //b c d".into());
        let expected = vec!["a".to_string()];
        assert_eq!(words, expected);
    }

    #[test]
    fn choose_some_words() {
        let words = vec!["a".to_string(), "b".into(), "c".into()];
        let mut rng = thread_rng();
        let chosen = choose_words(words.clone(), 3, &mut rng);
        assert_eq!(words, chosen);
    }

    // #[test]
    // fn split_words() {
    //     let text = "a word::here".to_string();
    //     let words = code_to_words(text);
    //     assert_eq!(words.len(), 3);
    // }
}
