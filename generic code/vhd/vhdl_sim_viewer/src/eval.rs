//! VHDL expression evaluation engine.

use crate::ast::*;
use crate::common::VhdlError;
use crate::value::{LogicVal, SimValue};
use std::collections::HashMap;

/// Scope containing signal/variable values and event tracking.
pub trait Scope {
    fn get_value(&self, name: &str) -> Option<SimValue>;
    fn had_event(&self, name: &str) -> bool;
    fn get_prev_value(&self, name: &str) -> Option<SimValue>;
    fn get_signal_width(&self, name: &str) -> Option<usize>;
}

/// Simple in-memory scope for local evaluation and testing.
#[derive(Debug, Clone, Default)]
pub struct SimpleScope {
    pub values: HashMap<String, SimValue>,
    pub prev_values: HashMap<String, SimValue>,
    pub events: HashMap<String, bool>,
    pub widths: HashMap<String, usize>,
}

impl Scope for SimpleScope {
    fn get_value(&self, name: &str) -> Option<SimValue> {
        self.values.get(name).cloned()
    }

    fn had_event(&self, name: &str) -> bool {
        self.events.get(name).copied().unwrap_or(false)
    }

    fn get_prev_value(&self, name: &str) -> Option<SimValue> {
        self.prev_values.get(name).cloned()
    }

    fn get_signal_width(&self, name: &str) -> Option<usize> {
        self.widths.get(name).copied()
    }
}

pub struct Evaluator<'a> {
    scope: &'a dyn Scope,
    target_width: Option<usize>,
}

impl<'a> Evaluator<'a> {
    pub fn new(scope: &'a dyn Scope) -> Self {
        Self {
            scope,
            target_width: None,
        }
    }

    pub fn with_target_width(scope: &'a dyn Scope, width: usize) -> Self {
        Self {
            scope,
            target_width: Some(width),
        }
    }

    pub fn eval(&self, expr: &Expr) -> Result<SimValue, VhdlError> {
        match expr {
            Expr::LitLogic(lv, _) => Ok(SimValue::Logic(*lv)),
            Expr::LitVector(bits, _) => Ok(SimValue::LogicVector(bits.clone())),
            Expr::LitInt(i, _) => Ok(SimValue::Integer(*i)),
            Expr::LitTime(fs, _) => Ok(SimValue::Time(*fs)),
            Expr::LitString(s, _) => Ok(SimValue::String(s.clone())),
            Expr::LitBool(b, _) => Ok(SimValue::Boolean(*b)),
            Expr::Ident(name, _span) => {
                let lower = name.to_ascii_lowercase();
                if let Some(val) = self.scope.get_value(&lower) {
                    Ok(val)
                } else {
                    // Check if it's an enum literal
                    Ok(SimValue::Enum(lower))
                }
            }
            Expr::UnaryOp(op, sub, _span) => {
                let val = self.eval(sub)?;
                match op {
                    UnaryOpKind::Not => match val {
                        SimValue::Logic(l) => Ok(SimValue::Logic(l.not())),
                        SimValue::Boolean(b) => Ok(SimValue::Boolean(!b)),
                        SimValue::LogicVector(vec) => {
                            let notted: Vec<LogicVal> = vec.iter().map(|b| b.not()).collect();
                            Ok(SimValue::LogicVector(notted))
                        }
                        _ => Err(VhdlError::SimError {
                            msg: format!("Cannot apply 'not' to {val}"),
                            time_fs: 0,
                        }),
                    },
                    UnaryOpKind::Neg => match val {
                        SimValue::Integer(i) => Ok(SimValue::Integer(-i)),
                        _ => Err(VhdlError::SimError {
                            msg: format!("Cannot negate non-integer {val}"),
                            time_fs: 0,
                        }),
                    },
                    UnaryOpKind::Pos => match val {
                        SimValue::Integer(i) => Ok(SimValue::Integer(i)),
                        _ => Err(VhdlError::SimError {
                            msg: format!("Cannot apply unary plus to {val}"),
                            time_fs: 0,
                        }),
                    },
                    UnaryOpKind::Abs => match val {
                        SimValue::Integer(i) => Ok(SimValue::Integer(i.abs())),
                        _ => Err(VhdlError::SimError {
                            msg: format!("Cannot apply abs to {val}"),
                            time_fs: 0,
                        }),
                    },
                }
            }
            Expr::BinaryOp(op, lhs, rhs, _span) => {
                let left = self.eval(lhs)?;
                let right = self.eval(rhs)?;
                self.eval_binary_op(*op, left, right)
            }
            Expr::Index(name, idx_expr, _span) => {
                let idx_val = self.eval(idx_expr)?;
                let idx = idx_val.as_i64().unwrap_or(0);

                let target_val = self.scope.get_value(name).ok_or_else(|| VhdlError::SimError {
                    msg: format!("Undefined signal/variable '{name}'"),
                    time_fs: 0,
                })?;

                match target_val {
                    SimValue::LogicVector(bits) => {
                        let len = bits.len() as i64;
                        // In VHDL, indexing (N downto 0) has index N at position 0, 0 at position N
                        // If index is within [0, len-1]:
                        if idx >= 0 && idx < len {
                            let bit_pos = (len - 1 - idx) as usize;
                            Ok(SimValue::Logic(bits[bit_pos]))
                        } else {
                            Err(VhdlError::SimError {
                                msg: format!("Index {idx} out of range for vector {name} of length {len}"),
                                time_fs: 0,
                            })
                        }
                    }
                    SimValue::Array(elems) => {
                        let len = elems.len() as i64;
                        if idx >= 0 && idx < len {
                            Ok(elems[idx as usize].clone())
                        } else {
                            Ok(elems.first().cloned().unwrap_or_default())
                        }
                    }
                    _ => Err(VhdlError::SimError {
                        msg: format!("Cannot index non-vector {name}"),
                        time_fs: 0,
                    }),
                }
            }
            Expr::Slice(name, range, _span) => {
                let target_val = self.scope.get_value(name).ok_or_else(|| VhdlError::SimError {
                    msg: format!("Undefined signal/variable '{name}'"),
                    time_fs: 0,
                })?;
                let left_val = self.eval(&range.left)?;
                let right_val = self.eval(&range.right)?;
                let left = left_val.as_i64().unwrap_or(0);
                let right = right_val.as_i64().unwrap_or(0);

                match target_val {
                    SimValue::LogicVector(bits) => {
                        let len = bits.len() as i64;
                        let mut sliced = Vec::new();
                        match range.direction {
                            RangeDirection::Downto => {
                                for i in (right..=left).rev() {
                                    if i >= 0 && i < len {
                                        let bit_pos = (len - 1 - i) as usize;
                                        sliced.push(bits[bit_pos]);
                                    } else {
                                        sliced.push(LogicVal::X);
                                    }
                                }
                            }
                            RangeDirection::To => {
                                for i in left..=right {
                                    if i >= 0 && i < len {
                                        let bit_pos = (len - 1 - i) as usize;
                                        sliced.push(bits[bit_pos]);
                                    } else {
                                        sliced.push(LogicVal::X);
                                    }
                                }
                            }
                        }
                        Ok(SimValue::LogicVector(sliced))
                    }
                    _ => Err(VhdlError::SimError {
                        msg: format!("Cannot slice non-vector {name}"),
                        time_fs: 0,
                    }),
                }
            }
            Expr::FunctionCall(name, args, _span) => self.eval_function_call(name, args),
            Expr::TypeCast(typ, expr, _span) => {
                let val = self.eval(expr)?;
                match typ.as_str() {
                    "std_logic_vector" | "unsigned" | "signed" => match val {
                        SimValue::LogicVector(_) => Ok(val),
                        SimValue::Integer(i) => {
                            let w = self.target_width.unwrap_or(32);
                            let mut bits = Vec::with_capacity(w);
                            for bit_idx in (0..w).rev() {
                                let b = ((i >> bit_idx) & 1) == 1;
                                bits.push(if b { LogicVal::One } else { LogicVal::Zero });
                            }
                            Ok(SimValue::LogicVector(bits))
                        }
                        SimValue::Logic(l) => Ok(SimValue::LogicVector(vec![l])),
                        _ => Ok(val),
                    },
                    _ => Ok(val),
                }
            }
            Expr::Aggregate(choices, _span) => {
                let width = self.target_width.unwrap_or(1);
                let mut bits = vec![LogicVal::U; width];

                for choice in choices {
                    match choice {
                        AggregateChoice::Others(val_expr) => {
                            let val = self.eval(val_expr)?;
                            let lv = match val {
                                SimValue::Logic(l) => l,
                                _ => LogicVal::Zero,
                            };
                            for b in &mut bits {
                                if *b == LogicVal::U {
                                    *b = lv;
                                }
                            }
                        }
                        AggregateChoice::Index(idx_expr, val_expr) => {
                            let idx = self.eval(idx_expr)?.as_i64().unwrap_or(0);
                            let val = self.eval(val_expr)?;
                            let lv = match val {
                                SimValue::Logic(l) => l,
                                _ => LogicVal::Zero,
                            };
                            let len = width as i64;
                            if idx >= 0 && idx < len {
                                let pos = (len - 1 - idx) as usize;
                                bits[pos] = lv;
                            }
                        }
                        AggregateChoice::Positional(val_expr) => {
                            let val = self.eval(val_expr)?;
                            let lv = match val {
                                SimValue::Logic(l) => l,
                                _ => LogicVal::Zero,
                            };
                            // Positional fills from left to right
                            if let Some(slot) = bits.iter_mut().find(|b| **b == LogicVal::U) {
                                *slot = lv;
                            }
                        }
                    }
                }
                Ok(SimValue::LogicVector(bits))
            }
        }
    }

    fn eval_function_call(&self, name: &str, args: &[Expr]) -> Result<SimValue, VhdlError> {
        let lower = name.to_ascii_lowercase();

        if lower == "rising_edge" {
            if args.is_empty() {
                return Err(VhdlError::SimError {
                    msg: "rising_edge requires 1 argument".to_string(),
                    time_fs: 0,
                });
            }
            if let Expr::Ident(ref sig_name, _) = args[0] {
                let curr = self.scope.get_value(sig_name);
                let prev = self.scope.get_prev_value(sig_name);
                let had_ev = self.scope.had_event(sig_name);

                let is_rise = had_ev
                    && matches!(
                        (prev, curr),
                        (
                            Some(SimValue::Logic(LogicVal::Zero | LogicVal::L)),
                            Some(SimValue::Logic(LogicVal::One | LogicVal::H))
                        )
                    );
                return Ok(SimValue::Boolean(is_rise));
            } else {
                let val = self.eval(&args[0])?;
                return Ok(SimValue::Boolean(val.as_bool().unwrap_or(false)));
            }
        }

        if lower == "falling_edge" {
            if args.is_empty() {
                return Err(VhdlError::SimError {
                    msg: "falling_edge requires 1 argument".to_string(),
                    time_fs: 0,
                });
            }
            if let Expr::Ident(ref sig_name, _) = args[0] {
                let curr = self.scope.get_value(sig_name);
                let prev = self.scope.get_prev_value(sig_name);
                let had_ev = self.scope.had_event(sig_name);

                let is_fall = had_ev
                    && matches!(
                        (prev, curr),
                        (
                            Some(SimValue::Logic(LogicVal::One | LogicVal::H)),
                            Some(SimValue::Logic(LogicVal::Zero | LogicVal::L))
                        )
                    );
                return Ok(SimValue::Boolean(is_fall));
            } else {
                return Ok(SimValue::Boolean(false));
            }
        }

        if lower.ends_with("'event") {
            let sig_name = lower.trim_end_matches("'event");
            let had_ev = self.scope.had_event(sig_name);
            return Ok(SimValue::Boolean(had_ev));
        }

        if lower == "to_integer" {
            if !args.is_empty() {
                let val = self.eval(&args[0])?;
                let i = val.as_i64().unwrap_or(0);
                return Ok(SimValue::Integer(i));
            }
        }

        if lower == "to_unsigned" || lower == "to_signed" {
            if args.len() >= 2 {
                let val = self.eval(&args[0])?.as_i64().unwrap_or(0);
                let width = self.eval(&args[1])?.as_i64().unwrap_or(32) as usize;
                let mut bits = Vec::with_capacity(width);
                for i in (0..width).rev() {
                    let b = ((val >> i) & 1) == 1;
                    bits.push(if b { LogicVal::One } else { LogicVal::Zero });
                }
                return Ok(SimValue::LogicVector(bits));
            }
        }

        if lower == "resize" {
            if args.len() >= 2 {
                let val = self.eval(&args[0])?;
                let width = self.eval(&args[1])?.as_i64().unwrap_or(8) as usize;
                if let SimValue::LogicVector(bits) = val {
                    let new_bits = bits;
                    if new_bits.len() < width {
                        let pad = width - new_bits.len();
                        let mut padded = vec![LogicVal::Zero; pad];
                        padded.extend(new_bits);
                        return Ok(SimValue::LogicVector(padded));
                    } else if new_bits.len() > width {
                        let start = new_bits.len() - width;
                        return Ok(SimValue::LogicVector(new_bits[start..].to_vec()));
                    } else {
                        return Ok(SimValue::LogicVector(new_bits));
                    }
                }
            }
        }

        // Default: try to evaluate identifier or first argument
        if !args.is_empty() {
            self.eval(&args[0])
        } else {
            Ok(SimValue::Null)
        }
    }

    fn eval_binary_op(
        &self,
        op: BinaryOpKind,
        left: SimValue,
        right: SimValue,
    ) -> Result<SimValue, VhdlError> {
        // Concatenation
        if op == BinaryOpKind::Concat {
            return left.concat(&right).map_err(|msg| VhdlError::SimError { msg, time_fs: 0 });
        }

        // Logic operations on LogicVal
        if let (SimValue::Logic(l1), SimValue::Logic(l2)) = (&left, &right) {
            match op {
                BinaryOpKind::And => return Ok(SimValue::Logic(l1.and(*l2))),
                BinaryOpKind::Or => return Ok(SimValue::Logic(l1.or(*l2))),
                BinaryOpKind::Nand => return Ok(SimValue::Logic(l1.nand(*l2))),
                BinaryOpKind::Nor => return Ok(SimValue::Logic(l1.nor(*l2))),
                BinaryOpKind::Xor => return Ok(SimValue::Logic(l1.xor(*l2))),
                BinaryOpKind::Xnor => return Ok(SimValue::Logic(l1.xnor(*l2))),
                BinaryOpKind::Eq => return Ok(SimValue::Boolean(l1 == l2)),
                BinaryOpKind::Ne => return Ok(SimValue::Boolean(l1 != l2)),
                _ => {}
            }
        }

        // Logic operations on LogicVector
        if let (SimValue::LogicVector(v1), SimValue::LogicVector(v2)) = (&left, &right) {
            let len = v1.len().min(v2.len());
            match op {
                BinaryOpKind::And => {
                    let res = (0..len).map(|i| v1[i].and(v2[i])).collect();
                    return Ok(SimValue::LogicVector(res));
                }
                BinaryOpKind::Or => {
                    let res = (0..len).map(|i| v1[i].or(v2[i])).collect();
                    return Ok(SimValue::LogicVector(res));
                }
                BinaryOpKind::Nand => {
                    let res = (0..len).map(|i| v1[i].nand(v2[i])).collect();
                    return Ok(SimValue::LogicVector(res));
                }
                BinaryOpKind::Nor => {
                    let res = (0..len).map(|i| v1[i].nor(v2[i])).collect();
                    return Ok(SimValue::LogicVector(res));
                }
                BinaryOpKind::Xor => {
                    let res = (0..len).map(|i| v1[i].xor(v2[i])).collect();
                    return Ok(SimValue::LogicVector(res));
                }
                BinaryOpKind::Xnor => {
                    let res = (0..len).map(|i| v1[i].xnor(v2[i])).collect();
                    return Ok(SimValue::LogicVector(res));
                }
                BinaryOpKind::Eq => return Ok(SimValue::Boolean(v1 == v2)),
                BinaryOpKind::Ne => return Ok(SimValue::Boolean(v1 != v2)),
                _ => {}
            }
        }

        // Boolean operations
        if let (SimValue::Boolean(b1), SimValue::Boolean(b2)) = (&left, &right) {
            match op {
                BinaryOpKind::And => return Ok(SimValue::Boolean(*b1 && *b2)),
                BinaryOpKind::Or => return Ok(SimValue::Boolean(*b1 || *b2)),
                BinaryOpKind::Nand => return Ok(SimValue::Boolean(!(*b1 && *b2))),
                BinaryOpKind::Nor => return Ok(SimValue::Boolean(!(*b1 || *b2))),
                BinaryOpKind::Xor => return Ok(SimValue::Boolean(*b1 ^ *b2)),
                BinaryOpKind::Xnor => return Ok(SimValue::Boolean(!(*b1 ^ *b2))),
                BinaryOpKind::Eq => return Ok(SimValue::Boolean(b1 == b2)),
                BinaryOpKind::Ne => return Ok(SimValue::Boolean(b1 != b2)),
                _ => {}
            }
        }

        // Integer arithmetic or Vector arithmetic treated as integer
        if let (Some(n1), Some(n2)) = (left.as_i64(), right.as_i64()) {
            match op {
                BinaryOpKind::Add => {
                    let sum = n1.wrapping_add(n2);
                    if let SimValue::LogicVector(v) = &left {
                        let w = v.len();
                        let mut bits = Vec::with_capacity(w);
                        for i in (0..w).rev() {
                            let b = ((sum >> i) & 1) == 1;
                            bits.push(if b { LogicVal::One } else { LogicVal::Zero });
                        }
                        return Ok(SimValue::LogicVector(bits));
                    }
                    return Ok(SimValue::Integer(sum));
                }
                BinaryOpKind::Sub => {
                    let diff = n1.wrapping_sub(n2);
                    if let SimValue::LogicVector(v) = &left {
                        let w = v.len();
                        let mut bits = Vec::with_capacity(w);
                        for i in (0..w).rev() {
                            let b = ((diff >> i) & 1) == 1;
                            bits.push(if b { LogicVal::One } else { LogicVal::Zero });
                        }
                        return Ok(SimValue::LogicVector(bits));
                    }
                    return Ok(SimValue::Integer(diff));
                }
                BinaryOpKind::Mul => return Ok(SimValue::Integer(n1.wrapping_mul(n2))),
                BinaryOpKind::Div => {
                    if n2 == 0 {
                        return Err(VhdlError::SimError {
                            msg: "Division by zero".to_string(),
                            time_fs: 0,
                        });
                    }
                    return Ok(SimValue::Integer(n1 / n2));
                }
                BinaryOpKind::Mod => {
                    if n2 == 0 {
                        return Err(VhdlError::SimError {
                            msg: "Modulo by zero".to_string(),
                            time_fs: 0,
                        });
                    }
                    return Ok(SimValue::Integer(n1 % n2));
                }
                BinaryOpKind::Rem => {
                    if n2 == 0 {
                        return Err(VhdlError::SimError {
                            msg: "Rem by zero".to_string(),
                            time_fs: 0,
                        });
                    }
                    return Ok(SimValue::Integer(n1 % n2));
                }
                BinaryOpKind::Eq => return Ok(SimValue::Boolean(n1 == n2)),
                BinaryOpKind::Ne => return Ok(SimValue::Boolean(n1 != n2)),
                BinaryOpKind::Lt => return Ok(SimValue::Boolean(n1 < n2)),
                BinaryOpKind::Le => return Ok(SimValue::Boolean(n1 <= n2)),
                BinaryOpKind::Gt => return Ok(SimValue::Boolean(n1 > n2)),
                BinaryOpKind::Ge => return Ok(SimValue::Boolean(n1 >= n2)),
                BinaryOpKind::Sll => return Ok(SimValue::Integer(n1 << n2)),
                BinaryOpKind::Srl => return Ok(SimValue::Integer((n1 as u64 >> n2) as i64)),
                _ => {}
            }
        }

        // Comparison of Enums or other SimValues
        match op {
            BinaryOpKind::Eq => Ok(SimValue::Boolean(left == right)),
            BinaryOpKind::Ne => Ok(SimValue::Boolean(left != right)),
            _ => Err(VhdlError::SimError {
                msg: format!("Unsupported binary operation {:?} between {left} and {right}", op),
                time_fs: 0,
            }),
        }
    }
}
