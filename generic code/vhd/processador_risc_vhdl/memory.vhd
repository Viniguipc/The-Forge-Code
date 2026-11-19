-- Memoria Unificada (Instrucoes e Dados - 64 palavras x 32 bits = 256 bytes)
-- Enderecamento a byte: indice da palavra = addr(7 downto 2)
library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

entity memory is
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
end entity memory;

architecture rtl of memory is
    type mem_array is array (0 to 63) of std_logic_vector(31 downto 0);
    signal mem : mem_array := (
        0  => X"1420000A", -- addi  r1, 10            (r1 = 10)
        1  => X"14400006", -- addi  r2, 6             (r2 = 6)
        2  => X"08221800", -- add   r3, r1, r2        (r3 = 16)
        3  => X"0C221800", -- sub   r3, r1, r2        (r3 = 4)
        4  => X"10221800", -- and   r3, r1, r2        (r3 = 2)
        5  => X"14800080", -- addi  r4, 128           (r4 = 0x80 -> palavra 32)
        6  => X"04810004", -- sw    r1, 4(r4)         (mem[33] = 10)
        7  => X"00830004", -- lw    r3, 4(r4)         (r3 = mem[33] = 10)
        8  => X"20240008", -- addl  r1, r4, 8(r4)     (mem[34] = r1 + r4 = 138)
        9  => X"00820008", -- lw    r2, 8(r4)         (r2 = mem[34] = 138)
        10 => X"2424050C", -- addli r1, 5, 12(r4)     (mem[35] = r1 + 5 = 15)
        11 => X"0082000C", -- lw    r2, 12(r4)        (r2 = mem[35] = 15)
        12 => X"1C230001", -- beq   r1, r3, 1         (r1==r3 -> pula p/ palavra 14)
        13 => X"14200063", -- addi  r1, 99            (pulado pelo beq)
        14 => X"1800000E", -- jump  14                (loop final)
        others => X"00000000"
    );
    signal word_idx : std_logic_vector(5 downto 0);
begin
    word_idx <= addr(7 downto 2);

    process(clk)
    begin
        if rising_edge(clk) then
            if mem_write = '1' then
                mem(to_integer(unsigned(word_idx))) <= write_data;
            end if;
        end if;
    end process;

    read_data <= mem(to_integer(unsigned(word_idx))) when mem_read = '1' else X"00000000";

    dbg_mem33 <= mem(33);
    dbg_mem34 <= mem(34);
    dbg_mem35 <= mem(35);
end architecture rtl;
