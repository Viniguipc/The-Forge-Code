-- Testbench para o Processador RISC Multiciclo (valida todas as 10 instrucoes)
library ieee;
use ieee.std_logic_1164.all;

entity cpu_tb is
end entity cpu_tb;

architecture sim of cpu_tb is
    component cpu_top is
        port (
            clk         : in  std_logic;
            reset       : in  std_logic;
            dbg_pc      : out std_logic_vector(31 downto 0);
            dbg_ir      : out std_logic_vector(31 downto 0);
            dbg_state   : out std_logic_vector(3 downto 0);
            dbg_alu_out : out std_logic_vector(31 downto 0);
            dbg_r1      : out std_logic_vector(31 downto 0);
            dbg_r2      : out std_logic_vector(31 downto 0);
            dbg_r3      : out std_logic_vector(31 downto 0);
            dbg_mem33   : out std_logic_vector(31 downto 0);
            dbg_mem34   : out std_logic_vector(31 downto 0);
            dbg_mem35   : out std_logic_vector(31 downto 0)
        );
    end component;

    signal clk         : std_logic := '0';
    signal reset       : std_logic := '1';
    signal dbg_pc      : std_logic_vector(31 downto 0);
    signal dbg_ir      : std_logic_vector(31 downto 0);
    signal dbg_state   : std_logic_vector(3 downto 0);
    signal dbg_alu_out : std_logic_vector(31 downto 0);
    signal dbg_r1      : std_logic_vector(31 downto 0);
    signal dbg_r2      : std_logic_vector(31 downto 0);
    signal dbg_r3      : std_logic_vector(31 downto 0);
    signal dbg_mem33   : std_logic_vector(31 downto 0);
    signal dbg_mem34   : std_logic_vector(31 downto 0);
    signal dbg_mem35   : std_logic_vector(31 downto 0);
begin
    dut: cpu_top port map (
        clk         => clk,
        reset       => reset,
        dbg_pc      => dbg_pc,
        dbg_ir      => dbg_ir,
        dbg_state   => dbg_state,
        dbg_alu_out => dbg_alu_out,
        dbg_r1      => dbg_r1,
        dbg_r2      => dbg_r2,
        dbg_r3      => dbg_r3,
        dbg_mem33   => dbg_mem33,
        dbg_mem34   => dbg_mem34,
        dbg_mem35   => dbg_mem35
    );

    -- Clock de 100 MHz (periodo 10 ns)
    clk_gen: process
    begin
        clk <= '0';
        wait for 5 ns;
        clk <= '1';
        wait for 5 ns;
    end process clk_gen;

    -- Estimulos e verificacoes
    stimulus: process
    begin
        reset <= '1';
        wait for 12 ns;
        reset <= '0';

        -- Aguarda execucao completa das 10 instrucoes (~60 ciclos = 600 ns)
        wait for 650 ns;

        assert dbg_r1    = X"0000000A" report "Erro: r1 deveria ser 10 (beq falhou em pular addi r1,99 ou addi inicial falhou)";
        assert dbg_r2    = X"0000000F" report "Erro: r2 deveria ser 15 (lw apos addli falhou)";
        assert dbg_r3    = X"0000000A" report "Erro: r3 deveria ser 10 (lw apos sw falhou)";
        assert dbg_mem33 = X"0000000A" report "Erro: mem[33] deveria ser 10 (sw falhou)";
        assert dbg_mem34 = X"0000008A" report "Erro: mem[34] deveria ser 138 (addl falhou)";
        assert dbg_mem35 = X"0000000F" report "Erro: mem[35] deveria ser 15 (addli falhou)";

        wait;
    end process stimulus;
end architecture sim;
