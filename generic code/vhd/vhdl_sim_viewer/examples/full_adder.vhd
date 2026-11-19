-- 1-bit Full Adder Circuit
library ieee;
use ieee.std_logic_1164.all;

entity FullAdder is
    port (
        a    : in  std_logic;
        b    : in  std_logic;
        cin  : in  std_logic;
        sum  : out std_logic;
        cout : out std_logic
    );
end entity FullAdder;

architecture rtl of FullAdder is
begin
    sum  <= (a xor b) xor cin;
    cout <= (a and b) or (cin and (a xor b));
end architecture rtl;
