use logos::Logos;

#[derive(Logos, Debug, PartialEq)]

pub enum Token {
    #[regex(r#""([^"]*)""#, |lex| lex.slice().trim_matches('"').to_string())]
    String(String),

    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i32>().ok())]
    Integer(i32),

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token(";")]
    Semicolon,

    #[token("let")]
    Let,

    #[token("=")]
    Eq,

    #[token("use")]
    Use,

    #[token(",")]
    Comma,

    #[token(".")]
    Dot,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    #[regex(r"[ \t\n\f]+", logos::skip)] // skip whitespace
    Error,
}

pub fn tokenizeFile(input: &str) -> Vec<Token> {
    Token::lexer(input)
        .filter_map(|tok| tok.ok()) // only keep valid tokens
        .collect()
}
