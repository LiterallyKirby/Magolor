use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
pub enum Token {
    #[regex(r#""([^"]*)""#, |lex| lex.slice().trim_matches('"').to_string())]
    String(String),
    
    // Numeric literals
    #[regex(r"[0-9]+\.[0-9]+f64", |lex| lex.slice().trim_end_matches("f64").parse::<f64>().ok())]
    Float64(f64),
    #[regex(r"[0-9]+\.[0-9]+f32", |lex| lex.slice().trim_end_matches("f32").parse::<f32>().ok())]
    Float32(f32),
    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f32>().ok())] // Default float is f32
    DefaultFloat(f32),
    #[regex(r"[0-9]+i64", |lex| lex.slice().trim_end_matches("i64").parse::<i64>().ok())]
    Integer64(i64),
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i32>().ok())]
    Integer(i32),
    
    // Boolean literals
    #[regex(r"true|false", |lex| lex.slice().parse::<bool>().ok())]
    Bool(bool),
    
    // Keywords
    #[regex("fn|func|Fn|Func")]
    Func,
    #[regex("return|Return")]
    Return,
    #[token("let")]
    Let,
    #[token("use")]
    Use,
    #[regex("void|Void")]
    Void,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("elif")]
    Elif,
    #[token(">")]
    Greater,
    #[token("<")]
    Less,
    #[token("!=")]
    NotEq,
    #[token("<=")]
    LessEq,
    #[token(">=")]
    GreaterEq,
    #[token("==")]
    EqEq,

    // Type keywords
    #[token("i32")]
    I32Type,
    #[token("i64")]
    I64Type,
    #[token("f32")]
    F32Type,
    #[token("f64")]
    F64Type,
    #[regex("String|string")]
    StringType,
    #[regex("bool|Bool")]
    BoolType,
    
    // Punctuation
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token(";")]
    Semicolon,
    #[token(":")]
    Colon,
    #[token("=")]
    Eq,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    
    // Identifiers (must come after keywords to avoid conflicts)
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),
    
    // Skip whitespace
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

pub fn tokenizeFile(input: &str) -> Vec<Token> {
    Token::lexer(input)
        .filter_map(|tok| tok.ok()) // only keep valid tokens
        .collect()
}
