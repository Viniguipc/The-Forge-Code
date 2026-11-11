export type Direction = 'N' | 'E' | 'S' | 'W';

export type CellType = 0 | 1 | 2 | 3;

export type RawCellTuple = [
  number, // L: row
  number, // C: col
  number, // CN: North (0 | 1)
  number, // CL: East (0 | 1)
  number, // CS: South (0 | 1)
  number, // CO: West (0 | 1)
  number  // TC: CellType (0 | 1 | 2 | 3)
];

export interface Position {
  row: number;
  col: number;
}

export interface Cell {
  row: number;
  col: number;
  connections: Record<Direction, boolean>;
  type: CellType;
  id: number;
}

export interface TreeNode {
  cellId: number;
  row: number;
  col: number;
  type: CellType;
  directionFromParent: Direction | null;
  children: TreeNode[];
  depth: number;
  outcome: 'goal' | 'hole' | 'dead_end' | 'branch';
  leadsToGoal: boolean;
  leadsToHole: boolean;
}

export type StepOutcome = 'goal' | 'hole' | 'step' | 'blocked';

export interface StepResult {
  nextState: Position;
  reward: number;
  done: boolean;
  outcome: StepOutcome;
}

export interface ValidationIssue {
  severity: 'error' | 'warning';
  message: string;
  row?: number;
  col?: number;
}

export type QValues = Record<Direction, number>;

export interface QLearningConfig {
  alpha: number;
  gamma: number;
  epsilon: number;
  epsilonMin: number;
  epsilonDecay: number;
  maxStepsPerEpisode: number;
  restrictToValidActions: boolean;
}

export interface EpisodeSummary {
  episode: number;
  totalReward: number;
  steps: number;
  outcome: 'goal' | 'hole' | 'timeout';
  epsilon: number;
}

export interface BestPathResult {
  path: Position[];
  cellIds: number[];
  actions: Direction[];
  reachedGoal: boolean;
  reason: 'goal' | 'hole' | 'loop' | 'blocked' | 'max_steps' | 'no_actions';
  totalReward: number;
}

