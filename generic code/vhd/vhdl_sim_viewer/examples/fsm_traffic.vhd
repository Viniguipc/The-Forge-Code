-- Traffic Light Controller Finite State Machine (FSM)
library ieee;
use ieee.std_logic_1164.all;

entity TrafficLightFSM is
    port (
        clk    : in  std_logic;
        reset  : in  std_logic;
        red    : out std_logic;
        yellow : out std_logic;
        green  : out std_logic
    );
end entity TrafficLightFSM;

architecture rtl of TrafficLightFSM is
    type state_type is (s_red, s_green, s_yellow);
    signal current_state : state_type := s_red;
    signal next_state    : state_type := s_red;
begin
    -- State register process
    process(clk, reset)
    begin
        if reset = '1' then
            current_state <= s_red;
        elsif rising_edge(clk) then
            current_state <= next_state;
        end if;
    end process;

    -- Next state logic
    process(current_state)
    begin
        case current_state is
            when s_red =>
                next_state <= s_green;
            when s_green =>
                next_state <= s_yellow;
            when s_yellow =>
                next_state <= s_red;
            when others =>
                next_state <= s_red;
        end case;
    end process;

    -- Output logic
    red    <= '1' when current_state = s_red else '0';
    yellow <= '1' when current_state = s_yellow else '0';
    green  <= '1' when current_state = s_green else '0';

end architecture rtl;
