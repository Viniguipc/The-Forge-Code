-- Testbench for Traffic Light Controller FSM
library ieee;
use ieee.std_logic_1164.all;

entity TrafficLightFSM_tb is
end entity TrafficLightFSM_tb;

architecture sim of TrafficLightFSM_tb is
    component TrafficLightFSM is
        port (
            clk    : in  std_logic;
            reset  : in  std_logic;
            red    : out std_logic;
            yellow : out std_logic;
            green  : out std_logic
        );
    end component;

    signal clk    : std_logic := '0';
    signal reset  : std_logic := '1';
    signal red    : std_logic;
    signal yellow : std_logic;
    signal green  : std_logic;
begin
    dut: TrafficLightFSM port map (
        clk    => clk,
        reset  => reset,
        red    => red,
        yellow => yellow,
        green  => green
    );

    -- Clock generation: 10 ns period
    clk_process: process
    begin
        clk <= '0';
        wait for 5 ns;
        clk <= '1';
        wait for 5 ns;
    end process clk_process;

    -- Stimulus
    stimulus: process
    begin
        -- Initial reset
        reset <= '1';
        wait for 15 ns;

        -- De-assert reset
        reset <= '0';
        wait for 100 ns;

        -- Pulse reset again
        reset <= '1';
        wait for 8 ns;
        reset <= '0';

        wait for 50 ns;
        wait;
    end process stimulus;
end architecture sim;
