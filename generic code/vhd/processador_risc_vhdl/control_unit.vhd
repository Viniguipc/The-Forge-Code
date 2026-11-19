-- Unidade de Controle Multiciclo (FSM)
library ieee;
use ieee.std_logic_1164.all;

entity control_unit is
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
end entity control_unit;

architecture rtl of control_unit is
    type state_t is (
        s_if,
        s_id,
        s_mem_addr,
        s_lw_mem,
        s_lw_wb,
        s_sw_mem,
        s_ex_r,
        s_wb_r,
        s_ex_addi,
        s_wb_addi,
        s_ex_beq,
        s_ex_jump,
        s_addl_ex1,
        s_addl_ex2,
        s_addl_mem,
        s_addli_ex1,
        s_addli_ex2,
        s_addli_mem
    );

    signal state      : state_t := s_if;
    signal next_state : state_t := s_if;
begin
    -- Registrador de estado
    process(clk, reset)
    begin
        if reset = '1' then
            state <= s_if;
        elsif rising_edge(clk) then
            state <= next_state;
        end if;
    end process;

    -- Logica de proximo estado e saidas de controle
    process(state, opcode)
    begin
        -- Valores padrao seguros
        pc_write      <= '0';
        pc_write_cond <= '0';
        i_or_d        <= '0';
        mem_read      <= '0';
        mem_write     <= '0';
        ir_write      <= '0';
        mdr_write     <= '0';
        reg_write     <= '0';
        reg_dst       <= "00";
        mem_to_reg    <= '0';
        alu_src_a     <= "00";
        alu_src_b     <= "000";
        alu_op        <= "00";
        pc_source     <= "00";
        temp_write    <= '0';
        mem_data_src  <= '0';
        dbg_state     <= "0000";
        next_state    <= s_if;

        case state is
            when s_if =>
                dbg_state  <= "0000";
                mem_read   <= '1';
                i_or_d     <= '0';
                ir_write   <= '1';
                alu_src_a  <= "00";
                alu_src_b  <= "001"; -- 4
                alu_op     <= "00";  -- ADD
                pc_source  <= "00";
                pc_write   <= '1';
                next_state <= s_id;

            when s_id =>
                dbg_state  <= "0001";
                alu_src_a  <= "00";  -- PC
                alu_src_b  <= "011"; -- sext(imm16) << 2
                alu_op     <= "00";  -- ADD (pre-calcula alvo do beq)
                case opcode is
                    when "000000" => next_state <= s_mem_addr; -- lw
                    when "000001" => next_state <= s_mem_addr; -- sw
                    when "000010" => next_state <= s_ex_r;     -- add
                    when "000011" => next_state <= s_ex_r;     -- sub
                    when "000100" => next_state <= s_ex_r;     -- and
                    when "000101" => next_state <= s_ex_addi;  -- addi
                    when "000110" => next_state <= s_ex_jump;  -- jump
                    when "000111" => next_state <= s_ex_beq;   -- beq
                    when "001000" => next_state <= s_addl_ex1; -- addl (complexa 1)
                    when "001001" => next_state <= s_addli_ex1;-- addli (complexa 2)
                    when others   => next_state <= s_if;
                end case;

            when s_mem_addr =>
                dbg_state  <= "0010";
                alu_src_a  <= "01";  -- A (r_base)
                alu_src_b  <= "010"; -- sext(imm16)
                alu_op     <= "00";  -- ADD
                if opcode = "000000" then
                    next_state <= s_lw_mem;
                else
                    next_state <= s_sw_mem;
                end if;

            when s_lw_mem =>
                dbg_state  <= "0011";
                mem_read   <= '1';
                i_or_d     <= '1'; -- ALUOut
                mdr_write  <= '1';
                next_state <= s_lw_wb;

            when s_lw_wb =>
                dbg_state  <= "0100";
                reg_dst    <= "00"; -- ir(20 downto 16)
                mem_to_reg <= '1';  -- MDR
                reg_write  <= '1';
                next_state <= s_if;

            when s_sw_mem =>
                dbg_state    <= "0101";
                mem_write    <= '1';
                i_or_d       <= '1'; -- ALUOut
                mem_data_src <= '0'; -- B
                next_state   <= s_if;

            when s_ex_r =>
                dbg_state  <= "0110";
                alu_src_a  <= "01";  -- A
                alu_src_b  <= "000"; -- B
                if opcode = "000010" then
                    alu_op <= "00"; -- ADD
                elsif opcode = "000011" then
                    alu_op <= "01"; -- SUB
                else
                    alu_op <= "10"; -- AND
                end if;
                next_state <= s_wb_r;

            when s_wb_r =>
                dbg_state  <= "0111";
                reg_dst    <= "01"; -- ir(15 downto 11)
                mem_to_reg <= '0';  -- ALUOut
                reg_write  <= '1';
                next_state <= s_if;

            when s_ex_addi =>
                dbg_state  <= "1000";
                alu_src_a  <= "01";  -- A (r1)
                alu_src_b  <= "010"; -- sext(imm16)
                alu_op     <= "00";  -- ADD
                next_state <= s_wb_addi;

            when s_wb_addi =>
                dbg_state  <= "1001";
                reg_dst    <= "10"; -- ir(25 downto 21) (r1)
                mem_to_reg <= '0';  -- ALUOut
                reg_write  <= '1';
                next_state <= s_if;

            when s_ex_beq =>
                dbg_state     <= "1010";
                alu_src_a     <= "01";  -- A
                alu_src_b     <= "000"; -- B
                alu_op        <= "01";  -- SUB
                pc_source     <= "01";  -- ALUOut
                pc_write_cond <= '1';
                next_state    <= s_if;

            when s_ex_jump =>
                dbg_state  <= "1011";
                pc_source  <= "10"; -- jump_addr
                pc_write   <= '1';
                next_state <= s_if;

            -- Complexa 1: addl r1, r2, offset(r2)
            when s_addl_ex1 =>
                dbg_state  <= "1100";
                alu_src_a  <= "01";  -- A (r1)
                alu_src_b  <= "000"; -- B (r2)
                alu_op     <= "00";  -- ADD (r1 + r2)
                next_state <= s_addl_ex2;

            when s_addl_ex2 =>
                dbg_state  <= "1101";
                temp_write <= '1';   -- salva r1 + r2 em TempReg
                alu_src_a  <= "10";  -- B (r2)
                alu_src_b  <= "010"; -- sext(imm16)
                alu_op     <= "00";  -- ADD (r2 + offset)
                next_state <= s_addl_mem;

            when s_addl_mem =>
                dbg_state    <= "1110";
                i_or_d       <= '1'; -- ALUOut (r2 + offset)
                mem_data_src <= '1'; -- TempReg (r1 + r2)
                mem_write    <= '1';
                next_state   <= s_if;

            -- Complexa 2: addli r1, imm8, off8(r2)
            when s_addli_ex1 =>
                dbg_state  <= "1100";
                alu_src_a  <= "01";  -- A (r1)
                alu_src_b  <= "100"; -- sext(imm8)
                alu_op     <= "00";  -- ADD (r1 + imm8)
                next_state <= s_addli_ex2;

            when s_addli_ex2 =>
                dbg_state  <= "1101";
                temp_write <= '1';   -- salva r1 + imm8 em TempReg
                alu_src_a  <= "10";  -- B (r2)
                alu_src_b  <= "101"; -- sext(off8)
                alu_op     <= "00";  -- ADD (r2 + off8)
                next_state <= s_addli_mem;

            when s_addli_mem =>
                dbg_state    <= "1111";
                i_or_d       <= '1'; -- ALUOut (r2 + off8)
                mem_data_src <= '1'; -- TempReg (r1 + imm8)
                mem_write    <= '1';
                next_state   <= s_if;
        end case;
    end process;
end architecture rtl;
