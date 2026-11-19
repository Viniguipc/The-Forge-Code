-- Entidade de Topo: Processador RISC Multiciclo
library ieee;
use ieee.std_logic_1164.all;

entity cpu_top is
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
end entity cpu_top;

architecture rtl of cpu_top is
    component control_unit is
        port (
            clk           : in  std_logic;
            reset         : in  std_logic;
            opcode        : in  std_logic_vector(5 downto 0);
            pc_write      : out std_logic;
            pc_write_cond : out std_logic;
            i_or_d        : out std_logic;
            mem_read      : out std_logic;
            mem_write     : out std_logic;
            ir_write      : out std_logic;
            mdr_write     : out std_logic;
            reg_write     : out std_logic;
            reg_dst       : out std_logic_vector(1 downto 0);
            mem_to_reg    : out std_logic;
            alu_src_a     : out std_logic_vector(1 downto 0);
            alu_src_b     : out std_logic_vector(2 downto 0);
            alu_op        : out std_logic_vector(1 downto 0);
            pc_source     : out std_logic_vector(1 downto 0);
            temp_write    : out std_logic;
            mem_data_src  : out std_logic;
            dbg_state     : out std_logic_vector(3 downto 0)
        );
    end component;

    component datapath is
        port (
            clk            : in  std_logic;
            reset          : in  std_logic;
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
            mem_read_data  : in  std_logic_vector(31 downto 0);
            mem_addr       : out std_logic_vector(31 downto 0);
            mem_write_data : out std_logic_vector(31 downto 0);
            opcode         : out std_logic_vector(5 downto 0);
            dbg_pc         : out std_logic_vector(31 downto 0);
            dbg_ir         : out std_logic_vector(31 downto 0);
            dbg_alu_out    : out std_logic_vector(31 downto 0);
            dbg_r1         : out std_logic_vector(31 downto 0);
            dbg_r2         : out std_logic_vector(31 downto 0);
            dbg_r3         : out std_logic_vector(31 downto 0)
        );
    end component;

    component memory is
        port (
            clk        : in  std_logic;
            mem_read   : in  std_logic;
            mem_write  : in  std_logic;
            addr       : in  std_logic_vector(31 downto 0);
            write_data : in  std_logic_vector(31 downto 0);
            read_data  : out std_logic_vector(31 downto 0);
            dbg_mem33  : out std_logic_vector(31 downto 0);
            dbg_mem34  : out std_logic_vector(31 downto 0);
            dbg_mem35  : out std_logic_vector(31 downto 0)
        );
    end component;

    signal opcode         : std_logic_vector(5 downto 0);
    signal pc_write       : std_logic;
    signal pc_write_cond  : std_logic;
    signal i_or_d         : std_logic;
    signal mem_read       : std_logic;
    signal mem_write      : std_logic;
    signal ir_write       : std_logic;
    signal mdr_write      : std_logic;
    signal reg_write      : std_logic;
    signal reg_dst        : std_logic_vector(1 downto 0);
    signal mem_to_reg     : std_logic;
    signal alu_src_a      : std_logic_vector(1 downto 0);
    signal alu_src_b      : std_logic_vector(2 downto 0);
    signal alu_op         : std_logic_vector(1 downto 0);
    signal pc_source      : std_logic_vector(1 downto 0);
    signal temp_write     : std_logic;
    signal mem_data_src   : std_logic;
    signal mem_read_data  : std_logic_vector(31 downto 0);
    signal mem_addr       : std_logic_vector(31 downto 0);
    signal mem_write_data : std_logic_vector(31 downto 0);
begin
    u_ctrl: control_unit port map (
        clk           => clk,
        reset         => reset,
        opcode        => opcode,
        pc_write      => pc_write,
        pc_write_cond => pc_write_cond,
        i_or_d        => i_or_d,
        mem_read      => mem_read,
        mem_write     => mem_write,
        ir_write      => ir_write,
        mdr_write     => mdr_write,
        reg_write     => reg_write,
        reg_dst       => reg_dst,
        mem_to_reg    => mem_to_reg,
        alu_src_a     => alu_src_a,
        alu_src_b     => alu_src_b,
        alu_op        => alu_op,
        pc_source     => pc_source,
        temp_write    => temp_write,
        mem_data_src  => mem_data_src,
        dbg_state     => dbg_state
    );

    u_dp: datapath port map (
        clk            => clk,
        reset          => reset,
        pc_write       => pc_write,
        pc_write_cond  => pc_write_cond,
        i_or_d         => i_or_d,
        ir_write       => ir_write,
        mdr_write      => mdr_write,
        reg_write      => reg_write,
        reg_dst        => reg_dst,
        mem_to_reg     => mem_to_reg,
        alu_src_a      => alu_src_a,
        alu_src_b      => alu_src_b,
        alu_op         => alu_op,
        pc_source      => pc_source,
        temp_write     => temp_write,
        mem_data_src   => mem_data_src,
        mem_read_data  => mem_read_data,
        mem_addr       => mem_addr,
        mem_write_data => mem_write_data,
        opcode         => opcode,
        dbg_pc         => dbg_pc,
        dbg_ir         => dbg_ir,
        dbg_alu_out    => dbg_alu_out,
        dbg_r1         => dbg_r1,
        dbg_r2         => dbg_r2,
        dbg_r3         => dbg_r3
    );

    u_mem: memory port map (
        clk        => clk,
        mem_read   => mem_read,
        mem_write  => mem_write,
        addr       => mem_addr,
        write_data => mem_write_data,
        read_data  => mem_read_data,
        dbg_mem33  => dbg_mem33,
        dbg_mem34  => dbg_mem34,
        dbg_mem35  => dbg_mem35
    );
end architecture rtl;
