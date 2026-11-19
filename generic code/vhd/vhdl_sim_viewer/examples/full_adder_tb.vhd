-- Testbench for 1-bit Full Adder
library ieee;
use ieee.std_logic_1164.all;

entity FullAdder_tb is
end entity FullAdder_tb;

architecture sim of FullAdder_tb is
    component FullAdder is
        port (
            a    : in  std_logic;
            b    : in  std_logic;
            cin  : in  std_logic;
            sum  : out std_logic;
            cout : out std_logic
        );
    end component;

    signal a    : std_logic := '0';
    signal b    : std_logic := '0';
    signal cin  : std_logic := '0';
    signal sum  : std_logic;
    signal cout : std_logic;
begin
    -- Device Under Test
    dut: FullAdder port map (
        a    => a,
        b    => b,
        cin  => cin,
        sum  => sum,
        cout => cout
    );

    stimulus: process
    begin
        -- Test 0: 0 + 0 + 0 = 0 (cout 0)
        a <= '0'; b <= '0'; cin <= '0';
        wait for 10 ns;
        assert sum = '0' and cout = '0' report "Failed 0+0+0";

        -- Test 1: 0 + 0 + 1 = 1 (cout 0)
        cin <= '1';
        wait for 10 ns;
        assert sum = '1' and cout = '0' report "Failed 0+0+1";

        -- Test 2: 0 + 1 + 0 = 1 (cout 0)
        b <= '1'; cin <= '0';
        wait for 10 ns;
        assert sum = '1' and cout = '0' report "Failed 0+1+0";

        -- Test 3: 0 + 1 + 1 = 0 (cout 1)
        cin <= '1';
        wait for 10 ns;
        assert sum = '0' and cout = '1' report "Failed 0+1+1";

        -- Test 4: 1 + 0 + 0 = 1 (cout 0)
        a <= '1'; b <= '0'; cin <= '0';
        wait for 10 ns;
        assert sum = '1' and cout = '0' report "Failed 1+0+0";

        -- Test 5: 1 + 0 + 1 = 0 (cout 1)
        cin <= '1';
        wait for 10 ns;
        assert sum = '0' and cout = '1' report "Failed 1+0+1";

        -- Test 6: 1 + 1 + 0 = 0 (cout 1)
        b <= '1'; cin <= '0';
        wait for 10 ns;
        assert sum = '0' and cout = '1' report "Failed 1+1+0";

        -- Test 7: 1 + 1 + 1 = 1 (cout 1)
        cin <= '1';
        wait for 10 ns;
        assert sum = '1' and cout = '1' report "Failed 1+1+1";

        wait;
    end process stimulus;
end architecture sim;
