#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    Slf,
    True,
    Let,
    While,

    Eof,
    Use,
    Export,
    LeftBracket,
    RightBracket,

    //YunGL
    Shader, Pipeline, Buffer, Render, Vertex, Fragment,
    Attributes, Primitive, Layout, Data, Source,

    Colon,
}
