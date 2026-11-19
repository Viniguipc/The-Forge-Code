//! Terminal ASCII / Unicode waveform renderer for quick CLI verification.

use crate::sim::SignalState;
use crate::value::{LogicVal, SimValue};
use std::collections::HashMap;

pub struct AsciiWaveRenderer;

impl AsciiWaveRenderer {
    pub fn render(
        signals: &HashMap<String, SignalState>,
        stop_time_fs: u64,
        columns: usize,
    ) -> String {
        if stop_time_fs == 0 {
            return "Simulation duration was 0 fs.\n".to_string();
        }

        let mut out = String::new();
        let mut sorted_names: Vec<String> = signals.keys().cloned().collect();
        sorted_names.sort();

        let max_name_len = sorted_names
            .iter()
            .map(|n| n.len())
            .max()
            .unwrap_or(8)
            .max(8);

        let wave_cols = columns.saturating_sub(max_name_len + 5).max(30);
        let time_per_col_fs = (stop_time_fs as f64 / wave_cols as f64).max(1.0);

        out.push_str("\n=== Simulation Waveform Diagram (ASCII) ===\n\n");

        // 1. Time header
        let stop_ns = stop_time_fs as f64 / 1_000_000.0;
        let mid_ns = stop_ns / 2.0;

        let left_pad = " ".repeat(max_name_len + 3);
        let mid_col = wave_cols / 2;
        let t_0 = "0 ns";
        let t_mid = format!("{mid_ns:.1} ns");
        let t_end = format!("{stop_ns:.1} ns");

        let spacing_1 = mid_col.saturating_sub(t_0.len());
        let spacing_2 = wave_cols.saturating_sub(mid_col + t_mid.len() + t_end.len());

        out.push_str(&format!(
            "{left_pad}{t_0}{}{t_mid}{}{t_end}\n",
            " ".repeat(spacing_1),
            " ".repeat(spacing_2)
        ));

        // Time ruler line
        let ruler: String = (0..wave_cols)
            .map(|i| if i % 10 == 0 { '|' } else { '-' })
            .collect();
        out.push_str(&format!("{left_pad}+{ruler}+\n"));

        // 2. Render each signal
        for name in &sorted_names {
            let sig = &signals[name];
            let name_padded = format!("{:>width$} : ", name, width = max_name_len);
            out.push_str(&name_padded);

            if sig.width == 1 {
                // 1-bit signal: render high/low transitions
                let mut wave = String::new();
                for col in 0..wave_cols {
                    let t_col_fs = (col as f64 * time_per_col_fs) as u64;
                    let val = Self::get_val_at_time(sig, t_col_fs);
                    let ch = match val {
                        SimValue::Logic(LogicVal::One | LogicVal::H) => '^',
                        SimValue::Logic(LogicVal::Zero | LogicVal::L) => '_',
                        SimValue::Logic(LogicVal::Z) => '~',
                        SimValue::Boolean(true) => '^',
                        SimValue::Boolean(false) => '_',
                        _ => 'x',
                    };
                    wave.push(ch);
                }
                out.push_str(&wave);
            } else {
                // Multi-bit bus: render hex transitions
                let mut bus = String::new();
                let mut last_hex = String::new();
                for col in 0..wave_cols {
                    let t_col_fs = (col as f64 * time_per_col_fs) as u64;
                    let val = Self::get_val_at_time(sig, t_col_fs);
                    let curr_hex = val.to_hex_string();
                    if curr_hex != last_hex {
                        bus.push('|');
                        last_hex = curr_hex;
                    } else {
                        bus.push('=');
                    }
                }
                out.push_str(&bus);
            }
            out.push('\n');
        }

        out.push_str(&format!("{left_pad}+{ruler}+\n"));
        out.push_str("Legend: ^ = High (1), _ = Low (0), ~ = High-Z (Z), x = Unknown (X/U), | = Bus transition\n\n");

        out
    }

    fn get_val_at_time(sig: &SignalState, time_fs: u64) -> SimValue {
        let mut curr = sig.curr_val.clone();
        for (t, v) in &sig.history {
            if *t <= time_fs {
                curr = v.clone();
            } else {
                break;
            }
        }
        curr
    }
}
