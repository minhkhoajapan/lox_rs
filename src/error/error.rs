#[derive(Debug, PartialEq, Clone)]
pub enum LoxError {
    UnterminatedString,
    UnknownSymbol(char),
}

impl std::fmt::Display for LoxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxError::UnterminatedString => write!(f, "Unterminated string"),
            LoxError::UnknownSymbol(c) => write!(f, "Unexpected character: '{}'", c),
        }
    }
}
