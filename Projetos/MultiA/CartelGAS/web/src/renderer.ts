import type { AclMessage, DriverSnapshot, SimSnapshot, StationSnapshot } from './types';

export interface VisualBeam {
  fromX: number;
  toX: number;
  color: string;
  label: string;
  ttl: number;
  maxTtl: number;
}

export interface HistoryPoint {
  tick: number;
  prices: Record<string, number>;
  cash: Record<string, number>;
}

const fmtBRL = (cents: number) => `R$ ${(cents / 100).toFixed(2).replace('.', ',')}`;

export function extractBeamsFromMessages(
  messages: AclMessage[],
  stations: StationSnapshot[],
): VisualBeam[] {
  const posMap = new Map<string, number>();
  for (const s of stations) posMap.set(s.name, s.position_x);

  const beams: VisualBeam[] = [];
  for (const m of messages) {
    if (m.sender.kind === 'Station' && m.receiver.kind === 'Station') {
      const fromX = posMap.get(m.sender.name);
      const toX = posMap.get(m.receiver.name);
      if (fromX === undefined || toX === undefined) continue;

      let color = '#a855f7';
      let label: string = m.performative;
      if (m.content.type === 'CartelBetrayalAlert') {
        color = '#f43f5e';
        label = 'TRAIÇÃO!';
      } else if (m.performative === 'AcceptProposal') {
        color = '#22c55e';
        label = 'ACCEPT';
      } else if (m.performative === 'Propose') {
        color = '#c084fc';
        label = 'PROPOSE';
      } else if (m.performative === 'RejectProposal') {
        color = '#f59e0b';
        label = 'REJECT';
      }

      beams.push({ fromX, toX, color, label, ttl: 28, maxTtl: 28 });
    }
  }
  return beams;
}

export function drawAvenue(
  canvas: HTMLCanvasElement,
  snap: SimSnapshot,
  beams: VisualBeam[],
) {
  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  const W = canvas.width;
  const H = canvas.height;
  const mapX = (x: number) => 60 + (x / 1000) * (W - 120);

  // Sky / atmosphere
  if (snap.is_night) {
    const grad = ctx.createLinearGradient(0, 0, 0, H);
    grad.addColorStop(0, '#040714');
    grad.addColorStop(1, '#0f172a');
    ctx.fillStyle = grad;
  } else {
    const grad = ctx.createLinearGradient(0, 0, 0, H);
    grad.addColorStop(0, '#0c192c');
    grad.addColorStop(1, '#111827');
    ctx.fillStyle = grad;
  }
  ctx.fillRect(0, 0, W, H);

  if (snap.is_night) {
    ctx.fillStyle = 'rgba(165, 180, 252, 0.85)';
    ctx.font = 'bold 12px Inter, sans-serif';
    ctx.fillText('🌙 CALADA DA NOITE — Risco de traição no cartel!', 18, 22);
  } else {
    ctx.fillStyle = 'rgba(253, 224, 71, 0.75)';
    ctx.font = 'bold 12px Inter, sans-serif';
    ctx.fillText('☀️ DIA — Movimento normal na avenida', 18, 22);
  }

  // Road
  const roadTop = 175;
  const roadH = 115;
  ctx.fillStyle = '#1e293b';
  ctx.fillRect(0, roadTop, W, roadH);
  ctx.strokeStyle = '#475569';
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.moveTo(0, roadTop);
  ctx.lineTo(W, roadTop);
  ctx.moveTo(0, roadTop + roadH);
  ctx.lineTo(W, roadTop + roadH);
  ctx.stroke();

  // Center dashed line
  ctx.save();
  ctx.setLineDash([20, 16]);
  ctx.strokeStyle = '#eab308';
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.moveTo(0, roadTop + roadH / 2);
  ctx.lineTo(W, roadTop + roadH / 2);
  ctx.stroke();
  ctx.restore();

  // Find cheapest open station for highlight
  const openStations = snap.stations.filter((s) => s.is_open);
  const minPrice = openStations.length > 0 ? Math.min(...openStations.map((s) => s.price_cents)) : -1;

  // Draw Gas Stations
  for (const st of snap.stations) {
    const cx = mapX(st.position_x);
    const baseY = 160;
    const isCheapest = st.is_open && st.price_cents === minPrice;

    // Station canopy
    ctx.save();
    if (!st.is_open) ctx.globalAlpha = 0.4;

    ctx.fillStyle = st.is_open ? '#0f172a' : '#1c1014';
    ctx.strokeStyle = isCheapest ? '#22c55e' : st.color;
    ctx.lineWidth = isCheapest ? 3 : 2;
    ctx.beginPath();
    ctx.roundRect(cx - 56, baseY - 90, 112, 82, 8);
    ctx.fill();
    ctx.stroke();

    // Top color bar
    ctx.fillStyle = st.color;
    ctx.fillRect(cx - 54, baseY - 88, 108, 6);

    // Station name
    ctx.fillStyle = '#f8fafc';
    ctx.font = 'bold 12px Inter, sans-serif';
    ctx.textAlign = 'center';
    ctx.fillText(st.name, cx, baseY - 66);

    // Price sign
    ctx.fillStyle = isCheapest ? '#4ade80' : '#f8fafc';
    ctx.font = 'bold 18px ui-monospace, monospace';
    ctx.fillText(st.is_open ? fmtBRL(st.price_cents) : 'FALIDO', cx, baseY - 42);

    // Status badge
    let badge = 'LIVRE';
    let badgeColor = '#94a3b8';
    if (st.cartel_status === 'CartelMember') {
      badge = '🤝 CARTEL';
      badgeColor = '#d8b4fe';
    } else if (st.cartel_status === 'Traitor') {
      badge = '🗡️ TRAÍRA';
      badgeColor = '#fca5a5';
    } else if (st.cartel_status === 'Bankrupt') {
      badge = '💀 FALIDO';
      badgeColor = '#ef4444';
    } else if (isCheapest) {
      badge = '🔥 +BARATO';
      badgeColor = '#4ade80';
    }
    ctx.fillStyle = badgeColor;
    ctx.font = 'bold 10px Inter, sans-serif';
    ctx.fillText(badge, cx, baseY - 24);

    // Cash mini bar
    const cashRatio = Math.max(0, Math.min(1, st.cash_cents / (st.initial_cash_cents * 1.5)));
    ctx.fillStyle = '#1e293b';
    ctx.fillRect(cx - 42, baseY - 15, 84, 5);
    ctx.fillStyle = st.cash_cents < 90_000 ? '#ef4444' : '#22c55e';
    ctx.fillRect(cx - 42, baseY - 15, 84 * cashRatio, 5);

    ctx.restore();
  }

  // Draw Private ACL Message Arcs between stations
  for (const b of beams) {
    const x1 = mapX(b.fromX);
    const x2 = mapX(b.toX);
    const alpha = Math.max(0.1, b.ttl / b.maxTtl);
    const midX = (x1 + x2) / 2;
    const arcHeight = Math.min(55, Math.abs(x2 - x1) * 0.16 + 18);

    ctx.save();
    ctx.globalAlpha = alpha;
    ctx.strokeStyle = b.color;
    ctx.lineWidth = 2.2;
    ctx.beginPath();
    ctx.moveTo(x1, 70);
    ctx.quadraticCurveTo(midX, 70 - arcHeight, x2, 70);
    ctx.stroke();

    ctx.fillStyle = b.color;
    ctx.font = 'bold 10px ui-monospace, monospace';
    ctx.textAlign = 'center';
    ctx.fillText(b.label, midX, 66 - arcHeight * 0.55);
    ctx.restore();
  }

  // Draw Drivers (20 cars)
  const stationPos = new Map(snap.stations.map((s) => [s.name, mapX(s.position_x)]));
  snap.drivers.forEach((dr: DriverSnapshot, idx: number) => {
    const cx = mapX(dr.position_x);
    const laneOffset = dr.direction > 0 ? roadTop + 28 : roadTop + 84;
    const jitter = ((idx % 3) - 1) * 7;
    const cy = dr.activity === 'Refueling' ? roadTop + 6 : laneOffset + jitter;

    // Target line if seeking fuel
    if (dr.activity === 'SeekingFuel' && dr.target_station) {
      const tx = stationPos.get(dr.target_station);
      if (tx !== undefined) {
        ctx.save();
        ctx.setLineDash([4, 4]);
        ctx.strokeStyle = 'rgba(239, 68, 68, 0.45)';
        ctx.lineWidth = 1.2;
        ctx.beginPath();
        ctx.moveTo(cx, cy);
        ctx.lineTo(tx, 162);
        ctx.stroke();
        ctx.restore();
      }
    }

    const onReserve = dr.fuel_liters <= dr.reserve_threshold;
    const carColor =
      dr.activity === 'Refueling'
        ? '#38bdf8'
        : onReserve
          ? '#ef4444'
          : dr.fuel_liters < 24
            ? '#f59e0b'
            : '#22c55e';

    ctx.save();
    ctx.fillStyle = carColor;
    ctx.beginPath();
    ctx.roundRect(cx - 10, cy - 6, 20, 12, 4);
    ctx.fill();

    // Headlights
    ctx.fillStyle = '#fef08a';
    const lightX = dr.direction > 0 ? cx + 8 : cx - 10;
    ctx.fillRect(lightX, cy - 4, 2, 3);
    ctx.fillRect(lightX, cy + 1, 2, 3);
    ctx.restore();
  });
}

export function drawLineChart(
  canvas: HTMLCanvasElement,
  history: HistoryPoint[],
  stations: StationSnapshot[],
  mode: 'price' | 'cash',
) {
  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  const W = canvas.width;
  const H = canvas.height;
  ctx.clearRect(0, 0, W, H);
  ctx.fillStyle = '#070b14';
  ctx.fillRect(0, 0, W, H);

  if (history.length < 2) {
    ctx.fillStyle = '#64748b';
    ctx.font = '12px Inter, sans-serif';
    ctx.fillText('Aguardando dados da simulação...', 20, H / 2);
    return;
  }

  const padL = 56;
  const padR = 14;
  const padT = 14;
  const padB = 22;
  const plotW = W - padL - padR;
  const plotH = H - padT - padB;

  let minVal = Infinity;
  let maxVal = -Infinity;
  for (const pt of history) {
    const dict = mode === 'price' ? pt.prices : pt.cash;
    for (const v of Object.values(dict)) {
      if (v < minVal) minVal = v;
      if (v > maxVal) maxVal = v;
    }
  }
  if (mode === 'price') {
    minVal = Math.min(minVal, 440);
    maxVal = Math.max(maxVal, 720);
  } else {
    minVal = 0;
    maxVal = Math.max(maxVal, 250_000);
  }
  const range = Math.max(1, maxVal - minVal);

  // Horizontal grid lines
  ctx.strokeStyle = '#1e293b';
  ctx.lineWidth = 1;
  ctx.fillStyle = '#64748b';
  ctx.font = '10px ui-monospace, monospace';
  ctx.textAlign = 'right';
  for (let i = 0; i <= 3; i++) {
    const ratio = i / 3;
    const y = padT + plotH - ratio * plotH;
    const val = minVal + ratio * range;
    ctx.beginPath();
    ctx.moveTo(padL, y);
    ctx.lineTo(W - padR, y);
    ctx.stroke();

    const label = mode === 'price' ? `R$${(val / 100).toFixed(2)}` : `R$${Math.round(val / 100)}`;
    ctx.fillText(label, padL - 6, y + 3);
  }

  // Series per station
  for (const st of stations) {
    ctx.strokeStyle = st.color;
    ctx.lineWidth = 2;
    ctx.beginPath();
    history.forEach((pt, idx) => {
      const val = (mode === 'price' ? pt.prices[st.name] : pt.cash[st.name]) ?? 0;
      const x = padL + (idx / (history.length - 1)) * plotW;
      const y = padT + plotH - ((val - minVal) / range) * plotH;
      if (idx === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    });
    ctx.stroke();
  }
}
