# Safe Rust VHDL Compiler, Simulator & Waveform Viewer

Uma aplicação completa desenvolvida em **100% Safe Rust** (`#![forbid(unsafe_code)]`) que implementa:
1. **Frontend VHDL**: Lexer & Parser recursivo descendente / Pratt parser para entidades, arquiteturas, tipos (IEEE 1164 `std_logic`, `std_logic_vector`, `bit`, `boolean`, `integer`, `enum`), processos concorrentes e sequenciais, instanciação de componentes e atribuições com atraso (`after`).
2. **Motor de Simulação de Eventos Discretos (IEEE 1076)**:
   - Resolução de ciclos delta ($\delta$-cycles) com convergência.
   - Escalonamento com precisão de femtossegundos (`fs`), picossegundos (`ps`), nanossegundos (`ns`).
   - Avaliação de sensibilidade, `rising_edge()`, `falling_edge()`, `'event`.
   - Execução de comandos sequenciais (`if/elsif/else`, `case/when`, `wait for/until/on`, laços `for/while`).
3. **Dois Visualizadores de Forma de Onda Embutidos**:
   - **Visualizador ASCII no Terminal**: Exibe diagramas de tempo com transições digitais (`^`, `_`, `~`, `x`) e ônibus hexadecimais em tempo real direto no console.
   - **Visualizador Interativo HTML5 / Canvas**: Interface gráfica dark-mode moderna, independente (sem dependências externas), com cursor interativo, zoom (mouse wheel / botões), pan por arrasto, alternância de radix (Hex, Binário, Decimal) e régua de tempo.
4. **Exportador VCD (Value Change Dump)**: Padrão IEEE 1364/1800 compatível com GTKWave, ModelSim e WaveTrace.
5. **Servidor Web Local Embutido**: Permite servir o visualizador interativo via `http://localhost:<porta>` com `--serve`.

---

## 🛡️ Garantia de Safe Rust

O projeto impõe `#![forbid(unsafe_code)]` tanto no `lib.rs` quanto no `main.rs`, garantindo conformidade rigorosa com segurança de memória e ausência de blocos `unsafe`.

---

## 🚀 Como Compilar e Executar

### 1. Compilação
```bash
cargo build --release
```
O executável estará disponível em `target/release/vhdl_sim.exe` (Windows) ou `target/release/vhdl_sim` (Linux/macOS).

### 2. Executar os Testes
```bash
cargo test
```
Todos os 9 testes de integração e unidade cobrem lexer, parser, avaliador de tipos lógicos de 9 valores, ciclos delta, somador completo, contador síncrono e máquina de estados.

---

## 💻 Uso da Linha de Comando (CLI)

```bash
vhdl_sim <arquivos.vhd...> [opções]
```

### Opções:
- `--ide [PORTA]`: Inicia a **IDE estilo Vivado** no navegador com editor VHDL, compilador integrado e visualizador de ondas (porta padrão: `8080`).
- `--top <ENTITY>`: Entidade de nível superior a ser elaborada (detecção automática por padrão).
- `--stop <TIME>`: Duração máxima da simulação (ex: `100ns`, `50ps`, `1us`, `10ms`) [padrão: `100ns`].
- `--vcd <ARQUIVO>`: Exporta as transições em formato padrão VCD (`.vcd`).
- `--html <ARQUIVO>`: Gera o visualizador de onda gráfico interativo em HTML5 (`.html`).
- `--serve [PORTA]`: Inicia o servidor HTTP local embutido (porta padrão: `8080`).
- `--view`: Gera o visualizador HTML e abre automaticamente no navegador padrão do sistema operacional.
- `--no-ascii`: Desativa a impressão da forma de onda ASCII no terminal.
- `--help, -h`: Exibe a ajuda.

---

## 📁 Exemplos Inclusos (`examples/`)

### 1. Somador Completo de 1 Bit (`full_adder.vhd` & `full_adder_tb.vhd`)
Demonstra circuitos combinacionais, instanciação hierárquica de componentes (`dut: FullAdder port map (...)`) e verificação com `assert`:
```bash
cargo run --bin vhdl_sim -- examples/full_adder.vhd examples/full_adder_tb.vhd --stop 80ns --vcd full_adder.vcd --html full_adder.html
```

### 2. Contador Binário Síncrono de 4 Bits (`counter_4bit.vhd` & `counter_4bit_tb.vhd`)
Demonstra registradores sensíveis a clock (`rising_edge(clk)`), reset assíncrono, sinal de enable e barramento `std_logic_vector(3 downto 0)`:
```bash
cargo run --bin vhdl_sim -- examples/counter_4bit.vhd examples/counter_4bit_tb.vhd --stop 180ns --vcd counter.vcd --html counter.html
```

### 3. Máquina de Estados Finito - Semáforo (`fsm_traffic.vhd` & `fsm_traffic_tb.vhd`)
Demonstra tipos enumerados (`type state_type is (s_red, s_green, s_yellow)`), lógica de próximo estado com `case/when` e transições de estado cíclicas:
```bash
cargo run --bin vhdl_sim -- examples/fsm_traffic.vhd examples/fsm_traffic_tb.vhd --stop 160ns --vcd traffic.vcd --html traffic.html
```

---

## 🌊 Visualizadores de Onda

### No Terminal (ASCII)
Exibe instantaneamente as transições digitais no console:
```text
           0 ns                             40.0 ns                    80.0 ns
           +|---------|---------|---------|---------|---------|---------|------+
       a : __________________________________^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
       b : _________________^^^^^^^^^^^^^^^^^_________________^^^^^^^^^^^^^^^^
     cin : _________^^^^^^^^_________^^^^^^^^________^^^^^^^^^________^^^^^^^^
    cout : __________________________^^^^^^^^________^^^^^^^^^^^^^^^^^^^^^^^^^
     sum : _________^^^^^^^^^^^^^^^^^________^^^^^^^^_________________^^^^^^^^
           +|---------|---------|---------|---------|---------|---------|------+
Legend: ^ = High (1), _ = Low (0), ~ = High-Z (Z), x = Unknown (X/U), | = Bus transition
```

### Gráfico Interativo no Navegador (HTML5 Canvas)
- **Zoom**: Roda do mouse ou botões `Zoom +`, `Zoom -`, `Fit`.
- **Pan / Rolagem**: Arraste do mouse horizontalmente.
- **Cursor de Medição**: Clique em qualquer instante de tempo para posicionar o cursor amarelo e visualizar o valor exato de todos os sinais naquele instante.
- **Radix**: Botões para alternar barramentos entre `HEX`, `BIN` e `DEC`.
- **Standalone**: Arquivo HTML único, auto-contido, que funciona 100% offline em qualquer navegador moderno.
