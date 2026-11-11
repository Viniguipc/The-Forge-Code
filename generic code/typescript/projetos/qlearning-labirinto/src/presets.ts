import type { RawCellTuple } from './types';

/**
 * Preset 1: Labirinto 3x3 do Enunciado (Prof. Andre Borin Soares)
 * Formato: [L, C, CN, CL, CS, CO, TC]
 */
export const MAZE_3X3_PROFESSOR: RawCellTuple[] = [
  [0, 0, 0, 1, 0, 0, 1], // (0,0) Início (azul)
  [0, 1, 0, 1, 1, 1, 0],
  [0, 2, 0, 0, 0, 1, 0],
  [1, 0, 0, 1, 1, 0, 0],
  [1, 1, 1, 1, 1, 1, 0],
  [1, 2, 0, 0, 0, 1, 3], // (1,2) Objetivo (verde)
  [2, 0, 1, 0, 0, 0, 0],
  [2, 1, 1, 1, 0, 0, 0],
  [2, 2, 0, 0, 0, 1, 2], // (2,2) Buraco (vermelho)
];

/**
 * Preset 2: Labirinto 5x5 Customizado
 * - 1 célula inicial (0,0)
 * - 1 célula objetivo (4,4)
 * - 4 células de buraco: (1,1), (2,3), (3,1), (4,2)
 * - Múltiplos caminhos possíveis (8, 10 e 12 passos) demonstrando a escolha do menor caminho.
 */
export const MAZE_5X5_CUSTOM: RawCellTuple[] = [
  // Linha 0
  [0, 0, 0, 1, 1, 0, 1], // (0,0) Início
  [0, 1, 0, 0, 1, 1, 0],
  [0, 2, 0, 1, 1, 0, 0],
  [0, 3, 0, 1, 0, 1, 0],
  [0, 4, 0, 0, 1, 1, 0],

  // Linha 1
  [1, 0, 1, 0, 1, 0, 0],
  [1, 1, 1, 0, 0, 0, 2], // (1,1) Buraco
  [1, 2, 1, 1, 1, 0, 0],
  [1, 3, 0, 1, 1, 1, 0],
  [1, 4, 1, 0, 1, 1, 0],

  // Linha 2
  [2, 0, 1, 1, 1, 0, 0],
  [2, 1, 0, 1, 0, 1, 0],
  [2, 2, 1, 0, 1, 1, 0],
  [2, 3, 1, 0, 0, 0, 2], // (2,3) Buraco
  [2, 4, 1, 0, 1, 0, 0],

  // Linha 3
  [3, 0, 1, 0, 1, 0, 0],
  [3, 1, 0, 0, 1, 0, 2], // (3,1) Buraco
  [3, 2, 1, 1, 1, 0, 0],
  [3, 3, 0, 1, 1, 1, 0],
  [3, 4, 1, 0, 1, 1, 0],

  // Linha 4
  [4, 0, 1, 1, 0, 0, 0],
  [4, 1, 1, 0, 0, 1, 0],
  [4, 2, 1, 0, 0, 0, 2], // (4,2) Buraco
  [4, 3, 1, 1, 0, 0, 0],
  [4, 4, 1, 0, 0, 1, 3], // (4,4) Objetivo
];

export interface MazePreset {
  id: string;
  name: string;
  description: string;
  data: RawCellTuple[];
}

/**
 * Preset 3: Labirinto 4x4 (Origem Inferior + Armadilhas)
 * - Início em (3,0), Objetivo em (0,3)
 * - Buracos em (1,2), (2,1), (3,2)
 */
export const MAZE_4X4_TRAPS: RawCellTuple[] = [
  // Linha 0
  [0, 0, 0, 1, 1, 0, 0],
  [0, 1, 0, 1, 1, 1, 0],
  [0, 2, 0, 1, 0, 1, 0],
  [0, 3, 0, 0, 1, 1, 3], // Objetivo

  // Linha 1
  [1, 0, 1, 1, 1, 0, 0],
  [1, 1, 1, 1, 0, 1, 0],
  [1, 2, 0, 1, 0, 1, 2], // Buraco
  [1, 3, 1, 0, 1, 1, 0],

  // Linha 2
  [2, 0, 1, 1, 1, 0, 0],
  [2, 1, 0, 1, 1, 1, 2], // Buraco
  [2, 2, 0, 1, 1, 1, 0],
  [2, 3, 1, 0, 1, 1, 0],

  // Linha 3
  [3, 0, 1, 1, 0, 0, 1], // Início
  [3, 1, 1, 1, 0, 1, 0],
  [3, 2, 1, 1, 0, 1, 2], // Buraco
  [3, 3, 1, 0, 0, 1, 0],
];

/**
 * Preset 4: Labirinto Retangular 4x6
 * - Início em (0,0), Objetivo em (3,5)
 * - Buracos em (0,3), (1,1), (2,4), (3,2)
 */
export const MAZE_4X6_RECT: RawCellTuple[] = [
  // Linha 0
  [0, 0, 0, 1, 1, 0, 1], // Início
  [0, 1, 0, 1, 1, 1, 0],
  [0, 2, 0, 1, 1, 1, 0],
  [0, 3, 0, 0, 1, 1, 2], // Buraco
  [0, 4, 0, 1, 1, 0, 0],
  [0, 5, 0, 0, 1, 1, 0],

  // Linha 1
  [1, 0, 1, 0, 1, 0, 0],
  [1, 1, 1, 1, 0, 0, 2], // Buraco
  [1, 2, 1, 1, 1, 1, 0],
  [1, 3, 1, 1, 1, 1, 0],
  [1, 4, 1, 1, 0, 1, 0],
  [1, 5, 1, 0, 1, 1, 0],

  // Linha 2
  [2, 0, 1, 1, 1, 0, 0],
  [2, 1, 0, 1, 1, 1, 0],
  [2, 2, 1, 1, 0, 1, 0],
  [2, 3, 1, 1, 1, 1, 0],
  [2, 4, 0, 1, 1, 1, 2], // Buraco
  [2, 5, 1, 0, 1, 1, 0],

  // Linha 3
  [3, 0, 1, 1, 0, 0, 0],
  [3, 1, 1, 1, 0, 1, 0],
  [3, 2, 0, 1, 0, 1, 2], // Buraco
  [3, 3, 1, 1, 0, 1, 0],
  [3, 4, 1, 1, 0, 1, 0],
  [3, 5, 1, 0, 0, 1, 3], // Objetivo
];

/**
 * Preset 5: Labirinto 5x5 com Dois Objetivos (Perto vs. Longe)
 * - Início em (2,0)
 * - Objetivo próximo em (0,3) (5 passos) e objetivo distante em (4,4) (8 passos)
 * - Buracos em (1,1), (2,2), (3,3)
 */
export const MAZE_5X5_DUAL_GOAL: RawCellTuple[] = [
  // Linha 0
  [0, 0, 0, 1, 1, 0, 0],
  [0, 1, 0, 1, 1, 1, 0],
  [0, 2, 0, 1, 0, 1, 0],
  [0, 3, 0, 1, 1, 1, 3], // Objetivo 1 (mais próximo: 5 passos)
  [0, 4, 0, 0, 1, 1, 0],

  // Linha 1
  [1, 0, 1, 1, 1, 0, 0],
  [1, 1, 1, 0, 1, 1, 2], // Buraco
  [1, 2, 0, 1, 1, 0, 0],
  [1, 3, 1, 0, 0, 1, 0],
  [1, 4, 1, 0, 1, 0, 0],

  // Linha 2
  [2, 0, 1, 1, 1, 0, 1], // Início
  [2, 1, 1, 1, 1, 1, 0],
  [2, 2, 1, 1, 1, 1, 2], // Buraco
  [2, 3, 0, 1, 1, 1, 0],
  [2, 4, 1, 0, 1, 1, 0],

  // Linha 3
  [3, 0, 1, 0, 1, 0, 0],
  [3, 1, 1, 1, 1, 0, 0],
  [3, 2, 1, 1, 1, 1, 0],
  [3, 3, 1, 1, 1, 1, 2], // Buraco
  [3, 4, 1, 0, 1, 1, 0],

  // Linha 4
  [4, 0, 1, 1, 0, 0, 0],
  [4, 1, 1, 0, 0, 1, 0],
  [4, 2, 1, 1, 0, 0, 0],
  [4, 3, 1, 1, 0, 1, 0],
  [4, 4, 1, 0, 0, 1, 3], // Objetivo 2 (mais distante: 8 passos)
];

/**
 * Preset 6: Labirinto Clássico 6x6 (Corredores, Becos e Atalhos)
 * - Início em (0,0), Objetivo em (5,5)
 * - Buracos em (1,3), (2,1), (3,4), (4,2), (5,0)
 */
export const MAZE_6X6_CLASSIC: RawCellTuple[] = [
  // Linha 0
  [0, 0, 0, 1, 1, 0, 1], // Início
  [0, 1, 0, 1, 0, 1, 0],
  [0, 2, 0, 1, 1, 1, 0],
  [0, 3, 0, 0, 1, 1, 0],
  [0, 4, 0, 1, 1, 0, 0],
  [0, 5, 0, 0, 1, 1, 0],

  // Linha 1
  [1, 0, 1, 1, 1, 0, 0],
  [1, 1, 0, 0, 1, 1, 0],
  [1, 2, 1, 1, 0, 0, 0],
  [1, 3, 1, 0, 0, 1, 2], // Buraco
  [1, 4, 1, 1, 1, 0, 0],
  [1, 5, 1, 0, 1, 1, 0],

  // Linha 2
  [2, 0, 1, 1, 1, 0, 0],
  [2, 1, 1, 1, 0, 1, 2], // Buraco
  [2, 2, 0, 1, 1, 1, 0],
  [2, 3, 0, 1, 0, 1, 0],
  [2, 4, 1, 0, 1, 1, 0],
  [2, 5, 1, 0, 1, 0, 0],

  // Linha 3
  [3, 0, 1, 0, 1, 0, 0],
  [3, 1, 0, 1, 1, 0, 0],
  [3, 2, 1, 1, 0, 1, 0],
  [3, 3, 0, 1, 1, 1, 0],
  [3, 4, 1, 1, 1, 1, 2], // Buraco
  [3, 5, 1, 0, 1, 1, 0],

  // Linha 4
  [4, 0, 1, 1, 1, 0, 0],
  [4, 1, 1, 1, 0, 1, 0],
  [4, 2, 0, 1, 1, 1, 2], // Buraco
  [4, 3, 1, 1, 0, 1, 0],
  [4, 4, 1, 1, 1, 1, 0],
  [4, 5, 1, 0, 1, 1, 0],

  // Linha 5
  [5, 0, 1, 1, 0, 0, 2], // Buraco
  [5, 1, 0, 1, 0, 1, 0],
  [5, 2, 1, 1, 0, 1, 0],
  [5, 3, 0, 1, 0, 1, 0],
  [5, 4, 1, 1, 0, 1, 0],
  [5, 5, 1, 0, 0, 1, 3], // Objetivo
];

/**
 * Preset 7: Labirinto 4x4 Diamante Simétrico (4 Rotas Empatadas de 6 Passos)
 * - Início em (0,0), Objetivo em (3,3)
 * - Buracos em (1,1) e (2,2)
 */
export const MAZE_4X4_DIAMOND: RawCellTuple[] = [
  // Linha 0
  [0, 0, 0, 1, 1, 0, 1], // Início
  [0, 1, 0, 1, 1, 1, 0],
  [0, 2, 0, 1, 1, 1, 0],
  [0, 3, 0, 0, 1, 1, 0],

  // Linha 1
  [1, 0, 1, 1, 1, 0, 0],
  [1, 1, 1, 0, 0, 1, 2], // Buraco
  [1, 2, 1, 1, 0, 0, 0],
  [1, 3, 1, 0, 1, 1, 0],

  // Linha 2
  [2, 0, 1, 1, 1, 0, 0],
  [2, 1, 0, 0, 1, 1, 0],
  [2, 2, 0, 1, 1, 0, 2], // Buraco
  [2, 3, 1, 0, 1, 1, 0],

  // Linha 3
  [3, 0, 1, 1, 0, 0, 0],
  [3, 1, 1, 1, 0, 1, 0],
  [3, 2, 1, 1, 0, 1, 0],
  [3, 3, 1, 0, 0, 1, 3], // Objetivo
];

/**
 * Preset 8: Labirinto 5x5 Rota Norte vs. Rota Sul (Simétrico, 8 Passos)
 * - Início em (2,0), Objetivo em (2,4)
 * - Buracos centrais em (1,2), (2,2), (3,2)
 */
export const MAZE_5X5_NORTH_SOUTH: RawCellTuple[] = [
  // Linha 0
  [0, 0, 0, 1, 1, 0, 0],
  [0, 1, 0, 1, 1, 1, 0],
  [0, 2, 0, 1, 1, 1, 0],
  [0, 3, 0, 1, 1, 1, 0],
  [0, 4, 0, 0, 1, 1, 0],

  // Linha 1
  [1, 0, 1, 1, 1, 0, 0],
  [1, 1, 1, 1, 0, 1, 0],
  [1, 2, 1, 1, 0, 1, 2], // Buraco
  [1, 3, 1, 1, 0, 1, 0],
  [1, 4, 1, 0, 1, 1, 0],

  // Linha 2
  [2, 0, 1, 0, 1, 0, 1], // Início (sobe para Norte ou desce para Sul)
  [2, 1, 0, 1, 0, 0, 0],
  [2, 2, 0, 1, 0, 1, 2], // Buraco
  [2, 3, 0, 0, 0, 1, 0],
  [2, 4, 1, 0, 1, 0, 3], // Objetivo

  // Linha 3
  [3, 0, 1, 1, 1, 0, 0],
  [3, 1, 0, 1, 1, 1, 0],
  [3, 2, 0, 1, 1, 1, 2], // Buraco
  [3, 3, 0, 1, 1, 1, 0],
  [3, 4, 1, 0, 1, 1, 0],

  // Linha 4
  [4, 0, 1, 1, 0, 0, 0],
  [4, 1, 1, 1, 0, 1, 0],
  [4, 2, 1, 1, 0, 1, 0],
  [4, 3, 1, 1, 0, 1, 0],
  [4, 4, 1, 0, 0, 1, 0],
];

export const MAZE_PRESETS: MazePreset[] = [
  {
    id: '3x3-professor',
    name: '3×3 do Professor (Padrão)',
    description: 'Labirinto clássico 3×3 do enunciado: início em (0,0), objetivo em (1,2) e buraco em (2,2).',
    data: MAZE_3X3_PROFESSOR,
  },
  {
    id: '4x4-traps',
    name: '4×4 Origem Inferior & Armadilhas',
    description: 'Grid 4×4 começando em (3,0) até (0,3), com 3 buracos estratégicos e rotas alternativas.',
    data: MAZE_4X4_TRAPS,
  },
  {
    id: '4x4-diamond',
    name: '4×4 Simétrico (4 Rotas Empatadas)',
    description: 'Grid 4×4 de (0,0) até (3,3) com 4 caminhos ótimos de 6 passos — ótimo para ver a IA alternar rotas ao resetar.',
    data: MAZE_4X4_DIAMOND,
  },
  {
    id: '4x6-rect',
    name: '4×6 Retangular (Assimétrico)',
    description: 'Grid não quadrado (4 linhas × 6 colunas) demonstrando adaptação automática de dimensões.',
    data: MAZE_4X6_RECT,
  },
  {
    id: '5x5-custom',
    name: '5×5 Customizado (Múltiplos Caminhos)',
    description: 'Grid 5×5 com 4 buracos, armadilhas e múltiplos caminhos (8, 10 e 12 passos) até (4,4).',
    data: MAZE_5X5_CUSTOM,
  },
  {
    id: '5x5-dual-goal',
    name: '5×5 Dois Objetivos (Perto vs. Longe)',
    description: 'Grid 5×5 com dois objetivos (TC=3): mostra a IA preferindo a saída de menor custo.',
    data: MAZE_5X5_DUAL_GOAL,
  },
  {
    id: '5x5-north-south',
    name: '5×5 Rota Norte vs. Sul (Empate)',
    description: 'Grid 5×5 espelhado de (2,0) até (2,4): a IA pode convergir tanto pela metade superior quanto pela inferior.',
    data: MAZE_5X5_NORTH_SOUTH,
  },
  {
    id: '6x6-classic',
    name: '6×6 Labirinto Clássico (36 células)',
    description: 'Labirinto 6×6 com 5 buracos, becos sem saída e múltiplos caminhos até (5,5).',
    data: MAZE_6X6_CLASSIC,
  },
];

