import { ALL_DIRECTIONS, MazeEnvironment } from './environment';
import type {
  BestPathResult,
  Direction,
  EpisodeSummary,
  Position,
  QLearningConfig,
  QValues,
  StepOutcome,
} from './types';

export interface VisualStepResult {
  state: Position;
  action: Direction;
  nextState: Position;
  reward: number;
  done: boolean;
  outcome: StepOutcome;
  episode: number;
  stepInEpisode: number;
  episodeReward: number;
  episodeFinished: boolean;
  epsilon: number;
  tdError: number;
}

export const DEFAULT_QLEARNING_CONFIG: QLearningConfig = {
  alpha: 0.1,
  gamma: 0.9,
  epsilon: 1.0,
  epsilonMin: 0.05,
  epsilonDecay: 0.985,
  maxStepsPerEpisode: 100,
  restrictToValidActions: true,
};

export class QLearningAgent {
  public readonly env: MazeEnvironment;
  private config: QLearningConfig;
  private qTable: Map<string, QValues>;
  private currentEpsilon: number;
  private episodesTrained: number;
  private history: EpisodeSummary[];

  // Estado para treinamento visual passo a passo
  private visualState: Position | null = null;
  private visualStepCount = 0;
  private visualEpisodeReward = 0;

  constructor(env: MazeEnvironment, config: Partial<QLearningConfig> = {}) {
    this.env = env;
    this.config = { ...DEFAULT_QLEARNING_CONFIG, ...config };
    this.qTable = new Map();
    this.currentEpsilon = this.config.epsilon;
    this.episodesTrained = 0;
    this.history = [];
    this.initQTable();
  }

  private stateKey(row: number, col: number): string {
    return `${row},${col}`;
  }

  private initQTable(): void {
    this.qTable.clear();
    for (const cell of this.env.getAllCells()) {
      this.qTable.set(this.stateKey(cell.row, cell.col), {
        N: 0,
        E: 0,
        S: 0,
        W: 0,
      });
    }
  }

  public reset(): void {
    this.initQTable();
    this.currentEpsilon = this.config.epsilon;
    this.episodesTrained = 0;
    this.history = [];
    this.visualState = null;
    this.visualStepCount = 0;
    this.visualEpisodeReward = 0;
  }

  public updateConfig(partial: Partial<QLearningConfig>): void {
    const prevInitialEpsilon = this.config.epsilon;
    this.config = { ...this.config, ...partial };
    if (
      partial.epsilon !== undefined &&
      partial.epsilon !== prevInitialEpsilon &&
      this.episodesTrained === 0
    ) {
      this.currentEpsilon = this.config.epsilon;
    }
    this.currentEpsilon = Math.max(this.config.epsilonMin, this.currentEpsilon);
  }

  public getConfig(): QLearningConfig {
    return { ...this.config };
  }

  public getCurrentEpsilon(): number {
    return this.currentEpsilon;
  }

  public getEpisodesTrained(): number {
    return this.episodesTrained;
  }

  public getHistory(): EpisodeSummary[] {
    return [...this.history];
  }

  public getQValues(row: number, col: number): QValues {
    const key = this.stateKey(row, col);
    const existing = this.qTable.get(key);
    if (existing) {
      return { ...existing };
    }
    const fresh: QValues = { N: 0, E: 0, S: 0, W: 0 };
    this.qTable.set(key, fresh);
    return { ...fresh };
  }

  private getCandidateActions(row: number, col: number): Direction[] {
    if (this.config.restrictToValidActions) {
      const valid = this.env.getValidActions(row, col);
      if (valid.length > 0) return valid;
    }
    return [...ALL_DIRECTIONS];
  }

  public getMaxQ(row: number, col: number): number {
    const candidates = this.getCandidateActions(row, col);
    const q = this.getQValues(row, col);
    let maxVal = -Infinity;
    for (const action of candidates) {
      if (q[action] > maxVal) {
        maxVal = q[action];
      }
    }
    return maxVal === -Infinity ? 0 : maxVal;
  }

  public getBestAction(row: number, col: number, ignoreAllZero = true): Direction | null {
    const cell = this.env.getCell(row, col);
    if (!cell || cell.type === 2 || cell.type === 3) {
      return null;
    }

    const candidates = this.getCandidateActions(row, col);
    if (candidates.length === 0) return null;

    const q = this.getQValues(row, col);
    let maxVal = -Infinity;
    let bestActions: Direction[] = [];

    for (const action of candidates) {
      const value = q[action];
      if (value > maxVal + 1e-12) {
        maxVal = value;
        bestActions = [action];
      } else if (Math.abs(value - maxVal) <= 1e-12) {
        bestActions.push(action);
      }
    }

    if (ignoreAllZero && Math.abs(maxVal) <= 1e-12 && bestActions.length === candidates.length) {
      return null;
    }

    return bestActions[Math.floor(Math.random() * bestActions.length)] ?? null;
  }

  public chooseAction(state: Position, epsilonOverride?: number): Direction {
    const candidates = this.getCandidateActions(state.row, state.col);
    const eps = epsilonOverride ?? this.currentEpsilon;

    if (Math.random() < eps) {
      return candidates[Math.floor(Math.random() * candidates.length)];
    }

    const q = this.getQValues(state.row, state.col);
    let maxVal = -Infinity;
    let bestActions: Direction[] = [];

    for (const action of candidates) {
      const val = q[action];
      if (val > maxVal + 1e-12) {
        maxVal = val;
        bestActions = [action];
      } else if (Math.abs(val - maxVal) <= 1e-12) {
        bestActions.push(action);
      }
    }

    return bestActions[Math.floor(Math.random() * bestActions.length)];
  }

  private updateQ(
    state: Position,
    action: Direction,
    reward: number,
    nextState: Position,
    done: boolean
  ): number {
    const key = this.stateKey(state.row, state.col);
    let qRow = this.qTable.get(key);
    if (!qRow) {
      qRow = { N: 0, E: 0, S: 0, W: 0 };
      this.qTable.set(key, qRow);
    }

    const currentQ = qRow[action];
    const maxNextQ = done ? 0 : this.getMaxQ(nextState.row, nextState.col);
    const target = reward + this.config.gamma * maxNextQ;
    const tdError = target - currentQ;
    qRow[action] = currentQ + this.config.alpha * tdError;
    return tdError;
  }

  private decayEpsilon(): void {
    this.currentEpsilon = Math.max(
      this.config.epsilonMin,
      this.currentEpsilon * this.config.epsilonDecay
    );
  }

  public trainEpisode(): EpisodeSummary {
    let state: Position = { ...this.env.startPosition };
    let totalReward = 0;
    let steps = 0;
    let finalOutcome: EpisodeSummary['outcome'] = 'timeout';

    while (steps < this.config.maxStepsPerEpisode) {
      const action = this.chooseAction(state);
      const result = this.env.step(state, action);
      this.updateQ(state, action, result.reward, result.nextState, result.done);

      totalReward += result.reward;
      steps++;
      state = result.nextState;

      if (result.done) {
        finalOutcome = result.outcome === 'goal' ? 'goal' : 'hole';
        break;
      }
    }

    this.episodesTrained++;
    const summary: EpisodeSummary = {
      episode: this.episodesTrained,
      totalReward,
      steps,
      outcome: finalOutcome,
      epsilon: this.currentEpsilon,
    };
    this.history.push(summary);
    this.decayEpsilon();
    return summary;
  }

  public trainEpisodes(count: number): EpisodeSummary[] {
    // Limpa eventual episódio visual em aberto
    this.visualState = null;
    this.visualStepCount = 0;
    this.visualEpisodeReward = 0;

    const batch: EpisodeSummary[] = [];
    for (let i = 0; i < count; i++) {
      batch.push(this.trainEpisode());
    }
    return batch;
  }

  public getVisualPosition(): Position {
    return this.visualState ? { ...this.visualState } : { ...this.env.startPosition };
  }

  public stepVisualEpisode(): VisualStepResult {
    if (!this.visualState) {
      this.visualState = { ...this.env.startPosition };
      this.visualStepCount = 0;
      this.visualEpisodeReward = 0;
    }

    const state = { ...this.visualState };
    const action = this.chooseAction(state);
    const result = this.env.step(state, action);
    const tdError = this.updateQ(
      state,
      action,
      result.reward,
      result.nextState,
      result.done
    );

    this.visualStepCount++;
    this.visualEpisodeReward += result.reward;
    this.visualState = { ...result.nextState };

    const reachedStepLimit =
      !result.done && this.visualStepCount >= this.config.maxStepsPerEpisode;
    const episodeFinished = result.done || reachedStepLimit;
    const episodeNumber = this.episodesTrained + 1;
    const stepInEpisode = this.visualStepCount;
    const episodeReward = this.visualEpisodeReward;

    if (episodeFinished) {
      this.episodesTrained++;
      const summaryOutcome: EpisodeSummary['outcome'] = result.done
        ? result.outcome === 'goal'
          ? 'goal'
          : 'hole'
        : 'timeout';

      this.history.push({
        episode: this.episodesTrained,
        totalReward: this.visualEpisodeReward,
        steps: this.visualStepCount,
        outcome: summaryOutcome,
        epsilon: this.currentEpsilon,
      });

      this.decayEpsilon();
      this.visualState = null;
      this.visualStepCount = 0;
      this.visualEpisodeReward = 0;
    }

    return {
      state,
      action,
      nextState: result.nextState,
      reward: result.reward,
      done: result.done,
      outcome: result.outcome,
      episode: episodeNumber,
      stepInEpisode,
      episodeReward,
      episodeFinished,
      epsilon: this.currentEpsilon,
      tdError,
    };
  }

  /**
   * Extrai a política ótima aprendida a partir da célula inicial seguindo argmax Q(s, a),
   * detectando ciclos ou bloqueios.
   */
  public getBestPath(): BestPathResult {
    const start = { ...this.env.startPosition };
    const startCell = this.env.getCell(start.row, start.col);
    if (!startCell) {
      return {
        path: [],
        cellIds: [],
        actions: [],
        reachedGoal: false,
        reason: 'no_actions',
        totalReward: 0,
      };
    }

    const path: Position[] = [start];
    const cellIds: number[] = [startCell.id];
    const actions: Direction[] = [];
    const visited = new Set<string>([this.stateKey(start.row, start.col)]);

    let current = start;
    let totalReward = 0;
    const maxSteps = Math.max(this.config.maxStepsPerEpisode, this.env.rows * this.env.cols * 2);

    for (let step = 0; step < maxSteps; step++) {
      const currentCell = this.env.getCell(current.row, current.col);
      if (!currentCell) {
        return {
          path,
          cellIds,
          actions,
          reachedGoal: false,
          reason: 'no_actions',
          totalReward,
        };
      }

      if (currentCell.type === 3) {
        return {
          path,
          cellIds,
          actions,
          reachedGoal: true,
          reason: 'goal',
          totalReward,
        };
      }

      if (currentCell.type === 2) {
        return {
          path,
          cellIds,
          actions,
          reachedGoal: false,
          reason: 'hole',
          totalReward,
        };
      }

      const candidates = this.getCandidateActions(current.row, current.col);
      if (candidates.length === 0) {
        return {
          path,
          cellIds,
          actions,
          reachedGoal: false,
          reason: 'no_actions',
          totalReward,
        };
      }

      // Deterministic argmax (keeps direction order N, E, S, W on exact ties)
      const q = this.getQValues(current.row, current.col);
      let bestAction = candidates[0];
      let bestVal = q[bestAction];
      for (let i = 1; i < candidates.length; i++) {
        const action = candidates[i];
        if (q[action] > bestVal) {
          bestVal = q[action];
          bestAction = action;
        }
      }

      const res = this.env.step(current, bestAction);
      actions.push(bestAction);
      totalReward += res.reward;

      if (res.outcome === 'blocked') {
        return {
          path,
          cellIds,
          actions,
          reachedGoal: false,
          reason: 'blocked',
          totalReward,
        };
      }

      const nextCell = this.env.getCell(res.nextState.row, res.nextState.col);
      const nextKey = this.stateKey(res.nextState.row, res.nextState.col);

      path.push(res.nextState);
      if (nextCell) cellIds.push(nextCell.id);

      if (res.outcome === 'goal') {
        return {
          path,
          cellIds,
          actions,
          reachedGoal: true,
          reason: 'goal',
          totalReward,
        };
      }

      if (res.outcome === 'hole') {
        return {
          path,
          cellIds,
          actions,
          reachedGoal: false,
          reason: 'hole',
          totalReward,
        };
      }

      if (visited.has(nextKey)) {
        return {
          path,
          cellIds,
          actions,
          reachedGoal: false,
          reason: 'loop',
          totalReward,
        };
      }

      visited.add(nextKey);
      current = res.nextState;
    }

    return {
      path,
      cellIds,
      actions,
      reachedGoal: false,
      reason: 'max_steps',
      totalReward,
    };
  }
}

