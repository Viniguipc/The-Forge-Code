//! Value representations, IEEE 1164 9-value logic, arithmetic and operations.

use std::fmt;

/// IEEE 1164 9-value logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum LogicVal {
    #[default]
    U, // Uninitialized
    X, // Forcing Unknown
    Zero, // '0' Forcing 0
    One,  // '1' Forcing 1
    Z, // High Impedance
    W, // Weak Unknown
    L, // Weak 0
    H, // Weak 1
    DontCare, // '-' Don't care
}

impl LogicVal {
    pub fn from_char(c: char) -> Option<Self> {
        match c.to_ascii_uppercase() {
            'U' => Some(LogicVal::U),
            'X' => Some(LogicVal::X),
            '0' => Some(LogicVal::Zero),
            '1' => Some(LogicVal::One),
            'Z' => Some(LogicVal::Z),
            'W' => Some(LogicVal::W),
            'L' => Some(LogicVal::L),
            'H' => Some(LogicVal::H),
            '-' => Some(LogicVal::DontCare),
            _ => None,
        }
    }

    pub fn to_char(self) -> char {
        match self {
            LogicVal::U => 'U',
            LogicVal::X => 'X',
            LogicVal::Zero => '0',
            LogicVal::One => '1',
            LogicVal::Z => 'Z',
            LogicVal::W => 'W',
            LogicVal::L => 'L',
            LogicVal::H => 'H',
            LogicVal::DontCare => '-',
        }
    }

    /// Convert to 2-value boolean if possible (0/L => false, 1/H => true)
    pub fn to_bool(self) -> Option<bool> {
        match self {
            LogicVal::Zero | LogicVal::L => Some(false),
            LogicVal::One | LogicVal::H => Some(true),
            _ => None,
        }
    }

    /// Logical NOT (IEEE 1164)
    pub fn not(self) -> Self {
        match self {
            LogicVal::Zero | LogicVal::L => LogicVal::One,
            LogicVal::One | LogicVal::H => LogicVal::Zero,
            _ => LogicVal::X,
        }
    }

    /// Logical AND (IEEE 1164)
    pub fn and(self, other: Self) -> Self {
        match (self, other) {
            (LogicVal::Zero | LogicVal::L, _) | (_, LogicVal::Zero | LogicVal::L) => LogicVal::Zero,
            (LogicVal::One | LogicVal::H, LogicVal::One | LogicVal::H) => LogicVal::One,
            _ => LogicVal::X,
        }
    }

    /// Logical OR (IEEE 1164)
    pub fn or(self, other: Self) -> Self {
        match (self, other) {
            (LogicVal::One | LogicVal::H, _) | (_, LogicVal::One | LogicVal::H) => LogicVal::One,
            (LogicVal::Zero | LogicVal::L, LogicVal::Zero | LogicVal::L) => LogicVal::Zero,
            _ => LogicVal::X,
        }
    }

    /// Logical NAND (IEEE 1164)
    pub fn nand(self, other: Self) -> Self {
        self.and(other).not()
    }

    /// Logical NOR (IEEE 1164)
    pub fn nor(self, other: Self) -> Self {
        self.or(other).not()
    }

    /// Logical XOR (IEEE 1164)
    pub fn xor(self, other: Self) -> Self {
        match (self.to_bool(), other.to_bool()) {
            (Some(a), Some(b)) => if a ^ b { LogicVal::One } else { LogicVal::Zero },
            _ => LogicVal::X,
        }
    }

    /// Logical XNOR (IEEE 1164)
    pub fn xnor(self, other: Self) -> Self {
        self.xor(other).not()
    }

    /// IEEE 1164 Resolution Table for multi-driver signals (wired-or/bus resolution)
    pub fn resolve(self, other: Self) -> Self {
        // Table 1164
        use LogicVal::*;
        match (self, other) {
            (U, _) | (_, U) => U,
            (X, _) | (_, X) => X,
            (Zero, Zero) | (Zero, L) | (L, Zero) | (Zero, W) | (W, Zero) => Zero,
            (One, One) | (One, H) | (H, One) | (One, W) | (W, One) => One,
            (Zero, One) | (One, Zero) => X,
            (Zero, H) | (H, Zero) => X,
            (One, L) | (L, One) => X,
            (Z, other) | (other, Z) => other,
            (W, W) => W,
            (W, L) | (L, W) => L,
            (W, H) | (H, W) => H,
            (L, L) => L,
            (H, H) => H,
            (L, H) | (H, L) => W,
            (DontCare, _) | (_, DontCare) => X,
        }
    }
}

impl fmt::Display for LogicVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

/// Dynamic Value in VHDL simulation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimValue {
    Logic(LogicVal),
    LogicVector(Vec<LogicVal>),
    Integer(i64),
    Boolean(bool),
    Time(u64), // femtoseconds
    Enum(String),
    String(String),
    Array(Vec<SimValue>),
    Null,
}

impl Default for SimValue {
    fn default() -> Self {
        SimValue::Logic(LogicVal::U)
    }
}

impl fmt::Display for SimValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SimValue::Logic(lv) => write!(f, "'{}'", lv.to_char()),
            SimValue::LogicVector(vec) => {
                let s: String = vec.iter().map(|v| v.to_char()).collect();
                write!(f, "\"{s}\"")
            }
            SimValue::Integer(i) => write!(f, "{i}"),
            SimValue::Boolean(b) => write!(f, "{}", if *b { "TRUE" } else { "FALSE" }),
            SimValue::Time(fs) => {
                if *fs >= 1_000_000_000_000_000 {
                    write!(f, "{} sec", *fs as f64 / 1_000_000_000_000_000.0)
                } else if *fs >= 1_000_000_000_000 {
                    write!(f, "{} ms", *fs as f64 / 1_000_000_000_000.0)
                } else if *fs >= 1_000_000_000 {
                    write!(f, "{} us", *fs as f64 / 1_000_000_000.0)
                } else if *fs >= 1_000_000 {
                    write!(f, "{} ns", *fs as f64 / 1_000_000.0)
                } else if *fs >= 1000 {
                    write!(f, "{} ps", *fs as f64 / 1000.0)
                } else {
                    write!(f, "{fs} fs")
                }
            }
            SimValue::Enum(s) => write!(f, "{s}"),
            SimValue::String(s) => write!(f, "\"{s}\""),
            SimValue::Array(arr) => write!(f, "[{} items]", arr.len()),
            SimValue::Null => write!(f, "NULL"),
        }
    }
}

impl SimValue {
    /// Format as hex string if it's a vector
    pub fn to_hex_string(&self) -> String {
        match self {
            SimValue::LogicVector(bits) => {
                if bits.is_empty() {
                    return "0".to_string();
                }
                // Group into 4 bits
                let mut hex = String::new();
                let len = bits.len();
                let chunks = (len + 3) / 4;
                for c in 0..chunks {
                    let start = len.saturating_sub((chunks - c) * 4);
                    let end = len.saturating_sub((chunks - c - 1) * 4);
                    let slice = &bits[start..end];
                    // check if any is X or U or Z
                    let has_special = slice.iter().find(|b| matches!(b, LogicVal::U | LogicVal::X | LogicVal::Z | LogicVal::W | LogicVal::DontCare));
                    if let Some(spec) = has_special {
                        hex.push(spec.to_char());
                    } else {
                        let mut val: u8 = 0;
                        for b in slice {
                            val = (val << 1) | if matches!(b, LogicVal::One | LogicVal::H) { 1 } else { 0 };
                        }
                        hex.push_str(&format!("{val:X}"));
                    }
                }
                hex
            }
            SimValue::Integer(i) => format!("{i:X}"),
            SimValue::Logic(l) => l.to_char().to_string(),
            _ => self.to_string(),
        }
    }

    /// Try to parse integer from vector or integer
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            SimValue::Integer(i) => Some(*i),
            SimValue::LogicVector(bits) => {
                let mut res: i64 = 0;
                for b in bits {
                    match b {
                        LogicVal::Zero | LogicVal::L => {
                            res <<= 1;
                        }
                        LogicVal::One | LogicVal::H => {
                            res = (res << 1) | 1;
                        }
                        _ => return None,
                    }
                }
                Some(res)
            }
            SimValue::Logic(LogicVal::One | LogicVal::H) => Some(1),
            SimValue::Logic(LogicVal::Zero | LogicVal::L) => Some(0),
            _ => None,
        }
    }

    /// Try to convert to bool
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            SimValue::Boolean(b) => Some(*b),
            SimValue::Logic(l) => l.to_bool(),
            SimValue::Integer(i) => Some(*i != 0),
            _ => None,
        }
    }

    /// Concatenate two values (mainly vectors and logic)
    pub fn concat(&self, other: &Self) -> Result<Self, String> {
        let mut bits = Vec::new();
        match self {
            SimValue::LogicVector(v) => bits.extend_from_slice(v),
            SimValue::Logic(l) => bits.push(*l),
            _ => return Err(format!("Cannot concatenate type {self}")),
        }
        match other {
            SimValue::LogicVector(v) => bits.extend_from_slice(v),
            SimValue::Logic(l) => bits.push(*l),
            _ => return Err(format!("Cannot concatenate type {other}")),
        }
        Ok(SimValue::LogicVector(bits))
    }
}
