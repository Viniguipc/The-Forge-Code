use vhdl_sim_viewer::ast::*;
use vhdl_sim_viewer::eval::{Evaluator, SimpleScope};
use vhdl_sim_viewer::html_viewer::HtmlViewer;
use vhdl_sim_viewer::lexer::Lexer;
use vhdl_sim_viewer::value::{LogicVal, SimValue};
use vhdl_sim_viewer::vcd::VcdWriter;
use vhdl_sim_viewer::*;

#[test]
fn test_lexer_tokens() {
    let src = "entity Mux is port ( a, b : in std_logic; sel : in std_logic; y : out std_logic ); end entity;";
    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().expect("Lexing failed");
    assert!(!tokens.is_empty());
}

#[test]
fn test_lexer_bit_strings_and_numbers() {
    let src = "signal data : std_logic_vector(7 downto 0) := X\"A5\"; signal b : std_logic_vector(3 downto 0) := B\"1010\";";
    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().expect("Lexing failed");
    let has_hex = tokens.iter().any(|t| match &t.kind {
        token::TokenKind::BitStringLit(bits) => bits.len() == 8,
        _ => false,
    });
    assert!(has_hex, "Hex bit string should expand to 8 bits");
}

#[test]
fn test_parser_entity_and_arch() {
    let src = r#"
        entity AndGate is
            port (
                a : in std_logic;
                b : in std_logic;
                y : out std_logic
            );
        end entity;

        architecture rtl of AndGate is
        begin
            y <= a and b;
        end architecture;
    "#;
    let design = parse_vhdl_source(src).expect("Parse failed");
    assert_eq!(design.units.len(), 2);
}

#[test]
fn test_evaluator_logic_and_vectors() {
    let mut scope = SimpleScope::default();
    scope.values.insert("a".to_string(), SimValue::Logic(LogicVal::One));
    scope.values.insert("b".to_string(), SimValue::Logic(LogicVal::Zero));
    scope.values.insert(
        "vec".to_string(),
        SimValue::LogicVector(vec![LogicVal::One, LogicVal::Zero, LogicVal::One, LogicVal::One]),
    );

    let eval = Evaluator::new(&scope);

    let expr = Expr::BinaryOp(
        BinaryOpKind::And,
        Box::new(Expr::Ident("a".to_string(), common::Span::default())),
        Box::new(Expr::Ident("b".to_string(), common::Span::default())),
        common::Span::default(),
    );
    let res = eval.eval(&expr).unwrap();
    assert_eq!(res, SimValue::Logic(LogicVal::Zero));
}

#[test]
fn test_simulation_and_gate() {
    let src = r#"
        entity TestAnd is
        end entity;

        architecture sim of TestAnd is
            signal a, b, y : std_logic := '0';
        begin
            process
            begin
                a <= '0';
                b <= '0';
                wait for 10 ns;
                assert y = '0' report "0 and 0 failed";

                a <= '1';
                wait for 10 ns;
                assert y = '0' report "1 and 0 failed";

                b <= '1';
                wait for 10 ns;
                assert y = '1' report "1 and 1 failed";

                wait;
            end process;

            y <= a and b;
        end architecture;
    "#;

    let sim = compile_and_simulate(&[src], Some("TestAnd"), 50_000_000).expect("Simulation failed");
    let y_sig = sim.signals.get("y").expect("Signal y should exist");
    assert_eq!(y_sig.curr_val, SimValue::Logic(LogicVal::One));
}

#[test]
fn test_simulation_counter() {
    let src = r#"
        entity CounterTest is
        end entity;

        architecture sim of CounterTest is
            signal clk : std_logic := '0';
            signal count : integer := 0;
        begin
            process
            begin
                clk <= '0';
                wait for 5 ns;
                clk <= '1';
                wait for 5 ns;
            end process;

            process(clk)
            begin
                if rising_edge(clk) then
                    count <= count + 1;
                end if;
            end process;
        end architecture;
    "#;

    let sim = compile_and_simulate(&[src], Some("CounterTest"), 50_000_000).expect("Counter simulation failed");
    let cnt = sim.signals.get("count").expect("count signal missing");
    let val = cnt.curr_val.as_i64().expect("count should be integer");
    assert_eq!(val, 5, "Counter should have incremented 5 times in 50 ns");
}

#[test]
fn test_full_adder_component_instantiation() {
    let fa_src = std::fs::read_to_string("examples/full_adder.vhd").expect("Failed to read full_adder.vhd");
    let fa_tb_src = std::fs::read_to_string("examples/full_adder_tb.vhd").expect("Failed to read full_adder_tb.vhd");

    let sim = compile_and_simulate(&[&fa_src, &fa_tb_src], Some("FullAdder_tb"), 80_000_000)
        .expect("Full adder simulation failed");

    // At 80 ns, a=1, b=1, cin=1 -> sum=1, cout=1
    let sum = sim.signals.get("sum").expect("sum signal missing");
    let cout = sim.signals.get("cout").expect("cout signal missing");

    assert_eq!(sum.curr_val, SimValue::Logic(LogicVal::One));
    assert_eq!(cout.curr_val, SimValue::Logic(LogicVal::One));
}

#[test]
fn test_fsm_traffic_light() {
    let fsm_src = std::fs::read_to_string("examples/fsm_traffic.vhd").expect("Failed to read fsm_traffic.vhd");
    let fsm_tb_src = std::fs::read_to_string("examples/fsm_traffic_tb.vhd").expect("Failed to read fsm_traffic_tb.vhd");

    let sim = compile_and_simulate(&[&fsm_src, &fsm_tb_src], Some("TrafficLightFSM_tb"), 160_000_000)
        .expect("Traffic light simulation failed");

    assert!(sim.signals.contains_key("red"));
    assert!(sim.signals.contains_key("green"));
    assert!(sim.signals.contains_key("yellow"));
}

#[test]
fn test_vcd_and_html_generation() {
    let src = r#"
        entity SigTest is
        end entity;

        architecture rtl of SigTest is
            signal clk : std_logic := '0';
        begin
            process
            begin
                clk <= '0';
                wait for 10 ns;
                clk <= '1';
                wait for 10 ns;
            end process;
        end architecture;
    "#;

    let sim = compile_and_simulate(&[src], Some("SigTest"), 30_000_000).expect("Simulation failed");

    let vcd = VcdWriter::generate_vcd(&sim.signals, "SigTest").expect("VCD generation failed");
    assert!(vcd.contains("$timescale"));
    assert!(vcd.contains("$scope module SigTest"));
    assert!(vcd.contains("$dumpvars"));

    let html = HtmlViewer::generate_html(&sim.signals, "SigTest", 30_000_000);
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("VHDL Waveform Viewer"));
    assert!(html.contains("SigTest"));
}

#[test]
fn test_json_parser_and_ide_api() {
    use vhdl_sim_viewer::ide::IdeViewer;
    use vhdl_sim_viewer::json::parse_json;
    use vhdl_sim_viewer::server::HttpServer;

    // 1. IDE HTML contains Vivado-style components
    let ide_html = IdeViewer::generate_html();
    assert!(ide_html.contains("VHDL Studio"));
    assert!(ide_html.contains("Flow Navigator"));
    assert!(ide_html.contains("/api/simulate"));
    assert!(ide_html.contains("btn-upload-files"));
    assert!(ide_html.contains("file-upload-input"));

    // 2. Built-in examples JSON is valid and contains 3 examples
    let ex_json = HttpServer::examples_json();
    let parsed_ex = parse_json(&ex_json).expect("Examples JSON should parse");
    assert_eq!(parsed_ex.as_array().unwrap().len(), 3);

    // 3. /api/simulate handler compiles and simulates VHDL
    let payload = r#"{
        "top": "ApiTest",
        "stop_time_fs": 20000000,
        "files": [
            {
                "name": "test.vhd",
                "content": "entity ApiTest is end entity; architecture sim of ApiTest is signal x : std_logic := '0'; begin process begin x <= '1'; wait for 10 ns; x <= '0'; wait; end process; end architecture;"
            }
        ]
    }"#;

    let resp_str = HttpServer::handle_simulate_json(payload);
    let resp = parse_json(&resp_str).expect("Simulate response should be valid JSON");
    let obj = resp.as_object().unwrap();
    assert_eq!(obj.get("ok"), Some(&vhdl_sim_viewer::json::JsonValue::Bool(true)));
    assert_eq!(obj.get("top_entity").and_then(|v| v.as_str()), Some("ApiTest"));
    assert!(!obj.get("signals").and_then(|v| v.as_array()).unwrap().is_empty());

    // 4. Error handling returns ok:false with clear message
    let bad_payload = r#"{"files":[{"name":"bad.vhd","content":"entity Broken is"}]}"#;
    let err_str = HttpServer::handle_simulate_json(bad_payload);
    let err_resp = parse_json(&err_str).expect("Error response should be valid JSON");
    assert_eq!(
        err_resp.as_object().unwrap().get("ok"),
        Some(&vhdl_sim_viewer::json::JsonValue::Bool(false))
    );
}

