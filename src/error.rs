pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    PathMissing,
    NoFiles,
    InsufficientWords,
    ZeroWordCount,
    NeedsHelp,
    InvalidColor,
}

impl Error {
    pub fn to_string(self) -> String {
        match self {
            Error::PathMissing => "provide a path to a Rust project".into(),
            Error::InvalidColor => "Color needs to be an u8 or a color string.".into(),
            Error::NoFiles => "No code files found".into(),
            Error::InsufficientWords => "Not enough words to meet word count".into(),
            Error::ZeroWordCount => "Word count can not be zero".into(),
            Error::NeedsHelp => "Usage: toggle_cool_cow_says_type -t rs -w 5 path_to_project
    -t : extension of files to use for words. Defaults to rs for Rust.
    -w : number of words to type against. Defaults to 10.
    -s : strict mode. Input must be matched perfectly, otherwise game can't end!".into(),
        }
    }
}
