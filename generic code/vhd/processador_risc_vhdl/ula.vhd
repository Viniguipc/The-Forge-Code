-- Unidade Logica e Aritmetica (ULA) de 32 bits
-- Operacoes:
--   "00" -> ADD (a + b)
--   "01" -> SUB (a - b)
--   "10" -> AND (a and b)
library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

entity ula is
    port (
        a      : in  std_logic_vector(31 downto 0);
        b      : in  std_logic_vector(31 downto 0);
        alu_op : in  std_logic_vector(1 downto 0);
        result : out std_logic_vector(31 downto 0);
        zero   : out std_logic
    );
end entity ula;

architecture rtl of ula is
    signal res_int : std_logic_vector(31 downto 0) := (others => '0');
begin
    process(a, b, alu_op)
    begin
        case alu_op is
            when "00" =>
                res_int <= std_logic_vector(unsigned(a) + unsigned(b));
            when "01" =>
                res_int <= std_logic_vector(unsigned(a) - unsigned(b));
            when "10" =>
                res_int <= a and b;
            when others =>
                res_int <= (others => '0');
        end case;
    end process;

    result <= res_int;
    zero   <= '1' when res_int = X"00000000" else '0';
end architecture rtl;
