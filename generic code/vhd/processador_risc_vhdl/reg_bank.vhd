-- Banco de Registradores (32 registradores x 32 bits)
-- r0 e fixo em 0x00000000 (leitura combinacional, escrita sincrona)
library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

entity reg_bank is
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
end entity reg_bank;

architecture rtl of reg_bank is
    type reg_array is array (0 to 31) of std_logic_vector(31 downto 0);
    signal regs : reg_array := (others => (others => '0'));
begin
    process(clk, reset)
    begin
        if reset = '1' then
            for i in 0 to 31 loop
                regs(i) <= X"00000000";
            end loop;
        elsif rising_edge(clk) then
            if reg_write = '1' then
                if write_reg /= "00000" then
                    regs(to_integer(unsigned(write_reg))) <= write_data;
                end if;
            end if;
        end if;
    end process;

    read_data1 <= X"00000000" when read_reg1 = "00000" else regs(to_integer(unsigned(read_reg1)));
    read_data2 <= X"00000000" when read_reg2 = "00000" else regs(to_integer(unsigned(read_reg2)));

    dbg_r1 <= regs(1);
    dbg_r2 <= regs(2);
    dbg_r3 <= regs(3);
end architecture rtl;
