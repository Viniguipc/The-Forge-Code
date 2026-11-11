import './style.css';
import {
  ALL_DIRECTIONS,
  DIRECTION_ARROWS,
  MazeEnvironment,
} from './environment';
import { MAZE_PRESETS } from './presets';
import { QLearningAgent } from './qlearning';
import { buildPathTree, getTreeStats } from './tree';
import type {
  CellType,
  Direction,
  Position,
  RawCellTuple,
  TreeNode,
} from './types';

const CELL_TYPE_LABELS: Record<CellType, string> = {
  0: 'Livre',
  1: 'Início',
  2: 'Buraco',
  3: 'Objetivo',
};

const DIRECTION_LABELS: Record<Direction, string> = {
  N: 'N ↑',
  E: 'L →',
  S: 'S ↓',
  W: 'O ←',
};

// Estado global da aplicação
let env: MazeEnvironment;
let agent: QLearningAgent;
let pathTree: TreeNode;
let agentPosition: Position;

let visualTimerId: number | null = null;
let playbackTimerId: number | null = null;
let treeViewMode: 'list' | 'svg' = 'list';
let defaultExpandedDepth = 6;

// Elementos do DOM
const elMazeGrid = document.getElementById('maze-grid') as HTMLDivElement;
const elMazeDimLabel = document.getElementById('maze-dimensions-label') as HTMLSpanElement;
const elValidationBanner = document.getElementById('validation-banner') as HTMLDivElement;
const elBestPathBadge = document.getElementById('best-path-badge') as HTMLSpanElement;
const elBestPathText = document.getElementById('best-path-text') as HTMLDivElement;
const elQTableBody = document.getElementById('q-table-body') as HTMLTableSectionElement;

const elPresetSelect = document.getElementById('preset-select') as HTMLSelectElement;
const elPresetDesc = document.getElementById('preset-description') as HTMLParagraphElement;
const elJsonInput = document.getElementById('maze-json-input') as HTMLTextAreaElement;
const elJsonError = document.getElementById('json-error') as HTMLSpanElement;
const elBtnFormatJson = document.getElementById('btn-format-json') as HTMLButtonElement;
const elBtnApplyJson = document.getElementById('btn-apply-json') as HTMLButtonElement;

const elParamAlpha = document.getElementById('param-alpha') as HTMLInputElement;
const elParamGamma = document.getElementById('param-gamma') as HTMLInputElement;
const elParamEpsilon = document.getElementById('param-epsilon') as HTMLInputElement;
const elParamSpeed = document.getElementById('param-speed') as HTMLInputElement;
const elParamEpisodes = document.getElementById('param-episodes') as HTMLInputElement;
const elParamMaxSteps = document.getElementById('param-max-steps') as HTMLInputElement;
const elParamRestrictValid = document.getElementById('param-restrict-valid') as HTMLInputElement;

const elValAlpha = document.getElementById('val-alpha') as HTMLSpanElement;
const elValGamma = document.getElementById('val-gamma') as HTMLSpanElement;
const elValEpsilon = document.getElementById('val-epsilon') as HTMLSpanElement;
const elValSpeed = document.getElementById('val-speed') as HTMLSpanElement;

const elBtnTrainInstant = document.getElementById('btn-train-instant') as HTMLButtonElement;
const elBtnTrainVisual = document.getElementById('btn-train-visual') as HTMLButtonElement;
const elBtnRunBest = document.getElementById('btn-run-best') as HTMLButtonElement;
const elBtnReset = document.getElementById('btn-reset') as HTMLButtonElement;

const elBadgeGridSize = document.getElementById('badge-grid-size') as HTMLSpanElement;
const elBadgeEpisodes = document.getElementById('badge-episodes') as HTMLSpanElement;
const elBadgeEpsilon = document.getElementById('badge-epsilon') as HTMLSpanElement;
const elBadgeGoalStatus = document.getElementById('badge-goal-status') as HTMLSpanElement;

const elMetricEpisodes = document.getElementById('metric-episodes') as HTMLElement;
const elMetricSuccessRate = document.getElementById('metric-success-rate') as HTMLElement;
const elMetricOutcomes = document.getElementById('metric-outcomes') as HTMLElement;
const elMetricLastReward = document.getElementById('metric-last-reward') as HTMLElement;
const elLiveLog = document.getElementById('live-log') as HTMLDivElement;
const elRewardChart = document.getElementById('reward-chart') as unknown as SVGSVGElement;

const elTreeStats = document.getElementById('tree-stats') as HTMLDivElement;
const elTreeWarning = document.getElementById('tree-warning') as HTMLDivElement;
const elTreeListContainer = document.getElementById('tree-list-container') as HTMLDivElement;
const elTreeSvgContainer = document.getElementById('tree-svg-container') as HTMLDivElement;
const elTabTreeList = document.getElementById('tab-tree-list') as HTMLButtonElement;
const elTabTreeSvg = document.getElementById('tab-tree-svg') as HTMLButtonElement;
const elBtnExpandAll = document.getElementById('btn-expand-all') as HTMLButtonElement;
const elBtnCollapseAll = document.getElementById('btn-collapse-all') as HTMLButtonElement;
const elChkOnlyGoalBranches = document.getElementById('chk-only-goal-branches') as HTMLInputElement;

function stopAnimations(): void {
  if (visualTimerId !== null) {
    window.clearInterval(visualTimerId);
    visualTimerId = null;
    elBtnTrainVisual.textContent = 'Treinar Visualmente (Passo a Passo)';
  }
  if (playbackTimerId !== null) {
    window.clearInterval(playbackTimerId);
    playbackTimerId = null;
    elBtnRunBest.textContent = 'Executar Melhor Caminho Aprendido';
  }
}

function readHyperparameters() {
  const alpha = Number(elParamAlpha.value);
  const gamma = Number(elParamGamma.value);
  const epsilon = Number(elParamEpsilon.value);
  const episodes = Math.max(1, Math.min(20000, Number(elParamEpisodes.value) || 300));
  const maxStepsPerEpisode = Math.max(
    10,
    Math.min(1000, Number(elParamMaxSteps.value) || 100)
  );
  const restrictToValidActions = elParamRestrictValid.checked;
  const speedMs = Number(elParamSpeed.value);

  return {
    alpha,
    gamma,
    epsilon,
    episodes,
    maxStepsPerEpisode,
    restrictToValidActions,
    speedMs,
  };
}

function syncHyperparamLabels(): void {
  const hp = readHyperparameters();
  elValAlpha.textContent = hp.alpha.toFixed(2);
  elValGamma.textContent = hp.gamma.toFixed(2);
  elValEpsilon.textContent = hp.epsilon.toFixed(2);
  elValSpeed.textContent = `${hp.speedMs}ms`;
  elBtnTrainInstant.textContent = `Treinar Instantâneo (${hp.episodes} episódios)`;

  if (agent) {
    const targetEpisodes = Math.max(50, hp.episodes);
    const epsilonDecay =
      hp.epsilon > 0.05
        ? Math.pow(0.05 / hp.epsilon, 1 / (targetEpisodes * 0.85))
        : 0.99;

    agent.updateConfig({
      alpha: hp.alpha,
      gamma: hp.gamma,
      epsilon: hp.epsilon,
      epsilonDecay: Math.min(0.995, Math.max(0.9, epsilonDecay)),
      maxStepsPerEpisode: hp.maxStepsPerEpisode,
      restrictToValidActions: hp.restrictToValidActions,
    });
  }
}

function formatTuplesForTextarea(tuples: RawCellTuple[]): string {
  const lines = tuples.map((t) => `  [${t.join(', ')}]`);
  return `[\n${lines.join(',\n')}\n]`;
}

function loadMaze(rawTuples: RawCellTuple[], descriptionText: string): void {
  stopAnimations();
  elJsonError.textContent = '';

  const newEnv = new MazeEnvironment(rawTuples);
  env = newEnv;

  const hp = readHyperparameters();
  // Ajusta maxSteps automaticamente se o grid for grande e o usuário estiver no padrão 100
  const recommendedSteps = Math.max(100, env.rows * env.cols * 4);
  if (Number(elParamMaxSteps.value) < env.rows * env.cols * 2) {
    elParamMaxSteps.value = String(recommendedSteps);
  }

  // Ajusta decaimento proporcional para convergir bem no número de episódios configurado
  const targetEpisodes = Math.max(50, hp.episodes);
  const epsilonDecay =
    hp.epsilon > 0.05
      ? Math.pow(0.05 / hp.epsilon, 1 / (targetEpisodes * 0.85))
      : 0.99;

  agent = new QLearningAgent(env, {
    alpha: hp.alpha,
    gamma: hp.gamma,
    epsilon: hp.epsilon,
    epsilonMin: 0.05,
    epsilonDecay: Math.min(0.995, Math.max(0.9, epsilonDecay)),
    maxStepsPerEpisode: Math.max(
      10,
      Number(elParamMaxSteps.value) || recommendedSteps
    ),
    restrictToValidActions: hp.restrictToValidActions,
  });

  agentPosition = { ...env.startPosition };
  pathTree = buildPathTree(env);
  defaultExpandedDepth = env.rows * env.cols <= 16 ? 8 : 4;

  elPresetDesc.textContent = descriptionText;
  elJsonInput.value = formatTuplesForTextarea(rawTuples);

  renderValidationBanner();
  renderAll();
  renderTree();
  elLiveLog.textContent = `Labirinto ${env.rows}×${env.cols} carregado. Início em (${env.startPosition.row}, ${env.startPosition.col}).`;
}

function renderValidationBanner(): void {
  const issues = env.validate();
  if (issues.length === 0) {
    elValidationBanner.classList.add('hidden');
    elValidationBanner.classList.remove('has-errors');
    elValidationBanner.replaceChildren();
    return;
  }

  const hasErrors = issues.some((i) => i.severity === 'error');
  elValidationBanner.classList.remove('hidden');
  elValidationBanner.classList.toggle('has-errors', hasErrors);

  const title = document.createElement('strong');
  title.textContent = hasErrors
    ? 'Erros de estrutura detectados no labirinto:'
    : 'Avisos de consistência do labirinto:';

  const list = document.createElement('ul');
  for (const issue of issues.slice(0, 6)) {
    const li = document.createElement('li');
    li.textContent = issue.message;
    list.appendChild(li);
  }
  if (issues.length > 6) {
    const li = document.createElement('li');
    li.textContent = `... e mais ${issues.length - 6} aviso(s).`;
    list.appendChild(li);
  }

  elValidationBanner.replaceChildren(title, list);
}

function renderMazeGrid(): void {
  const maxDim = Math.max(env.rows, env.cols);
  const cellSize = Math.max(48, Math.min(96, Math.floor(380 / maxDim)));

  elMazeGrid.style.gridTemplateColumns = `repeat(${env.cols}, ${cellSize}px)`;
  elMazeDimLabel.textContent = `${env.rows * env.cols} células (${env.rows} linhas × ${env.cols} colunas)`;
  elBadgeGridSize.textContent = `Grid: ${env.rows}×${env.cols}`;

  const bestPath = agent.getBestPath();
  const bestPathSet = new Set<string>();
  if (bestPath.reachedGoal) {
    for (const pos of bestPath.path) {
      bestPathSet.add(`${pos.row},${pos.col}`);
    }
  }

  const fragment = document.createDocumentFragment();

  for (let r = 0; r < env.rows; r++) {
    for (let c = 0; c < env.cols; c++) {
      const cell = env.getCell(r, c);
      const cellEl = document.createElement('div');
      cellEl.className = 'maze-cell';
      cellEl.style.width = `${cellSize}px`;
      cellEl.style.height = `${cellSize}px`;

      if (!cell) {
        cellEl.classList.add('wall-n', 'wall-e', 'wall-s', 'wall-w');
        cellEl.style.background = '#cbd5e1';
        const missing = document.createElement('div');
        missing.className = 'cell-center';
        missing.textContent = '—';
        cellEl.appendChild(missing);
        fragment.appendChild(cellEl);
        continue;
      }

      cellEl.classList.add(`type-${cell.type}`);
      cellEl.classList.add(cell.connections.N ? 'open-n' : 'wall-n');
      cellEl.classList.add(cell.connections.E ? 'open-e' : 'wall-e');
      cellEl.classList.add(cell.connections.S ? 'open-s' : 'wall-s');
      cellEl.classList.add(cell.connections.W ? 'open-w' : 'wall-w');

      if (bestPathSet.has(`${r},${c}`)) {
        cellEl.classList.add('on-best-path');
      }

      const topEl = document.createElement('div');
      topEl.className = 'cell-top';
      const idSpan = document.createElement('span');
      idSpan.textContent = `${cell.id}`;
      const coordSpan = document.createElement('span');
      coordSpan.className = 'cell-coord';
      coordSpan.textContent = `(${r},${c})`;
      topEl.append(idSpan, coordSpan);

      const centerEl = document.createElement('div');
      centerEl.className = 'cell-center';

      const bestAction = agent.getBestAction(r, c, true);
      if (cell.type === 3) {
        const icon = document.createElement('span');
        icon.className = 'cell-terminal-icon';
        icon.textContent = 'SAÍDA';
        centerEl.appendChild(icon);
      } else if (cell.type === 2) {
        const icon = document.createElement('span');
        icon.className = 'cell-terminal-icon';
        icon.textContent = 'BURACO';
        centerEl.appendChild(icon);
      } else if (bestAction) {
        const arrow = document.createElement('span');
        arrow.className = 'cell-arrow';
        arrow.textContent = DIRECTION_ARROWS[bestAction];
        centerEl.appendChild(arrow);
      } else {
        const dot = document.createElement('span');
        dot.style.opacity = '0.35';
        dot.textContent = '·';
        centerEl.appendChild(dot);
      }

      const qMaxEl = document.createElement('div');
      qMaxEl.className = 'cell-qmax';
      if (cell.type === 3) {
        qMaxEl.textContent = '+100';
      } else if (cell.type === 2) {
        qMaxEl.textContent = '-100';
      } else {
        const maxQ = agent.getMaxQ(r, c);
        qMaxEl.textContent = maxQ === 0 ? '0.0' : maxQ.toFixed(1);
      }

      cellEl.append(topEl, centerEl, qMaxEl);

      if (agentPosition.row === r && agentPosition.col === c) {
        const token = document.createElement('div');
        token.className = 'agent-token';
        const dot = document.createElement('div');
        dot.className = 'agent-dot';
        dot.textContent = '🤖';
        dot.title = `Agente em (${r}, ${c})`;
        token.appendChild(dot);
        cellEl.appendChild(token);
      }

      fragment.appendChild(cellEl);
    }
  }

  elMazeGrid.replaceChildren(fragment);
}

function renderBestPath(): void {
  const result = agent.getBestPath();
  const episodes = agent.getEpisodesTrained();

  if (episodes === 0) {
    elBestPathBadge.className = 'status-tag status-neutral';
    elBestPathBadge.textContent = 'Aguardando treino';
    elBestPathText.textContent = 'Treine o agente para descobrir a política ótima.';
    elBadgeGoalStatus.className = 'pill pill-status';
    elBadgeGoalStatus.textContent = 'Não treinado';
    return;
  }

  const coordsStr = result.path.map((p) => `(${p.row},${p.col})`).join(' → ');
  const idsStr = result.cellIds.join(' → ');

  if (result.reachedGoal) {
    elBestPathBadge.className = 'status-tag status-success';
    elBestPathBadge.textContent = `Objetivo alcançado (${result.actions.length} passos)`;
    elBestPathText.textContent = `${coordsStr}  [células: ${idsStr}] | Retorno: ${result.totalReward}`;
    elBadgeGoalStatus.className = 'pill pill-status ok';
    elBadgeGoalStatus.textContent = `Caminho ótimo: ${result.actions.length} passos`;
  } else {
    elBestPathBadge.className = 'status-tag status-warn';
    const reasonMap: Record<typeof result.reason, string> = {
      goal: 'Objetivo',
      hole: 'Caiu em buraco',
      loop: 'Ciclo detectado',
      blocked: 'Bloqueado na parede',
      max_steps: 'Limite de passos',
      no_actions: 'Sem ações válidas',
    };
    elBestPathBadge.textContent = reasonMap[result.reason];
    elBestPathText.textContent = `${coordsStr} (política ainda incompleta — treine mais episódios)`;
    elBadgeGoalStatus.className = 'pill pill-status warn';
    elBadgeGoalStatus.textContent = reasonMap[result.reason];
  }
}

function renderQTable(): void {
  const fragment = document.createDocumentFragment();

  for (const cell of env.getAllCells()) {
    const tr = document.createElement('tr');
    const q = agent.getQValues(cell.row, cell.col);
    const bestAction = agent.getBestAction(cell.row, cell.col, true);
    const maxQ = agent.getMaxQ(cell.row, cell.col);

    const tdCell = document.createElement('td');
    tdCell.textContent = `#${cell.id} (${cell.row},${cell.col})`;
    tr.appendChild(tdCell);

    const tdType = document.createElement('td');
    const pill = document.createElement('span');
    pill.className = `type-pill type-pill-${cell.type}`;
    pill.textContent = CELL_TYPE_LABELS[cell.type];
    tdType.appendChild(pill);
    tr.appendChild(tdType);

    for (const dir of ALL_DIRECTIONS) {
      const td = document.createElement('td');
      const isAllowed = cell.connections[dir];
      const val = q[dir];

      if (!isAllowed && agent.getConfig().restrictToValidActions) {
        td.textContent = '×';
        td.style.opacity = '0.35';
      } else {
        td.textContent = val.toFixed(2);
        if (val > 0.005) td.classList.add('q-pos');
        else if (val < -0.005) td.classList.add('q-neg');
      }

      if (bestAction === dir) {
        td.classList.add('q-best-cell');
      }
      tr.appendChild(td);
    }

    const tdBest = document.createElement('td');
    tdBest.textContent =
      cell.type === 2 || cell.type === 3
        ? '—'
        : bestAction
          ? DIRECTION_LABELS[bestAction]
          : '·';
    tr.appendChild(tdBest);

    const tdMax = document.createElement('td');
    tdMax.textContent =
      cell.type === 3 ? '+100' : cell.type === 2 ? '-100' : maxQ.toFixed(2);
    tr.appendChild(tdMax);

    fragment.appendChild(tr);
  }

  elQTableBody.replaceChildren(fragment);
}

function renderMetrics(): void {
  const history = agent.getHistory();
  const total = history.length;
  const eps = agent.getCurrentEpsilon();

  elBadgeEpisodes.textContent = `Episódios: ${total}`;
  elBadgeEpsilon.textContent = `ε: ${eps.toFixed(3)}`;
  elMetricEpisodes.textContent = String(total);

  if (total === 0) {
    elMetricSuccessRate.textContent = '0%';
    elMetricOutcomes.textContent = '0 / 0';
    elMetricLastReward.textContent = '—';
    elRewardChart.replaceChildren();
    return;
  }

  const goals = history.filter((h) => h.outcome === 'goal').length;
  const holes = history.filter((h) => h.outcome === 'hole').length;
  const last = history[history.length - 1];

  elMetricSuccessRate.textContent = `${((goals / total) * 100).toFixed(1)}%`;
  elMetricOutcomes.textContent = `${goals} / ${holes}`;
  elMetricLastReward.textContent = `${last.totalReward.toFixed(0)} (${last.steps}p)`;

  renderRewardChart(history.map((h) => h.totalReward));
}

function renderRewardChart(rewards: number[]): void {
  const svgNS = 'http://www.w3.org/2000/svg';
  elRewardChart.replaceChildren();

  if (rewards.length === 0) return;

  // Amostragem para no máximo 120 pontos no gráfico
  const maxPoints = 120;
  const sampled: number[] = [];
  if (rewards.length <= maxPoints) {
    sampled.push(...rewards);
  } else {
    const stride = rewards.length / maxPoints;
    for (let i = 0; i < maxPoints; i++) {
      sampled.push(rewards[Math.floor(i * stride)]);
    }
  }

  const width = 360;
  const height = 90;
  const pad = 8;

  const minR = Math.min(...sampled, -100);
  const maxR = Math.max(...sampled, 100);
  const range = Math.max(1, maxR - minR);

  const zeroY = pad + ((maxR - 0) / range) * (height - pad * 2);
  const zeroLine = document.createElementNS(svgNS, 'line');
  zeroLine.setAttribute('x1', '0');
  zeroLine.setAttribute('x2', String(width));
  zeroLine.setAttribute('y1', zeroY.toFixed(1));
  zeroLine.setAttribute('y2', zeroY.toFixed(1));
  zeroLine.setAttribute('stroke', '#cbd5e1');
  zeroLine.setAttribute('stroke-dasharray', '3 3');
  elRewardChart.appendChild(zeroLine);

  const points = sampled
    .map((r, idx) => {
      const x =
        sampled.length === 1
          ? width / 2
          : pad + (idx / (sampled.length - 1)) * (width - pad * 2);
      const y = pad + ((maxR - r) / range) * (height - pad * 2);
      return `${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(' ');

  const polyline = document.createElementNS(svgNS, 'polyline');
  polyline.setAttribute('fill', 'none');
  polyline.setAttribute('stroke', '#2563eb');
  polyline.setAttribute('stroke-width', '2');
  polyline.setAttribute('stroke-linejoin', 'round');
  polyline.setAttribute('points', points);
  elRewardChart.appendChild(polyline);
}

function renderAll(): void {
  renderMazeGrid();
  renderBestPath();
  renderQTable();
  renderMetrics();
}

// ============================================================================
// RENDERIZAÇÃO DA ÁRVORE DE ESTADOS / CAMINHOS
// ============================================================================

function renderTree(): void {
  const stats = getTreeStats(pathTree);

  const chips: Array<{ label: string; cls?: string }> = [
    { label: `Nós: ${stats.totalNodes}` },
    { label: `Caminhos: ${stats.totalPaths}` },
    { label: `Objetivos: ${stats.goalPaths}`, cls: 'goal' },
    { label: `Buracos: ${stats.holePaths}`, cls: 'hole' },
    { label: `Becos: ${stats.deadEndPaths}` },
    { label: `Células únicas: ${stats.uniqueCellsVisited}/${env.getAllCells().length}` },
  ];

  elTreeStats.replaceChildren(
    ...chips.map((c) => {
      const span = document.createElement('span');
      span.className = `tree-stat-chip${c.cls ? ` ${c.cls}` : ''}`;
      span.textContent = c.label;
      return span;
    })
  );

  if (stats.truncated) {
    elTreeWarning.classList.remove('hidden');
    elTreeWarning.textContent =
      'Árvore muito densa: exibição limitada aos primeiros 8.000 nós para manter a fluidez.';
  } else {
    elTreeWarning.classList.add('hidden');
  }

  if (treeViewMode === 'list') {
    elTreeListContainer.classList.remove('hidden');
    elTreeSvgContainer.classList.add('hidden');
    renderHierarchicalTree();
  } else {
    elTreeListContainer.classList.add('hidden');
    elTreeSvgContainer.classList.remove('hidden');
    renderSvgTree();
  }
}

function createTreeListNode(
  node: TreeNode,
  onlyGoalBranches: boolean,
  isRoot: boolean
): HTMLLIElement | null {
  if (onlyGoalBranches && !node.leadsToGoal) {
    return null;
  }

  const visibleChildren = node.children.filter(
    (c) => !onlyGoalBranches || c.leadsToGoal
  );
  const hasChildren = visibleChildren.length > 0;

  const li = document.createElement('li');
  li.className = 'tree-li';

  const row = document.createElement('div');
  row.className = 'tree-node-row';
  if (isRoot) row.classList.add('is-start');
  if (node.outcome === 'goal') row.classList.add('is-goal');
  else if (node.outcome === 'hole') row.classList.add('is-hole');
  else if (node.outcome === 'dead_end') row.classList.add('is-dead-end');
  if (node.leadsToGoal) row.classList.add('on-goal-branch');

  let childUl: HTMLUListElement | null = null;
  const initiallyExpanded = node.depth < defaultExpandedDepth;

  if (hasChildren) {
    const toggleBtn = document.createElement('button');
    toggleBtn.type = 'button';
    toggleBtn.className = 'tree-toggle';
    toggleBtn.textContent = initiallyExpanded ? '−' : '+';
    toggleBtn.title = 'Expandir / Recolher ramo';
    toggleBtn.addEventListener('click', () => {
      if (!childUl) return;
      const collapsed = childUl.classList.toggle('hidden');
      toggleBtn.textContent = collapsed ? '+' : '−';
    });
    row.appendChild(toggleBtn);
  }

  const dirBadge = document.createElement('span');
  dirBadge.className = 'tree-dir-badge';
  dirBadge.textContent = node.directionFromParent
    ? DIRECTION_LABELS[node.directionFromParent]
    : 'Início';
  row.appendChild(dirBadge);

  const label = document.createElement('span');
  label.textContent = `#${node.cellId} (${node.row},${node.col})`;
  row.appendChild(label);

  if (node.outcome === 'goal') {
    const tag = document.createElement('span');
    tag.className = 'tree-outcome-tag goal';
    tag.textContent = 'Objetivo (+100)';
    row.appendChild(tag);
  } else if (node.outcome === 'hole') {
    const tag = document.createElement('span');
    tag.className = 'tree-outcome-tag hole';
    tag.textContent = 'Buraco (-100)';
    row.appendChild(tag);
  } else if (node.outcome === 'dead_end') {
    const tag = document.createElement('span');
    tag.className = 'tree-outcome-tag dead';
    tag.textContent = 'Fim do ramo';
    row.appendChild(tag);
  }

  li.appendChild(row);

  if (hasChildren) {
    childUl = document.createElement('ul');
    childUl.className = 'tree-ul';
    if (!initiallyExpanded) {
      childUl.classList.add('hidden');
    }

    for (const child of visibleChildren) {
      const childLi = createTreeListNode(child, onlyGoalBranches, false);
      if (childLi) childUl.appendChild(childLi);
    }

    li.appendChild(childUl);
  }

  return li;
}

function renderHierarchicalTree(): void {
  const onlyGoalBranches = elChkOnlyGoalBranches.checked;
  const rootUl = document.createElement('ul');
  rootUl.className = 'tree-ul tree-root';

  const rootLi = createTreeListNode(pathTree, onlyGoalBranches, true);
  if (rootLi) {
    rootUl.appendChild(rootLi);
    elTreeListContainer.replaceChildren(rootUl);
  } else {
    const empty = document.createElement('p');
    empty.className = 'muted';
    empty.textContent = 'Nenhum caminho encontrado até uma célula objetivo (TC = 3).';
    elTreeListContainer.replaceChildren(empty);
  }
}

interface PositionedTreeNode {
  node: TreeNode;
  x: number;
  y: number;
  children: PositionedTreeNode[];
}

function renderSvgTree(): void {
  const onlyGoal = elChkOnlyGoalBranches.checked;
  const svgNS = 'http://www.w3.org/2000/svg';
  elTreeSvgContainer.replaceChildren();

  // Limite de proteção para diagramas SVG muito gigantes
  const stats = getTreeStats(pathTree);
  if (stats.totalNodes > 450 && !onlyGoal) {
    const msg = document.createElement('p');
    msg.className = 'muted';
    msg.style.padding = '12px';
    msg.textContent = `O diagrama SVG completo possui ${stats.totalNodes} nós. Ative "Filtrar ramos até Objetivo" ou use a aba "Árvore Hierárquica" para explorar com clareza.`;
    elTreeSvgContainer.appendChild(msg);
    return;
  }

  const levelGap = 72;
  const siblingGap = 78;
  let leafIndex = 0;
  let maxDepth = 0;

  function layout(node: TreeNode): PositionedTreeNode | null {
    if (onlyGoal && !node.leadsToGoal) return null;
    if (node.depth > maxDepth) maxDepth = node.depth;

    const visibleChildren = node.children
      .map(layout)
      .filter((c): c is PositionedTreeNode => c !== null);

    let x: number;
    if (visibleChildren.length === 0) {
      x = leafIndex * siblingGap + 48;
      leafIndex++;
    } else {
      const firstX = visibleChildren[0].x;
      const lastX = visibleChildren[visibleChildren.length - 1].x;
      x = (firstX + lastX) / 2;
    }

    return {
      node,
      x,
      y: node.depth * levelGap + 38,
      children: visibleChildren,
    };
  }

  const rootPos = layout(pathTree);
  if (!rootPos) {
    const empty = document.createElement('p');
    empty.className = 'muted';
    empty.style.padding = '12px';
    empty.textContent = 'Nenhum caminho até o objetivo para exibir.';
    elTreeSvgContainer.appendChild(empty);
    return;
  }

  const width = Math.max(360, leafIndex * siblingGap + 40);
  const height = Math.max(180, (maxDepth + 1) * levelGap + 24);

  const svg = document.createElementNS(svgNS, 'svg');
  svg.setAttribute('width', String(width));
  svg.setAttribute('height', String(height));
  svg.setAttribute('viewBox', `0 0 ${width} ${height}`);

  function drawEdges(item: PositionedTreeNode): void {
    for (const child of item.children) {
      const line = document.createElementNS(svgNS, 'line');
      line.setAttribute('x1', String(item.x));
      line.setAttribute('y1', String(item.y + 16));
      line.setAttribute('x2', String(child.x));
      line.setAttribute('y2', String(child.y - 16));
      line.setAttribute(
        'stroke',
        child.node.leadsToGoal
          ? '#16a34a'
          : child.node.outcome === 'hole'
            ? '#dc2626'
            : '#94a3b8'
      );
      line.setAttribute('stroke-width', child.node.leadsToGoal ? '2.5' : '1.5');
      svg.appendChild(line);

      if (child.node.directionFromParent) {
        const midX = (item.x + child.x) / 2;
        const midY = (item.y + child.y) / 2;
        const dirText = document.createElementNS(svgNS, 'text');
        dirText.setAttribute('x', String(midX + 4));
        dirText.setAttribute('y', String(midY));
        dirText.setAttribute('font-size', '10');
        dirText.setAttribute('font-weight', '700');
        dirText.setAttribute('fill', '#475569');
        dirText.textContent = DIRECTION_ARROWS[child.node.directionFromParent];
        svg.appendChild(dirText);
      }

      drawEdges(child);
    }
  }

  function drawNodes(item: PositionedTreeNode, isRoot: boolean): void {
    const g = document.createElementNS(svgNS, 'g');
    const rect = document.createElementNS(svgNS, 'rect');
    rect.setAttribute('x', String(item.x - 28));
    rect.setAttribute('y', String(item.y - 16));
    rect.setAttribute('width', '56');
    rect.setAttribute('height', '32');
    rect.setAttribute('rx', '8');

    let fill = '#ffffff';
    let stroke = '#94a3b8';
    let textFill = '#0f172a';

    if (isRoot) {
      fill = '#3b82f6';
      stroke = '#1d4ed8';
      textFill = '#ffffff';
    } else if (item.node.outcome === 'goal') {
      fill = '#22c55e';
      stroke = '#15803d';
      textFill = '#052e16';
    } else if (item.node.outcome === 'hole') {
      fill = '#ef4444';
      stroke = '#b91c1c';
      textFill = '#ffffff';
    } else if (item.node.leadsToGoal) {
      fill = '#f0fdf4';
      stroke = '#4ade80';
    }

    rect.setAttribute('fill', fill);
    rect.setAttribute('stroke', stroke);
    rect.setAttribute('stroke-width', '2');
    g.appendChild(rect);

    const txtId = document.createElementNS(svgNS, 'text');
    txtId.setAttribute('x', String(item.x));
    txtId.setAttribute('y', String(item.y - 2));
    txtId.setAttribute('text-anchor', 'middle');
    txtId.setAttribute('font-size', '11');
    txtId.setAttribute('font-weight', '700');
    txtId.setAttribute('fill', textFill);
    txtId.textContent = `#${item.node.cellId}`;
    g.appendChild(txtId);

    const txtCoord = document.createElementNS(svgNS, 'text');
    txtCoord.setAttribute('x', String(item.x));
    txtCoord.setAttribute('y', String(item.y + 11));
    txtCoord.setAttribute('text-anchor', 'middle');
    txtCoord.setAttribute('font-size', '9.5');
    txtCoord.setAttribute('fill', textFill);
    txtCoord.textContent = `(${item.node.row},${item.node.col})`;
    g.appendChild(txtCoord);

    svg.appendChild(g);

    for (const child of item.children) {
      drawNodes(child, false);
    }
  }

  drawEdges(rootPos);
  drawNodes(rootPos, true);
  elTreeSvgContainer.appendChild(svg);
}

// ============================================================================
// AÇÕES DE TREINAMENTO E CONTROLES
// ============================================================================

function handleTrainInstant(): void {
  stopAnimations();
  syncHyperparamLabels();
  const hp = readHyperparameters();

  const batch = agent.trainEpisodes(hp.episodes);
  agentPosition = { ...env.startPosition };
  renderAll();

  const goalsInBatch = batch.filter((b) => b.outcome === 'goal').length;
  const best = agent.getBestPath();

  elLiveLog.textContent = best.reachedGoal
    ? `Treino concluído (+${hp.episodes} episódios, ${goalsInBatch} vitórias). Caminho ótimo: ${best.path
        .map((p) => `(${p.row},${p.col})`)
        .join(' → ')}.`
    : `Treino concluído (+${hp.episodes} episódios, ${goalsInBatch} vitórias). Treine mais episódios ou ajuste ε.`;
}

function handleToggleVisualTraining(): void {
  if (visualTimerId !== null) {
    stopAnimations();
    elLiveLog.textContent = 'Treino visual pausado.';
    return;
  }

  stopAnimations();
  syncHyperparamLabels();
  const speedMs = readHyperparameters().speedMs;
  elBtnTrainVisual.textContent = 'Pausar Treino Visual';

  visualTimerId = window.setInterval(() => {
    const step = agent.stepVisualEpisode();
    agentPosition = step.episodeFinished
      ? { ...env.startPosition }
      : { ...step.nextState };

    renderAll();

    const outcomeNote = step.done
      ? step.outcome === 'goal'
        ? 'OBJETIVO (+100)!'
        : 'BURACO (-100)!'
      : `recompensa ${step.reward}`;

    elLiveLog.textContent = `Ep. ${step.episode} | Passo ${step.stepInEpisode}: (${step.state.row},${step.state.col}) ${DIRECTION_ARROWS[step.action]} (${step.nextState.row},${step.nextState.col}) → ${outcomeNote} | ε=${step.epsilon.toFixed(3)}`;
  }, speedMs);
}

function handleRunBestPath(): void {
  if (playbackTimerId !== null) {
    stopAnimations();
    agentPosition = { ...env.startPosition };
    renderMazeGrid();
    return;
  }

  stopAnimations();
  const best = agent.getBestPath();
  if (best.path.length <= 1) {
    elLiveLog.textContent = 'Nenhum movimento aprendido ainda. Treine o agente primeiro.';
    return;
  }

  let idx = 0;
  agentPosition = { ...best.path[0] };
  renderMazeGrid();
  elBtnRunBest.textContent = 'Parar Execução';

  const speedMs = Math.max(120, readHyperparameters().speedMs);
  playbackTimerId = window.setInterval(() => {
    idx++;
    if (idx >= best.path.length) {
      stopAnimations();
      elLiveLog.textContent = best.reachedGoal
        ? `Execução concluída com sucesso em ${best.actions.length} passos! Retorno total: ${best.totalReward}.`
        : `Execução encerrada (${best.reason}) após ${best.actions.length} passos.`;
      return;
    }

    agentPosition = { ...best.path[idx] };
    renderMazeGrid();
    elLiveLog.textContent = `Executando melhor política: passo ${idx}/${best.path.length - 1} → (${agentPosition.row}, ${agentPosition.col})`;
  }, speedMs);
}

function handleReset(): void {
  stopAnimations();
  syncHyperparamLabels();
  agent.reset();
  agentPosition = { ...env.startPosition };
  renderAll();
  elLiveLog.textContent = 'Q-Table resetada para 0.';
}

function applyJsonFromTextarea(): void {
  elJsonError.textContent = '';
  try {
    const parsed: unknown = JSON.parse(elJsonInput.value);
    if (!Array.isArray(parsed)) {
      throw new Error('O JSON deve ser um array de células [[L, C, CN, CL, CS, CO, TC], ...].');
    }
    elPresetSelect.value = 'custom-json';
    loadMaze(
      parsed as RawCellTuple[],
      'Labirinto personalizado carregado via editor JSON.'
    );
  } catch (err) {
    elJsonError.textContent =
      err instanceof Error ? err.message : 'Erro ao interpretar JSON do labirinto.';
  }
}

function bindEvents(): void {
  elPresetSelect.addEventListener('change', () => {
    const val = elPresetSelect.value;
    if (val === 'custom-json') {
      elPresetDesc.textContent =
        'Cole ou edite a matriz [L, C, CN, CL, CS, CO, TC] abaixo e clique em "Aplicar Labirinto JSON".';
      return;
    }
    const preset = MAZE_PRESETS.find((p) => p.id === val);
    if (preset) {
      loadMaze(preset.data, preset.description);
    }
  });

  elBtnFormatJson.addEventListener('click', () => {
    elJsonError.textContent = '';
    try {
      const parsed = JSON.parse(elJsonInput.value) as RawCellTuple[];
      elJsonInput.value = formatTuplesForTextarea(parsed);
    } catch {
      elJsonError.textContent = 'JSON inválido para formatar.';
    }
  });

  elBtnApplyJson.addEventListener('click', applyJsonFromTextarea);

  for (const input of [
    elParamAlpha,
    elParamGamma,
    elParamEpsilon,
    elParamSpeed,
    elParamEpisodes,
    elParamMaxSteps,
    elParamRestrictValid,
  ]) {
    input.addEventListener('input', syncHyperparamLabels);
    input.addEventListener('change', syncHyperparamLabels);
  }

  elBtnTrainInstant.addEventListener('click', handleTrainInstant);
  elBtnTrainVisual.addEventListener('click', handleToggleVisualTraining);
  elBtnRunBest.addEventListener('click', handleRunBestPath);
  elBtnReset.addEventListener('click', handleReset);

  elTabTreeList.addEventListener('click', () => {
    treeViewMode = 'list';
    elTabTreeList.classList.add('active');
    elTabTreeSvg.classList.remove('active');
    renderTree();
  });

  elTabTreeSvg.addEventListener('click', () => {
    treeViewMode = 'svg';
    elTabTreeSvg.classList.add('active');
    elTabTreeList.classList.remove('active');
    renderTree();
  });

  elBtnExpandAll.addEventListener('click', () => {
    defaultExpandedDepth = 999;
    renderTree();
  });

  elBtnCollapseAll.addEventListener('click', () => {
    defaultExpandedDepth = 1;
    renderTree();
  });

  elChkOnlyGoalBranches.addEventListener('change', () => {
    renderTree();
  });
}

// Inicialização
bindEvents();
syncHyperparamLabels();
loadMaze(MAZE_PRESETS[0].data, MAZE_PRESETS[0].description);
