#![forbid(unsafe_code)]

//! # Safe Rust VHDL Compiler, Simulator, and Waveform Viewer
//!
//! A pure Safe Rust implementation of a VHDL parser, elaboration engine,
//! discrete-event simulator conforming to IEEE 1076, and waveform visualizers (VCD, ASCII, HTML5 Canvas).

pub mod ascii_wave;
pub mod ast;
pub mod common;
pub mod elaboration;
pub mod eval;
pub mod html_viewer;
pub mod ide;
pub mod json;
pub mod lexer;
pub mod parser;
pub mod server;
pub mod sim;
pub mod token;
pub mod value;
pub mod vcd;

use common::VhdlError;
use elaboration::Elaborator;
use lexer::Lexer;
use parser::Parser;
use sim::Simulator;

/// Parse a single VHDL source string into an AST DesignFile.
pub fn parse_vhdl_source(source: &str) -> Result<ast::DesignFile, VhdlError> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse_design_file()
}

/// Compile multiple VHDL sources, elaborate top entity, and run simulation up to stop_time_fs.
pub fn compile_and_simulate(
    sources: &[&str],
    top_entity: Option<&str>,
    stop_time_fs: u64,
) -> Result<Simulator, VhdlError> {
    let mut files = Vec::new();
    for src in sources {
        files.push(parse_vhdl_source(src)?);
    }

    let elaborator = Elaborator::new(files);
    let top = match top_entity {
        Some(t) => t.to_string(),
        None => elaborator.find_default_top()?,
    };

    let model = elaborator.elaborate(&top)?;
    let mut sim = Simulator::new(model);
    sim.run_until(stop_time_fs)?;

    Ok(sim)
}
