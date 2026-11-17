import {
  drawAvenue,
  drawLineChart,
  extractBeamsFromMessages,
  type HistoryPoint,
  type VisualBeam,
} from './renderer';
import type { AclMessage, SimCommand, SimSnapshot } from './types';

const avenueCanvas = document.getElementById('avenue-canvas') as HTMLCanvasElement;
const priceCanvas = document.getElementById('price-chart') as HTMLCanvasElement;
const cashCanvas = document.getElementById('cash-chart') as HTMLCanvasElement;
const stationsRow = document.getElementById('stations-row') as HTMLElement;
const dfTbody = document.getElementById('df-tbody') as HTMLElement;
const dfCount = document.getElementById('df-count') as HTMLElement;
const aclLogEl = document.getElementById('acl-log') as HTMLElement;

const connBadge = document.getElementById('conn-badge') as HTMLElement;
const dayBadge = document.getElementById('day-badge') as HTMLElement;
const regimeBadge = document.getElementById('regime-badge') as HTMLElement;
const tickBadge = document.getElementById('tick-badge') as HTMLElement;

const btnPlayPause = document.getElementById('btn-play-pause') as HTMLButtonElement;
const btnStep = document.getElementById('btn-step') as HTMLButtonElement;
const btnCrisis = document.getElementById('btn-crisis') as HTMLButtonElement;
const btnReset = document.getElementById('btn-reset') as HTMLButtonElement;
const speedRange = document.getElementById('speed-range') as HTMLInputElement;
const aclTabs = document.getElementById('acl-tabs') as HTMLElement;

let latestSnap: SimSnapshot | null = null;
let history: HistoryPoint[] = [];
let allMessages: AclMessage[] = [];
let beams: VisualBeam[] = [];
let activeFilter: 'all' | 'cartel' | 'df' | 'fuel' = 'all';
let ws: WebSocket | null = null;

const fmtBRL = (cents: number) => `R$ ${(cents / 100).toFixed(2).replace('.', ',')}`;

function sendCmd(cmd: SimCommand) {
  if (ws && ws.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify(cmd));
  }
}

function connectWs() {
  const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
  const url = `${proto}//${location.host}/ws`;
  ws = new WebSocket(url);

  ws.onopen = () => {
    connBadge.textContent = '🟢 Conectado';
    connBadge.className = 'pill pill-ok';
  };

  ws.onclose = () => {
    connBadge.textContent = '🟠 Reconectando...';
    connBadge.className = 'pill pill-warn';
    setTimeout(connectWs, 1200);
  };

  ws.onmessage = (ev) => {
    const snap: SimSnapshot = JSON.parse(ev.data);

    // Detect reset
    if (latestSnap && snap.tick < latestSnap.tick) {
      history = [];
      allMessages = [];
      beams = [];
    }

    latestSnap = snap;

    // Record history point if tick advanced
    if (history.length === 0 || history[history.length - 1].tick !== snap.tick) {
      const prices: Record<string, number> = {};
      const cash: Record<string, number> = {};
      for (const s of snap.stations) {
        if (s.is_open) prices[s.name] = s.price_cents;
        cash[s.name] = s.cash_cents;
      }
      history.push({ tick: snap.tick, prices, cash });
      if (history.length > 160) history.shift();
    }

    if (snap.new_messages.length > 0) {
      beams.push(...extractBeamsFromMessages(snap.new_messages, snap.stations));
      allMessages.push(...snap.new_messages);
      if (allMessages.length > 250) {
        allMessages = allMessages.slice(-250);
      }
    }

    renderUI(snap);
  };
}

function renderUI(snap: SimSnapshot) {
  tickBadge.textContent = `Tick: ${snap.tick}`;
  btnPlayPause.textContent = snap.paused ? '▶ Continuar' : '⏸ Pausar';

  if (snap.is_night) {
    dayBadge.textContent = '🌙 Calada da Noite';
    dayBadge.className = 'pill pill-night';
  } else {
    dayBadge.textContent = '☀️ Dia';
    dayBadge.className = 'pill pill-day';
  }

  const regimeMap = {
    FreeMarket: ['Mercado Livre', 'pill pill-free'],
    CartelActive: ['🤝 Cartel Ativo', 'pill pill-cartel'],
    PriceWar: ['⚔️ Guerra de Preços!', 'pill pill-war'],
    Monopoly: ['👑 Monopólio', 'pill pill-warn'],
  } as const;
  const [rLabel, rClass] = regimeMap[snap.market_regime];
  regimeBadge.textContent = rLabel;
  regimeBadge.className = rClass;

  // Render Station Cards
  stationsRow.innerHTML = snap.stations
    .map((st) => {
      const tagMap = {
        Independent: ['Independente', 'tag-indep'],
        CartelMember: ['Cartel', 'tag-cartel'],
        Traitor: ['Traíra!', 'tag-traitor'],
        Bankrupt: ['Falido', 'tag-bankrupt'],
      } as const;
      const [tLabel, tClass] = tagMap[st.cartel_status];
      const cashPct = Math.max(0, Math.min(100, (st.cash_cents / (st.initial_cash_cents * 1.5)) * 100));
      const cashColor = st.cash_cents < 90_000 ? '#ef4444' : '#22c55e';

      return `
        <div class="station-card ${st.is_open ? '' : 'bankrupt'}" style="border-top-color: ${st.color}">
          <div class="station-top">
            <span class="station-name" style="color: ${st.color}">${st.name}</span>
            <span class="status-tag ${tClass}">${tLabel}</span>
          </div>
          <div class="station-price">${st.is_open ? fmtBRL(st.price_cents) : 'FECHADO'}</div>
          <div class="station-metrics">
            <span>Caixa: <strong>${fmtBRL(st.cash_cents)}</strong></span>
            <span>Estoque: <strong>${Math.round(st.stock_liters)}L</strong></span>
            <span>Vendas: <strong>${Math.round(st.liters_sold_total)}L</strong></span>
            <span>Clientes: <strong>${st.customers_served}</strong></span>
          </div>
          <div class="cash-bar-bg">
            <div class="cash-bar-fill" style="width: ${cashPct}%; background: ${cashColor}"></div>
          </div>
          <div class="station-action">${st.last_action}</div>
        </div>
      `;
    })
    .join('');

  // Render DF Table
  dfCount.textContent = `${snap.df_listings.length} serviços ativos`;
  const sortedDf = [...snap.df_listings].sort((a, b) => a.price_cents - b.price_cents);
  dfTbody.innerHTML =
    sortedDf.length === 0
      ? `<tr><td colspan="5" style="color:#64748b">Nenhum posto aberto nas Páginas Amarelas</td></tr>`
      : sortedDf
          .map(
            (s, idx) => `
          <tr>
            <td><strong>${s.agent_name}</strong></td>
            <td><code>${s.service_type}</code></td>
            <td style="font-family: monospace; color: ${idx === 0 ? '#4ade80' : '#e2e8f0'}">
              ${fmtBRL(s.price_cents)} ${idx === 0 ? '🔥' : ''}
            </td>
            <td>${Math.round(s.stock_liters)} L</td>
            <td>${s.is_open ? '🟢 Aberto' : '🔴 Fechado'}</td>
          </tr>
        `,
          )
          .join('');

  renderAclLog();
  drawLineChart(priceCanvas, history, snap.stations, 'price');
  drawLineChart(cashCanvas, history, snap.stations, 'cash');
}

function renderAclLog() {
  const filtered = allMessages.filter((m) => {
    if (activeFilter === 'cartel') {
      return m.sender.kind === 'Station' && m.receiver.kind === 'Station';
    }
    if (activeFilter === 'df') {
      return m.sender.kind === 'DirectoryFacilitator' || m.receiver.kind === 'DirectoryFacilitator';
    }
    if (activeFilter === 'fuel') {
      return m.content.type === 'FuelRequest' || m.content.type === 'FuelReceipt' || m.content.type === 'FuelRefused';
    }
    return true;
  });

  const recent = filtered.slice(-45).reverse();
  aclLogEl.innerHTML = recent
    .map((m) => {
      let cls = 'acl-item';
      if (m.content.type === 'CartelBetrayalAlert') cls += ' perf-inform-betrayal';
      else if (m.performative === 'Propose') cls += ' perf-propose';
      else if (m.performative === 'AcceptProposal') cls += ' perf-accept';
      else if (m.performative === 'RejectProposal') cls += ' perf-reject';
      else if (m.sender.kind === 'DirectoryFacilitator' || m.receiver.kind === 'DirectoryFacilitator') {
        cls += ' perf-df';
      }

      return `
        <div class="${cls}">
          <div class="acl-meta">
            <span class="acl-perf">[${m.performative.toUpperCase()}] ${m.sender.name} → ${m.receiver.name}</span>
            <span>Tick ${m.tick}</span>
          </div>
          <div>${m.summary}</div>
        </div>
      `;
    })
    .join('');
}

// Smooth animation loop for avenue canvas & fading ACL message arcs
function animate() {
  if (latestSnap) {
    drawAvenue(avenueCanvas, latestSnap, beams);
    for (const b of beams) b.ttl -= 1;
    beams = beams.filter((b) => b.ttl > 0);
  }
  requestAnimationFrame(animate);
}

// Event Listeners
btnPlayPause.addEventListener('click', () => {
  if (!latestSnap) return;
  sendCmd({ cmd: latestSnap.paused ? 'Play' : 'Pause' });
});
btnStep.addEventListener('click', () => sendCmd({ cmd: 'Step' }));
btnCrisis.addEventListener('click', () => sendCmd({ cmd: 'InjectCashCrisis' }));
btnReset.addEventListener('click', () => sendCmd({ cmd: 'Reset' }));
speedRange.addEventListener('input', () => {
  const tick_ms = Number(speedRange.value);
  sendCmd({ cmd: 'SetSpeed', payload: { tick_ms } });
});

aclTabs.addEventListener('click', (ev) => {
  const target = ev.target as HTMLElement;
  const filter = target.dataset.filter as typeof activeFilter | undefined;
  if (!filter) return;
  activeFilter = filter;
  aclTabs.querySelectorAll('.tab').forEach((t) => t.classList.remove('active'));
  target.classList.add('active');
  renderAclLog();
});

connectWs();
requestAnimationFrame(animate);
