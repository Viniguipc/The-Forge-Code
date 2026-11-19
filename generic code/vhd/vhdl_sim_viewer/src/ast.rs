//! Abstract Syntax Tree (AST) definitions for VHDL.

use crate::common::Span;
use crate::value::LogicVal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortMode {
    In,
    Out,
    InOut,
    Buffer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeDirection {
    Downto,
    To,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RangeExpr {
    pub left: Box<Expr>,
    pub direction: RangeDirection,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeDefKind {
    StdLogic,
    StdLogicVector(RangeExpr),
    Bit,
    BitVector(RangeExpr),
    Integer(Option<RangeExpr>),
    Natural,
    Positive,
    Boolean,
    Time,
    Enum(Vec<String>),
    Array(RangeExpr, Box<TypeDefKind>),
    Alias(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PortDecl {
    pub names: Vec<String>,
    pub mode: PortMode,
    pub typ: TypeDefKind,
    pub default: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenericDecl {
    pub names: Vec<String>,
    pub typ: TypeDefKind,
    pub default: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
    Signal {
        names: Vec<String>,
        typ: TypeDefKind,
        default: Option<Expr>,
        span: Span,
    },
    Variable {
        names: Vec<String>,
        typ: TypeDefKind,
        default: Option<Expr>,
        span: Span,
    },
    Constant {
        names: Vec<String>,
        typ: TypeDefKind,
        value: Expr,
        span: Span,
    },
    Component {
        name: String,
        generics: Vec<GenericDecl>,
        ports: Vec<PortDecl>,
        span: Span,
    },
    Type {
        name: String,
        kind: TypeDefKind,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum TargetExpr {
    Ident(String),
    Index(String, Box<Expr>),
    Slice(String, RangeExpr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOpKind {
    Not,
    Neg,
    Pos,
    Abs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOpKind {
    And,
    Or,
    Nand,
    Nor,
    Xor,
    Xnor,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Concat,
    Sll,
    Srl,
    Sla,
    Sra,
    Rol,
    Ror,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AggregateChoice {
    Others(Box<Expr>),
    Index(Box<Expr>, Box<Expr>),
    Positional(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    LitLogic(LogicVal, Span),
    LitVector(Vec<LogicVal>, Span),
    LitInt(i64, Span),
    LitTime(u64, Span), // femtoseconds
    LitString(String, Span),
    LitBool(bool, Span),
    Ident(String, Span),
    Index(String, Box<Expr>, Span),
    Slice(String, RangeExpr, Span),
    UnaryOp(UnaryOpKind, Box<Expr>, Span),
    BinaryOp(BinaryOpKind, Box<Expr>, Box<Expr>, Span),
    FunctionCall(String, Vec<Expr>, Span),
    Aggregate(Vec<AggregateChoice>, Span),
    TypeCast(String, Box<Expr>, Span),
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::LitLogic(_, s)
            | Expr::LitVector(_, s)
            | Expr::LitInt(_, s)
            | Expr::LitTime(_, s)
            | Expr::LitString(_, s)
            | Expr::LitBool(_, s)
            | Expr::Ident(_, s)
            | Expr::Index(_, _, s)
            | Expr::Slice(_, _, s)
            | Expr::UnaryOp(_, _, s)
            | Expr::BinaryOp(_, _, _, s)
            | Expr::FunctionCall(_, _, s)
            | Expr::Aggregate(_, s)
            | Expr::TypeCast(_, _, s) => *s,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Choice {
    Expr(Expr),
    Range(RangeExpr),
    Others,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CaseArm {
    pub choices: Vec<Choice>,
    pub stmts: Vec<SeqStmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SeqStmt {
    SignalAssign {
        target: TargetExpr,
        value: Expr,
        delay: Option<Expr>,
        span: Span,
    },
    VariableAssign {
        target: TargetExpr,
        value: Expr,
        span: Span,
    },
    If {
        cond: Expr,
        then_branch: Vec<SeqStmt>,
        elsif_branches: Vec<(Expr, Vec<SeqStmt>)>,
        else_branch: Option<Vec<SeqStmt>>,
        span: Span,
    },
    Case {
        expr: Expr,
        arms: Vec<CaseArm>,
        span: Span,
    },
    WaitFor {
        duration: Expr,
        span: Span,
    },
    WaitUntil {
        cond: Expr,
        span: Span,
    },
    WaitOn {
        signals: Vec<String>,
        span: Span,
    },
    Wait {
        span: Span,
    },
    Assert {
        cond: Expr,
        report: Option<String>,
        severity: Option<String>,
        span: Span,
    },
    ForLoop {
        var_name: String,
        range: RangeExpr,
        body: Vec<SeqStmt>,
        span: Span,
    },
    WhileLoop {
        cond: Expr,
        body: Vec<SeqStmt>,
        span: Span,
    },
    Null(Span),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConcurrentStmt {
    Process {
        label: Option<String>,
        sensitivity: Option<Vec<String>>,
        decls: Vec<Declaration>,
        body: Vec<SeqStmt>,
        span: Span,
    },
    SignalAssign {
        target: TargetExpr,
        expr: Expr,
        delay: Option<Expr>,
        span: Span,
    },
    ConditionalSignalAssign {
        target: TargetExpr,
        when_else: Vec<(Expr, Expr)>, // (value, condition)
        default: Expr,
        delay: Option<Expr>,
        span: Span,
    },
    ComponentInst {
        label: String,
        comp_name: String,
        generic_map: Vec<(Option<String>, Expr)>,
        port_map: Vec<(Option<String>, Expr)>,
        span: Span,
    },
    Assert {
        cond: Expr,
        report: Option<String>,
        severity: Option<String>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntityDecl {
    pub name: String,
    pub generics: Vec<GenericDecl>,
    pub ports: Vec<PortDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArchDecl {
    pub name: String,
    pub entity_name: String,
    pub decls: Vec<Declaration>,
    pub stmts: Vec<ConcurrentStmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DesignUnit {
    Entity(EntityDecl),
    Architecture(ArchDecl),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DesignFile {
    pub units: Vec<DesignUnit>,
}
