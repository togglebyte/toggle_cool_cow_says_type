use std::env::Args;
use std::path::PathBuf;

use crate::error::{Error, Result};
use tinybit::Color;

#[derive(Debug)] 
pub struct Config {
    pub project_path: PathBuf,
    pub file_extension: String,
    pub word_count: usize,
    pub strict: bool,
    pub skip_word_on_space: bool,
    pub min_accuracy: Option<f32>,
    pub cursor_foreground_color: Color,
    pub cursor_background_color: Color,
}

impl Config {
    pub fn from_iter(mut args: impl Iterator<Item = String>) -> Result<Self> {
        let mut word_count = 10;
        let mut project_path = None;
        let mut file_extension = "rs".to_string();
        let mut foreground_color = None;
        let mut background_color = None;
        let mut min_accuracy = None;
        let mut skip_word_on_space = false;

        let mut argc = 0;
        let mut strict = false;

        while let Some(arg) = args.next() {
            argc += 1;
            match arg.as_ref() {
                "-h" | "-?" | "--h" | "--?" => return Err(Error::NeedsHelp),
                "-w" => {
                    word_count = args
                        .next()
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(10)
                }
                "-t" => {
                    file_extension = args.next().unwrap_or("rs".to_string());
                    if file_extension.starts_with('.') {
                        file_extension.remove(0);
                    }
                }
                "-ma" => {
                    min_accuracy = args
                        .next()
                        .and_then(|s| s.parse::<f32>().ok())
                }
                "-v" => return Err(Error::Version),
                "-s" => strict = true,
                "-ss" => skip_word_on_space = true,
                "-cf" => {
                    let front_color = args.next().unwrap_or("green".to_string());
                    if let Ok(c) = front_color.parse::<u8>() {
                        foreground_color = Some(Color::AnsiValue(c));
                    } else {
                        if let Ok(c) = front_color.parse::<Color>() {
                            foreground_color = Some(c);
                        } else {
                            return Err(Error::InvalidColor);
                        }
                    }
                }
                "-cb" => {
                    let back_color = args.next().unwrap_or("dark_grey".to_string());
                    if let Ok(c) = back_color.parse::<u8>() {
                        background_color = Some(Color::AnsiValue(c));
                    } else {
                        if let Ok(c) = back_color.parse::<Color>() {
                            background_color = Some(c);
                        } else {
                            return Err(Error::InvalidColor);
                        }
                    }
                }
                arg => {
                    let path: String = shellexpand::tilde(arg).to_string();
                    project_path = Some(path);
                }
            }
        }

        if argc <= 1 {
            return Err(Error::NeedsHelp);
        }

        let project_path = match project_path {
            Some(p) => p,
            None => return Err(Error::PathMissing),
        };

        if word_count == 0 {
            return Err(Error::ZeroWordCount);
        }

        let inst = Self {
            word_count,
            project_path: project_path.into(),
            file_extension,
            strict,
            cursor_foreground_color: foreground_color.unwrap_or(Color::Green),
            cursor_background_color: background_color.unwrap_or(Color::DarkGrey),
            min_accuracy,
            skip_word_on_space,
        };

        Ok(inst)
    }

    pub fn from_args(args: Args) -> Result<Self> {
        Self::from_iter(args)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::error::Error::PathMissing;

    #[test]
    fn parse_word_count() {
        let args = "-w 12 /".split_whitespace().map(str::to_owned);
        let config = Config::from_iter(args).unwrap();
        assert_eq!(config.word_count, 12);
    }

    #[test]
    fn parse_error() {
        // Missing path arg
        let args = vec!["-s".to_string(), "-t".into(), "c".into()];
        assert!(matches!(
            Config::from_iter(args.into_iter()),
            Err(PathMissing)
        ));
    }

    #[test]
    fn parse_extension() {
        let args = "-p / -t .c".split_whitespace().map(str::to_owned);
        let config = Config::from_iter(args).unwrap();
        assert_eq!(config.file_extension, "c".to_string());

        let args = "-p / ".split_whitespace().map(str::to_owned);
        let config = Config::from_iter(args).unwrap();
        assert_eq!(config.file_extension, "rs".to_string());
    }
}
