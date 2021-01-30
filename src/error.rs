pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    PathMissing,
    NoFiles,
    InsufficientWords,
    ZeroWordCount,
}

impl Error {
    pub fn to_string(self) -> String {   
        match self {
            Error::PathMissing => "provide a path to a Rust project".into(),
            Error::NoFiles => "No code files found".into(),
            Error::InsufficientWords => "Not enough words to meet word count".into(),
            Error::ZeroWordCount => "Word count can not be zero".into(),
        }
    }
}
