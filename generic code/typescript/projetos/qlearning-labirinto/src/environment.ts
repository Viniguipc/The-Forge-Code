import type {
  Cell,
  CellType,
  Direction,
  Position,
  RawCellTuple,
  StepResult,
  ValidationIssue,
} from './types';

export const ALL_DIRECTIONS: readonly Direction[] = ['N', 'E', 'S', 'W'];

export const DIRECTION_DELTAS: Record<Direction, Position> = {
  N: { row: -1, col: 0 },
  E: { row: 0, col: 1 },
  S: { row: 1, col: 0 },
  W: { row: 0, col: -1 },
};

export const OPPOSITE_DIRECTION: Record<Direction, Direction> = {
  N: 'S',
  E: 'W',
  S: 'N',
  W: 'E',
};

export const DIRECTION_ARROWS: Record<Direction, string> = {
  N: '↑',
  E: '→',
  S: '↓',
  W: '←',
};

export class MazeEnvironment {
  public readonly rows: number;
  public readonly cols: number;
  public readonly startPosition: Position;
  public readonly goalPositions: Position[];
  public readonly holePositions: Position[];
  public readonly validationIssues: ValidationIssue[];

  private readonly grid: (Cell | null)[][];
  private readonly cells: Cell[];

  constructor(rawCells: RawCellTuple[] | number[][]) {
    if (!Array.isArray(rawCells) || rawCells.length === 0) {
      throw new Error('A lista de células do labirinto não pode estar vazia.');
    }

    let maxRow = 0;
    let maxCol = 0;

    for (let i = 0; i < rawCells.length; i++) {
      const tuple = rawCells[i];
      if (!Array.isArray(tuple) || tuple.length !== 7) {
        throw new Error(
          `Célula no índice ${i} inválida: esperado vetor [L, C, CN, CL, CS, CO, TC] com 7 números.`
        );
      }
      const [row, col, cn, cl, cs, co, tc] = tuple;
      if (!Number.isInteger(row) || row < 0 || !Number.isInteger(col) || col < 0) {
        throw new Error(
          `Coordenadas inválidas na célula ${i}: L=${row}, C=${col}. Devem ser inteiros >= 0.`
        );
      }
      for (const flag of [cn, cl, cs, co]) {
        if (flag !== 0 && flag !== 1) {
          throw new Error(
            `Conectividade inválida em (${row}, ${col}): valores devem ser 0 ou 1.`
          );
        }
      }
      if (tc !== 0 && tc !== 1 && tc !== 2 && tc !== 3) {
        throw new Error(
          `Tipo de célula inválido em (${row}, ${col}): TC=${tc}. Valores permitidos: 0, 1, 2 ou 3.`
        );
      }

      if (row > maxRow) maxRow = row;
      if (col > maxCol) maxCol = col;
    }

    this.rows = maxRow + 1;
    this.cols = maxCol + 1;
    this.grid = Array.from({ length: this.rows }, () =>
      Array.from({ length: this.cols }, () => null)
    );
    this.cells = [];
    this.goalPositions = [];
    this.holePositions = [];
    this.validationIssues = [];

    let foundStart: Position | null = null;
    let startCount = 0;

    for (const tuple of rawCells) {
      const [row, col, cn, cl, cs, co, tc] = tuple as RawCellTuple;
      const cellType = tc as CellType;
      const id = row * this.cols + col;

      if (this.grid[row][col] !== null) {
        this.validationIssues.push({
          severity: 'warning',
          row,
          col,
          message: `Célula duplicada em (${row}, ${col}); a última definição foi mantida.`,
        });
      }

      const cell: Cell = {
        row,
        col,
        id,
        type: cellType,
        connections: {
          N: cn === 1,
          E: cl === 1,
          S: cs === 1,
          W: co === 1,
        },
      };

      this.grid[row][col] = cell;

      if (cellType === 1) {
        startCount++;
        if (!foundStart) {
          foundStart = { row, col };
        }
      } else if (cellType === 2) {
        this.holePositions.push({ row, col });
      } else if (cellType === 3) {
        this.goalPositions.push({ row, col });
      }
    }

    for (let r = 0; r < this.rows; r++) {
      for (let c = 0; c < this.cols; c++) {
        const cell = this.grid[r][c];
        if (cell) {
          this.cells.push(cell);
        } else {
          this.validationIssues.push({
            severity: 'warning',
            row: r,
            col: c,
            message: `Célula (${r}, ${c}) não foi informada na lista e será tratada como bloqueada.`,
          });
        }
      }
    }

    if (!foundStart) {
      throw new Error(
        'Nenhuma célula inicial (TC = 1) encontrada no labirinto.'
      );
    }
    if (startCount > 1) {
      this.validationIssues.push({
        severity: 'warning',
        message: `Foram encontradas ${startCount} células iniciais (TC = 1). Usando (${foundStart.row}, ${foundStart.col}) como origem.`,
      });
    }
    if (this.goalPositions.length === 0) {
      this.validationIssues.push({
        severity: 'warning',
        message: 'Nenhuma célula objetivo (TC = 3) definida no labirinto.',
      });
    }

    this.startPosition = foundStart;
    this.validateConsistency();
  }

  private validateConsistency(): void {
    for (const cell of this.cells) {
      const { row, col, connections } = cell;

      for (const dir of ALL_DIRECTIONS) {
        if (!connections[dir]) continue;

        const neighborPos = this.getNeighborPosition(row, col, dir);
        if (!this.isInsideBounds(neighborPos.row, neighborPos.col)) {
          this.validationIssues.push({
            severity: 'error',
            row,
            col,
            message: `Célula (${row}, ${col}) possui saída para ${dir} fora dos limites do grid (${this.rows}×${this.cols}).`,
          });
          continue;
        }

        const neighbor = this.getCell(neighborPos.row, neighborPos.col);
        if (!neighbor) {
          this.validationIssues.push({
            severity: 'error',
            row,
            col,
            message: `Célula (${row}, ${col}) conecta ao ${dir} com (${neighborPos.row}, ${neighborPos.col}), que não existe.`,
          });
          continue;
        }

        const opposite = OPPOSITE_DIRECTION[dir];
        if (!neighbor.connections[opposite]) {
          this.validationIssues.push({
            severity: 'warning',
            row,
            col,
            message: `Inconsistência de parede: (${row}, ${col}) conecta ao ${dir}, mas (${neighbor.row}, ${neighbor.col}) não conecta ao ${opposite}.`,
          });
        }
      }
    }
  }

  public validate(): ValidationIssue[] {
    return [...this.validationIssues];
  }

  public isInsideBounds(row: number, col: number): boolean {
    return row >= 0 && row < this.rows && col >= 0 && col < this.cols;
  }

  public getCell(row: number, col: number): Cell | null {
    if (!this.isInsideBounds(row, col)) return null;
    return this.grid[row][col];
  }

  public getCellById(id: number): Cell | null {
    const row = Math.floor(id / this.cols);
    const col = id % this.cols;
    return this.getCell(row, col);
  }

  public getAllCells(): Cell[] {
    return [...this.cells];
  }

  public getNeighborPosition(row: number, col: number, action: Direction): Position {
    const delta = DIRECTION_DELTAS[action];
    return {
      row: row + delta.row,
      col: col + delta.col,
    };
  }

  public getValidActions(row: number, col: number): Direction[] {
    const cell = this.getCell(row, col);
    if (!cell) return [];

    const valid: Direction[] = [];
    for (const dir of ALL_DIRECTIONS) {
      if (!cell.connections[dir]) continue;
      const next = this.getNeighborPosition(row, col, dir);
      if (this.getCell(next.row, next.col)) {
        valid.push(dir);
      }
    }
    return valid;
  }

  public isTerminal(row: number, col: number): boolean {
    const cell = this.getCell(row, col);
    return cell !== null && (cell.type === 2 || cell.type === 3);
  }

  public step(state: Position, action: Direction): StepResult {
    const current = this.getCell(state.row, state.col);
    if (!current || !current.connections[action]) {
      return {
        nextState: { row: state.row, col: state.col },
        reward: -10,
        done: false,
        outcome: 'blocked',
      };
    }

    const nextPos = this.getNeighborPosition(state.row, state.col, action);
    const nextCell = this.getCell(nextPos.row, nextPos.col);
    if (!nextCell) {
      return {
        nextState: { row: state.row, col: state.col },
        reward: -10,
        done: false,
        outcome: 'blocked',
      };
    }

    if (nextCell.type === 3) {
      return {
        nextState: nextPos,
        reward: 100,
        done: true,
        outcome: 'goal',
      };
    }

    if (nextCell.type === 2) {
      return {
        nextState: nextPos,
        reward: -100,
        done: true,
        outcome: 'hole',
      };
    }

    return {
      nextState: nextPos,
      reward: -1,
      done: false,
      outcome: 'step',
    };
  }
}

