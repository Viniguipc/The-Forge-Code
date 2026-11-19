//! Value Change Dump (VCD) generator compatible with GTKWave, ModelSim, and custom viewers.

use crate::common::VhdlError;
use crate::sim::SignalState;
use crate::value::{LogicVal, SimValue};
use std::collections::{BTreeMap, HashMap};

pub struct VcdWriter;

impl VcdWriter {
    /// Generate standard VCD string from simulation signal states
    pub fn generate_vcd(
        signals: &HashMap<String, SignalState>,
        top_name: &str,
    ) -> Result<String, VhdlError> {
        let mut out = String::new();

        out.push_str("$date\n   2026-11-19\n$end\n");
        out.push_str("$version\n   Safe Rust VHDL Discrete-Event Simulator\n$end\n");
        out.push_str("$timescale 1ps $end\n");
        out.push_str(&format!("$scope module {top_name} $end\n"));

        // Assign unique short VCD identifier tokens (!, ", #, $, %, ...)
        let mut sorted_names: Vec<String> = signals.keys().cloned().collect();
        sorted_names.sort();

        let mut id_map: HashMap<String, String> = HashMap::new();
        for (i, name) in sorted_names.iter().enumerate() {
            let id = Self::generate_id(i);
            id_map.insert(name.clone(), id.clone());
            let sig = &signals[name];
            let width = sig.width.max(1);
            if width == 1 {
                out.push_str(&format!("$var wire 1 {id} {name} $end\n"));
            } else {
                let msb = width - 1;
                out.push_str(&format!("$var wire {width} {id} {name} [{msb}:0] $end\n"));
            }
        }

        out.push_str("$upscope $end\n");
        out.push_str("$enddefinitions $end\n");

        // Collect all events sorted by time (in picoseconds)
        // Group by time_ps -> list of (sig_name, SimValue)
        let mut time_events: BTreeMap<u64, Vec<(String, SimValue)>> = BTreeMap::new();

        for (name, sig) in signals {
            for &(time_fs, ref val) in &sig.history {
                let time_ps = time_fs / 1000;
                time_events
                    .entry(time_ps)
                    .or_default()
                    .push((name.clone(), val.clone()));
            }
        }

        // Initial dump ($dumpvars)
        out.push_str("$dumpvars\n");
        if let Some(init_events) = time_events.get(&0) {
            for (name, val) in init_events {
                let id = &id_map[name];
                out.push_str(&Self::format_vcd_val(val, id));
            }
        }
        out.push_str("$end\n");

        // Timestamped changes
        for (&time_ps, events) in &time_events {
            if time_ps == 0 {
                continue;
            }
            out.push_str(&format!("#{time_ps}\n"));
            for (name, val) in events {
                let id = &id_map[name];
                out.push_str(&Self::format_vcd_val(val, id));
            }
        }

        Ok(out)
    }

    fn generate_id(index: usize) -> String {
        let mut n = index;
        let mut s = String::new();
        loop {
            let rem = (n % 94) as u8;
            s.push((b'!' + rem) as char);
            n /= 94;
            if n == 0 {
                break;
            }
        }
        s
    }

    fn format_vcd_val(val: &SimValue, id: &str) -> String {
        match val {
            SimValue::Logic(l) => {
                let c = match l {
                    LogicVal::One | LogicVal::H => '1',
                    LogicVal::Zero | LogicVal::L => '0',
                    LogicVal::Z => 'z',
                    _ => 'x',
                };
                format!("{c}{id}\n")
            }
            SimValue::LogicVector(bits) => {
                let bit_str: String = bits
                    .iter()
                    .map(|b| match b {
                        LogicVal::One | LogicVal::H => '1',
                        LogicVal::Zero | LogicVal::L => '0',
                        LogicVal::Z => 'z',
                        _ => 'x',
                    })
                    .collect();
                format!("b{bit_str} {id}\n")
            }
            SimValue::Integer(i) => {
                format!("b{:b} {id}\n", *i as u64)
            }
            SimValue::Boolean(b) => {
                let c = if *b { '1' } else { '0' };
                format!("{c}{id}\n")
            }
            _ => format!("x{id}\n"),
        }
    }
}
