//! Safe, lightweight HTTP server for the waveform viewer and Vivado-style VHDL IDE.

use crate::ascii_wave::AsciiWaveRenderer;
use crate::elaboration::Elaborator;
use crate::ide::IdeViewer;
use crate::json::{escape_json_str, parse_json};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::sim::Simulator;
use crate::value::{LogicVal, SimValue};
use crate::vcd::VcdWriter;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

const EX_COUNTER_VHD: &str = include_str!("../examples/counter_4bit.vhd");
const EX_COUNTER_TB_VHD: &str = include_str!("../examples/counter_4bit_tb.vhd");
const EX_ADDER_VHD: &str = include_str!("../examples/full_adder.vhd");
const EX_ADDER_TB_VHD: &str = include_str!("../examples/full_adder_tb.vhd");
const EX_FSM_VHD: &str = include_str!("../examples/fsm_traffic.vhd");
const EX_FSM_TB_VHD: &str = include_str!("../examples/fsm_traffic_tb.vhd");

pub struct HttpServer;

impl HttpServer {
    /// Serve static HTML content on localhost:port
    pub fn serve(html_content: String, port: u16) -> Result<(), std::io::Error> {
        let addr = format!("127.0.0.1:{port}");
        let listener = TcpListener::bind(&addr)?;
        println!("\n=======================================================");
        println!("🚀 Waveform Viewer Web Server active at:");
        println!("   http://localhost:{port}");
        println!("   Press Ctrl+C in this terminal to stop the server.");
        println!("=======================================================\n");

        for stream in listener.incoming() {
            if let Ok(mut socket) = stream {
                let mut buf = [0u8; 1024];
                let _ = socket.read(&mut buf);
                Self::send_response(&mut socket, "200 OK", "text/html; charset=UTF-8", &html_content);
            }
        }
        Ok(())
    }

    /// Serve the full Vivado-style VHDL IDE and simulation REST API on localhost:port
    pub fn serve_ide(port: u16) -> Result<(), std::io::Error> {
        let addr = format!("127.0.0.1:{port}");
        let listener = TcpListener::bind(&addr)?;
        println!("\n=======================================================");
        println!("⚡ VHDL Studio (Vivado-Style IDE) active at:");
        println!("   http://localhost:{port}");
        println!("   Press Ctrl+C in this terminal to stop the server.");
        println!("=======================================================\n");

        for stream in listener.incoming() {
            if let Ok(mut socket) = stream {
                if let Some((method, path, body)) = Self::read_http_request(&mut socket) {
                    match (method.as_str(), path.as_str()) {
                        ("GET", "/") | ("GET", "/index.html") => {
                            Self::send_response(
                                &mut socket,
                                "200 OK",
                                "text/html; charset=UTF-8",
                                IdeViewer::generate_html(),
                            );
                        }
                        ("GET", "/api/examples") => {
                            let json = Self::examples_json();
                            Self::send_response(
                                &mut socket,
                                "200 OK",
                                "application/json; charset=UTF-8",
                                &json,
                            );
                        }
                        ("POST", "/api/simulate") => {
                            let json = Self::handle_simulate_json(&body);
                            Self::send_response(
                                &mut socket,
                                "200 OK",
                                "application/json; charset=UTF-8",
                                &json,
                            );
                        }
                        _ => {
                            Self::send_response(
                                &mut socket,
                                "404 Not Found",
                                "text/plain; charset=UTF-8",
                                "Not Found",
                            );
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn read_http_request(socket: &mut TcpStream) -> Option<(String, String, String)> {
        let mut raw = Vec::new();
        let mut chunk = [0u8; 4096];
        let mut header_end = None;

        // Read until headers end (\r\n\r\n)
        loop {
            let n = socket.read(&mut chunk).ok()?;
            if n == 0 {
                break;
            }
            raw.extend_from_slice(&chunk[..n]);
            if let Some(pos) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
                header_end = Some(pos + 4);
                break;
            }
            if raw.len() > 128 * 1024 {
                return None;
            }
        }

        let hdr_end = header_end?;
        let header_str = String::from_utf8_lossy(&raw[..hdr_end]).to_string();
        let mut lines = header_str.lines();
        let req_line = lines.next()?;
        let mut parts = req_line.split_whitespace();
        let method = parts.next()?.to_string();
        let path = parts.next()?.split('?').next()?.to_string();

        let mut content_length: usize = 0;
        for line in lines {
            if let Some((k, v)) = line.split_once(':') {
                if k.trim().eq_ignore_ascii_case("content-length") {
                    content_length = v.trim().parse().unwrap_or(0);
                }
            }
        }

        // Read remaining body bytes if needed
        let target_len = hdr_end + content_length;
        while raw.len() < target_len {
            let n = socket.read(&mut chunk).ok()?;
            if n == 0 {
                break;
            }
            raw.extend_from_slice(&chunk[..n]);
        }

        let body_end = target_len.min(raw.len());
        let body = String::from_utf8_lossy(&raw[hdr_end..body_end]).to_string();
        Some((method, path, body))
    }

    fn send_response(socket: &mut TcpStream, status: &str, content_type: &str, body: &str) {
        let header = format!(
            "HTTP/1.1 {status}\r\n\
             Content-Type: {content_type}\r\n\
             Content-Length: {}\r\n\
             Cache-Control: no-store\r\n\
             Connection: close\r\n\r\n",
            body.len()
        );
        let _ = socket.write_all(header.as_bytes());
        let _ = socket.write_all(body.as_bytes());
        let _ = socket.flush();
    }

    pub fn examples_json() -> String {
        let examples = [
            (
                "4-Bit Synchronous Counter",
                "Counter4Bit_tb",
                "180ns",
                vec![
                    ("counter_4bit.vhd", EX_COUNTER_VHD),
                    ("counter_4bit_tb.vhd", EX_COUNTER_TB_VHD),
                ],
            ),
            (
                "1-Bit Full Adder",
                "FullAdder_tb",
                "80ns",
                vec![
                    ("full_adder.vhd", EX_ADDER_VHD),
                    ("full_adder_tb.vhd", EX_ADDER_TB_VHD),
                ],
            ),
            (
                "Traffic Light FSM",
                "TrafficLightFSM_tb",
                "160ns",
                vec![
                    ("fsm_traffic.vhd", EX_FSM_VHD),
                    ("fsm_traffic_tb.vhd", EX_FSM_TB_VHD),
                ],
            ),
        ];

        let mut out = String::from("[");
        for (i, (title, top, stop, files)) in examples.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str(&format!(
                "{{\"title\":{},\"top\":{},\"stop\":{},\"files\":[",
                escape_json_str(title),
                escape_json_str(top),
                escape_json_str(stop),
            ));
            for (j, (fname, fcontent)) in files.iter().enumerate() {
                if j > 0 {
                    out.push(',');
                }
                out.push_str(&format!(
                    "{{\"name\":{},\"content\":{}}}",
                    escape_json_str(fname),
                    escape_json_str(fcontent),
                ));
            }
            out.push_str("]}");
        }
        out.push(']');
        out
    }

    pub fn handle_simulate_json(body: &str) -> String {
        let parsed = match parse_json(body) {
            Ok(v) => v,
            Err(e) => return Self::err_json(&format!("Invalid JSON payload: {e}")),
        };

        let obj = match parsed.as_object() {
            Some(o) => o,
            None => return Self::err_json("Expected JSON object"),
        };

        let stop_time_fs = obj
            .get("stop_time_fs")
            .and_then(|v| v.as_u64())
            .unwrap_or(100_000_000);

        let top_req = obj
            .get("top")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let files_arr = match obj.get("files").and_then(|v| v.as_array()) {
            Some(a) if !a.is_empty() => a,
            _ => return Self::err_json("No VHDL files provided"),
        };

        let mut design_files = Vec::new();
        for item in files_arr {
            let fobj = match item.as_object() {
                Some(o) => o,
                None => continue,
            };
            let fname = fobj.get("name").and_then(|v| v.as_str()).unwrap_or("source.vhd");
            let content = fobj.get("content").and_then(|v| v.as_str()).unwrap_or("");

            let mut lexer = Lexer::new(content);
            let tokens = match lexer.tokenize() {
                Ok(t) => t,
                Err(e) => return Self::err_json(&format!("[{fname}] {e}")),
            };
            let mut parser = Parser::new(tokens);
            let df = match parser.parse_design_file() {
                Ok(d) => d,
                Err(e) => return Self::err_json(&format!("[{fname}] {e}")),
            };
            design_files.push(df);
        }

        let elaborator = Elaborator::new(design_files);
        let resolved_top = match top_req {
            Some(t) => t,
            None => match elaborator.find_default_top() {
                Ok(t) => t,
                Err(e) => return Self::err_json(&format!("{e}")),
            },
        };

        let elab_model = match elaborator.elaborate(&resolved_top) {
            Ok(m) => m,
            Err(e) => return Self::err_json(&format!("{e}")),
        };
        let process_count = elab_model.processes.len();

        let mut simulator = Simulator::new(elab_model);
        if let Err(e) = simulator.run_until(stop_time_fs) {
            return Self::err_json(&format!("{e}"));
        }

        let ascii_wave = AsciiWaveRenderer::render(&simulator.signals, stop_time_fs, 80);
        let vcd = VcdWriter::generate_vcd(&simulator.signals, &resolved_top).unwrap_or_default();

        // Build signals JSON
        let mut sorted_names: Vec<String> = simulator.signals.keys().cloned().collect();
        sorted_names.sort();

        let mut sigs_json = String::from("[");
        for (idx, name) in sorted_names.iter().enumerate() {
            if idx > 0 {
                sigs_json.push(',');
            }
            let sig = &simulator.signals[name];
            let width = sig.width.max(1);
            sigs_json.push_str(&format!(
                "{{\"name\":{},\"width\":{width},\"transitions\":[",
                escape_json_str(name)
            ));

            for (t_idx, (time_fs, val)) in sig.history.iter().enumerate() {
                if t_idx > 0 {
                    sigs_json.push(',');
                }
                let val_str = match val {
                    SimValue::Logic(l) => l.to_char().to_string(),
                    SimValue::LogicVector(bits) => bits.iter().map(|b| b.to_char()).collect(),
                    SimValue::Integer(i) => i.to_string(),
                    SimValue::Boolean(b) => (if *b { "1" } else { "0" }).to_string(),
                    _ => val.to_string(),
                };
                let hex_str = val.to_hex_string();
                let bin_str = match val {
                    SimValue::Logic(l) => l.to_char().to_string(),
                    SimValue::LogicVector(bits) => bits.iter().map(|b| b.to_char()).collect(),
                    SimValue::Integer(i) => format!("{i:b}"),
                    SimValue::Boolean(b) => if *b { "1" } else { "0" }.to_string(),
                    _ => val_str.clone(),
                };
                let dec_str = val.as_i64().map_or_else(|| val_str.clone(), |i| i.to_string());

                let is_high = matches!(
                    val,
                    SimValue::Logic(LogicVal::One | LogicVal::H) | SimValue::Boolean(true)
                );
                let is_low = matches!(
                    val,
                    SimValue::Logic(LogicVal::Zero | LogicVal::L) | SimValue::Boolean(false)
                );
                let is_z = matches!(val, SimValue::Logic(LogicVal::Z));

                sigs_json.push_str(&format!(
                    "{{\"t\":{time_fs},\"val\":{},\"hex\":{},\"bin\":{},\"dec\":{},\"h\":{is_high},\"l\":{is_low},\"z\":{is_z}}}",
                    escape_json_str(&val_str),
                    escape_json_str(&hex_str),
                    escape_json_str(&bin_str),
                    escape_json_str(&dec_str),
                ));
            }
            sigs_json.push_str("]}");
        }
        sigs_json.push(']');

        // Logs JSON
        let mut logs_json = String::from("[");
        for (i, log) in simulator.logs.iter().enumerate() {
            if i > 0 {
                logs_json.push(',');
            }
            logs_json.push_str(&escape_json_str(log));
        }
        logs_json.push(']');

        format!(
            "{{\"ok\":true,\"top_entity\":{},\"stop_time_fs\":{stop_time_fs},\"process_count\":{process_count},\"logs\":{logs_json},\"signals\":{sigs_json},\"ascii_wave\":{},\"vcd\":{}}}",
            escape_json_str(&resolved_top),
            escape_json_str(&ascii_wave),
            escape_json_str(&vcd),
        )
    }

    fn err_json(msg: &str) -> String {
        format!("{{\"ok\":false,\"error\":{}}}", escape_json_str(msg))
    }
}
