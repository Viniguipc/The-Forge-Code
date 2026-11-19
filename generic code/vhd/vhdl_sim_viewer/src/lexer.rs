//! Lexer for VHDL source code.

use crate::common::{Span, VhdlError};
use crate::token::{Token, TokenKind};
use crate::value::LogicVal;

pub struct Lexer<'a> {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            _marker: std::marker::PhantomData,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.get(self.pos).copied();
        if let Some(c) = ch {
            self.pos += 1;
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        ch
    }

    fn skip_whitespace_and_comments(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else if c == '-' && self.peek_next() == Some('-') {
                // Comment until newline
                while let Some(ch) = self.peek() {
                    self.advance();
                    if ch == '\n' {
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, VhdlError> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token()?;
            let is_eof = tok.kind == TokenKind::Eof;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }

    pub fn next_token(&mut self) -> Result<Token, VhdlError> {
        self.skip_whitespace_and_comments();
        let span = Span::new(self.line, self.col);

        let ch = match self.peek() {
            Some(c) => c,
            None => return Ok(Token::new(TokenKind::Eof, span)),
        };

        // Identifiers or Bit strings (B"...", X"...", O"...") or Keywords
        if ch.is_alphabetic() || ch == '_' {
            // Check bit string: B"..." or X"..." or O"..."
            if (ch == 'B' || ch == 'b' || ch == 'X' || ch == 'x' || ch == 'O' || ch == 'o')
                && self.peek_next() == Some('"')
            {
                let prefix = ch.to_ascii_uppercase();
                self.advance(); // consume prefix
                self.advance(); // consume opening quote
                let mut bits = Vec::new();
                while let Some(bc) = self.peek() {
                    if bc == '"' {
                        self.advance();
                        break;
                    }
                    if bc == '_' {
                        self.advance();
                        continue;
                    }
                    self.advance();
                    match prefix {
                        'B' => {
                            if let Some(lv) = LogicVal::from_char(bc) {
                                bits.push(lv);
                            } else {
                                return Err(VhdlError::LexError {
                                    msg: format!("Invalid character in binary string: '{bc}'"),
                                    span,
                                });
                            }
                        }
                        'X' => {
                            if let Some(digit) = bc.to_digit(16) {
                                for i in (0..4).rev() {
                                    let b = (digit >> i) & 1;
                                    bits.push(if b == 1 { LogicVal::One } else { LogicVal::Zero });
                                }
                            } else if let Some(lv) = LogicVal::from_char(bc) {
                                // e.g. X"Z" or X"X" -> repeat 4 times
                                for _ in 0..4 {
                                    bits.push(lv);
                                }
                            } else {
                                return Err(VhdlError::LexError {
                                    msg: format!("Invalid hex digit in bit string: '{bc}'"),
                                    span,
                                });
                            }
                        }
                        'O' => {
                            if let Some(digit) = bc.to_digit(8) {
                                for i in (0..3).rev() {
                                    let b = (digit >> i) & 1;
                                    bits.push(if b == 1 { LogicVal::One } else { LogicVal::Zero });
                                }
                            } else {
                                return Err(VhdlError::LexError {
                                    msg: format!("Invalid octal digit in bit string: '{bc}'"),
                                    span,
                                });
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                return Ok(Token::new(TokenKind::BitStringLit(bits), span));
            }

            // Normal identifier or keyword
            let mut word = String::new();
            while let Some(c) = self.peek() {
                if c.is_alphanumeric() || c == '_' {
                    word.push(c);
                    self.advance();
                } else {
                    break;
                }
            }

            let lower = word.to_ascii_lowercase();
            let kind = match lower.as_str() {
                "entity" => TokenKind::Entity,
                "architecture" => TokenKind::Architecture,
                "is" => TokenKind::Is,
                "port" => TokenKind::Port,
                "generic" => TokenKind::Generic,
                "end" => TokenKind::End,
                "of" => TokenKind::Of,
                "begin" => TokenKind::Begin,
                "signal" => TokenKind::Signal,
                "variable" => TokenKind::Variable,
                "constant" => TokenKind::Constant,
                "component" => TokenKind::Component,
                "map" => TokenKind::Map,
                "process" => TokenKind::Process,
                "if" => TokenKind::If,
                "then" => TokenKind::Then,
                "elsif" => TokenKind::Elsif,
                "else" => TokenKind::Else,
                "case" => TokenKind::Case,
                "when" => TokenKind::When,
                "for" => TokenKind::For,
                "while" => TokenKind::While,
                "loop" => TokenKind::Loop,
                "wait" => TokenKind::Wait,
                "until" => TokenKind::Until,
                "on" => TokenKind::On,
                "after" => TokenKind::After,
                "assert" => TokenKind::Assert,
                "report" => TokenKind::Report,
                "severity" => TokenKind::Severity,
                "in" => TokenKind::In,
                "out" => TokenKind::Out,
                "inout" => TokenKind::Inout,
                "buffer" => TokenKind::Buffer,
                "range" => TokenKind::Range,
                "downto" => TokenKind::Downto,
                "to" => TokenKind::To,
                "type" => TokenKind::Type,
                "subtype" => TokenKind::Subtype,
                "package" => TokenKind::Package,
                "library" => TokenKind::Library,
                "use" => TokenKind::Use,
                "all" => TokenKind::All,
                "others" => TokenKind::Others,
                "null" => TokenKind::Null,
                "and" => TokenKind::And,
                "or" => TokenKind::Or,
                "nand" => TokenKind::Nand,
                "nor" => TokenKind::Nor,
                "xor" => TokenKind::Xor,
                "xnor" => TokenKind::Xnor,
                "not" => TokenKind::Not,
                "mod" => TokenKind::Mod,
                "rem" => TokenKind::Rem,
                "abs" => TokenKind::Abs,
                "sll" => TokenKind::Sll,
                "srl" => TokenKind::Srl,
                "sla" => TokenKind::Sla,
                "sra" => TokenKind::Sra,
                "rol" => TokenKind::Rol,
                "ror" => TokenKind::Ror,
                _ => TokenKind::Ident(lower),
            };
            return Ok(Token::new(kind, span));
        }

        // Numbers (integers, based integers, or floats)
        if ch.is_ascii_digit() {
            let mut num_str = String::new();
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() || c == '_' {
                    if c != '_' {
                        num_str.push(c);
                    }
                    self.advance();
                } else {
                    break;
                }
            }

            // Check if based integer, e.g. 2#1010# or 16#FF#
            if self.peek() == Some('#') {
                self.advance(); // consume '#'
                let base: u32 = num_str.parse().map_err(|_| VhdlError::LexError {
                    msg: format!("Invalid base in based integer: '{num_str}'"),
                    span,
                })?;
                let mut based_val_str = String::new();
                while let Some(c) = self.peek() {
                    if c == '#' {
                        self.advance(); // consume ending '#'
                        break;
                    }
                    if c != '_' {
                        based_val_str.push(c);
                    }
                    self.advance();
                }
                let val = i64::from_str_radix(&based_val_str, base).map_err(|_| VhdlError::LexError {
                    msg: format!("Invalid based literal '{based_val_str}' for base {base}"),
                    span,
                })?;
                return Ok(Token::new(TokenKind::IntLit(val), span));
            }

            // Check if float (period followed by digit)
            if self.peek() == Some('.') && self.peek_next().map_or(false, |c| c.is_ascii_digit()) {
                self.advance(); // consume '.'
                let mut frac_str = String::new();
                while let Some(c) = self.peek() {
                    if c.is_ascii_digit() || c == '_' {
                        if c != '_' {
                            frac_str.push(c);
                        }
                        self.advance();
                    } else {
                        break;
                    }
                }
                let full = format!("{num_str}.{frac_str}");
                let f: f64 = full.parse().unwrap_or(0.0);
                return Ok(Token::new(TokenKind::RealLit(f), span));
            }

            let val: i64 = num_str.parse().map_err(|_| VhdlError::LexError {
                msg: format!("Invalid integer literal '{num_str}'"),
                span,
            })?;

            return Ok(Token::new(TokenKind::IntLit(val), span));
        }

        // Single quote: could be character literal '0' or tick attribute 'event
        if ch == '\'' {
            // Lookahead: 'c'
            if let Some(_c1) = self.peek_next() {
                if self.chars.get(self.pos + 2).copied() == Some('\'') {
                    self.advance(); // consume first '
                    let c = self.advance().unwrap(); // char
                    self.advance(); // consume second '
                    return Ok(Token::new(TokenKind::CharLit(c), span));
                }
            }
            self.advance(); // lone tick
            return Ok(Token::new(TokenKind::Tick, span));
        }

        // String literal: "..."
        if ch == '"' {
            self.advance(); // consume '"'
            let mut s = String::new();
            while let Some(c) = self.peek() {
                if c == '"' {
                    self.advance();
                    // In VHDL, "" inside a string is an escaped quote
                    if self.peek() == Some('"') {
                        s.push('"');
                        self.advance();
                    } else {
                        break;
                    }
                } else {
                    s.push(c);
                    self.advance();
                }
            }
            return Ok(Token::new(TokenKind::StringLit(s), span));
        }

        // Two-character and one-character symbols
        self.advance();
        let kind = match ch {
            ';' => TokenKind::Semicolon,
            ',' => TokenKind::Comma,
            '.' => TokenKind::Dot,
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '&' => TokenKind::Ampersand,
            '*' => {
                if self.peek() == Some('*') {
                    self.advance();
                    TokenKind::DoubleStar
                } else {
                    TokenKind::Star
                }
            }
            '/' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::Ne
                } else {
                    TokenKind::Slash
                }
            }
            ':' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::Assign
                } else {
                    TokenKind::Colon
                }
            }
            '=' => {
                if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::Arrow
                } else {
                    TokenKind::Eq
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::SignalAssign // Can also be interpreted as Le
                } else {
                    TokenKind::Lt
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::Ge
                } else {
                    TokenKind::Gt
                }
            }
            _ => {
                return Err(VhdlError::LexError {
                    msg: format!("Unexpected character: '{ch}'"),
                    span,
                })
            }
        };

        Ok(Token::new(kind, span))
    }
}
