import type { MazeEnvironment } from './environment';
import type { Direction, TreeNode } from './types';

export interface TreeBuildOptions {
  maxNodes?: number;
}

export interface TreeStats {
  totalNodes: number;
  totalPaths: number;
  goalPaths: number;
  holePaths: number;
  deadEndPaths: number;
  maxDepth: number;
  uniqueCellsVisited: number;
  truncated: boolean;
}

/**
 * Constrói a árvore de todos os caminhos simples possíveis a partir da célula inicial (TC = 1),
 * evitando ciclos no mesmo ramo usando `visitedInCurrentPath`.
 */
export function buildPathTree(
  env: MazeEnvironment,
  options: TreeBuildOptions = {}
): TreeNode {
  const maxNodes = options.maxNodes ?? 8000;
  const startCell = env.getCell(env.startPosition.row, env.startPosition.col);

  if (!startCell) {
    throw new Error('Célula inicial não encontrada ao construir árvore de caminhos.');
  }

  let nodeCount = 0;
  const visitedInCurrentPath = new Set<number>();

  function expand(
    row: number,
    col: number,
    directionFromParent: Direction | null,
    depth: number
  ): TreeNode {
    const cell = env.getCell(row, col)!;
    nodeCount++;
    visitedInCurrentPath.add(cell.id);

    const isGoal = cell.type === 3;
    const isHole = cell.type === 2;

    const children: TreeNode[] = [];

    if (!isGoal && !isHole && nodeCount < maxNodes) {
      const validDirections = env.getValidActions(row, col);
      for (const dir of validDirections) {
        if (nodeCount >= maxNodes) break;
        const nextPos = env.getNeighborPosition(row, col, dir);
        const neighbor = env.getCell(nextPos.row, nextPos.col);
        if (!neighbor || visitedInCurrentPath.has(neighbor.id)) {
          continue;
        }
        children.push(expand(neighbor.row, neighbor.col, dir, depth + 1));
      }
    }

    visitedInCurrentPath.delete(cell.id);

    let outcome: TreeNode['outcome'];
    if (isGoal) {
      outcome = 'goal';
    } else if (isHole) {
      outcome = 'hole';
    } else if (children.length === 0) {
      outcome = 'dead_end';
    } else {
      outcome = 'branch';
    }

    const leadsToGoal = isGoal || children.some((child) => child.leadsToGoal);
    const leadsToHole = isHole || children.some((child) => child.leadsToHole);

    return {
      cellId: cell.id,
      row: cell.row,
      col: cell.col,
      type: cell.type,
      directionFromParent,
      children,
      depth,
      outcome,
      leadsToGoal,
      leadsToHole,
    };
  }

  return expand(startCell.row, startCell.col, null, 0);
}

export function getTreeStats(root: TreeNode, maxNodes = 8000): TreeStats {
  let totalNodes = 0;
  let totalPaths = 0;
  let goalPaths = 0;
  let holePaths = 0;
  let deadEndPaths = 0;
  let maxDepth = 0;
  const uniqueCells = new Set<number>();

  function walk(node: TreeNode): void {
    totalNodes++;
    uniqueCells.add(node.cellId);
    if (node.depth > maxDepth) {
      maxDepth = node.depth;
    }

    if (node.children.length === 0) {
      totalPaths++;
      if (node.outcome === 'goal') goalPaths++;
      else if (node.outcome === 'hole') holePaths++;
      else deadEndPaths++;
      return;
    }

    for (const child of node.children) {
      walk(child);
    }
  }

  walk(root);

  return {
    totalNodes,
    totalPaths,
    goalPaths,
    holePaths,
    deadEndPaths,
    maxDepth,
    uniqueCellsVisited: uniqueCells.size,
    truncated: totalNodes >= maxNodes,
  };
}

