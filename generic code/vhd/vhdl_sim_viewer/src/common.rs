//! Common data structures, spans, and error handling for the VHDL engine.

use std::fmt;

/// Represents a location in source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

impl Span {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

/// Engine errors for lexing, parsing, elaboration, and simulation.
#[derive(Debug, Clone, PartialEq)]
pub enum VhdlError {
    LexError {
        msg: String,
        span: Span,
    },
    ParseError {
        msg: String,
        span: Span,
    },
    ElabError {
        msg: String,
    },
    SimError {
        msg: String,
        time_fs: u64,
    },
}

impl fmt::Display for VhdlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VhdlError::LexError { msg, span } => write!(f, "Lexical error at {span}: {msg}"),
            VhdlError::ParseError { msg, span } => write!(f, "Parse error at {span}: {msg}"),
            VhdlError::ElabError { msg } => write!(f, "Elaboration error: {msg}"),
            VhdlError::SimError { msg, time_fs } => {
                let time_ns = *time_fs as f64 / 1_000_000.0;
                write!(f, "Simulation error at {time_ns:.3} ns: {msg}")
            }
        }
    }
}

impl std::error::Error for VhdlError {}
