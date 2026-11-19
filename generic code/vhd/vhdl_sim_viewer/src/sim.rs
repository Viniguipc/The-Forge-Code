//! Discrete-Event Simulation Engine conforming to IEEE 1076 simulation semantics.

use crate::ast::*;
use crate::common::VhdlError;
use crate::elaboration::ElabModel;
use crate::eval::{Evaluator, Scope};
use crate::value::SimValue;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone)]
pub struct SignalState {
    pub name: String,
    pub width: usize,
    pub curr_val: SimValue,
    pub prev_val: SimValue,
    pub had_event: bool,
    pub history: Vec<(u64, SimValue)>, // (time_fs, value)
}

#[derive(Debug, Clone, PartialEq)]
pub enum WaitState {
    Active,
    SuspendedSensitivity(Vec<String>),
    SuspendedUntilTime(u64),
    SuspendedUntilCond(Expr),
    SuspendedOnSignals(Vec<String>),
    SuspendedForever,
}

#[derive(Debug, Clone)]
pub enum StmtFrame {
    Block {
        stmts: Vec<SeqStmt>,
        pc: usize,
    },
    ForLoop {
        var_name: String,
        curr: i64,
        end: i64,
        step: i64,
        body: Vec<SeqStmt>,
        pc: usize,
    },
    WhileLoop {
        cond: Expr,
        body: Vec<SeqStmt>,
        pc: usize,
    },
}

#[derive(Debug, Clone)]
pub struct ProcessRuntime {
    pub name: String,
    pub sensitivity: Option<Vec<String>>,
    pub var_scope: HashMap<String, SimValue>,
    pub initial_body: Vec<SeqStmt>,
    pub call_stack: Vec<StmtFrame>,
    pub wait_state: WaitState,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub sig_name: String,
    pub value: SimValue,
    pub time_fs: u64,
    pub delta: usize,
}

/// Simulation Scope providing values to expression evaluator.
pub struct SimScope<'a> {
    pub signals: &'a HashMap<String, SignalState>,
    pub local_vars: &'a HashMap<String, SimValue>,
    pub constants: &'a HashMap<String, SimValue>,
}

impl<'a> Scope for SimScope<'a> {
    fn get_value(&self, name: &str) -> Option<SimValue> {
        let lower = name.to_ascii_lowercase();
        if let Some(v) = self.local_vars.get(&lower) {
            return Some(v.clone());
        }
        if let Some(c) = self.constants.get(&lower) {
            return Some(c.clone());
        }
        if let Some(sig) = self.signals.get(&lower) {
            return Some(sig.curr_val.clone());
        }
        None
    }

    fn had_event(&self, name: &str) -> bool {
        let lower = name.to_ascii_lowercase();
        self.signals.get(&lower).map_or(false, |s| s.had_event)
    }

    fn get_prev_value(&self, name: &str) -> Option<SimValue> {
        let lower = name.to_ascii_lowercase();
        self.signals.get(&lower).map(|s| s.prev_val.clone())
    }

    fn get_signal_width(&self, name: &str) -> Option<usize> {
        let lower = name.to_ascii_lowercase();
        self.signals.get(&lower).map(|s| s.width)
    }
}

pub struct Simulator {
    pub current_time_fs: u64,
    pub current_delta: usize,
    pub signals: HashMap<String, SignalState>,
    pub constants: HashMap<String, SimValue>,
    pub processes: Vec<ProcessRuntime>,
    pub event_queue: VecDeque<Transaction>,
    pub max_delta_cycles: usize,
    pub logs: Vec<String>,
}

impl Simulator {
    pub fn new(model: ElabModel) -> Self {
        let mut signals = HashMap::new();
        for sig in model.signals {
            signals.insert(
                sig.name.clone(),
                SignalState {
                    name: sig.name,
                    width: sig.width,
                    curr_val: sig.init_val.clone(),
                    prev_val: sig.init_val.clone(),
                    had_event: false,
                    history: vec![(0, sig.init_val)],
                },
            );
        }

        let mut processes = Vec::new();
        for proc in model.processes {
            let stack = vec![StmtFrame::Block {
                stmts: proc.body.clone(),
                pc: 0,
            }];
            processes.push(ProcessRuntime {
                name: proc.name,
                sensitivity: proc.sensitivity,
                var_scope: proc.var_decls,
                initial_body: proc.body,
                call_stack: stack,
                wait_state: WaitState::Active,
            });
        }

        Self {
            current_time_fs: 0,
            current_delta: 0,
            signals,
            constants: model.constants,
            processes,
            event_queue: VecDeque::new(),
            max_delta_cycles: 10_000,
            logs: Vec::new(),
        }
    }

    /// Run simulation up to stop_time_fs
    pub fn run_until(&mut self, stop_time_fs: u64) -> Result<(), VhdlError> {
        // Initialization Phase (T = 0, delta = 0)
        self.current_time_fs = 0;
        self.current_delta = 0;

        // Execute all active processes initially
        let active_indices: Vec<usize> = (0..self.processes.len()).collect();
        self.execute_active_processes(&active_indices)?;

        // Simulation Cycle Loop
        while self.current_time_fs <= stop_time_fs {
            // Check delta transactions at current time and current delta + 1
            let next_delta = self.current_delta + 1;
            let has_delta_events = self
                .event_queue
                .iter()
                .any(|tx| tx.time_fs == self.current_time_fs && tx.delta == next_delta);

            if has_delta_events {
                self.current_delta = next_delta;
                if self.current_delta > self.max_delta_cycles {
                    return Err(VhdlError::SimError {
                        msg: format!(
                            "Delta cycle limit ({}) exceeded at time {}. Probable combinational loop.",
                            self.max_delta_cycles, self.current_time_fs
                        ),
                        time_fs: self.current_time_fs,
                    });
                }

                // Apply scheduled transactions for this delta
                let active = self.apply_transactions_for_current_step()?;
                if !active.is_empty() {
                    self.execute_active_processes(&active)?;
                }
                continue;
            }

            // Delta cycles for current time have converged.
            // Reset delta and clear had_event flags.
            self.current_delta = 0;
            for sig in self.signals.values_mut() {
                sig.had_event = false;
            }

            // Find earliest next scheduled time
            let mut next_time = u64::MAX;

            for tx in &self.event_queue {
                if tx.time_fs > self.current_time_fs && tx.time_fs < next_time {
                    next_time = tx.time_fs;
                }
            }

            for p in &self.processes {
                if let WaitState::SuspendedUntilTime(wake_time) = p.wait_state {
                    if wake_time > self.current_time_fs && wake_time < next_time {
                        next_time = wake_time;
                    }
                }
            }

            if next_time == u64::MAX || next_time > stop_time_fs {
                // No more events within time limit
                break;
            }

            // Advance time
            self.current_time_fs = next_time;
            self.current_delta = 0;

            // Wake up processes whose wait-for expired
            let mut awakened = Vec::new();
            for (idx, p) in self.processes.iter_mut().enumerate() {
                if let WaitState::SuspendedUntilTime(wake_time) = p.wait_state {
                    if wake_time <= self.current_time_fs {
                        p.wait_state = WaitState::Active;
                        awakened.push(idx);
                    }
                }
            }

            // Apply transactions for current_time_fs at delta 0
            let tx_active = self.apply_transactions_for_current_step()?;
            for idx in tx_active {
                if !awakened.contains(&idx) {
                    awakened.push(idx);
                }
            }

            if !awakened.is_empty() {
                self.execute_active_processes(&awakened)?;
            }
        }

        // Post-simulation final sample recording
        for sig in self.signals.values_mut() {
            if let Some(last) = sig.history.last() {
                if last.0 < self.current_time_fs {
                    let val = sig.curr_val.clone();
                    sig.history.push((self.current_time_fs, val));
                }
            }
        }

        Ok(())
    }

    fn apply_transactions_for_current_step(&mut self) -> Result<Vec<usize>, VhdlError> {
        let mut changed_signals = HashSet::new();

        // Extract transactions matching current time and delta
        let mut i = 0;
        while i < self.event_queue.len() {
            if self.event_queue[i].time_fs == self.current_time_fs
                && self.event_queue[i].delta == self.current_delta
            {
                let tx = self.event_queue.remove(i).unwrap();
                let lower_name = tx.sig_name.to_ascii_lowercase();

                if let Some(sig) = self.signals.get_mut(&lower_name) {
                    if sig.curr_val != tx.value {
                        sig.prev_val = sig.curr_val.clone();
                        sig.curr_val = tx.value.clone();
                        sig.had_event = true;
                        sig.history.push((self.current_time_fs, tx.value));
                        changed_signals.insert(lower_name);
                    }
                }
            } else {
                i += 1;
            }
        }

        // Detect which processes to wake up based on changed signals
        let mut awakened = Vec::new();
        for (idx, p) in self.processes.iter_mut().enumerate() {
            let mut wake = false;
            match &p.wait_state {
                WaitState::SuspendedSensitivity(sens) => {
                    for s in sens {
                        if changed_signals.contains(s) {
                            wake = true;
                            break;
                        }
                    }
                }
                WaitState::SuspendedOnSignals(sigs) => {
                    for s in sigs {
                        if changed_signals.contains(s) {
                            wake = true;
                            break;
                        }
                    }
                }
                WaitState::SuspendedUntilCond(cond) => {
                    // Check if any changed signal is referenced, and if cond is true
                    let scope = SimScope {
                        signals: &self.signals,
                        local_vars: &p.var_scope,
                        constants: &self.constants,
                    };
                    let eval = Evaluator::new(&scope);
                    if let Ok(val) = eval.eval(cond) {
                        if val.as_bool().unwrap_or(false) {
                            wake = true;
                        }
                    }
                }
                WaitState::Active => {
                    wake = true;
                }
                WaitState::SuspendedUntilTime(_) | WaitState::SuspendedForever => {}
            }

            if wake {
                p.wait_state = WaitState::Active;
                awakened.push(idx);
            }
        }

        Ok(awakened)
    }

    fn execute_active_processes(&mut self, active_indices: &[usize]) -> Result<(), VhdlError> {
        for &idx in active_indices {
            self.execute_process(idx)?;
        }
        Ok(())
    }

    fn execute_process(&mut self, proc_idx: usize) -> Result<(), VhdlError> {
        let mut new_transactions = Vec::new();
        let mut new_logs = Vec::new();

        let current_time_fs = self.current_time_fs;
        let current_delta = self.current_delta;
        let signals = &self.signals;
        let constants = &self.constants;

        {
            let proc = &mut self.processes[proc_idx];
            if proc.wait_state != WaitState::Active {
                return Ok(());
            }

            let mut steps_budget = 100_000usize;

            while !proc.call_stack.is_empty() && proc.wait_state == WaitState::Active {
                steps_budget = steps_budget.saturating_sub(1);
                if steps_budget == 0 {
                    return Err(VhdlError::SimError {
                        msg: format!(
                            "Execution step limit exceeded in process '{}'. Possible infinite loop without wait.",
                            proc.name
                        ),
                        time_fs: current_time_fs,
                    });
                }

                let last_idx = proc.call_stack.len() - 1;

                let is_block_done = match &proc.call_stack[last_idx] {
                    StmtFrame::Block { stmts, pc } => *pc >= stmts.len(),
                    _ => false,
                };
                if is_block_done {
                    if last_idx == 0 {
                        if let Some(sens) = &proc.sensitivity {
                            proc.wait_state = WaitState::SuspendedSensitivity(sens.clone());
                            if let StmtFrame::Block { pc, .. } = &mut proc.call_stack[0] {
                                *pc = 0;
                            }
                            break;
                        } else {
                            if let StmtFrame::Block { pc, .. } = &mut proc.call_stack[0] {
                                *pc = 0;
                            }
                            continue;
                        }
                    } else {
                        proc.call_stack.pop();
                        continue;
                    }
                }

                match &mut proc.call_stack[last_idx] {
                    StmtFrame::Block { stmts, pc } => {
                        let stmt = stmts[*pc].clone();
                        *pc += 1;

                        Self::execute_seq_stmt(
                            signals,
                            constants,
                            current_time_fs,
                            current_delta,
                            stmt,
                            proc,
                            &mut new_transactions,
                            &mut new_logs,
                        )?;
                    }
                    StmtFrame::ForLoop {
                        var_name,
                        curr,
                        end,
                        step,
                        body,
                        pc,
                    } => {
                        if *pc >= body.len() {
                            // Advance loop iteration
                            let var_n = var_name.clone();
                            let next_val = *curr + *step;
                            let continues = if *step > 0 {
                                next_val <= *end
                            } else {
                                next_val >= *end
                            };

                            if continues {
                                *curr = next_val;
                                *pc = 0;
                                proc.var_scope.insert(var_n, SimValue::Integer(next_val));
                            } else {
                                proc.call_stack.pop();
                            }
                            continue;
                        }

                        let stmt = body[*pc].clone();
                        *pc += 1;

                        Self::execute_seq_stmt(
                            signals,
                            constants,
                            current_time_fs,
                            current_delta,
                            stmt,
                            proc,
                            &mut new_transactions,
                            &mut new_logs,
                        )?;
                    }
                    StmtFrame::WhileLoop { cond, body, pc } => {
                        if *pc >= body.len() {
                            *pc = 0;
                        }

                        if *pc == 0 {
                            let scope = SimScope {
                                signals,
                                local_vars: &proc.var_scope,
                                constants,
                            };
                            let eval = Evaluator::new(&scope);
                            let cond_val = eval.eval(cond)?;
                            if !cond_val.as_bool().unwrap_or(false) {
                                proc.call_stack.pop();
                                continue;
                            }
                        }

                        let stmt = body[*pc].clone();
                        *pc += 1;

                        Self::execute_seq_stmt(
                            signals,
                            constants,
                            current_time_fs,
                            current_delta,
                            stmt,
                            proc,
                            &mut new_transactions,
                            &mut new_logs,
                        )?;
                    }
                }
            }

            // Check if call stack emptied (process reached the end of its body)
            if proc.call_stack.is_empty() {
                if let Some(sens) = &proc.sensitivity {
                    // Process with sensitivity list suspends at end and resets stack
                    proc.wait_state = WaitState::SuspendedSensitivity(sens.clone());
                    proc.call_stack.push(StmtFrame::Block {
                        stmts: proc.initial_body.clone(),
                        pc: 0,
                    });
                } else if proc.wait_state == WaitState::Active {
                    // Process without sensitivity list loops back to beginning
                    proc.call_stack.push(StmtFrame::Block {
                        stmts: proc.initial_body.clone(),
                        pc: 0,
                    });
                }
            }
        }

        // Add generated transactions to simulator queue
        for tx in new_transactions {
            self.event_queue.push_back(tx);
        }
        self.logs.extend(new_logs);

        Ok(())
    }

    fn execute_seq_stmt(
        signals: &HashMap<String, SignalState>,
        constants: &HashMap<String, SimValue>,
        current_time_fs: u64,
        current_delta: usize,
        stmt: SeqStmt,
        proc: &mut ProcessRuntime,
        tx_out: &mut Vec<Transaction>,
        logs_out: &mut Vec<String>,
    ) -> Result<(), VhdlError> {
        let scope = SimScope {
            signals,
            local_vars: &proc.var_scope,
            constants,
        };

        match stmt {
            SeqStmt::SignalAssign {
                target,
                value,
                delay,
                span: _,
            } => {
                let target_name = match &target {
                    TargetExpr::Ident(n) => n.clone(),
                    TargetExpr::Index(n, _) => n.clone(),
                    TargetExpr::Slice(n, _) => n.clone(),
                };

                let target_width = scope.get_signal_width(&target_name).unwrap_or(1);
                let eval = Evaluator::with_target_width(&scope, target_width);
                let evaluated_val = eval.eval(&value)?;

                // Handle sub-element assignments (index or slice)
                let final_val = match &target {
                    TargetExpr::Ident(_) => evaluated_val,
                    TargetExpr::Index(_, idx_expr) => {
                        let idx = eval.eval(idx_expr)?.as_i64().unwrap_or(0);
                        let curr_sig = scope.get_value(&target_name).unwrap_or_default();
                        match curr_sig {
                            SimValue::LogicVector(mut bits) => {
                                let len = bits.len() as i64;
                                if idx >= 0 && idx < len {
                                    let pos = (len - 1 - idx) as usize;
                                    if let SimValue::Logic(lv) = evaluated_val {
                                        bits[pos] = lv;
                                    }
                                }
                                SimValue::LogicVector(bits)
                            }
                            SimValue::Array(mut elems) => {
                                if idx >= 0 && (idx as usize) < elems.len() {
                                    elems[idx as usize] = evaluated_val;
                                }
                                SimValue::Array(elems)
                            }
                            _ => evaluated_val,
                        }
                    }
                    TargetExpr::Slice(_, range) => {
                        let left = eval.eval(&range.left)?.as_i64().unwrap_or(0);
                        let right = eval.eval(&range.right)?.as_i64().unwrap_or(0);
                        let curr_sig = scope.get_value(&target_name).unwrap_or_default();
                        if let (SimValue::LogicVector(mut bits), SimValue::LogicVector(new_bits)) =
                            (curr_sig, evaluated_val.clone())
                        {
                            let len = bits.len() as i64;
                            let mut val_idx = 0;
                            match range.direction {
                                RangeDirection::Downto => {
                                    for i in (right..=left).rev() {
                                        if i >= 0 && i < len && val_idx < new_bits.len() {
                                            let pos = (len - 1 - i) as usize;
                                            bits[pos] = new_bits[val_idx];
                                            val_idx += 1;
                                        }
                                    }
                                }
                                RangeDirection::To => {
                                    for i in left..=right {
                                        if i >= 0 && i < len && val_idx < new_bits.len() {
                                            let pos = (len - 1 - i) as usize;
                                            bits[pos] = new_bits[val_idx];
                                            val_idx += 1;
                                        }
                                    }
                                }
                            }
                            SimValue::LogicVector(bits)
                        } else {
                            evaluated_val
                        }
                    }
                };

                let delay_fs = if let Some(d_expr) = delay {
                    eval.eval(&d_expr)?.as_i64().unwrap_or(0).max(0) as u64
                } else {
                    0
                };

                let (target_time, target_delta) = if delay_fs > 0 {
                    (current_time_fs + delay_fs, 0)
                } else {
                    (current_time_fs, current_delta + 1)
                };

                tx_out.push(Transaction {
                    sig_name: target_name,
                    value: final_val,
                    time_fs: target_time,
                    delta: target_delta,
                });
            }
            SeqStmt::VariableAssign {
                target,
                value,
                span: _,
            } => {
                let eval = Evaluator::new(&scope);
                let val = eval.eval(&value)?;
                match target {
                    TargetExpr::Ident(name) => {
                        proc.var_scope.insert(name.to_ascii_lowercase(), val);
                    }
                    _ => {}
                }
            }
            SeqStmt::If {
                cond,
                then_branch,
                elsif_branches,
                else_branch,
                span: _,
            } => {
                let eval = Evaluator::new(&scope);
                let cond_val = eval.eval(&cond)?;
                if cond_val.as_bool().unwrap_or(false) {
                    proc.call_stack.push(StmtFrame::Block {
                        stmts: then_branch,
                        pc: 0,
                    });
                    return Ok(());
                }

                for (el_cond, el_branch) in elsif_branches {
                    let c_val = eval.eval(&el_cond)?;
                    if c_val.as_bool().unwrap_or(false) {
                        proc.call_stack.push(StmtFrame::Block {
                            stmts: el_branch,
                            pc: 0,
                        });
                        return Ok(());
                    }
                }

                if let Some(el_branch) = else_branch {
                    proc.call_stack.push(StmtFrame::Block {
                        stmts: el_branch,
                        pc: 0,
                    });
                }
            }
            SeqStmt::Case { expr, arms, span: _ } => {
                let eval = Evaluator::new(&scope);
                let val = eval.eval(&expr)?;

                for arm in arms {
                    let mut matched = false;
                    for choice in &arm.choices {
                        match choice {
                            Choice::Expr(c_expr) => {
                                let c_val = eval.eval(c_expr)?;
                                if c_val == val {
                                    matched = true;
                                    break;
                                }
                            }
                            Choice::Range(range) => {
                                let left = eval.eval(&range.left)?.as_i64().unwrap_or(0);
                                let right = eval.eval(&range.right)?.as_i64().unwrap_or(0);
                                if let Some(i) = val.as_i64() {
                                    let (min, max) = (left.min(right), left.max(right));
                                    if i >= min && i <= max {
                                        matched = true;
                                        break;
                                    }
                                }
                            }
                            Choice::Others => {
                                matched = true;
                                break;
                            }
                        }
                    }

                    if matched {
                        proc.call_stack.push(StmtFrame::Block {
                            stmts: arm.stmts,
                            pc: 0,
                        });
                        break;
                    }
                }
            }
            SeqStmt::WaitFor { duration, span: _ } => {
                let eval = Evaluator::new(&scope);
                let dur_val = eval.eval(&duration)?;
                let dur_fs = match dur_val {
                    SimValue::Time(fs) => fs,
                    SimValue::Integer(i) => i.max(0) as u64,
                    _ => 0,
                };
                let wake_time = current_time_fs + dur_fs;
                proc.wait_state = WaitState::SuspendedUntilTime(wake_time);
            }
            SeqStmt::WaitUntil { cond, span: _ } => {
                proc.wait_state = WaitState::SuspendedUntilCond(cond);
            }
            SeqStmt::WaitOn { signals, span: _ } => {
                proc.wait_state = WaitState::SuspendedOnSignals(signals);
            }
            SeqStmt::Wait { span: _ } => {
                proc.wait_state = WaitState::SuspendedForever;
            }
            SeqStmt::Assert {
                cond,
                report,
                severity,
                span: _,
            } => {
                let eval = Evaluator::new(&scope);
                let val = eval.eval(&cond)?;
                if !val.as_bool().unwrap_or(false) {
                    let rep = report.unwrap_or_else(|| "Assertion condition failed".to_string());
                    let sev = severity.unwrap_or_else(|| "NOTE".to_string());
                    let time_ns = current_time_fs as f64 / 1_000_000.0;
                    let msg = format!("[ASSERT {sev} at {time_ns:.3} ns] {rep}");
                    println!("{msg}");
                    logs_out.push(msg);
                }
            }
            SeqStmt::ForLoop {
                var_name,
                range,
                body,
                span: _,
            } => {
                let eval = Evaluator::new(&scope);
                let left = eval.eval(&range.left)?.as_i64().unwrap_or(0);
                let right = eval.eval(&range.right)?.as_i64().unwrap_or(0);

                let (start, end, step) = match range.direction {
                    RangeDirection::Downto => (left, right, -1),
                    RangeDirection::To => (left, right, 1),
                };

                proc.var_scope
                    .insert(var_name.clone(), SimValue::Integer(start));

                proc.call_stack.push(StmtFrame::ForLoop {
                    var_name,
                    curr: start,
                    end,
                    step,
                    body,
                    pc: 0,
                });
            }
            SeqStmt::WhileLoop { cond, body, span: _ } => {
                proc.call_stack.push(StmtFrame::WhileLoop {
                    cond,
                    body,
                    pc: 0,
                });
            }
            SeqStmt::Null(_) => {}
        }

        Ok(())
    }
}
