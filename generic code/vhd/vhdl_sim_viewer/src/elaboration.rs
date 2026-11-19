//! Elaboration module: resolves design units, flattens components, and builds the simulation model.

use crate::ast::*;
use crate::common::VhdlError;
use crate::value::{LogicVal, SimValue};
use std::collections::{HashMap, HashSet};

/// Elaborated signal definition.
#[derive(Debug, Clone)]
pub struct ElabSignal {
    pub name: String,
    pub typ: TypeDefKind,
    pub width: usize,
    pub init_val: SimValue,
}

/// Elaborated process definition.
#[derive(Debug, Clone)]
pub struct ElabProcess {
    pub name: String,
    pub sensitivity: Option<Vec<String>>,
    pub var_decls: HashMap<String, SimValue>,
    pub body: Vec<SeqStmt>,
}

/// Elaborated model ready for simulation.
#[derive(Debug, Clone, Default)]
pub struct ElabModel {
    pub top_name: String,
    pub signals: Vec<ElabSignal>,
    pub signal_map: HashMap<String, usize>,
    pub processes: Vec<ElabProcess>,
    pub constants: HashMap<String, SimValue>,
}

impl ElabModel {
    pub fn get_signal_index(&self, name: &str) -> Option<usize> {
        self.signal_map.get(&name.to_ascii_lowercase()).copied()
    }
}

pub struct Elaborator {
    _files: Vec<DesignFile>,
    entities: HashMap<String, EntityDecl>,
    architectures: HashMap<String, Vec<ArchDecl>>,
}

impl Elaborator {
    pub fn new(files: Vec<DesignFile>) -> Self {
        let mut entities = HashMap::new();
        let mut architectures = HashMap::new();

        for file in &files {
            for unit in &file.units {
                match unit {
                    DesignUnit::Entity(ent) => {
                        entities.insert(ent.name.to_ascii_lowercase(), ent.clone());
                    }
                    DesignUnit::Architecture(arch) => {
                        let ent_key = arch.entity_name.to_ascii_lowercase();
                        architectures
                            .entry(ent_key)
                            .or_insert_with(Vec::new)
                            .push(arch.clone());
                    }
                }
            }
        }

        Self {
            _files: files,
            entities,
            architectures,
        }
    }

    /// Find default top-level entity if none provided
    pub fn find_default_top(&self) -> Result<String, VhdlError> {
        // Preference: testbench entity (no ports), or last declared entity
        for (name, ent) in &self.entities {
            if ent.ports.is_empty() {
                return Ok(name.clone());
            }
        }
        if let Some((name, _)) = self.entities.iter().last() {
            return Ok(name.clone());
        }
        Err(VhdlError::ElabError {
            msg: "No entities found in design files".to_string(),
        })
    }

    pub fn elaborate(&self, top_name: &str) -> Result<ElabModel, VhdlError> {
        let top_key = top_name.to_ascii_lowercase();
        let entity = self.entities.get(&top_key).ok_or_else(|| VhdlError::ElabError {
            msg: format!("Top entity '{top_name}' not found"),
        })?;

        let archs = self.architectures.get(&top_key).ok_or_else(|| VhdlError::ElabError {
            msg: format!("No architecture found for entity '{top_name}'"),
        })?;

        let arch = archs.last().unwrap(); // Use latest declared architecture

        let mut model = ElabModel {
            top_name: top_name.to_string(),
            signals: Vec::new(),
            signal_map: HashMap::new(),
            processes: Vec::new(),
            constants: HashMap::new(),
        };

        let empty_types = HashMap::new();

        // 1. Register Ports as Signals
        for port in &entity.ports {
            for name in &port.names {
                let sig_name = name.to_ascii_lowercase();
                let (width, init_val) = self.resolve_type_info(&port.typ, &port.default, &empty_types)?;
                let idx = model.signals.len();
                model.signals.push(ElabSignal {
                    name: sig_name.clone(),
                    typ: port.typ.clone(),
                    width,
                    init_val,
                });
                model.signal_map.insert(sig_name, idx);
            }
        }

        // 2. Elaborate Architecture Scope
        self.elaborate_architecture(arch, "", &mut model, &HashMap::new())?;

        Ok(model)
    }

    fn elaborate_architecture(
        &self,
        arch: &ArchDecl,
        prefix: &str,
        model: &mut ElabModel,
        port_bindings: &HashMap<String, String>,
    ) -> Result<(), VhdlError> {
        // Collect enum literals and custom type definitions
        let mut enum_literals = HashSet::new();
        let mut type_defs = HashMap::new();
        for decl in &arch.decls {
            if let Declaration::Type { name, kind, .. } = decl {
                type_defs.insert(name.to_ascii_lowercase(), kind.clone());
                if let TypeDefKind::Enum(variants) = kind {
                    for v in variants {
                        enum_literals.insert(v.to_ascii_lowercase());
                    }
                }
            }
        }

        // Component declarations in this architecture
        let mut comp_decls = HashMap::new();

        // Architectural signals and constants
        for decl in &arch.decls {
            match decl {
                Declaration::Signal { names, typ, default, .. } => {
                    for name in names {
                        let full_name = if prefix.is_empty() {
                            name.to_ascii_lowercase()
                        } else {
                            format!("{prefix}.{}", name.to_ascii_lowercase())
                        };
                        let (width, init_val) = self.resolve_type_info(typ, default, &type_defs)?;
                        let idx = model.signals.len();
                        model.signals.push(ElabSignal {
                            name: full_name.clone(),
                            typ: typ.clone(),
                            width,
                            init_val,
                        });
                        model.signal_map.insert(full_name, idx);
                    }
                }
                Declaration::Constant { names, typ, value, .. } => {
                    let (_, val) = self.resolve_type_info(typ, &Some(value.clone()), &type_defs)?;
                    for name in names {
                        let full_name = if prefix.is_empty() {
                            name.to_ascii_lowercase()
                        } else {
                            format!("{prefix}.{}", name.to_ascii_lowercase())
                        };
                        model.constants.insert(full_name, val.clone());
                    }
                }
                Declaration::Component { name, generics, ports, .. } => {
                    comp_decls.insert(name.to_ascii_lowercase(), (generics.clone(), ports.clone()));
                }
                _ => {}
            }
        }

        // Concurrent statements
        let mut proc_idx = 0;
        for stmt in &arch.stmts {
            match stmt {
                ConcurrentStmt::Process {
                    label,
                    sensitivity,
                    decls,
                    body,
                    span: _,
                } => {
                    proc_idx += 1;
                    let p_name = label.clone().unwrap_or_else(|| format!("{prefix}proc_{proc_idx}"));
                    let mut var_decls = HashMap::new();
                    for d in decls {
                        if let Declaration::Variable { names, typ, default, .. } = d {
                            for v_name in names {
                                let (_, init) = self.resolve_type_info(typ, default, &type_defs)?;
                                var_decls.insert(v_name.to_ascii_lowercase(), init);
                            }
                        }
                    }

                    // Map sensitivity signals with prefix / bindings
                    let mapped_sens = sensitivity.as_ref().map(|list| {
                        list.iter()
                            .map(|s| self.resolve_signal_ref(s, prefix, port_bindings, &enum_literals))
                            .collect()
                    });

                    // Rename signal references inside process body if prefix is active
                    let mapped_body = self.remap_body_signals(body, prefix, port_bindings, &enum_literals);

                    model.processes.push(ElabProcess {
                        name: p_name,
                        sensitivity: mapped_sens,
                        var_decls,
                        body: mapped_body,
                    });
                }
                ConcurrentStmt::SignalAssign {
                    target,
                    expr,
                    delay,
                    span,
                } => {
                    // Convert to process sensitive to all signals on RHS
                    proc_idx += 1;
                    let p_name = format!("{prefix}assign_{proc_idx}");
                    let mut rhs_signals = Vec::new();
                    self.collect_signals_in_expr(expr, &mut rhs_signals, &enum_literals);

                    let mapped_sens: Vec<String> = rhs_signals
                        .into_iter()
                        .map(|s| self.resolve_signal_ref(&s, prefix, port_bindings, &enum_literals))
                        .collect();

                    let mapped_target = self.remap_target(target, prefix, port_bindings, &enum_literals);
                    let mapped_expr = self.remap_expr(expr, prefix, port_bindings, &enum_literals);
                    let mapped_delay = delay.as_ref().map(|d| self.remap_expr(d, prefix, port_bindings, &enum_literals));

                    let body = vec![SeqStmt::SignalAssign {
                        target: mapped_target,
                        value: mapped_expr,
                        delay: mapped_delay,
                        span: *span,
                    }];

                    model.processes.push(ElabProcess {
                        name: p_name,
                        sensitivity: Some(mapped_sens),
                        var_decls: HashMap::new(),
                        body,
                    });
                }
                ConcurrentStmt::ConditionalSignalAssign {
                    target,
                    when_else,
                    default,
                    delay,
                    span,
                } => {
                    // Convert to process with if/elsif/else
                    proc_idx += 1;
                    let p_name = format!("{prefix}cond_assign_{proc_idx}");
                    let mut sens_set = HashSet::new();
                    for (val, cond) in when_else {
                        self.collect_signals_in_expr(val, &mut sens_set, &enum_literals);
                        self.collect_signals_in_expr(cond, &mut sens_set, &enum_literals);
                    }
                    self.collect_signals_in_expr(default, &mut sens_set, &enum_literals);

                    let mapped_sens: Vec<String> = sens_set
                        .into_iter()
                        .map(|s| self.resolve_signal_ref(&s, prefix, port_bindings, &enum_literals))
                        .collect();

                    let mapped_target = self.remap_target(target, prefix, port_bindings, &enum_literals);
                    let mapped_delay = delay.as_ref().map(|d| self.remap_expr(d, prefix, port_bindings, &enum_literals));

                    // Build If / Elsif / Else tree
                    let first = &when_else[0];
                    let first_cond = self.remap_expr(&first.1, prefix, port_bindings, &enum_literals);
                    let first_val = self.remap_expr(&first.0, prefix, port_bindings, &enum_literals);
                    let then_branch = vec![SeqStmt::SignalAssign {
                        target: mapped_target.clone(),
                        value: first_val,
                        delay: mapped_delay.clone(),
                        span: *span,
                    }];

                    let mut elsif_branches = Vec::new();
                    for (val, cond) in &when_else[1..] {
                        let c = self.remap_expr(cond, prefix, port_bindings, &enum_literals);
                        let v = self.remap_expr(val, prefix, port_bindings, &enum_literals);
                        elsif_branches.push((
                            c,
                            vec![SeqStmt::SignalAssign {
                                target: mapped_target.clone(),
                                value: v,
                                delay: mapped_delay.clone(),
                                span: *span,
                            }],
                        ));
                    }

                    let def_v = self.remap_expr(default, prefix, port_bindings, &enum_literals);
                    let else_branch = Some(vec![SeqStmt::SignalAssign {
                        target: mapped_target,
                        value: def_v,
                        delay: mapped_delay,
                        span: *span,
                    }]);

                    let if_stmt = SeqStmt::If {
                        cond: first_cond,
                        then_branch,
                        elsif_branches,
                        else_branch,
                        span: *span,
                    };

                    model.processes.push(ElabProcess {
                        name: p_name,
                        sensitivity: Some(mapped_sens),
                        var_decls: HashMap::new(),
                        body: vec![if_stmt],
                    });
                }
                ConcurrentStmt::ComponentInst {
                    label,
                    comp_name,
                    generic_map: _,
                    port_map,
                    span: _,
                } => {
                    // Elaborate instantiated component
                    let comp_key = comp_name.to_ascii_lowercase();
                    let child_entity = self.entities.get(&comp_key).ok_or_else(|| {
                        VhdlError::ElabError {
                            msg: format!("Instantiated component/entity '{comp_name}' not found"),
                        }
                    })?;

                    let child_archs = self.architectures.get(&comp_key).ok_or_else(|| {
                        VhdlError::ElabError {
                            msg: format!("No architecture found for component '{comp_name}'"),
                        }
                    })?;
                    let child_arch = child_archs.last().unwrap();

                    let child_prefix = if prefix.is_empty() {
                        label.to_ascii_lowercase()
                    } else {
                        format!("{prefix}.{}", label.to_ascii_lowercase())
                    };

                    // Build port bindings between child ports and parent signals
                    let mut child_bindings = HashMap::new();
                    for (i, (formal, actual)) in port_map.iter().enumerate() {
                        let formal_name = match formal {
                            Some(f) => f.to_ascii_lowercase(),
                            None => {
                                // Positional binding
                                let mut all_ports = Vec::new();
                                for p in &child_entity.ports {
                                    for p_name in &p.names {
                                        all_ports.push(p_name.to_ascii_lowercase());
                                    }
                                }
                                all_ports.get(i).cloned().unwrap_or_default()
                            }
                        };

                        if let Expr::Ident(actual_name, _) = actual {
                            let mapped_actual = self.resolve_signal_ref(actual_name, prefix, port_bindings, &enum_literals);
                            child_bindings.insert(formal_name, mapped_actual);
                        }
                    }

                    // Recursively elaborate child architecture
                    self.elaborate_architecture(child_arch, &child_prefix, model, &child_bindings)?;
                }
                ConcurrentStmt::Assert { cond, report, severity, span } => {
                    // Concurrent assertion
                    let mut rhs_signals = Vec::new();
                    self.collect_signals_in_expr(cond, &mut rhs_signals, &enum_literals);
                    let mapped_sens: Vec<String> = rhs_signals
                        .into_iter()
                        .map(|s| self.resolve_signal_ref(&s, prefix, port_bindings, &enum_literals))
                        .collect();

                    let mapped_cond = self.remap_expr(cond, prefix, port_bindings, &enum_literals);
                    model.processes.push(ElabProcess {
                        name: format!("{prefix}assert_{proc_idx}"),
                        sensitivity: Some(mapped_sens),
                        var_decls: HashMap::new(),
                        body: vec![SeqStmt::Assert {
                            cond: mapped_cond,
                            report: report.clone(),
                            severity: severity.clone(),
                            span: *span,
                        }],
                    });
                }
            }
        }

        Ok(())
    }

    fn resolve_signal_ref(
        &self,
        name: &str,
        prefix: &str,
        port_bindings: &HashMap<String, String>,
        enum_literals: &HashSet<String>,
    ) -> String {
        let lower = name.to_ascii_lowercase();
        if enum_literals.contains(&lower) {
            return lower;
        }
        if let Some(bound) = port_bindings.get(&lower) {
            return bound.clone();
        }
        if prefix.is_empty() {
            lower
        } else {
            format!("{prefix}.{lower}")
        }
    }

    fn resolve_type_info(
        &self,
        typ: &TypeDefKind,
        default: &Option<Expr>,
        type_defs: &HashMap<String, TypeDefKind>,
    ) -> Result<(usize, SimValue), VhdlError> {
        match typ {
            TypeDefKind::StdLogic | TypeDefKind::Bit => {
                let init = if let Some(Expr::LitLogic(lv, _)) = default {
                    SimValue::Logic(*lv)
                } else {
                    SimValue::Logic(LogicVal::U)
                };
                Ok((1, init))
            }
            TypeDefKind::StdLogicVector(r) | TypeDefKind::BitVector(r) => {
                let left = self.eval_const_int(&r.left)?;
                let right = self.eval_const_int(&r.right)?;
                let width = ((left - right).abs() + 1) as usize;
                let init = match default {
                    Some(Expr::LitVector(vec, _)) => SimValue::LogicVector(vec.clone()),
                    Some(expr @ Expr::Aggregate(_, _)) => {
                        let empty_scope = crate::eval::SimpleScope::default();
                        let eval = crate::eval::Evaluator::with_target_width(&empty_scope, width);
                        eval.eval(expr).unwrap_or_else(|_| SimValue::LogicVector(vec![LogicVal::U; width]))
                    }
                    _ => SimValue::LogicVector(vec![LogicVal::U; width]),
                };
                Ok((width, init))
            }
            TypeDefKind::Integer(_) | TypeDefKind::Natural | TypeDefKind::Positive => {
                let init = if let Some(e) = default {
                    SimValue::Integer(self.eval_const_int(e)?)
                } else {
                    SimValue::Integer(0)
                };
                Ok((32, init))
            }
            TypeDefKind::Boolean => {
                let init = if let Some(Expr::LitBool(b, _)) = default {
                    SimValue::Boolean(*b)
                } else {
                    SimValue::Boolean(false)
                };
                Ok((1, init))
            }
            TypeDefKind::Time => Ok((64, SimValue::Time(0))),
            TypeDefKind::Enum(variants) => {
                let init = variants.first().cloned().unwrap_or_default();
                Ok((8, SimValue::Enum(init.to_ascii_lowercase())))
            }
            TypeDefKind::Array(r, elem_typ) => {
                let left = self.eval_const_int(&r.left)?;
                let right = self.eval_const_int(&r.right)?;
                let len = ((left - right).abs() + 1) as usize;
                let (_, elem_default) = self.resolve_type_info(elem_typ, &None, type_defs)?;
                let mut arr = vec![elem_default; len];

                if let Some(Expr::Aggregate(choices, _)) = default {
                    let mut explicit = vec![false; len];
                    let mut pos_idx = 0usize;
                    let mut others_val = None;

                    for choice in choices {
                        match choice {
                            AggregateChoice::Others(val_expr) => {
                                let (_, v) = self.resolve_type_info(elem_typ, &Some((**val_expr).clone()), type_defs)?;
                                others_val = Some(v);
                            }
                            AggregateChoice::Index(idx_expr, val_expr) => {
                                let idx = self.eval_const_int(idx_expr)? as usize;
                                if idx < len {
                                    let (_, v) = self.resolve_type_info(elem_typ, &Some((**val_expr).clone()), type_defs)?;
                                    arr[idx] = v;
                                    explicit[idx] = true;
                                }
                            }
                            AggregateChoice::Positional(val_expr) => {
                                if pos_idx < len {
                                    let (_, v) = self.resolve_type_info(elem_typ, &Some((**val_expr).clone()), type_defs)?;
                                    arr[pos_idx] = v;
                                    explicit[pos_idx] = true;
                                    pos_idx += 1;
                                }
                            }
                        }
                    }

                    if let Some(ov) = others_val {
                        for i in 0..len {
                            if !explicit[i] {
                                arr[i] = ov.clone();
                            }
                        }
                    }
                }

                Ok((32, SimValue::Array(arr)))
            }
            TypeDefKind::Alias(name) => {
                let lower = name.to_ascii_lowercase();
                if let Some(resolved) = type_defs.get(&lower) {
                    self.resolve_type_info(resolved, default, type_defs)
                } else {
                    Ok((32, SimValue::Integer(0)))
                }
            }
        }
    }

    fn eval_const_int(&self, expr: &Expr) -> Result<i64, VhdlError> {
        match expr {
            Expr::LitInt(i, _) => Ok(*i),
            Expr::UnaryOp(UnaryOpKind::Neg, sub, _) => Ok(-self.eval_const_int(sub)?),
            Expr::BinaryOp(BinaryOpKind::Add, l, r, _) => {
                Ok(self.eval_const_int(l)? + self.eval_const_int(r)?)
            }
            Expr::BinaryOp(BinaryOpKind::Sub, l, r, _) => {
                Ok(self.eval_const_int(l)? - self.eval_const_int(r)?)
            }
            _ => Ok(0),
        }
    }

    fn collect_signals_in_expr(
        &self,
        expr: &Expr,
        signals: &mut impl SignalCollector,
        enum_literals: &HashSet<String>,
    ) {
        match expr {
            Expr::Ident(name, _) => {
                let lower = name.to_ascii_lowercase();
                if lower != "true" && lower != "false" && !enum_literals.contains(&lower) {
                    signals.add_signal(lower);
                }
            }
            Expr::Index(name, idx, _) => {
                if !enum_literals.contains(&name.to_ascii_lowercase()) {
                    signals.add_signal(name.to_ascii_lowercase());
                }
                self.collect_signals_in_expr(idx, signals, enum_literals);
            }
            Expr::Slice(name, range, _) => {
                if !enum_literals.contains(&name.to_ascii_lowercase()) {
                    signals.add_signal(name.to_ascii_lowercase());
                }
                self.collect_signals_in_expr(&range.left, signals, enum_literals);
                self.collect_signals_in_expr(&range.right, signals, enum_literals);
            }
            Expr::UnaryOp(_, sub, _) => self.collect_signals_in_expr(sub, signals, enum_literals),
            Expr::BinaryOp(_, l, r, _) => {
                self.collect_signals_in_expr(l, signals, enum_literals);
                self.collect_signals_in_expr(r, signals, enum_literals);
            }
            Expr::FunctionCall(_, args, _) => {
                for arg in args {
                    self.collect_signals_in_expr(arg, signals, enum_literals);
                }
            }
            Expr::TypeCast(_, sub, _) => self.collect_signals_in_expr(sub, signals, enum_literals),
            Expr::Aggregate(choices, _) => {
                for choice in choices {
                    match choice {
                        AggregateChoice::Others(e) | AggregateChoice::Positional(e) => {
                            self.collect_signals_in_expr(e, signals, enum_literals);
                        }
                        AggregateChoice::Index(i, v) => {
                            self.collect_signals_in_expr(i, signals, enum_literals);
                            self.collect_signals_in_expr(v, signals, enum_literals);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn remap_target(
        &self,
        target: &TargetExpr,
        prefix: &str,
        bindings: &HashMap<String, String>,
        enum_literals: &HashSet<String>,
    ) -> TargetExpr {
        match target {
            TargetExpr::Ident(name) => {
                TargetExpr::Ident(self.resolve_signal_ref(name, prefix, bindings, enum_literals))
            }
            TargetExpr::Index(name, idx) => TargetExpr::Index(
                self.resolve_signal_ref(name, prefix, bindings, enum_literals),
                Box::new(self.remap_expr(idx, prefix, bindings, enum_literals)),
            ),
            TargetExpr::Slice(name, range) => TargetExpr::Slice(
                self.resolve_signal_ref(name, prefix, bindings, enum_literals),
                RangeExpr {
                    left: Box::new(self.remap_expr(&range.left, prefix, bindings, enum_literals)),
                    direction: range.direction.clone(),
                    right: Box::new(self.remap_expr(&range.right, prefix, bindings, enum_literals)),
                },
            ),
        }
    }

    fn remap_expr(
        &self,
        expr: &Expr,
        prefix: &str,
        bindings: &HashMap<String, String>,
        enum_literals: &HashSet<String>,
    ) -> Expr {
        match expr {
            Expr::Ident(name, span) => {
                let lower = name.to_ascii_lowercase();
                if lower == "true" || lower == "false" || enum_literals.contains(&lower) {
                    Expr::Ident(lower, *span)
                } else {
                    Expr::Ident(self.resolve_signal_ref(&lower, prefix, bindings, enum_literals), *span)
                }
            }
            Expr::Index(name, idx, span) => Expr::Index(
                self.resolve_signal_ref(name, prefix, bindings, enum_literals),
                Box::new(self.remap_expr(idx, prefix, bindings, enum_literals)),
                *span,
            ),
            Expr::Slice(name, range, span) => Expr::Slice(
                self.resolve_signal_ref(name, prefix, bindings, enum_literals),
                RangeExpr {
                    left: Box::new(self.remap_expr(&range.left, prefix, bindings, enum_literals)),
                    direction: range.direction.clone(),
                    right: Box::new(self.remap_expr(&range.right, prefix, bindings, enum_literals)),
                },
                *span,
            ),
            Expr::UnaryOp(op, sub, span) => {
                Expr::UnaryOp(op.clone(), Box::new(self.remap_expr(sub, prefix, bindings, enum_literals)), *span)
            }
            Expr::BinaryOp(op, l, r, span) => Expr::BinaryOp(
                op.clone(),
                Box::new(self.remap_expr(l, prefix, bindings, enum_literals)),
                Box::new(self.remap_expr(r, prefix, bindings, enum_literals)),
                *span,
            ),
            Expr::FunctionCall(name, args, span) => {
                let remapped_args = args
                    .iter()
                    .map(|a| self.remap_expr(a, prefix, bindings, enum_literals))
                    .collect();
                Expr::FunctionCall(name.clone(), remapped_args, *span)
            }
            Expr::TypeCast(t, sub, span) => {
                Expr::TypeCast(t.clone(), Box::new(self.remap_expr(sub, prefix, bindings, enum_literals)), *span)
            }
            _ => expr.clone(),
        }
    }

    fn remap_body_signals(
        &self,
        stmts: &[SeqStmt],
        prefix: &str,
        bindings: &HashMap<String, String>,
        enum_literals: &HashSet<String>,
    ) -> Vec<SeqStmt> {
        stmts
            .iter()
            .map(|stmt| self.remap_seq_stmt(stmt, prefix, bindings, enum_literals))
            .collect()
    }

    fn remap_seq_stmt(
        &self,
        stmt: &SeqStmt,
        prefix: &str,
        bindings: &HashMap<String, String>,
        enum_literals: &HashSet<String>,
    ) -> SeqStmt {
        match stmt {
            SeqStmt::SignalAssign {
                target,
                value,
                delay,
                span,
            } => SeqStmt::SignalAssign {
                target: self.remap_target(target, prefix, bindings, enum_literals),
                value: self.remap_expr(value, prefix, bindings, enum_literals),
                delay: delay.as_ref().map(|d| self.remap_expr(d, prefix, bindings, enum_literals)),
                span: *span,
            },
            SeqStmt::VariableAssign { target, value, span } => SeqStmt::VariableAssign {
                target: target.clone(),
                value: self.remap_expr(value, prefix, bindings, enum_literals),
                span: *span,
            },
            SeqStmt::If {
                cond,
                then_branch,
                elsif_branches,
                else_branch,
                span,
            } => SeqStmt::If {
                cond: self.remap_expr(cond, prefix, bindings, enum_literals),
                then_branch: self.remap_body_signals(then_branch, prefix, bindings, enum_literals),
                elsif_branches: elsif_branches
                    .iter()
                    .map(|(c, b)| {
                        (
                            self.remap_expr(c, prefix, bindings, enum_literals),
                            self.remap_body_signals(b, prefix, bindings, enum_literals),
                        )
                    })
                    .collect(),
                else_branch: else_branch
                    .as_ref()
                    .map(|b| self.remap_body_signals(b, prefix, bindings, enum_literals)),
                span: *span,
            },
            SeqStmt::Case { expr, arms, span } => SeqStmt::Case {
                expr: self.remap_expr(expr, prefix, bindings, enum_literals),
                arms: arms
                    .iter()
                    .map(|arm| CaseArm {
                        choices: arm.choices.iter().map(|choice| match choice {
                            Choice::Expr(c_expr) => Choice::Expr(self.remap_expr(c_expr, prefix, bindings, enum_literals)),
                            Choice::Range(r) => Choice::Range(RangeExpr {
                                left: Box::new(self.remap_expr(&r.left, prefix, bindings, enum_literals)),
                                direction: r.direction.clone(),
                                right: Box::new(self.remap_expr(&r.right, prefix, bindings, enum_literals)),
                            }),
                            Choice::Others => Choice::Others,
                        }).collect(),
                        stmts: self.remap_body_signals(&arm.stmts, prefix, bindings, enum_literals),
                        span: arm.span,
                    })
                    .collect(),
                span: *span,
            },
            SeqStmt::WaitFor { duration, span } => SeqStmt::WaitFor {
                duration: self.remap_expr(duration, prefix, bindings, enum_literals),
                span: *span,
            },
            SeqStmt::WaitUntil { cond, span } => SeqStmt::WaitUntil {
                cond: self.remap_expr(cond, prefix, bindings, enum_literals),
                span: *span,
            },
            SeqStmt::WaitOn { signals, span } => SeqStmt::WaitOn {
                signals: signals
                    .iter()
                    .map(|s| self.resolve_signal_ref(s, prefix, bindings, enum_literals))
                    .collect(),
                span: *span,
            },
            SeqStmt::Assert {
                cond,
                report,
                severity,
                span,
            } => SeqStmt::Assert {
                cond: self.remap_expr(cond, prefix, bindings, enum_literals),
                report: report.clone(),
                severity: severity.clone(),
                span: *span,
            },
            SeqStmt::ForLoop {
                var_name,
                range,
                body,
                span,
            } => SeqStmt::ForLoop {
                var_name: var_name.clone(),
                range: RangeExpr {
                    left: Box::new(self.remap_expr(&range.left, prefix, bindings, enum_literals)),
                    direction: range.direction.clone(),
                    right: Box::new(self.remap_expr(&range.right, prefix, bindings, enum_literals)),
                },
                body: self.remap_body_signals(body, prefix, bindings, enum_literals),
                span: *span,
            },
            SeqStmt::WhileLoop { cond, body, span } => SeqStmt::WhileLoop {
                cond: self.remap_expr(cond, prefix, bindings, enum_literals),
                body: self.remap_body_signals(body, prefix, bindings, enum_literals),
                span: *span,
            },
            SeqStmt::Wait { span } => SeqStmt::Wait { span: *span },
            SeqStmt::Null(span) => SeqStmt::Null(*span),
        }
    }
}

trait SignalCollector {
    fn add_signal(&mut self, s: String);
}

impl SignalCollector for Vec<String> {
    fn add_signal(&mut self, s: String) {
        if !self.contains(&s) {
            self.push(s);
        }
    }
}

impl SignalCollector for HashSet<String> {
    fn add_signal(&mut self, s: String) {
        self.insert(s);
    }
}
