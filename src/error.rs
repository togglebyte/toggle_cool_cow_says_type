pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    PathMissing,
    NoFiles,
    InsufficientWords,
    ZeroWordCount,
    NeedsHelp,
}

impl Error {
    pub fn to_string(self) -> String {
        match self {
            Error::PathMissing => "provide a path to a Rust project".into(),
            Error::NoFiles => "No code files found".into(),
            Error::InsufficientWords => "Not enough words to meet word count".into(),
            Error::ZeroWordCount => "Word count can not be zero".into(),
            Error::NeedsHelp => "Usage: toggle_cool_cow_says_type -p path_to_project -t rs -w 5
    -p : path to a code project or word list files. REQUIRED.
    -t : extension of files to use for words. Defaults to rs for Rust.
    -w : number of words to type against. Defaults to 10.".into(),
        }
    }
}
