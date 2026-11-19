//! Tokens for the VHDL lexer.

use crate::common::Span;
use crate::value::LogicVal;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Entity,
    Architecture,
    Is,
    Port,
    Generic,
    End,
    Of,
    Begin,
    Signal,
    Variable,
    Constant,
    Component,
    Map,
    Process,
    If,
    Then,
    Elsif,
    Else,
    Case,
    When,
    For,
    While,
    Loop,
    Wait,
    Until,
    On,
    After,
    Assert,
    Report,
    Severity,
    In,
    Out,
    Inout,
    Buffer,
    Range,
    Downto,
    To,
    Type,
    Subtype,
    Package,
    Library,
    Use,
    All,
    Others,
    Null,
    // Logical & Arithmetic Operators
    And,
    Or,
    Nand,
    Nor,
    Xor,
    Xnor,
    Not,
    Mod,
    Rem,
    Abs,
    Sll,
    Srl,
    Sla,
    Sra,
    Rol,
    Ror,

    // Symbols
    Semicolon,
    Colon,
    Comma,
    Dot,
    Tick,
    DoubleQuote,
    LParen,
    RParen,
    Assign,       // :=
    SignalAssign, // <=
    Arrow,        // =>
    Eq,           // =
    Ne,           // /=
    Lt,           // <
    Le,           // <= (context sensitive, can be SignalAssign or Le)
    Gt,           // >
    Ge,           // >=
    Plus,         // +
    Minus,        // -
    Star,         // *
    Slash,        // /
    Ampersand,    // &
    DoubleStar,   // **

    // Literals
    Ident(String),
    IntLit(i64),
    RealLit(f64),
    CharLit(char),
    StringLit(String),
    BitStringLit(Vec<LogicVal>),
    TimeLit(u64), // femtoseconds

    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}
