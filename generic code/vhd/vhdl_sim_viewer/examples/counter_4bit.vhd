-- 4-bit Synchronous Binary Counter with Async Reset & Enable
library ieee;
use ieee.std_logic_1164.all;

entity Counter4Bit is
    port (
        clk    : in  std_logic;
        reset  : in  std_logic;
        enable : in  std_logic;
        count  : out std_logic_vector(3 downto 0)
    );
end entity Counter4Bit;

architecture rtl of Counter4Bit is
    signal count_reg : std_logic_vector(3 downto 0) := "0000";
begin
    process(clk, reset)
    begin
        if reset = '1' then
            count_reg <= "0000";
        elsif rising_edge(clk) then
            if enable = '1' then
                count_reg <= count_reg + 1;
            end if;
        end if;
    end process;

    count <= count_reg;
end architecture rtl;
