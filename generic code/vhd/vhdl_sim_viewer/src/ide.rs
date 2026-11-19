//! Vivado-style Web IDE & Integrated Waveform Viewer served by the Safe Rust backend.

pub struct IdeViewer;

impl IdeViewer {
    pub fn generate_html() -> &'static str {
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>VHDL Studio — Safe Rust Compiler & Vivado-Style Waveform IDE</title>
  <style>
    :root {
      --bg-app: #0b0f17;
      --bg-panel: #111827;
      --bg-header: #161f30;
      --bg-hover: #1e293b;
      --bg-active: #0f2942;
      --border: #243044;
      --text-main: #e2e8f0;
      --text-dim: #94a3b8;
      --accent: #0ea5e9;
      --accent-bright: #38bdf8;
      --green: #22c55e;
      --green-dark: #15803d;
      --amber: #f59e0b;
      --red: #ef4444;
      --font-mono: 'Consolas', 'Cascadia Code', 'Fira Code', 'JetBrains Mono', monospace;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      background: var(--bg-app);
      color: var(--text-main);
      height: 100vh;
      display: flex;
      flex-direction: column;
      overflow: hidden;
      font-size: 13px;
    }

    /* Top Vivado-style Toolbar */
    .topbar {
      height: 44px;
      background: var(--bg-header);
      border-bottom: 1px solid var(--border);
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 0 12px;
      gap: 12px;
      user-select: none;
    }
    .brand {
      display: flex;
      align-items: center;
      gap: 10px;
      font-weight: 700;
      font-size: 14px;
      color: #f8fafc;
    }
    .brand-tag {
      font-size: 10px;
      font-weight: 600;
      text-transform: uppercase;
      padding: 2px 6px;
      border-radius: 3px;
      background: #0c4a6e;
      color: #38bdf8;
      border: 1px solid #0284c7;
    }
    .toolbar-group {
      display: flex;
      align-items: center;
      gap: 8px;
    }
    label.inline-lbl {
      color: var(--text-dim);
      font-size: 11px;
      font-weight: 600;
    }
    select, input[type="text"] {
      background: #0b0f17;
      border: 1px solid var(--border);
      color: var(--text-main);
      padding: 4px 8px;
      border-radius: 4px;
      font-size: 12px;
      outline: none;
    }
    select:focus, input[type="text"]:focus {
      border-color: var(--accent);
    }
    button {
      background: #1e293b;
      border: 1px solid #334155;
      color: var(--text-main);
      padding: 5px 10px;
      border-radius: 4px;
      font-size: 12px;
      cursor: pointer;
      display: inline-flex;
      align-items: center;
      gap: 5px;
      transition: background 0.12s, border-color 0.12s;
    }
    button:hover { background: #2d3b52; border-color: #475569; }
    button.active { background: #0284c7; border-color: #38bdf8; color: #fff; }
    button.btn-run {
      background: #16a34a;
      border-color: #22c55e;
      color: #fff;
      font-weight: 600;
      padding: 5px 14px;
    }
    button.btn-run:hover { background: #15803d; }
    button.btn-run:disabled { opacity: 0.6; cursor: wait; }

    /* Main Workspace Layout */
    .workspace {
      flex: 1;
      display: flex;
      overflow: hidden;
      min-height: 0;
    }

    /* Left Sidebar: Flow Navigator & Sources */
    .sidebar {
      width: 240px;
      min-width: 200px;
      background: var(--bg-panel);
      border-right: 1px solid var(--border);
      display: flex;
      flex-direction: column;
      overflow-y: auto;
      user-select: none;
    }
    .side-section {
      border-bottom: 1px solid var(--border);
    }
    .side-header {
      background: var(--bg-header);
      padding: 6px 10px;
      font-size: 11px;
      font-weight: 700;
      text-transform: uppercase;
      letter-spacing: 0.4px;
      color: var(--text-dim);
      display: flex;
      justify-content: space-between;
      align-items: center;
    }
    .flow-item {
      padding: 7px 12px;
      cursor: pointer;
      display: flex;
      align-items: center;
      gap: 8px;
      font-size: 12px;
      color: var(--text-main);
      border-left: 2px solid transparent;
    }
    .flow-item:hover {
      background: var(--bg-hover);
      border-left-color: var(--accent);
    }
    .source-item {
      padding: 6px 10px;
      cursor: pointer;
      display: flex;
      align-items: center;
      justify-content: space-between;
      font-family: var(--font-mono);
      font-size: 12px;
      border-left: 2px solid transparent;
    }
    .source-item:hover { background: var(--bg-hover); }
    .source-item.active {
      background: var(--bg-active);
      border-left-color: var(--accent-bright);
      color: #fff;
    }
    .source-del {
      color: var(--text-dim);
      font-size: 14px;
      padding: 0 4px;
      border-radius: 3px;
    }
    .source-del:hover { color: var(--red); background: rgba(239,68,68,0.15); }

    .elab-stat {
      padding: 5px 12px;
      font-size: 12px;
      color: var(--text-dim);
      display: flex;
      justify-content: space-between;
    }
    .elab-stat span.val { color: var(--accent-bright); font-family: var(--font-mono); font-weight: 600; }

    /* Center Column: Main Area + Bottom Console */
    .center-col {
      flex: 1;
      display: flex;
      flex-direction: column;
      min-width: 0;
      overflow: hidden;
    }

    .main-split {
      flex: 1;
      display: flex;
      min-height: 0;
      overflow: hidden;
    }

    /* Code Editor Pane */
    .editor-pane {
      flex: 1;
      display: flex;
      flex-direction: column;
      border-right: 1px solid var(--border);
      min-width: 260px;
      background: #0d1117;
    }
    .pane-tabs {
      height: 32px;
      background: var(--bg-header);
      border-bottom: 1px solid var(--border);
      display: flex;
      align-items: center;
      overflow-x: auto;
      user-select: none;
    }
    .tab-item {
      height: 100%;
      padding: 0 12px;
      display: flex;
      align-items: center;
      gap: 6px;
      font-size: 12px;
      font-family: var(--font-mono);
      color: var(--text-dim);
      border-right: 1px solid var(--border);
      cursor: pointer;
      white-space: nowrap;
    }
    .tab-item:hover { background: var(--bg-hover); color: var(--text-main); }
    .tab-item.active {
      background: #0d1117;
      color: var(--accent-bright);
      border-top: 2px solid var(--accent-bright);
    }

    .editor-error-bar {
      background: rgba(239, 68, 68, 0.15);
      border-bottom: 1px solid var(--red);
      color: #fca5a5;
      padding: 5px 10px;
      font-family: var(--font-mono);
      font-size: 12px;
      display: none;
    }

    .editor-body {
      flex: 1;
      display: flex;
      position: relative;
      overflow: hidden;
    }
    .gutter {
      width: 44px;
      background: #0b0f17;
      border-right: 1px solid var(--border);
      color: #475569;
      font-family: var(--font-mono);
      font-size: 13px;
      line-height: 20px;
      padding: 10px 6px 10px 0;
      text-align: right;
      overflow: hidden;
      user-select: none;
    }
    .code-container {
      flex: 1;
      position: relative;
      overflow: hidden;
    }
    #code-highlight, #code-input {
      position: absolute;
      top: 0; left: 0;
      width: 100%; height: 100%;
      margin: 0;
      padding: 10px 12px;
      font-family: var(--font-mono);
      font-size: 13px;
      line-height: 20px;
      tab-size: 4;
      white-space: pre;
      overflow: auto;
      border: none;
      outline: none;
    }
    #code-highlight {
      pointer-events: none;
      color: #e2e8f0;
      z-index: 1;
    }
    #code-input {
      background: transparent;
      color: transparent;
      caret-color: #38bdf8;
      resize: none;
      z-index: 2;
    }
    /* VHDL Syntax Highlighting classes */
    .vhdl-kw { color: #c084fc; font-weight: 600; }
    .vhdl-type { color: #38bdf8; }
    .vhdl-comment { color: #64748b; font-style: italic; }
    .vhdl-str { color: #4ade80; }
    .vhdl-num { color: #fbbf24; }

    /* Vivado Waveform Pane */
    .wave-pane {
      flex: 1.25;
      display: flex;
      flex-direction: column;
      min-width: 300px;
      background: #090d16;
    }
    .wave-toolbar {
      height: 32px;
      background: var(--bg-header);
      border-bottom: 1px solid var(--border);
      display: flex;
      align-items: center;
      padding: 0 8px;
      gap: 6px;
      user-select: none;
      overflow-x: auto;
    }
    .wave-toolbar button {
      padding: 2px 7px;
      font-size: 11px;
    }
    .cursor-pill {
      font-family: var(--font-mono);
      font-size: 11px;
      padding: 2px 8px;
      border-radius: 3px;
      background: #451a03;
      color: #fbbf24;
      border: 1px solid #b45309;
      white-space: nowrap;
    }
    .delta-pill {
      font-family: var(--font-mono);
      font-size: 11px;
      padding: 2px 8px;
      border-radius: 3px;
      background: #083344;
      color: #22d3ee;
      border: 1px solid #0e7490;
      white-space: nowrap;
      display: none;
    }

    .wave-main {
      flex: 1;
      display: flex;
      overflow: hidden;
      position: relative;
    }
    .wave-signals {
      width: 250px;
      min-width: 210px;
      background: var(--bg-panel);
      border-right: 1px solid var(--border);
      display: flex;
      flex-direction: column;
      user-select: none;
    }
    .wave-sig-hdr {
      height: 32px;
      background: #161f30;
      border-bottom: 1px solid var(--border);
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 0 10px;
      font-size: 11px;
      font-weight: 700;
      color: var(--text-dim);
      text-transform: uppercase;
    }
    .wave-sig-list {
      flex: 1;
      overflow-y: hidden;
    }
    .ws-row {
      height: 32px;
      border-bottom: 1px solid #1a2333;
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 0 8px;
      cursor: pointer;
      font-family: var(--font-mono);
      font-size: 12px;
    }
    .ws-row:hover { background: var(--bg-hover); }
    .ws-row.selected { background: var(--bg-active); }
    .ws-row.bit-child { padding-left: 24px; background: #0d131f; }
    .ws-row.bit-child:hover { background: #172133; }
    .ws-left {
      display: flex;
      align-items: center;
      gap: 5px;
      overflow: hidden;
    }
    .ws-expand {
      width: 14px;
      color: var(--text-dim);
      font-size: 10px;
      text-align: center;
      flex-shrink: 0;
    }
    .ws-name {
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      color: #f1f5f9;
    }
    .ws-val {
      font-weight: 600;
      font-size: 11px;
      margin-left: 6px;
      flex-shrink: 0;
    }

    .wave-canvas-wrap {
      flex: 1;
      position: relative;
      overflow: hidden;
      cursor: crosshair;
      background: #070a11;
    }
    #wave-canvas { display: block; }

    /* Bottom Console / Messages Panel */
    .console-pane {
      height: 175px;
      min-height: 110px;
      background: var(--bg-panel);
      border-top: 1px solid var(--border);
      display: flex;
      flex-direction: column;
    }
    .console-tabs {
      height: 28px;
      background: var(--bg-header);
      border-bottom: 1px solid var(--border);
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding-right: 10px;
      user-select: none;
    }
    .console-tab-list { display: flex; height: 100%; }
    .ctab {
      padding: 0 12px;
      display: flex;
      align-items: center;
      font-size: 11px;
      font-weight: 600;
      color: var(--text-dim);
      border-right: 1px solid var(--border);
      cursor: pointer;
    }
    .ctab.active { background: var(--bg-panel); color: var(--accent-bright); }
    .console-body {
      flex: 1;
      overflow: auto;
      padding: 8px 12px;
      font-family: var(--font-mono);
      font-size: 12px;
      line-height: 1.5;
    }
    .msg-ok { color: #4ade80; }
    .msg-err { color: #f87171; }
    .msg-info { color: #38bdf8; }
    .msg-warn { color: #fbbf24; }
    .hidden { display: none !important; }
  </style>
</head>
<body>
  <!-- Top Toolbar -->
  <header class="topbar">
    <div class="brand">
      <span>⚡ VHDL Studio</span>
      <span class="brand-tag">Safe Rust · Vivado-Style IDE</span>
    </div>

    <div class="toolbar-group">
      <label class="inline-lbl">Example:</label>
      <select id="example-select"></select>
      <button id="btn-load-example" title="Load selected example project">Load</button>
    </div>

    <div class="toolbar-group">
      <label class="inline-lbl">Top:</label>
      <input type="text" id="input-top" placeholder="auto-detect" style="width: 120px;" />
      <label class="inline-lbl">Stop:</label>
      <input type="text" id="input-stop" value="180ns" style="width: 75px;" />
      <button id="btn-run" class="btn-run" title="Compile & Run Simulation (Ctrl+Enter)">▶ Run Simulation</button>
      <button id="btn-vcd" title="Download VCD waveform file">⬇ Export VCD</button>
    </div>

    <div class="toolbar-group">
      <button id="btn-layout-split" class="active">Split</button>
      <button id="btn-layout-editor">Editor</button>
      <button id="btn-layout-wave">Waveform</button>
    </div>
  </header>

  <!-- Main Workspace -->
  <div class="workspace">
    <!-- Left Sidebar -->
    <aside class="sidebar">
      <div class="side-section">
        <div class="side-header">Flow Navigator</div>
        <div class="flow-item" onclick="runSimulation()">▶ Run Behavioral Simulation</div>
        <div class="flow-item" onclick="document.getElementById('file-upload-input').click()">⬆ Upload VHDL Files (.vhd)</div>
        <div class="flow-item" onclick="zoomFit()">🔍 Zoom Waveform to Fit</div>
        <div class="flow-item" onclick="downloadVcd()">💾 Export .VCD Trace</div>
      </div>

      <div class="side-section" id="sources-section">
        <div class="side-header">
          <span>Sources (.vhd)</span>
          <div style="display: flex; gap: 4px;">
            <button id="btn-upload-files" style="padding: 1px 6px; font-size: 11px; background: #0369a1; border-color: #0ea5e9;" title="Upload one or more .vhd files">⬆ Upload</button>
            <button id="btn-add-file" style="padding: 1px 6px; font-size: 11px;" title="Create new empty .vhd file">+ New</button>
            <button id="btn-clear-files" style="padding: 1px 6px; font-size: 11px;" title="Clear all files to start a new project">Clear</button>
          </div>
        </div>
        <input type="file" id="file-upload-input" multiple accept=".vhd,.vhdl" style="display: none;" />
        <div id="sources-list"></div>
      </div>

      <div class="side-section">
        <div class="side-header">Elaborated Design</div>
        <div class="elab-stat"><span>Top Entity:</span><span class="val" id="elab-top">-</span></div>
        <div class="elab-stat"><span>Signals:</span><span class="val" id="elab-sigs">0</span></div>
        <div class="elab-stat"><span>Processes:</span><span class="val" id="elab-procs">0</span></div>
        <div class="elab-stat"><span>Stop Time:</span><span class="val" id="elab-stop">0 ns</span></div>
      </div>
    </aside>

    <!-- Center Column -->
    <div class="center-col">
      <div class="main-split">
        <!-- VHDL Editor Pane -->
        <section class="editor-pane" id="editor-pane">
          <div class="pane-tabs" id="editor-tabs"></div>
          <div class="editor-error-bar" id="editor-error-bar"></div>
          <div class="editor-body">
            <div class="gutter" id="editor-gutter">1</div>
            <div class="code-container">
              <pre id="code-highlight" aria-hidden="true"></pre>
              <textarea id="code-input" spellcheck="false" autocomplete="off"></textarea>
            </div>
          </div>
        </section>

        <!-- Vivado Waveform Pane -->
        <section class="wave-pane" id="wave-pane">
          <div class="wave-toolbar">
            <span class="cursor-pill" id="cursor-readout">Cursor: 0.000 ns</span>
            <span class="delta-pill" id="delta-readout">Δt: 0.000 ns</span>
            <button id="w-zoom-in" title="Zoom In">+</button>
            <button id="w-zoom-out" title="Zoom Out">-</button>
            <button id="w-zoom-fit" title="Zoom Fit">Fit</button>
            <span style="border-left: 1px solid #334155; height: 14px;"></span>
            <button id="w-edge-start" title="Jump to 0 ns">|&lt;</button>
            <button id="w-edge-prev" title="Previous Transition of Selected Signal">&lt;|</button>
            <button id="w-edge-next" title="Next Transition of Selected Signal">|&gt;</button>
            <button id="w-edge-end" title="Jump to End">&gt;|</button>
            <span style="border-left: 1px solid #334155; height: 14px;"></span>
            <button class="radix-btn active" data-radix="hex">HEX</button>
            <button class="radix-btn" data-radix="bin">BIN</button>
            <button class="radix-btn" data-radix="dec">DEC</button>
          </div>

          <div class="wave-main">
            <div class="wave-signals">
              <div class="wave-sig-hdr">
                <span>Name</span>
                <span>Value</span>
              </div>
              <div class="wave-sig-list" id="wave-sig-list"></div>
            </div>
            <div class="wave-canvas-wrap" id="wave-canvas-wrap">
              <canvas id="wave-canvas"></canvas>
            </div>
          </div>
        </section>
      </div>

      <!-- Bottom Console -->
      <section class="console-pane">
        <div class="console-tabs">
          <div class="console-tab-list">
            <div class="ctab active" data-ctab="messages">Tcl / Messages</div>
            <div class="ctab" data-ctab="netlist">Elaborated Signals</div>
            <div class="ctab" data-ctab="ascii">ASCII Wave</div>
          </div>
          <span style="font-size: 11px; color: var(--text-dim);">Tip: Shift+Click on waveform to measure Δt · Scroll wheel to Zoom · Drag to Pan</span>
        </div>
        <div class="console-body" id="console-messages">
          <div class="msg-info">VHDL Studio initialized. Ready to compile and simulate.</div>
        </div>
        <div class="console-body hidden" id="console-netlist"></div>
        <pre class="console-body hidden" id="console-ascii" style="margin: 0;"></pre>
      </section>
    </div>
  </div>

  <script>
    // --- Project & Editor State ---
    let examples = [];
    let files = [];
    let activeFileIdx = 0;

    // --- Simulation & Waveform State ---
    let simResult = null;
    let displayRows = []; // flattened list including expanded bus bits
    let expandedBuses = new Set();
    let selectedSignalName = null;
    let radix = 'hex';
    let cursorTimeFs = 0;
    let markerTimeFs = null; // Shift+Click secondary marker for delta measurement
    let timeStartFs = 0;
    let timeEndFs = 180_000_000;
    let scrollY = 0;
    let isDragging = false;
    let dragStartX = 0;
    let dragStartTimeFs = 0;

    const ROW_H = 32;
    const HDR_H = 32;

    const codeInput = document.getElementById('code-input');
    const codeHighlight = document.getElementById('code-highlight');
    const gutter = document.getElementById('editor-gutter');
    const canvasWrap = document.getElementById('wave-canvas-wrap');
    const canvas = document.getElementById('wave-canvas');
    const ctx = canvas.getContext('2d');
    const sigListEl = document.getElementById('wave-sig-list');

    // --- VHDL Syntax Highlighter ---
    function highlightVhdl(src) {
      const esc = src
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;');

      const tokens = [];
      // Tokenize comments, strings, chars, words, numbers
      const regex = /(--[^\n]*)|("(?:[^"\\]|\\.)*")|('.')|(\b(?:entity|architecture|is|port|generic|end|of|begin|signal|variable|constant|component|map|process|if|then|elsif|else|case|when|for|while|loop|wait|until|on|after|assert|report|severity|in|out|inout|buffer|range|downto|to|type|subtype|package|library|use|all|others|null|and|or|nand|nor|xor|xnor|not|rising_edge|falling_edge)\b)|(\b(?:std_logic_vector|std_logic|bit_vector|bit|integer|natural|positive|boolean|time)\b)|(\b\d+(?:\.\d+)?\b)/gi;

      let last = 0;
      let out = '';
      let m;
      while ((m = regex.exec(esc)) !== null) {
        out += esc.slice(last, m.index);
        if (m[1]) out += `<span class="vhdl-comment">${m[1]}</span>`;
        else if (m[2] || m[3]) out += `<span class="vhdl-str">${m[2] || m[3]}</span>`;
        else if (m[4]) out += `<span class="vhdl-kw">${m[4]}</span>`;
        else if (m[5]) out += `<span class="vhdl-type">${m[5]}</span>`;
        else if (m[6]) out += `<span class="vhdl-num">${m[6]}</span>`;
        last = regex.lastIndex;
      }
      out += esc.slice(last);
      if (src.endsWith('\n')) out += ' ';
      return out;
    }

    function updateEditorHighlight() {
      const val = codeInput.value;
      codeHighlight.innerHTML = highlightVhdl(val);
      const lines = val.split('\n').length;
      let g = '';
      for (let i = 1; i <= lines; i++) g += i + '\n';
      gutter.textContent = g;
    }

    codeInput.addEventListener('input', () => {
      if (files[activeFileIdx]) {
        files[activeFileIdx].content = codeInput.value;
      }
      updateEditorHighlight();
    });

    codeInput.addEventListener('scroll', () => {
      codeHighlight.scrollTop = codeInput.scrollTop;
      codeHighlight.scrollLeft = codeInput.scrollLeft;
      gutter.scrollTop = codeInput.scrollTop;
    });

    codeInput.addEventListener('keydown', (e) => {
      if (e.key === 'Tab') {
        e.preventDefault();
        const s = codeInput.selectionStart;
        const end = codeInput.selectionEnd;
        codeInput.value = codeInput.value.substring(0, s) + '    ' + codeInput.value.substring(end);
        codeInput.selectionStart = codeInput.selectionEnd = s + 4;
        if (files[activeFileIdx]) files[activeFileIdx].content = codeInput.value;
        updateEditorHighlight();
      } else if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
        e.preventDefault();
        runSimulation();
      }
    });

    function renderFileLists() {
      const srcEl = document.getElementById('sources-list');
      const tabsEl = document.getElementById('editor-tabs');
      srcEl.innerHTML = '';
      tabsEl.innerHTML = '';

      files.forEach((f, idx) => {
        const sItem = document.createElement('div');
        sItem.className = 'source-item' + (idx === activeFileIdx ? ' active' : '');
        sItem.innerHTML = `<span>📄 ${f.name}</span>` +
          (files.length > 1 ? `<span class="source-del" title="Remove file">×</span>` : '');
        sItem.onclick = (e) => {
          if (e.target.classList.contains('source-del')) {
            files.splice(idx, 1);
            if (activeFileIdx >= files.length) activeFileIdx = files.length - 1;
            selectFile(activeFileIdx);
          } else {
            selectFile(idx);
          }
        };
        srcEl.appendChild(sItem);

        const tab = document.createElement('div');
        tab.className = 'tab-item' + (idx === activeFileIdx ? ' active' : '');
        tab.textContent = f.name;
        tab.onclick = () => selectFile(idx);
        tabsEl.appendChild(tab);
      });
    }

    function selectFile(idx) {
      activeFileIdx = idx;
      if (files[idx]) {
        codeInput.value = files[idx].content;
      } else {
        codeInput.value = '';
      }
      updateEditorHighlight();
      renderFileLists();
    }

    document.getElementById('btn-add-file').onclick = () => {
      const name = prompt('New VHDL file name (e.g. design.vhd):', `module_${files.length + 1}.vhd`);
      if (!name) return;
      files.push({
        name: name.trim(),
        content: `-- ${name.trim()}\nlibrary ieee;\nuse ieee.std_logic_1164.all;\n\nentity MyModule is\nend entity;\n\narchitecture rtl of MyModule is\nbegin\nend architecture;\n`
      });
      selectFile(files.length - 1);
    };

    document.getElementById('btn-clear-files').onclick = () => {
      files = [];
      activeFileIdx = 0;
      document.getElementById('input-top').value = '';
      selectFile(0);
      logMessage('Cleared project files. Upload or create new .vhd files.', 'info');
    };

    const uploadInput = document.getElementById('file-upload-input');
    document.getElementById('btn-upload-files').onclick = () => {
      uploadInput.value = '';
      uploadInput.click();
    };

    async function handleUploadedFiles(fileList) {
      if (!fileList || fileList.length === 0) return;
      const loaded = await Promise.all(
        Array.from(fileList).map(async (f) => ({
          name: f.name,
          content: await f.text()
        }))
      );

      if (loaded.length >= 2) {
        // Multi-file project upload: reset Top to auto-detect so old example top doesn't interfere
        document.getElementById('input-top').value = '';
      }

      let lastIdx = activeFileIdx;
      loaded.forEach((item) => {
        const existingIdx = files.findIndex(
          (ex) => ex.name.toLowerCase() === item.name.toLowerCase()
        );
        if (existingIdx >= 0) {
          files[existingIdx].content = item.content;
          lastIdx = existingIdx;
        } else {
          files.push(item);
          lastIdx = files.length - 1;
        }
      });

      selectFile(lastIdx);
      const names = loaded.map((f) => f.name).join(', ');
      logMessage(`Uploaded ${loaded.length} file(s): ${names}`, 'ok');
    }

    uploadInput.addEventListener('change', (e) => {
      handleUploadedFiles(e.target.files);
    });

    // Drag & Drop support on Sources sidebar
    const sourcesSection = document.getElementById('sources-section');
    sourcesSection.addEventListener('dragover', (e) => {
      e.preventDefault();
      sourcesSection.style.outline = '1px dashed var(--accent-bright)';
    });
    sourcesSection.addEventListener('dragleave', () => {
      sourcesSection.style.outline = 'none';
    });
    sourcesSection.addEventListener('drop', (e) => {
      e.preventDefault();
      sourcesSection.style.outline = 'none';
      handleUploadedFiles(e.dataTransfer.files);
    });

    // --- Load Examples ---
    async function fetchExamples() {
      try {
        const res = await fetch('/api/examples');
        examples = await res.json();
        const sel = document.getElementById('example-select');
        sel.innerHTML = '';
        examples.forEach((ex, i) => {
          const opt = document.createElement('option');
          opt.value = i;
          opt.textContent = ex.title;
          sel.appendChild(opt);
        });
        if (examples.length > 0) {
          loadExample(0);
        }
      } catch (err) {
        logMessage('Failed to load built-in examples: ' + err, 'err');
      }
    }

    function loadExample(index) {
      const ex = examples[index];
      if (!ex) return;
      files = ex.files.map(f => ({ name: f.name, content: f.content }));
      document.getElementById('input-top').value = ex.top || '';
      document.getElementById('input-stop').value = ex.stop || '180ns';
      selectFile(0);
      logMessage(`Loaded example: ${ex.title}`, 'info');
      runSimulation();
    }

    document.getElementById('btn-load-example').onclick = () => {
      const idx = parseInt(document.getElementById('example-select').value, 10) || 0;
      loadExample(idx);
    };

    // --- Time Parsing & Formatting ---
    function parseTimeFs(str) {
      const s = (str || '100ns').trim().toLowerCase();
      const m = s.match(/^([\d.]+)\s*(fs|ps|ns|us|ms|sec|s)?$/);
      if (!m) return 100_000_000;
      const val = parseFloat(m[1]);
      const unit = m[2] || 'ns';
      const mult = {
        fs: 1, ps: 1e3, ns: 1e6, us: 1e9, ms: 1e12, sec: 1e15, s: 1e15
      }[unit] || 1e6;
      return Math.round(val * mult);
    }

    function formatTime(fs) {
      if (fs >= 1e15) return (fs / 1e15).toFixed(2) + ' s';
      if (fs >= 1e12) return (fs / 1e12).toFixed(2) + ' ms';
      if (fs >= 1e9) return (fs / 1e9).toFixed(2) + ' us';
      if (fs >= 1e6) return (fs / 1e6).toFixed(3) + ' ns';
      if (fs >= 1e3) return (fs / 1e3).toFixed(1) + ' ps';
      return Math.round(fs) + ' fs';
    }

    function logMessage(text, kind = 'info') {
      const box = document.getElementById('console-messages');
      const div = document.createElement('div');
      div.className = 'msg-' + kind;
      div.textContent = `[${new Date().toLocaleTimeString()}] ${text}`;
      box.appendChild(div);
      box.scrollTop = box.scrollHeight;
    }

    // --- Simulation API Call ---
    async function runSimulation() {
      const btn = document.getElementById('btn-run');
      const errBar = document.getElementById('editor-error-bar');
      errBar.style.display = 'none';
      btn.disabled = true;
      btn.textContent = '⏳ Simulating...';

      const top = document.getElementById('input-top').value.trim();
      const stopTimeFs = parseTimeFs(document.getElementById('input-stop').value);

      try {
        const res = await fetch('/api/simulate', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            top,
            stop_time_fs: stopTimeFs,
            files
          })
        });
        const data = await res.json();

        if (!data.ok) {
          errBar.textContent = '❌ ' + data.error;
          errBar.style.display = 'block';
          logMessage('ERROR: ' + data.error, 'err');
          return;
        }

        simResult = data;
        document.getElementById('elab-top').textContent = data.top_entity;
        document.getElementById('elab-sigs').textContent = data.signals.length;
        document.getElementById('elab-procs').textContent = data.process_count;
        document.getElementById('elab-stop').textContent = formatTime(data.stop_time_fs);

        logMessage(`Compiled & simulated '${data.top_entity}' (${data.signals.length} signals, ${data.process_count} processes) up to ${formatTime(data.stop_time_fs)}.`, 'ok');
        if (data.logs && data.logs.length > 0) {
          data.logs.forEach(l => logMessage(l, 'warn'));
        }

        // Update Netlist & ASCII console tabs
        document.getElementById('console-ascii').textContent = data.ascii_wave || '';
        const netEl = document.getElementById('console-netlist');
        netEl.innerHTML = data.signals.map(s =>
          `<div>• <b>${s.name}</b> — width: ${s.width} bit(s), transitions: ${s.transitions.length}</div>`
        ).join('');

        timeStartFs = 0;
        timeEndFs = data.stop_time_fs > 0 ? data.stop_time_fs : 100_000_000;
        cursorTimeFs = 0;
        markerTimeFs = null;

        rebuildDisplayRows();
        resizeCanvas();
      } catch (err) {
        logMessage('Network / Server error: ' + err, 'err');
      } finally {
        btn.disabled = false;
        btn.textContent = '▶ Run Simulation';
      }
    }

    document.getElementById('btn-run').onclick = runSimulation;

    function downloadVcd() {
      if (!simResult || !simResult.vcd) {
        logMessage('Run simulation first before exporting VCD.', 'warn');
        return;
      }
      const blob = new Blob([simResult.vcd], { type: 'text/plain' });
      const a = document.createElement('a');
      a.href = URL.createObjectURL(blob);
      a.download = `${simResult.top_entity || 'wave'}.vcd`;
      a.click();
    }
    document.getElementById('btn-vcd').onclick = downloadVcd;

    // --- Expandable Bus & Signal Display Rows ---
    function rebuildDisplayRows() {
      displayRows = [];
      if (!simResult) return;

      simResult.signals.forEach(sig => {
        const isLogicBus = sig.width > 1 && sig.transitions.some(t => t.bin && t.bin.length === sig.width);
        displayRows.push({
          key: sig.name,
          label: sig.width > 1 ? `${sig.name} [${sig.width - 1}:0]` : sig.name,
          width: sig.width,
          isBus: sig.width > 1,
          expandable: isLogicBus,
          isChild: false,
          transitions: sig.transitions
        });

        if (isLogicBus && expandedBuses.has(sig.name)) {
          for (let b = 0; b < sig.width; b++) {
            const bitIdx = sig.width - 1 - b;
            const childTransitions = sig.transitions.map(t => {
              const ch = (t.bin && t.bin[b]) ? t.bin[b] : 'U';
              return {
                t: t.t,
                val: ch,
                hex: ch,
                bin: ch,
                dec: ch,
                h: ch === '1' || ch === 'H',
                l: ch === '0' || ch === 'L',
                z: ch === 'Z'
              };
            });
            displayRows.push({
              key: `${sig.name}[${bitIdx}]`,
              label: `${sig.name}[${bitIdx}]`,
              width: 1,
              isBus: false,
              expandable: false,
              isChild: true,
              transitions: childTransitions
            });
          }
        }
      });

      renderSignalList();
    }

    function getValAtTime(transitions, tFs) {
      let curr = transitions[0];
      for (let i = 0; i < transitions.length; i++) {
        if (transitions[i].t <= tFs) curr = transitions[i];
        else break;
      }
      return curr;
    }

    function renderSignalList() {
      sigListEl.innerHTML = '';
      displayRows.forEach((row, idx) => {
        const div = document.createElement('div');
        div.className = 'ws-row' +
          (row.isChild ? ' bit-child' : '') +
          (selectedSignalName === row.key ? ' selected' : '');

        const arrow = row.expandable ? (expandedBuses.has(row.key) ? '▼' : '▶') : '';
        div.innerHTML = `
          <div class="ws-left">
            <span class="ws-expand">${arrow}</span>
            <span class="ws-name" title="${row.label}">${row.label}</span>
          </div>
          <span class="ws-val" id="wval-${idx}">-</span>
        `;

        div.onclick = (e) => {
          if (row.expandable && e.target.classList.contains('ws-expand')) {
            if (expandedBuses.has(row.key)) expandedBuses.delete(row.key);
            else expandedBuses.add(row.key);
            rebuildDisplayRows();
            renderWaveform();
          } else {
            selectedSignalName = row.key;
            renderSignalList();
          }
        };
        div.ondblclick = () => {
          if (row.expandable) {
            if (expandedBuses.has(row.key)) expandedBuses.delete(row.key);
            else expandedBuses.add(row.key);
            rebuildDisplayRows();
            renderWaveform();
          }
        };

        sigListEl.appendChild(div);
      });
      updateCursorReadouts();
    }

    function updateCursorReadouts() {
      displayRows.forEach((row, idx) => {
        const el = document.getElementById('wval-' + idx);
        if (!el) return;
        const v = getValAtTime(row.transitions, cursorTimeFs);
        if (!v) return;
        if (row.width === 1) {
          el.textContent = v.val;
          el.style.color = v.h ? '#4ade80' : v.l ? '#94a3b8' : v.z ? '#38bdf8' : '#f87171';
        } else {
          el.textContent = v[radix] || v.hex;
          el.style.color = '#38bdf8';
        }
      });

      document.getElementById('cursor-readout').textContent = 'Cursor: ' + formatTime(cursorTimeFs);
      const deltaEl = document.getElementById('delta-readout');
      if (markerTimeFs !== null) {
        deltaEl.style.display = 'inline-block';
        deltaEl.textContent = 'Δt: ' + formatTime(Math.abs(cursorTimeFs - markerTimeFs));
      } else {
        deltaEl.style.display = 'none';
      }
    }

    // --- Vivado Canvas Waveform Renderer ---
    function resizeCanvas() {
      const rect = canvasWrap.getBoundingClientRect();
      if (rect.width <= 0 || rect.height <= 0) return;
      const dpr = window.devicePixelRatio || 1;
      canvas.width = rect.width * dpr;
      canvas.height = rect.height * dpr;
      canvas.style.width = rect.width + 'px';
      canvas.style.height = rect.height + 'px';
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      renderWaveform();
    }

    function tToX(t, w) {
      return ((t - timeStartFs) / (timeEndFs - timeStartFs)) * w;
    }

    function xToT(x, w) {
      return timeStartFs + (x / w) * (timeEndFs - timeStartFs);
    }

    function renderWaveform() {
      const rect = canvasWrap.getBoundingClientRect();
      const w = rect.width;
      const h = rect.height;
      if (w <= 0 || h <= 0) return;

      ctx.clearRect(0, 0, w, h);

      // Ruler header
      ctx.fillStyle = '#111827';
      ctx.fillRect(0, 0, w, HDR_H);
      ctx.strokeStyle = '#243044';
      ctx.beginPath();
      ctx.moveTo(0, HDR_H);
      ctx.lineTo(w, HDR_H);
      ctx.stroke();

      const span = Math.max(1, timeEndFs - timeStartFs);
      const rawStep = span / 9;
      const pow = Math.pow(10, Math.floor(Math.log10(rawStep)));
      const norm = rawStep / pow;
      const mult = norm < 2 ? 1 : norm < 5 ? 2 : 5;
      const gridStep = Math.max(1, mult * pow);
      const firstGrid = Math.ceil(timeStartFs / gridStep) * gridStep;

      ctx.font = '10px Consolas, monospace';
      ctx.textAlign = 'center';
      for (let t = firstGrid; t <= timeEndFs; t += gridStep) {
        const x = tToX(t, w);
        ctx.strokeStyle = '#141c2b';
        ctx.beginPath();
        ctx.moveTo(x, HDR_H);
        ctx.lineTo(x, h);
        ctx.stroke();

        ctx.strokeStyle = '#475569';
        ctx.beginPath();
        ctx.moveTo(x, HDR_H - 5);
        ctx.lineTo(x, HDR_H);
        ctx.stroke();

        ctx.fillStyle = '#94a3b8';
        ctx.fillText(formatTime(t), x, HDR_H - 9);
      }

      // Signal Tracks
      displayRows.forEach((row, idx) => {
        const yTop = HDR_H + idx * ROW_H - scrollY;
        const yBot = yTop + ROW_H;
        if (yBot < HDR_H || yTop > h) return;

        ctx.strokeStyle = '#151e2e';
        ctx.beginPath();
        ctx.moveTo(0, yBot);
        ctx.lineTo(w, yBot);
        ctx.stroke();

        const yHi = yTop + 6;
        const yLo = yBot - 7;
        const yMid = (yHi + yLo) / 2;
        const trs = row.transitions;

        if (row.width === 1) {
          ctx.lineWidth = 1.6;
          for (let i = 0; i < trs.length; i++) {
            const tr = trs[i];
            const nextT = (i + 1 < trs.length) ? trs[i + 1].t : timeEndFs;
            if (nextT < timeStartFs) continue;
            if (tr.t > timeEndFs) break;

            const x0 = Math.max(0, tToX(tr.t, w));
            const x1 = Math.min(w, tToX(nextT, w));
            const y = tr.h ? yHi : tr.l ? yLo : yMid;

            ctx.strokeStyle = tr.h ? '#22c55e' : tr.l ? '#64748b' : tr.z ? '#38bdf8' : '#ef4444';
            ctx.beginPath();
            ctx.moveTo(x0, y);
            ctx.lineTo(x1, y);
            ctx.stroke();

            if (i > 0 && tr.t >= timeStartFs && tr.t <= timeEndFs) {
              const prev = trs[i - 1];
              const prevY = prev.h ? yHi : prev.l ? yLo : yMid;
              ctx.strokeStyle = '#22c55e';
              ctx.beginPath();
              ctx.moveTo(x0, prevY);
              ctx.lineTo(x0, y);
              ctx.stroke();
            }
          }
        } else {
          ctx.lineWidth = 1.4;
          for (let i = 0; i < trs.length; i++) {
            const tr = trs[i];
            const nextT = (i + 1 < trs.length) ? trs[i + 1].t : timeEndFs;
            if (nextT < timeStartFs) continue;
            if (tr.t > timeEndFs) break;

            const x0 = Math.max(0, tToX(tr.t, w));
            const x1 = Math.min(w, tToX(nextT, w));
            const segW = x1 - x0;
            const slant = Math.min(4, segW / 4);

            ctx.strokeStyle = '#22c55e';
            ctx.fillStyle = 'rgba(34, 197, 94, 0.08)';
            ctx.beginPath();
            ctx.moveTo(x0 + slant, yHi);
            ctx.lineTo(x1 - slant, yHi);
            ctx.lineTo(x1, yMid);
            ctx.lineTo(x1 - slant, yLo);
            ctx.lineTo(x0 + slant, yLo);
            ctx.lineTo(x0, yMid);
            ctx.closePath();
            ctx.fill();
            ctx.stroke();

            if (segW > 22) {
              ctx.fillStyle = '#e2e8f0';
              ctx.font = '11px Consolas, monospace';
              ctx.textAlign = 'center';
              ctx.textBaseline = 'middle';
              ctx.fillText(tr[radix] || tr.hex, x0 + segW / 2, yMid);
            }
          }
        }
      });

      // Secondary Marker (Delta)
      if (markerTimeFs !== null && markerTimeFs >= timeStartFs && markerTimeFs <= timeEndFs) {
        const mx = tToX(markerTimeFs, w);
        ctx.strokeStyle = '#22d3ee';
        ctx.setLineDash([4, 3]);
        ctx.lineWidth = 1.2;
        ctx.beginPath();
        ctx.moveTo(mx, 0);
        ctx.lineTo(mx, h);
        ctx.stroke();
        ctx.setLineDash([]);
      }

      // Primary Cursor
      if (cursorTimeFs >= timeStartFs && cursorTimeFs <= timeEndFs) {
        const cx = tToX(cursorTimeFs, w);
        ctx.strokeStyle = '#f59e0b';
        ctx.lineWidth = 1.6;
        ctx.beginPath();
        ctx.moveTo(cx, 0);
        ctx.lineTo(cx, h);
        ctx.stroke();
      }

      updateCursorReadouts();
    }

    // --- Canvas Interactions ---
    canvasWrap.addEventListener('mousedown', (e) => {
      const rect = canvasWrap.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const clickedT = Math.max(0, Math.round(xToT(x, rect.width)));

      if (e.shiftKey) {
        markerTimeFs = clickedT;
        renderWaveform();
        return;
      }

      cursorTimeFs = clickedT;
      isDragging = true;
      dragStartX = e.clientX;
      dragStartTimeFs = timeStartFs;
      renderWaveform();
    });

    window.addEventListener('mousemove', (e) => {
      if (!isDragging) return;
      const rect = canvasWrap.getBoundingClientRect();
      const dx = e.clientX - dragStartX;
      if (Math.abs(dx) > 3) {
        const span = timeEndFs - timeStartFs;
        const dt = (dx / rect.width) * span;
        timeStartFs = Math.max(0, dragStartTimeFs - dt);
        timeEndFs = timeStartFs + span;
        renderWaveform();
      }
    });

    window.addEventListener('mouseup', () => { isDragging = false; });

    canvasWrap.addEventListener('wheel', (e) => {
      e.preventDefault();
      if (e.shiftKey) {
        // Vertical scroll synced with signal list
        scrollY = Math.max(0, scrollY + e.deltaY * 0.5);
        sigListEl.scrollTop = scrollY;
        renderWaveform();
        return;
      }
      const rect = canvasWrap.getBoundingClientRect();
      const mx = e.clientX - rect.left;
      const mt = xToT(mx, rect.width);
      const factor = e.deltaY < 0 ? 0.75 : 1.35;
      const newSpan = Math.max(1000, (timeEndFs - timeStartFs) * factor);
      const ratio = mx / rect.width;
      timeStartFs = Math.max(0, mt - newSpan * ratio);
      timeEndFs = timeStartFs + newSpan;
      renderWaveform();
    }, { passive: false });

    function zoomFit() {
      timeStartFs = 0;
      timeEndFs = (simResult && simResult.stop_time_fs > 0) ? simResult.stop_time_fs : 100_000_000;
      renderWaveform();
    }

    document.getElementById('w-zoom-in').onclick = () => {
      const span = (timeEndFs - timeStartFs) * 0.7;
      const mid = (cursorTimeFs >= timeStartFs && cursorTimeFs <= timeEndFs) ? cursorTimeFs : (timeStartFs + timeEndFs) / 2;
      timeStartFs = Math.max(0, mid - span / 2);
      timeEndFs = timeStartFs + span;
      renderWaveform();
    };
    document.getElementById('w-zoom-out').onclick = () => {
      const span = (timeEndFs - timeStartFs) * 1.4;
      const mid = (timeStartFs + timeEndFs) / 2;
      timeStartFs = Math.max(0, mid - span / 2);
      timeEndFs = timeStartFs + span;
      renderWaveform();
    };
    document.getElementById('w-zoom-fit').onclick = zoomFit;
    document.getElementById('w-edge-start').onclick = () => { cursorTimeFs = 0; renderWaveform(); };
    document.getElementById('w-edge-end').onclick = () => {
      cursorTimeFs = simResult ? simResult.stop_time_fs : 0;
      renderWaveform();
    };

    function jumpEdge(dir) {
      const row = displayRows.find(r => r.key === selectedSignalName) || displayRows[0];
      if (!row) return;
      if (dir > 0) {
        const next = row.transitions.find(tr => tr.t > cursorTimeFs);
        if (next) cursorTimeFs = next.t;
      } else {
        const prevs = row.transitions.filter(tr => tr.t < cursorTimeFs);
        if (prevs.length > 0) cursorTimeFs = prevs[prevs.length - 1].t;
      }
      renderWaveform();
    }
    document.getElementById('w-edge-prev').onclick = () => jumpEdge(-1);
    document.getElementById('w-edge-next').onclick = () => jumpEdge(1);

    document.querySelectorAll('.radix-btn').forEach(btn => {
      btn.onclick = () => {
        radix = btn.dataset.radix;
        document.querySelectorAll('.radix-btn').forEach(b => b.classList.remove('active'));
        btn.classList.add('active');
        renderWaveform();
      };
    });

    // Layout Switcher
    const edPane = document.getElementById('editor-pane');
    const wvPane = document.getElementById('wave-pane');
    function setLayout(mode) {
      document.getElementById('btn-layout-split').classList.toggle('active', mode === 'split');
      document.getElementById('btn-layout-editor').classList.toggle('active', mode === 'editor');
      document.getElementById('btn-layout-wave').classList.toggle('active', mode === 'wave');
      edPane.classList.toggle('hidden', mode === 'wave');
      wvPane.classList.toggle('hidden', mode === 'editor');
      resizeCanvas();
    }
    document.getElementById('btn-layout-split').onclick = () => setLayout('split');
    document.getElementById('btn-layout-editor').onclick = () => setLayout('editor');
    document.getElementById('btn-layout-wave').onclick = () => setLayout('wave');

    // Bottom Console Tabs
    document.querySelectorAll('.ctab').forEach(tab => {
      tab.onclick = () => {
        document.querySelectorAll('.ctab').forEach(t => t.classList.remove('active'));
        tab.classList.add('active');
        const target = tab.dataset.ctab;
        document.getElementById('console-messages').classList.toggle('hidden', target !== 'messages');
        document.getElementById('console-netlist').classList.toggle('hidden', target !== 'netlist');
        document.getElementById('console-ascii').classList.toggle('hidden', target !== 'ascii');
      };
    });

    window.addEventListener('resize', resizeCanvas);
    fetchExamples();
  </script>
</body>
</html>
"#
    }
}
