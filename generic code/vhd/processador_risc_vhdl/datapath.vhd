-- Unidade Operativa (Datapath) do Processador RISC Multiciclo
library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

entity datapath is
    port (
        clk            : in  std_logic;
        reset          : in  std_logic;
        -- Sinais de controle
        pc_write       : in  std_logic;
        pc_write_cond  : in  std_logic;
        i_or_d         : in  std_logic;
        ir_write       : in  std_logic;
        mdr_write      : in  std_logic;
        reg_write      : in  std_logic;
        reg_dst        : in  std_logic_vector(1 downto 0);
        mem_to_reg     : in  std_logic;
        alu_src_a      : in  std_logic_vector(1 downto 0);
        alu_src_b      : in  std_logic_vector(2 downto 0);
        alu_op         : in  std_logic_vector(1 downto 0);
        pc_source      : in  std_logic_vector(1 downto 0);
        temp_write     : in  std_logic;
        mem_data_src   : in  std_logic;
        -- Interface com Memoria
        mem_read_data  : in  std_logic_vector(31 downto 0);
        mem_addr       : out std_logic_vector(31 downto 0);
        mem_write_data : out std_logic_vector(31 downto 0);
        -- Saidas para Controle e Debug
        opcode         : out std_logic_vector(5 downto 0);
        dbg_pc         : out std_logic_vector(31 downto 0);
        dbg_ir         : out std_logic_vector(31 downto 0);
        dbg_alu_out    : out std_logic_vector(31 downto 0);
        dbg_r1         : out std_logic_vector(31 downto 0);
        dbg_r2         : out std_logic_vector(31 downto 0);
        dbg_r3         : out std_logic_vector(31 downto 0)
    );
end entity datapath;

architecture rtl of datapath is
    component reg_bank is
        port (
            clk        : in  std_logic;
            reset      : in  std_logic;
            reg_write  : in  std_logic;
            read_reg1  : in  std_logic_vector(4 downto 0);
            read_reg2  : in  std_logic_vector(4 downto 0);
            write_reg  : in  std_logic_vector(4 downto 0);
            write_data : in  std_logic_vector(31 downto 0);
            read_data1 : out std_logic_vector(31 downto 0);
            read_data2 : out std_logic_vector(31 downto 0);
            dbg_r1     : out std_logic_vector(31 downto 0);
            dbg_r2     : out std_logic_vector(31 downto 0);
            dbg_r3     : out std_logic_vector(31 downto 0)
        );
    end component;

    component ula is
        port (
            a      : in  std_logic_vector(31 downto 0);
            b      : in  std_logic_vector(31 downto 0);
            alu_op : in  std_logic_vector(1 downto 0);
            result : out std_logic_vector(31 downto 0);
            zero   : out std_logic
        );
    end component;

    -- Registradores do Datapath
    signal pc_reg      : std_logic_vector(31 downto 0) := (others => '0');
    signal ir_reg      : std_logic_vector(31 downto 0) := (others => '0');
    signal mdr_reg     : std_logic_vector(31 downto 0) := (others => '0');
    signal a_reg       : std_logic_vector(31 downto 0) := (others => '0');
    signal b_reg       : std_logic_vector(31 downto 0) := (others => '0');
    signal alu_out_reg : std_logic_vector(31 downto 0) := (others => '0');
    signal temp_reg    : std_logic_vector(31 downto 0) := (others => '0');

    -- Campos da Instrucao
    signal rs_addr     : std_logic_vector(4 downto 0) := (others => '0');
    signal rt_addr     : std_logic_vector(4 downto 0) := (others => '0');
    signal rd_addr     : std_logic_vector(4 downto 0) := (others => '0');
    signal imm16       : std_logic_vector(15 downto 0) := (others => '0');
    signal imm8_hi     : std_logic_vector(7 downto 0) := (others => '0');
    signal off8_lo     : std_logic_vector(7 downto 0) := (others => '0');
    signal target26    : std_logic_vector(25 downto 0) := (others => '0');

    -- Extensoes de sinal e deslocamentos
    signal sext_imm16     : std_logic_vector(31 downto 0);
    signal sext_imm16_sl2 : std_logic_vector(31 downto 0);
    signal sext_imm8_hi   : std_logic_vector(31 downto 0);
    signal sext_off8_lo   : std_logic_vector(31 downto 0);
    signal jump_addr      : std_logic_vector(31 downto 0);

    -- Sinais internos dos MUXes
    signal write_reg_addr : std_logic_vector(4 downto 0);
    signal reg_write_data : std_logic_vector(31 downto 0);
    signal rf_read_data1  : std_logic_vector(31 downto 0);
    signal rf_read_data2  : std_logic_vector(31 downto 0);
    signal alu_in_a       : std_logic_vector(31 downto 0);
    signal alu_in_b       : std_logic_vector(31 downto 0);
    signal alu_result     : std_logic_vector(31 downto 0);
    signal alu_zero       : std_logic;
    signal next_pc        : std_logic_vector(31 downto 0);
    signal pc_en          : std_logic;
begin
    -- Fatiamento da instrucao
    opcode   <= ir_reg(31 downto 26);
    rs_addr  <= ir_reg(25 downto 21);
    rt_addr  <= ir_reg(20 downto 16);
    rd_addr  <= ir_reg(15 downto 11);
    imm16    <= ir_reg(15 downto 0);
    imm8_hi  <= ir_reg(15 downto 8);
    off8_lo  <= ir_reg(7 downto 0);
    target26 <= ir_reg(25 downto 0);

    -- Extensao de sinal e shift left 2
    sext_imm16     <= X"FFFF" & imm16 when imm16(15) = '1' else X"0000" & imm16;
    sext_imm16_sl2 <= sext_imm16(29 downto 0) & "00";
    sext_imm8_hi   <= X"FFFFFF" & imm8_hi when imm8_hi(7) = '1' else X"000000" & imm8_hi;
    sext_off8_lo   <= X"FFFFFF" & off8_lo when off8_lo(7) = '1' else X"000000" & off8_lo;
    jump_addr      <= pc_reg(31 downto 28) & target26 & "00";

    -- MUX 7: IorD (endereco da memoria)
    mem_addr <= alu_out_reg when i_or_d = '1' else pc_reg;

    -- MUX dado de escrita na memoria (sw vs addl/addli)
    mem_write_data <= temp_reg when mem_data_src = '1' else b_reg;

    -- MUX 1: RegDst (registrador de destino)
    write_reg_addr <= rs_addr when reg_dst = "10" else
                      rd_addr when reg_dst = "01" else
                      rt_addr;

    -- MUX 3: MemtoReg (dado de escrita no banco de registradores)
    reg_write_data <= mdr_reg when mem_to_reg = '1' else alu_out_reg;

    -- Instancia Banco de Registradores
    u_regs: reg_bank port map (
        clk        => clk,
        reset      => reset,
        reg_write  => reg_write,
        read_reg1  => rs_addr,
        read_reg2  => rt_addr,
        write_reg  => write_reg_addr,
        write_data => reg_write_data,
        read_data1 => rf_read_data1,
        read_data2 => rf_read_data2,
        dbg_r1     => dbg_r1,
        dbg_r2     => dbg_r2,
        dbg_r3     => dbg_r3
    );

    -- MUX 4: ALUSrcA
    alu_in_a <= b_reg  when alu_src_a = "10" else
                a_reg  when alu_src_a = "01" else
                pc_reg;

    -- MUX 5: ALUSrcB
    alu_in_b <= sext_off8_lo   when alu_src_b = "101" else
                sext_imm8_hi   when alu_src_b = "100" else
                sext_imm16_sl2 when alu_src_b = "011" else
                sext_imm16     when alu_src_b = "010" else
                X"00000004"    when alu_src_b = "001" else
                b_reg;

    -- Instancia ULA
    u_ula: ula port map (
        a      => alu_in_a,
        b      => alu_in_b,
        alu_op => alu_op,
        result => alu_result,
        zero   => alu_zero
    );

    -- MUX 6: PCSource
    next_pc <= jump_addr   when pc_source = "10" else
               alu_out_reg when pc_source = "01" else
               alu_result;

    pc_en <= pc_write or (pc_write_cond and alu_zero);

    -- Atualizacao dos registradores na borda de subida do clock
    process(clk, reset)
    begin
        if reset = '1' then
            pc_reg      <= X"00000000";
            ir_reg      <= X"00000000";
            mdr_reg     <= X"00000000";
            a_reg       <= X"00000000";
            b_reg       <= X"00000000";
            alu_out_reg <= X"00000000";
            temp_reg    <= X"00000000";
        elsif rising_edge(clk) then
            if pc_en = '1' then
                pc_reg <= next_pc;
            end if;
            if ir_write = '1' then
                ir_reg <= mem_read_data;
            end if;
            if mdr_write = '1' then
                mdr_reg <= mem_read_data;
            end if;
            if temp_write = '1' then
                temp_reg <= alu_out_reg;
            end if;
            a_reg       <= rf_read_data1;
            b_reg       <= rf_read_data2;
            alu_out_reg <= alu_result;
        end if;
    end process;

    dbg_pc      <= pc_reg;
    dbg_ir      <= ir_reg;
    dbg_alu_out <= alu_out_reg;
end architecture rtl;
