-- Testbench for 4-bit Synchronous Binary Counter
library ieee;
use ieee.std_logic_1164.all;

entity Counter4Bit_tb is
end entity Counter4Bit_tb;

architecture sim of Counter4Bit_tb is
    component Counter4Bit is
        port (
            clk    : in  std_logic;
            reset  : in  std_logic;
            enable : in  std_logic;
            count  : out std_logic_vector(3 downto 0)
        );
    end component;

    signal clk    : std_logic := '0';
    signal reset  : std_logic := '1';
    signal enable : std_logic := '0';
    signal count  : std_logic_vector(3 downto 0);
begin
    -- Instantiate DUT
    dut: Counter4Bit port map (
        clk    => clk,
        reset  => reset,
        enable => enable,
        count  => count
    );

    -- Clock generation: 10 ns period (50 MHz)
    clk_gen: process
    begin
        clk <= '0';
        wait for 5 ns;
        clk <= '1';
        wait for 5 ns;
    end process clk_gen;

    -- Stimulus process
    stimulus: process
    begin
        -- Assert reset for 12 ns
        reset <= '1';
        enable <= '0';
        wait for 12 ns;

        -- Release reset, enable counter
        reset <= '0';
        enable <= '1';
        wait for 80 ns;

        -- Disable counter for 20 ns
        enable <= '0';
        wait for 20 ns;

        -- Re-enable counter
        enable <= '1';
        wait for 40 ns;

        -- Async reset pulse
        reset <= '1';
        wait for 5 ns;
        reset <= '0';

        wait for 30 ns;
        wait;
    end process stimulus;
end architecture sim;
