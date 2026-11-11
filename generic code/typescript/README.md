# TypeScript — The-Forge-Code

Área dedicada a estudos e projetos em **TypeScript**.

## Estrutura

- `estudos/` — Exercícios, testes rápidos e algoritmos avulsos em `.ts`.
- `projetos/` — Aplicações completas com `package.json` próprio.
  - `projetos/qlearning-labirinto/` — Agente de Aprendizado por Reforço (Q-Learning) + árvore de caminhos em labirinto (`Vite + TypeScript`).
- `tsconfig.base.json` — Configuração estrita base reutilizável em novos projetos (`"extends": "../../tsconfig.base.json"`).

## Como rodar um projeto Vite + TS aqui (PowerShell)

```powershell
cd ".\projetos\qlearning-labirinto"
npm.cmd install
npm.cmd run dev
```

Para checar tipos e gerar build de produção:

```powershell
npm.cmd run build
```
