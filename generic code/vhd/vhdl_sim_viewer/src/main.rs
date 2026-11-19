#![forbid(unsafe_code)]

//! CLI binary for the Safe Rust VHDL Simulator and Waveform Viewer.

use std::env;
use std::fs;
use std::process::Command;
use vhdl_sim_viewer::ascii_wave::AsciiWaveRenderer;
use vhdl_sim_viewer::elaboration::Elaborator;
use vhdl_sim_viewer::html_viewer::HtmlViewer;
use vhdl_sim_viewer::lexer::Lexer;
use vhdl_sim_viewer::parser::Parser;
use vhdl_sim_viewer::server::HttpServer;
use vhdl_sim_viewer::sim::Simulator;
use vhdl_sim_viewer::vcd::VcdWriter;

fn parse_time_arg(s: &str) -> Option<u64> {
    let lower = s.to_ascii_lowercase();
    let num_part: String = lower.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
    let unit_part: String = lower.chars().skip_while(|c| c.is_ascii_digit() || *c == '.').collect();

    let val: f64 = num_part.parse().ok()?;
    let mult = match unit_part.trim() {
        "fs" => 1.0,
        "ps" => 1_000.0,
        "ns" | "" => 1_000_000.0, // default ns
        "us" => 1_000_000_000.0,
        "ms" => 1_000_000_000_000.0,
        "sec" | "s" => 1_000_000_000_000_000.0,
        _ => return None,
    };
    Some((val * mult).round() as u64)
}

fn print_help() {
    println!(
        r#"
Safe Rust VHDL Compiler, Simulator & Waveform Viewer
====================================================
Usage:
  vhdl_sim --ide [PORT]
  vhdl_sim <files.vhd...> [options]

Options:
  --ide [PORT]          Launch Vivado-style VHDL IDE & Waveform Viewer (default port 8080)
  --top <ENTITY>        Top-level entity to elaborate (default: auto-detect)
  --stop <TIME>         Simulation duration (e.g. 100ns, 50ps, 1us) [default: 100ns]
  --vcd <FILE>          Export simulation trace as Value Change Dump (.vcd)
  --html <FILE>         Generate standalone interactive HTML waveform viewer
  --serve [PORT]        Launch lightweight local web server (default port 8080)
  --view                Generate HTML waveform and open in default web browser
  --no-ascii            Disable ASCII waveform printing to terminal
  --help, -h            Show this help message

Examples:
  vhdl_sim --ide
  vhdl_sim --ide 9000
  vhdl_sim counter_4bit.vhd counter_4bit_tb.vhd --stop 200ns
  vhdl_sim fsm_traffic.vhd --stop 150ns --html wave.html --view
  vhdl_sim full_adder.vhd full_adder_tb.vhd --vcd trace.vcd --serve 8080
"#
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_help();
        return Ok(());
    }

    let mut files = Vec::new();
    let mut top_entity = None;
    let mut stop_time_fs = 100_000_000u64; // default 100 ns
    let mut vcd_path = None;
    let mut html_path = None;
    let mut serve_port = None;
    let mut ide_port = None;
    let mut auto_view = false;
    let mut show_ascii = true;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--ide" => {
                let mut port = 8080;
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    if let Ok(p) = args[i + 1].parse::<u16>() {
                        port = p;
                        i += 1;
                    }
                }
                ide_port = Some(port);
            }
            "--top" => {
                i += 1;
                if i < args.len() {
                    top_entity = Some(args[i].clone());
                }
            }
            "--stop" => {
                i += 1;
                if i < args.len() {
                    if let Some(fs) = parse_time_arg(&args[i]) {
                        stop_time_fs = fs;
                    } else {
                        eprintln!("Invalid time format: '{}'. Using default 100ns.", args[i]);
                    }
                }
            }
            "--vcd" => {
                i += 1;
                if i < args.len() {
                    vcd_path = Some(args[i].clone());
                }
            }
            "--html" => {
                i += 1;
                if i < args.len() {
                    html_path = Some(args[i].clone());
                }
            }
            "--serve" => {
                let mut port = 8080;
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    if let Ok(p) = args[i + 1].parse::<u16>() {
                        port = p;
                        i += 1;
                    }
                }
                serve_port = Some(port);
            }
            "--view" => {
                auto_view = true;
            }
            "--no-ascii" => {
                show_ascii = false;
            }
            arg if arg.starts_with('-') => {
                eprintln!("Unknown option: {arg}");
                print_help();
                return Ok(());
            }
            file => {
                files.push(file.to_string());
            }
        }
        i += 1;
    }

    if let Some(port) = ide_port {
        let url = format!("http://localhost:{port}");
        #[cfg(target_os = "windows")]
        let _ = Command::new("cmd").args(["/c", "start", &url]).spawn();
        #[cfg(target_os = "macos")]
        let _ = Command::new("open").arg(&url).spawn();
        #[cfg(target_os = "linux")]
        let _ = Command::new("xdg-open").arg(&url).spawn();

        HttpServer::serve_ide(port)?;
        return Ok(());
    }

    if files.is_empty() {
        eprintln!("Error: No VHDL source files specified.");
        print_help();
        return Ok(());
    }

    println!(">>> Parsing {} VHDL source file(s)...", files.len());
    let mut design_files = Vec::new();

    for filepath in &files {
        let content = fs::read_to_string(filepath)
            .map_err(|e| format!("Failed to read file '{filepath}': {e}"))?;
        let mut lexer = Lexer::new(&content);
        let tokens = lexer.tokenize().map_err(|e| format!("{e}"))?;
        let mut parser = Parser::new(tokens);
        let design_file = parser.parse_design_file().map_err(|e| format!("{e}"))?;
        println!("  ✓ Successfully parsed: {filepath}");
        design_files.push(design_file);
    }

    let elaborator = Elaborator::new(design_files);
    let resolved_top = match top_entity {
        Some(ref t) => t.clone(),
        None => elaborator.find_default_top().map_err(|e| format!("{e}"))?,
    };

    println!(">>> Elaborating top-level entity: '{resolved_top}'...");
    let elab_model = elaborator.elaborate(&resolved_top).map_err(|e| format!("{e}"))?;
    println!(
        "  ✓ Elaborated {} signals and {} processes.",
        elab_model.signals.len(),
        elab_model.processes.len()
    );

    let stop_ns = stop_time_fs as f64 / 1_000_000.0;
    println!(">>> Starting discrete-event simulation up to {stop_ns:.3} ns...");
    let mut simulator = Simulator::new(elab_model);
    simulator.run_until(stop_time_fs).map_err(|e| format!("{e}"))?;
    println!("  ✓ Simulation finished successfully at {stop_ns:.3} ns.");

    // Terminal ASCII Waveform
    if show_ascii {
        let ascii_diagram = AsciiWaveRenderer::render(&simulator.signals, stop_time_fs, 80);
        println!("{ascii_diagram}");
    }

    // Export VCD if requested
    if let Some(ref v_path) = vcd_path {
        let vcd_content = VcdWriter::generate_vcd(&simulator.signals, &resolved_top)
            .map_err(|e| format!("{e}"))?;
        fs::write(v_path, vcd_content)?;
        println!(">>> Exported VCD waveform to: {v_path}");
    }

    // Generate HTML viewer if requested or if --view or --serve is active
    let need_html = html_path.is_some() || serve_port.is_some() || auto_view;
    let html_content = if need_html {
        Some(HtmlViewer::generate_html(
            &simulator.signals,
            &resolved_top,
            stop_time_fs,
        ))
    } else {
        None
    };

    if let (Some(h_path), Some(content)) = (&html_path, &html_content) {
        fs::write(h_path, content)?;
        println!(">>> Generated interactive HTML waveform viewer: {h_path}");
    }

    if auto_view {
        let target_html = html_path
            .clone()
            .unwrap_or_else(|| format!("{resolved_top}_wave.html"));
        if html_path.is_none() {
            if let Some(content) = &html_content {
                fs::write(&target_html, content)?;
                println!(">>> Saved HTML waveform to: {target_html}");
            }
        }
        println!(">>> Opening waveform viewer in default browser...");
        #[cfg(target_os = "windows")]
        let _ = Command::new("cmd").args(["/c", "start", &target_html]).spawn();
        #[cfg(target_os = "macos")]
        let _ = Command::new("open").arg(&target_html).spawn();
        #[cfg(target_os = "linux")]
        let _ = Command::new("xdg-open").arg(&target_html).spawn();
    }

    // Serve via HTTP if requested
    if let (Some(port), Some(content)) = (serve_port, html_content) {
        HttpServer::serve(content, port)?;
    }

    Ok(())
}
