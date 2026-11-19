//! Standalone interactive HTML5 Canvas waveform viewer.

use crate::sim::SignalState;
use crate::value::{LogicVal, SimValue};
use std::collections::HashMap;

pub struct HtmlViewer;

impl HtmlViewer {
    pub fn generate_html(
        signals: &HashMap<String, SignalState>,
        top_name: &str,
        stop_time_fs: u64,
    ) -> String {
        let mut sorted_names: Vec<String> = signals.keys().cloned().collect();
        sorted_names.sort();

        // Build JSON representation of signals and transitions
        let mut signals_json = String::from("[\n");
        for (idx, name) in sorted_names.iter().enumerate() {
            let sig = &signals[name];
            let width = sig.width.max(1);

            let mut transitions_json = String::from("[");
            for (t_idx, (time_fs, val)) in sig.history.iter().enumerate() {
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

                let is_high = match val {
                    SimValue::Logic(LogicVal::One | LogicVal::H) | SimValue::Boolean(true) => true,
                    _ => false,
                };
                let is_low = match val {
                    SimValue::Logic(LogicVal::Zero | LogicVal::L) | SimValue::Boolean(false) => true,
                    _ => false,
                };
                let is_z = match val {
                    SimValue::Logic(LogicVal::Z) => true,
                    _ => false,
                };

                transitions_json.push_str(&format!(
                    r#"{{"t":{time_fs},"val":"{val_str}","hex":"{hex_str}","bin":"{bin_str}","dec":"{dec_str}","h":{is_high},"l":{is_low},"z":{is_z}}}"#
                ));
                if t_idx + 1 < sig.history.len() {
                    transitions_json.push(',');
                }
            }
            transitions_json.push(']');

            signals_json.push_str(&format!(
                r#"  {{"name":"{name}","width":{width},"transitions":{transitions_json}}}"#
            ));

            if idx + 1 < sorted_names.len() {
                signals_json.push_str(",\n");
            } else {
                signals_json.push('\n');
            }
        }
        signals_json.push(']');

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>VHDL Waveform Viewer - {top_name}</title>
  <style>
    * {{ box-sizing: border-box; margin: 0; padding: 0; }}
    body {{
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
      background-color: #0b0f19;
      color: #e2e8f0;
      overflow: hidden;
      height: 100vh;
      display: flex;
      flex-direction: column;
    }}
    header {{
      background: #111827;
      border-bottom: 1px solid #1f2937;
      padding: 10px 18px;
      display: flex;
      align-items: center;
      justify-content: space-between;
      user-select: none;
    }}
    .title-area {{
      display: flex;
      align-items: center;
      gap: 12px;
    }}
    .badge {{
      background: #1e293b;
      border: 1px solid #334155;
      padding: 3px 8px;
      border-radius: 4px;
      font-size: 12px;
      font-weight: 600;
      color: #38bdf8;
    }}
    .toolbar {{
      display: flex;
      align-items: center;
      gap: 10px;
    }}
    button {{
      background: #1e293b;
      border: 1px solid #334155;
      color: #e2e8f0;
      padding: 5px 12px;
      border-radius: 5px;
      font-size: 13px;
      cursor: pointer;
      transition: background 0.15s, border-color 0.15s;
    }}
    button:hover {{
      background: #334155;
      border-color: #64748b;
    }}
    button.active {{
      background: #0284c7;
      border-color: #38bdf8;
      color: #fff;
    }}
    .cursor-badge {{
      font-family: monospace;
      font-size: 13px;
      background: #172554;
      color: #60a5fa;
      border: 1px solid #1d4ed8;
      padding: 4px 10px;
      border-radius: 4px;
    }}
    #app-container {{
      flex: 1;
      display: flex;
      overflow: hidden;
      position: relative;
    }}
    #signals-panel {{
      width: 280px;
      min-width: 280px;
      background: #0f172a;
      border-right: 1px solid #1e293b;
      display: flex;
      flex-direction: column;
      z-index: 10;
    }}
    .panel-header {{
      height: 36px;
      border-bottom: 1px solid #1e293b;
      display: flex;
      align-items: center;
      padding: 0 12px;
      font-size: 12px;
      font-weight: 600;
      text-transform: uppercase;
      color: #94a3b8;
      background: #1e293b;
      justify-content: space-between;
    }}
    #signals-list {{
      flex: 1;
      overflow-y: hidden;
    }}
    .sig-row {{
      height: 40px;
      border-bottom: 1px solid #1e293b;
      display: flex;
      align-items: center;
      padding: 0 12px;
      justify-content: space-between;
      cursor: pointer;
    }}
    .sig-row:hover {{
      background: #1e293b;
    }}
    .sig-name {{
      font-family: monospace;
      font-size: 13px;
      font-weight: 500;
      color: #f1f5f9;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      max-width: 150px;
    }}
    .sig-val {{
      font-family: monospace;
      font-size: 13px;
      font-weight: 600;
      color: #38bdf8;
      text-align: right;
    }}
    #waveform-viewport {{
      flex: 1;
      position: relative;
      overflow: hidden;
      background: #090d16;
      cursor: crosshair;
    }}
    canvas {{
      display: block;
    }}
  </style>
</head>
<body>
  <header>
    <div class="title-area">
      <h2 style="font-size: 16px; font-weight: 700; color: #f8fafc;">VHDL Waveform Viewer</h2>
      <span class="badge">TOP: {top_name}</span>
      <span class="badge" id="duration-badge">STOP: 0 ns</span>
    </div>
    <div class="toolbar">
      <div class="cursor-badge" id="cursor-readout">Cursor: 0.000 ns</div>
      <button id="btn-zoom-in" title="Zoom In (+)">Zoom +</button>
      <button id="btn-zoom-out" title="Zoom Out (-)">Zoom -</button>
      <button id="btn-zoom-fit" title="Zoom to Fit">Fit</button>
      <span style="border-left: 1px solid #334155; height: 18px; margin: 0 4px;"></span>
      <button id="btn-radix-hex" class="active">HEX</button>
      <button id="btn-radix-bin">BIN</button>
      <button id="btn-radix-dec">DEC</button>
    </div>
  </header>

  <div id="app-container">
    <div id="signals-panel">
      <div class="panel-header">
        <span>Signal Name</span>
        <span>Value</span>
      </div>
      <div id="signals-list"></div>
    </div>
    <div id="waveform-viewport">
      <canvas id="wave-canvas"></canvas>
    </div>
  </div>

  <script>
    const SIGNALS = {signals_json};
    const STOP_TIME_FS = {stop_time_fs};
    const ROW_HEIGHT = 40;
    const HEADER_HEIGHT = 36;

    let radix = 'hex'; // 'hex', 'bin', 'dec'
    let cursorTimeFs = 0;
    let timeStartFs = 0;
    let timeEndFs = STOP_TIME_FS > 0 ? STOP_TIME_FS : 100_000_000; // default 100ns if 0
    let scrollY = 0;
    let isDragging = false;
    let dragStartX = 0;
    let dragStartTimeStartFs = 0;

    const viewport = document.getElementById('waveform-viewport');
    const canvas = document.getElementById('wave-canvas');
    const ctx = canvas.getContext('2d');
    const sigListEl = document.getElementById('signals-list');
    const cursorReadoutEl = document.getElementById('cursor-readout');
    const durationBadgeEl = document.getElementById('duration-badge');

    durationBadgeEl.textContent = 'STOP: ' + formatTime(STOP_TIME_FS);

    function formatTime(fs) {{
      if (fs >= 1_000_000_000_000_000) return (fs / 1_000_000_000_000_000).toFixed(2) + ' s';
      if (fs >= 1_000_000_000_000) return (fs / 1_000_000_000_000).toFixed(2) + ' ms';
      if (fs >= 1_000_000_000) return (fs / 1_000_000_000).toFixed(2) + ' us';
      if (fs >= 1_000_000) return (fs / 1_000_000).toFixed(3) + ' ns';
      if (fs >= 1_000) return (fs / 1000).toFixed(1) + ' ps';
      return fs + ' fs';
    }}

    function buildSignalList() {{
      sigListEl.innerHTML = '';
      SIGNALS.forEach((sig, idx) => {{
        const row = document.createElement('div');
        row.className = 'sig-row';
        row.innerHTML = `
          <span class="sig-name" title="${{sig.name}}">${{sig.name}}</span>
          <span class="sig-val" id="val-${{idx}}">-</span>
        `;
        sigListEl.appendChild(row);
      }});
    }}

    function getValAtTime(sig, timeFs) {{
      let curr = sig.transitions[0];
      for (let tr of sig.transitions) {{
        if (tr.t <= timeFs) {{
          curr = tr;
        }} else {{
          break;
        }}
      }}
      return curr;
    }}

    function updateSignalValues() {{
      SIGNALS.forEach((sig, idx) => {{
        const valObj = getValAtTime(sig, cursorTimeFs);
        const valEl = document.getElementById('val-' + idx);
        if (valEl && valObj) {{
          if (sig.width === 1) {{
            valEl.textContent = valObj.val;
            valEl.style.color = valObj.h ? '#4ade80' : valObj.l ? '#94a3b8' : '#facc15';
          }} else {{
            valEl.textContent = valObj[radix] || valObj.hex;
            valEl.style.color = '#38bdf8';
          }}
        }}
      }});
      cursorReadoutEl.textContent = 'Cursor: ' + formatTime(cursorTimeFs);
    }}

    function resize() {{
      const rect = viewport.getBoundingClientRect();
      const dpr = window.devicePixelRatio || 1;
      canvas.width = rect.width * dpr;
      canvas.height = rect.height * dpr;
      canvas.style.width = rect.width + 'px';
      canvas.style.height = rect.height + 'px';
      ctx.scale(dpr, dpr);
      render();
    }}

    function timeToX(tFs, width) {{
      return ((tFs - timeStartFs) / (timeEndFs - timeStartFs)) * width;
    }}

    function xToTime(x, width) {{
      return timeStartFs + (x / width) * (timeEndFs - timeStartFs);
    }}

    function render() {{
      const rect = viewport.getBoundingClientRect();
      const width = rect.width;
      const height = rect.height;

      ctx.clearRect(0, 0, width, height);

      // 1. Time Grid and Ruler
      ctx.fillStyle = '#111827';
      ctx.fillRect(0, 0, width, HEADER_HEIGHT);
      ctx.strokeStyle = '#1e293b';
      ctx.beginPath();
      ctx.moveTo(0, HEADER_HEIGHT);
      ctx.lineTo(width, HEADER_HEIGHT);
      ctx.stroke();

      const timeRange = timeEndFs - timeStartFs;
      const idealIntervals = 10;
      let rawStep = timeRange / idealIntervals;
      let power = Math.pow(10, Math.floor(Math.log10(rawStep)));
      let norm = rawStep / power;
      let stepMultiplier = norm < 2 ? 1 : norm < 5 ? 2 : 5;
      let gridStepFs = Math.max(1, stepMultiplier * power);

      let firstGridTime = Math.ceil(timeStartFs / gridStepFs) * gridStepFs;

      ctx.fillStyle = '#64748b';
      ctx.font = '11px monospace';
      ctx.textAlign = 'center';

      for (let t = firstGridTime; t <= timeEndFs; t += gridStepFs) {{
        let x = timeToX(t, width);
        // Vertical grid line
        ctx.strokeStyle = '#162032';
        ctx.beginPath();
        ctx.moveTo(x, HEADER_HEIGHT);
        ctx.lineTo(x, height);
        ctx.stroke();

        // Ruler tick & text
        ctx.strokeStyle = '#475569';
        ctx.beginPath();
        ctx.moveTo(x, HEADER_HEIGHT - 6);
        ctx.lineTo(x, HEADER_HEIGHT);
        ctx.stroke();

        ctx.fillText(formatTime(t), x, HEADER_HEIGHT - 10);
      }}

      // 2. Render Signal Tracks
      SIGNALS.forEach((sig, idx) => {{
        const yTop = HEADER_HEIGHT + idx * ROW_HEIGHT - scrollY;
        const yBottom = yTop + ROW_HEIGHT;

        if (yBottom < HEADER_HEIGHT || yTop > height) return;

        // Row background separator
        ctx.strokeStyle = '#1e293b';
        ctx.beginPath();
        ctx.moveTo(0, yBottom);
        ctx.lineTo(width, yBottom);
        ctx.stroke();

        const waveHighY = yTop + 8;
        const waveLowY = yTop + ROW_HEIGHT - 10;
        const waveMidY = (waveHighY + waveLowY) / 2;

        if (sig.width === 1) {{
          // Digital 1-bit waveform
          ctx.lineWidth = 1.8;

          for (let i = 0; i < sig.transitions.length; i++) {{
            const tr = sig.transitions[i];
            const nextT = (i + 1 < sig.transitions.length) ? sig.transitions[i+1].t : timeEndFs;

            if (nextT < timeStartFs) continue;
            if (tr.t > timeEndFs) break;

            const xStart = Math.max(0, timeToX(tr.t, width));
            const xEnd = Math.min(width, timeToX(nextT, width));

            let curY = tr.h ? waveHighY : tr.l ? waveLowY : waveMidY;
            ctx.strokeStyle = tr.h ? '#22c55e' : tr.l ? '#64748b' : '#f59e0b';

            ctx.beginPath();
            ctx.moveTo(xStart, curY);
            ctx.lineTo(xEnd, curY);
            ctx.stroke();

            // Vertical edge transition
            if (i > 0 && tr.t >= timeStartFs && tr.t <= timeEndFs) {{
              const prevTr = sig.transitions[i-1];
              const prevY = prevTr.h ? waveHighY : prevTr.l ? waveLowY : waveMidY;
              ctx.strokeStyle = '#94a3b8';
              ctx.beginPath();
              ctx.moveTo(xStart, prevY);
              ctx.lineTo(xStart, curY);
              ctx.stroke();
            }}
          }}
        }} else {{
          // Multi-bit Bus waveform
          ctx.lineWidth = 1.5;

          for (let i = 0; i < sig.transitions.length; i++) {{
            const tr = sig.transitions[i];
            const nextT = (i + 1 < sig.transitions.length) ? sig.transitions[i+1].t : timeEndFs;

            if (nextT < timeStartFs) continue;
            if (tr.t > timeEndFs) break;

            const xStart = Math.max(0, timeToX(tr.t, width));
            const xEnd = Math.min(width, timeToX(nextT, width));
            const segW = xEnd - xStart;

            ctx.strokeStyle = '#38bdf8';
            ctx.fillStyle = '#0c4a6e33';

            // Draw bus polygon (diamond transition edges)
            const slant = Math.min(4, segW / 4);

            ctx.beginPath();
            ctx.moveTo(xStart + slant, waveHighY);
            ctx.lineTo(xEnd - slant, waveHighY);
            ctx.lineTo(xEnd, waveMidY);
            ctx.lineTo(xEnd - slant, waveLowY);
            ctx.lineTo(xStart + slant, waveLowY);
            ctx.lineTo(xStart, waveMidY);
            ctx.closePath();
            ctx.fill();
            ctx.stroke();

            // Label inside bus cell
            if (segW > 24) {{
              const label = tr[radix] || tr.hex;
              ctx.fillStyle = '#bae6fd';
              ctx.font = '12px monospace';
              ctx.textAlign = 'center';
              ctx.textBaseline = 'middle';
              ctx.fillText(label, xStart + segW / 2, waveMidY);
            }}
          }}
        }}
      }});

      // 3. Render Cursor Line
      if (cursorTimeFs >= timeStartFs && cursorTimeFs <= timeEndFs) {{
        const curX = timeToX(cursorTimeFs, width);
        ctx.strokeStyle = '#f59e0b';
        ctx.lineWidth = 1.5;
        ctx.beginPath();
        ctx.moveTo(curX, 0);
        ctx.lineTo(curX, height);
        ctx.stroke();

        // Cursor header tag
        ctx.fillStyle = '#f59e0b';
        ctx.fillRect(curX - 35, 2, 70, 18);
        ctx.fillStyle = '#0f172a';
        ctx.font = 'bold 10px monospace';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText(formatTime(cursorTimeFs), curX, 11);
      }}

      updateSignalValues();
    }}

    // Mouse and Keyboard Navigation
    viewport.addEventListener('mousedown', (e) => {{
      const rect = viewport.getBoundingClientRect();
      const x = e.clientX - rect.left;
      if (e.button === 0) {{ // Left click: place cursor or drag pan
        cursorTimeFs = Math.max(0, Math.round(xToTime(x, rect.width)));
        isDragging = true;
        dragStartX = e.clientX;
        dragStartTimeStartFs = timeStartFs;
        render();
      }}
    }});

    window.addEventListener('mousemove', (e) => {{
      if (isDragging) {{
        const rect = viewport.getBoundingClientRect();
        const dx = e.clientX - dragStartX;
        const dt = (dx / rect.width) * (timeEndFs - timeStartFs);
        if (Math.abs(dx) > 3) {{
          let newStart = Math.max(0, dragStartTimeStartFs - dt);
          let span = timeEndFs - timeStartFs;
          timeStartFs = newStart;
          timeEndFs = newStart + span;
          render();
        }}
      }}
    }});

    window.addEventListener('mouseup', () => {{
      isDragging = false;
    }});

    // Wheel: zoom in/out centered on mouse cursor
    viewport.addEventListener('wheel', (e) => {{
      e.preventDefault();
      const rect = viewport.getBoundingClientRect();
      const mouseX = e.clientX - rect.left;
      const mouseTime = xToTime(mouseX, rect.width);
      const zoomFactor = e.deltaY < 0 ? 0.75 : 1.35;

      const newSpan = Math.max(1000, (timeEndFs - timeStartFs) * zoomFactor);
      const ratio = mouseX / rect.width;

      timeStartFs = Math.max(0, mouseTime - newSpan * ratio);
      timeEndFs = timeStartFs + newSpan;
      render();
    }}, {{ passive: false }});

    // Toolbar controls
    document.getElementById('btn-zoom-in').addEventListener('click', () => {{
      const span = (timeEndFs - timeStartFs) * 0.7;
      const mid = cursorTimeFs >= timeStartFs && cursorTimeFs <= timeEndFs ? cursorTimeFs : (timeStartFs + timeEndFs) / 2;
      timeStartFs = Math.max(0, mid - span / 2);
      timeEndFs = timeStartFs + span;
      render();
    }});

    document.getElementById('btn-zoom-out').addEventListener('click', () => {{
      const span = (timeEndFs - timeStartFs) * 1.4;
      const mid = (timeStartFs + timeEndFs) / 2;
      timeStartFs = Math.max(0, mid - span / 2);
      timeEndFs = timeStartFs + span;
      render();
    }});

    document.getElementById('btn-zoom-fit').addEventListener('click', () => {{
      timeStartFs = 0;
      timeEndFs = STOP_TIME_FS > 0 ? STOP_TIME_FS : 100_000_000;
      render();
    }});

    ['hex', 'bin', 'dec'].forEach(r => {{
      document.getElementById('btn-radix-' + r).addEventListener('click', (e) => {{
        radix = r;
        document.querySelectorAll('.toolbar button').forEach(b => {{
          if (b.id.startsWith('btn-radix-')) b.classList.remove('active');
        }});
        e.target.classList.add('active');
        render();
      }});
    }});

    window.addEventListener('resize', resize);

    // Initial setup
    buildSignalList();
    resize();
  </script>
</body>
</html>
"#
        )
    }
}
