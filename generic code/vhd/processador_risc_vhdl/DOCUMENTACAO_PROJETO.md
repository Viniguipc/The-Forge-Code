# Documentação do Projeto: Processador RISC Multiciclo em VHDL

**Disciplina:** Organização de Computadores – 2026/2  
**Curso:** Engenharia de Computação  
**Autores:** Arthur Ferreira Borba e Vinícius Guilherme Pereira Cruz  
**Professora:** Débora da Silva Motta Matos  

---

## a. Instruções Escolhidas

Foram definidas **10 instruções de 32 bits** (8 instruções simples + 2 instruções complexas para trabalho em dupla):

| Código (Opcode) | Binário `[31:26]` | Mnemônico | Exemplo | Tipo | Descrição |
|---|---|---|---|---|---|
| 0 | `000000` | `lw` | `lw r1, 4(r4)` | Tipo-I | Lê palavra da memória em `Reg[r4] + sext(offset)` e grava em `r1` |
| 1 | `000001` | `sw` | `sw r1, 4(r4)` | Tipo-I | Grava `Reg[r1]` na memória no endereço `Reg[r4] + sext(offset)` |
| 2 | `000010` | `add` | `add r3, r1, r2` | Tipo-R | `Reg[r3] <- Reg[r1] + Reg[r2]` |
| 3 | `000011` | `sub` | `sub r3, r1, r2` | Tipo-R | `Reg[r3] <- Reg[r1] - Reg[r2]` |
| 4 | `000100` | `and` | `and r3, r1, r2` | Tipo-R | `Reg[r3] <- Reg[r1] AND Reg[r2]` (bit a bit) |
| 5 | `000101` | `addi` | `addi r1, 10` | Tipo-I | `Reg[r1] <- Reg[r1] + sext(imm16)` |
| 6 | `000110` | `jump` | `jump 14` | Tipo-J | `PC <- PC[31:28] & target26 & "00"` |
| 7 | `000111` | `beq` | `beq r1, r3, 1` | Tipo-I | Se `Reg[r1] = Reg[r3]`, `PC <- PC + (sext(offset) << 2)` |
| 8 | `001000` | `addl` | `addl r1, r4, 8(r4)` | Complexa 1 | Soma `Reg[r1] + Reg[r4]` e grava na memória em `Reg[r4] + sext(offset)` |
| 9 | `001001` | `addli` | `addli r1, 5, 12(r4)` | Complexa 2 | Soma `Reg[r1] + sext(imm8)` e grava na memória em `Reg[r4] + sext(off8)` |

---

## b. Número de Registradores

- **Banco de Registradores:** **32 registradores** de propósito geral de **32 bits** (`r0` a `r31`).
- O registrador **`r0`** é fixo em `0x00000000` (leituras sempre retornam 0 e escritas em `r0` são ignoradas).
- **Registradores Internos (não visíveis ao programador, modelo Multiciclo):**
  - `PC` (32 bits): Program Counter (incrementa de `+4` por instrução).
  - `IR` (32 bits): Instruction Register.
  - `MDR` (32 bits): Memory Data Register.
  - `A` e `B` (32 bits): Registradores de operandos lidos do banco.
  - `ALUOut` (32 bits): Armazena o resultado da ULA entre ciclos.
  - `TempReg` (32 bits): Registrador auxiliar para reter a soma das instruções complexas `addl` e `addli` enquanto a ULA calcula o endereço de memória.

---

## c. Definição das Instruções e Formato dos Campos (32 bits)

Todas as instruções possuem largura fixa de **32 bits**, com `opcode` nos 6 bits mais significativos (`[31:26]`):

1. **Tipo-R (`add`, `sub`, `and`):**
   - `[31:26]` `opcode` (6 bits)
   - `[25:21]` `rs` (5 bits – 1º operando)
   - `[20:16]` `rt` (5 bits – 2º operando)
   - `[15:11]` `rd` (5 bits – registrador de destino)
   - `[10:0]` `resto` (11 bits preenchidos com `0`)

2. **Tipo-I (`lw`, `sw`, `addi`, `beq`, `addl`):**
   - `[31:26]` `opcode` (6 bits)
   - `[25:21]` `reg1` (5 bits – base em `lw`/`sw`, destino em `addi`, `r1` em `beq`/`addl`)
   - `[20:16]` `reg2` (5 bits – destino em `lw`, fonte em `sw`, `00000` em `addi`, `r2` em `beq`/`addl`)
   - `[15:0]` `imediato` (16 bits com extensão de sinal para 32 bits)

3. **Tipo-I Dual-Imm (`addli` – Complexa 2):**
   - `[31:26]` `opcode` (6 bits)
   - `[25:21]` `r1` (5 bits)
   - `[20:16]` `r2` (5 bits)
   - `[15:8]` `imm8` (8 bits – constante somada a `r1`, com extensão de sinal)
   - `[7:0]` `off8` (8 bits – deslocamento somado a `r2`, com extensão de sinal)

4. **Tipo-J (`jump`):**
   - `[31:26]` `opcode` (6 bits)
   - `[25:0]` `target26` (26 bits – endereço de palavra deslocado de 2 bits à esquerda `<< 2`)

---

## d. Modos de Endereçamento (`lw` e `sw`)

- Utiliza **endereçamento por base + deslocamento** (*Base + Offset Addressing*).
- O endereço efetivo em bytes é calculado pela ULA somando o registrador base (`A`) com o imediato de 16 bits estendido em sinal para 32 bits (`sext_imm16`).
- Como a memória é organizada em palavras de 32 bits (4 bytes) e o `PC` avança de `+4`, os bits `addr(7 downto 2)` selecionam a palavra correspondente na memória.

---

## e. Especificação da Memória

- **Arquitetura:** **1 Memória Unificada** (Von Neumann – instruções e dados compartilham o mesmo espaço de endereçamento), implementada em [`memory.vhd`](memory.vhd).
- **Tamanho:** **64 palavras de 32 bits** (**256 bytes**), endereçada a byte (`addr(7 downto 2)`).
- **Operação:** Leitura assíncrona controlada por `mem_read` e escrita síncrona na borda de subida do clock controlada por `mem_write`.

---

## f. Projeto da Unidade Operativa (`datapath.vhd`, `ula.vhd`, `reg_bank.vhd`)

- **ULA (`ula.vhd`):** Executa `ADD` (`"00"`), `SUB` (`"01"`) e `AND` (`"10"`), gerando `result` (32 bits) e flag `zero`.
- **Banco de Registradores (`reg_bank.vhd`):** 2 portas de leitura combinacionais e 1 porta de escrita síncrona.
- **Multiplexadores no Caminho de Dados (`datapath.vhd`):**
  - `MUX 1` (`reg_dst`): Seleciona registrador de escrita (`rt`, `rd` ou `rs`).
  - `MUX 3` (`mem_to_reg`): Seleciona dado para o banco (`ALUOut` ou `MDR`).
  - `MUX 4` (`alu_src_a`): Seleciona entrada A da ULA (`PC`, `A` ou `B`).
  - `MUX 5` (`alu_src_b`): Seleciona entrada B da ULA (`B`, `4`, `sext_imm16`, `sext_imm16 << 2`, `sext_imm8`, `sext_off8`).
  - `MUX 6` (`pc_source`): Seleciona próximo `PC` (`alu_result`, `ALUOut` ou `jump_addr`).
  - `MUX 7` (`i_or_d`): Seleciona endereço da memória (`PC` ou `ALUOut`).
  - `mem_data_mux` (`mem_data_src`): Seleciona dado de escrita na memória (`B` para `sw`, `TempReg` para `addl`/`addli`).

---

## g. Projeto da Unidade de Controle (`control_unit.vhd`)

Implementada como **Máquina de Estados Finitos (FSM)** com **18 estados**:
- **Busca e Decodificação:** `s_if` -> `s_id`
- **Memória (`lw`/`sw`):** `s_mem_addr` -> `s_lw_mem` -> `s_lw_wb` (ou `s_sw_mem`)
- **Tipo-R (`add`/`sub`/`and`):** `s_ex_r` -> `s_wb_r`
- **Imediato (`addi`):** `s_ex_addi` -> `s_wb_addi`
- **Desvios (`beq`/`jump`):** `s_ex_beq`, `s_ex_jump`
- **Complexa 1 (`addl`):** `s_addl_ex1` (`A+B`) -> `s_addl_ex2` (salva `TempReg` e calcula `B+offset`) -> `s_addl_mem` (`Mem[ALUOut] <= TempReg`)
- **Complexa 2 (`addli`):** `s_addli_ex1` (`A+imm8`) -> `s_addli_ex2` (salva `TempReg` e calcula `B+off8`) -> `s_addli_mem` (`Mem[ALUOut] <= TempReg`)

---

## Arquivos do Projeto

- [`ula.vhd`](ula.vhd) – Unidade Lógica e Aritmética
- [`reg_bank.vhd`](reg_bank.vhd) – Banco de Registradores
- [`memory.vhd`](memory.vhd) – Memória Unificada pré-carregada com programa de teste
- [`control_unit.vhd`](control_unit.vhd) – Unidade de Controle FSM Multiciclo
- [`datapath.vhd`](datapath.vhd) – Unidade Operativa (Datapath)
- [`cpu_top.vhd`](cpu_top.vhd) – Entidade de Topo do Processador
- [`cpu_tb.vhd`](cpu_tb.vhd) – Testbench com verificação automática (`assert`) das 10 instruções
