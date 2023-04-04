use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
pub enum Token {
    #[regex("[a-zA-Z0-9]+")]
    Text,
    
    #[regex("[\\(\\)\\[\\]\\{\\},.:;\"\']+")]
    Punctuation,

    #[error]
    #[regex(r"[ \s\t\n\f\r]+", logos::skip)]
    Error,
}

