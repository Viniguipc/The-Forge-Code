//! Recursive-descent parser for VHDL.

use crate::ast::*;
use crate::common::{Span, VhdlError};
use crate::token::{Token, TokenKind};
use crate::value::LogicVal;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    eof_token: Token,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            eof_token: Token {
                kind: TokenKind::Eof,
                span: Span::default(),
            },
        }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&self.eof_token)
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.peek().kind
    }

    fn advance(&mut self) -> Token {
        if self.pos < self.tokens.len() {
            let tok = self.tokens[self.pos].clone();
            self.pos += 1;
            tok
        } else {
            Token {
                kind: TokenKind::Eof,
                span: Span::default(),
            }
        }
    }

    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if std::mem::discriminant(self.peek_kind()) == std::mem::discriminant(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, VhdlError> {
        let tok = self.advance();
        if std::mem::discriminant(&tok.kind) == std::mem::discriminant(&kind) {
            Ok(tok)
        } else {
            Err(VhdlError::ParseError {
                msg: format!("Expected token {:?}, found {:?}", kind, tok.kind),
                span: tok.span,
            })
        }
    }

    fn expect_ident(&mut self) -> Result<String, VhdlError> {
        let tok = self.advance();
        match tok.kind {
            TokenKind::Ident(s) => Ok(s),
            _ => Err(VhdlError::ParseError {
                msg: format!("Expected identifier, found {:?}", tok.kind),
                span: tok.span,
            }),
        }
    }

    /// Parse complete design file
    pub fn parse_design_file(&mut self) -> Result<DesignFile, VhdlError> {
        let mut file = DesignFile::default();

        while *self.peek_kind() != TokenKind::Eof {
            match self.peek_kind() {
                TokenKind::Library | TokenKind::Use => {
                    self.skip_library_or_use()?;
                }
                TokenKind::Entity => {
                    let ent = self.parse_entity()?;
                    file.units.push(DesignUnit::Entity(ent));
                }
                TokenKind::Architecture => {
                    let arch = self.parse_architecture()?;
                    file.units.push(DesignUnit::Architecture(arch));
                }
                TokenKind::Package => {
                    self.skip_package()?;
                }
                _ => {
                    let tok = self.peek();
                    return Err(VhdlError::ParseError {
                        msg: format!("Unexpected top-level token: {:?}", tok.kind),
                        span: tok.span,
                    });
                }
            }
        }

        Ok(file)
    }

    fn skip_library_or_use(&mut self) -> Result<(), VhdlError> {
        self.advance(); // consume 'library' or 'use'
        while *self.peek_kind() != TokenKind::Semicolon && *self.peek_kind() != TokenKind::Eof {
            self.advance();
        }
        self.expect(TokenKind::Semicolon)?;
        Ok(())
    }

    fn skip_package(&mut self) -> Result<(), VhdlError> {
        self.advance(); // consume 'package'
        // Skip until 'end [package] [<name>];'
        let mut depth = 1;
        while depth > 0 && *self.peek_kind() != TokenKind::Eof {
            let tok = self.advance();
            if tok.kind == TokenKind::Package {
                depth += 1;
            } else if tok.kind == TokenKind::End {
                // optionally skip 'package' and ident
                if *self.peek_kind() == TokenKind::Package {
                    self.advance();
                }
                if let TokenKind::Ident(_) = self.peek_kind() {
                    self.advance();
                }
                if *self.peek_kind() == TokenKind::Semicolon {
                    self.advance();
                }
                depth -= 1;
            }
        }
        Ok(())
    }

    // --- Entity ---
    fn parse_entity(&mut self) -> Result<EntityDecl, VhdlError> {
        let start_tok = self.expect(TokenKind::Entity)?;
        let name = self.expect_ident()?;
        self.expect(TokenKind::Is)?;

        let mut generics = Vec::new();
        let mut ports = Vec::new();

        while *self.peek_kind() != TokenKind::End && *self.peek_kind() != TokenKind::Eof {
            match self.peek_kind() {
                TokenKind::Generic => {
                    generics = self.parse_generics()?;
                }
                TokenKind::Port => {
                    ports = self.parse_ports()?;
                }
                _ => {
                    let tok = self.peek();
                    return Err(VhdlError::ParseError {
                        msg: format!("Unexpected token in entity declaration: {:?}", tok.kind),
                        span: tok.span,
                    });
                }
            }
        }

        self.expect(TokenKind::End)?;
        if *self.peek_kind() == TokenKind::Entity {
            self.advance();
        }
        if let TokenKind::Ident(_) = self.peek_kind() {
            self.advance();
        }
        self.expect(TokenKind::Semicolon)?;

        Ok(EntityDecl {
            name,
            generics,
            ports,
            span: start_tok.span,
        })
    }

    fn parse_ports(&mut self) -> Result<Vec<PortDecl>, VhdlError> {
        self.expect(TokenKind::Port)?;
        self.expect(TokenKind::LParen)?;
        let mut ports = Vec::new();

        loop {
            let p_span = self.peek().span;
            let mut names = vec![self.expect_ident()?];
            while self.match_token(&TokenKind::Comma) {
                names.push(self.expect_ident()?);
            }
            self.expect(TokenKind::Colon)?;

            // Mode
            let mode = match self.peek_kind() {
                TokenKind::In => {
                    self.advance();
                    PortMode::In
                }
                TokenKind::Out => {
                    self.advance();
                    PortMode::Out
                }
                TokenKind::Inout => {
                    self.advance();
                    PortMode::InOut
                }
                TokenKind::Buffer => {
                    self.advance();
                    PortMode::Buffer
                }
                _ => PortMode::In, // Default mode is IN
            };

            let typ = self.parse_type_kind()?;
            let mut default = None;
            if self.match_token(&TokenKind::Assign) {
                default = Some(self.parse_expr()?);
            }

            ports.push(PortDecl {
                names,
                mode,
                typ,
                default,
                span: p_span,
            });

            if self.match_token(&TokenKind::Semicolon) {
                if *self.peek_kind() == TokenKind::RParen {
                    break;
                }
            } else {
                break;
            }
        }

        self.expect(TokenKind::RParen)?;
        self.expect(TokenKind::Semicolon)?;
        Ok(ports)
    }

    fn parse_generics(&mut self) -> Result<Vec<GenericDecl>, VhdlError> {
        self.expect(TokenKind::Generic)?;
        self.expect(TokenKind::LParen)?;
        let mut generics = Vec::new();

        loop {
            let g_span = self.peek().span;
            let mut names = vec![self.expect_ident()?];
            while self.match_token(&TokenKind::Comma) {
                names.push(self.expect_ident()?);
            }
            self.expect(TokenKind::Colon)?;
            let typ = self.parse_type_kind()?;
            let mut default = None;
            if self.match_token(&TokenKind::Assign) {
                default = Some(self.parse_expr()?);
            }

            generics.push(GenericDecl {
                names,
                typ,
                default,
                span: g_span,
            });

            if self.match_token(&TokenKind::Semicolon) {
                if *self.peek_kind() == TokenKind::RParen {
                    break;
                }
            } else {
                break;
            }
        }

        self.expect(TokenKind::RParen)?;
        self.expect(TokenKind::Semicolon)?;
        Ok(generics)
    }

    // --- Architecture ---
    fn parse_architecture(&mut self) -> Result<ArchDecl, VhdlError> {
        let start_tok = self.expect(TokenKind::Architecture)?;
        let name = self.expect_ident()?;
        self.expect(TokenKind::Of)?;
        let entity_name = self.expect_ident()?;
        self.expect(TokenKind::Is)?;

        let mut decls = Vec::new();
        while *self.peek_kind() != TokenKind::Begin && *self.peek_kind() != TokenKind::Eof {
            decls.push(self.parse_declaration()?);
        }

        self.expect(TokenKind::Begin)?;

        let mut stmts = Vec::new();
        while *self.peek_kind() != TokenKind::End && *self.peek_kind() != TokenKind::Eof {
            stmts.push(self.parse_concurrent_stmt()?);
        }

        self.expect(TokenKind::End)?;
        if *self.peek_kind() == TokenKind::Architecture {
            self.advance();
        }
        if let TokenKind::Ident(_) = self.peek_kind() {
            self.advance();
        }
        self.expect(TokenKind::Semicolon)?;

        Ok(ArchDecl {
            name,
            entity_name,
            decls,
            stmts,
            span: start_tok.span,
        })
    }

    // --- Declarations ---
    fn parse_declaration(&mut self) -> Result<Declaration, VhdlError> {
        let tok = self.peek().clone();
        match tok.kind {
            TokenKind::Signal => {
                self.advance();
                let mut names = vec![self.expect_ident()?];
                while self.match_token(&TokenKind::Comma) {
                    names.push(self.expect_ident()?);
                }
                self.expect(TokenKind::Colon)?;
                let typ = self.parse_type_kind()?;
                let mut default = None;
                if self.match_token(&TokenKind::Assign) {
                    default = Some(self.parse_expr()?);
                }
                self.expect(TokenKind::Semicolon)?;
                Ok(Declaration::Signal {
                    names,
                    typ,
                    default,
                    span: tok.span,
                })
            }
            TokenKind::Variable => {
                self.advance();
                let mut names = vec![self.expect_ident()?];
                while self.match_token(&TokenKind::Comma) {
                    names.push(self.expect_ident()?);
                }
                self.expect(TokenKind::Colon)?;
                let typ = self.parse_type_kind()?;
                let mut default = None;
                if self.match_token(&TokenKind::Assign) {
                    default = Some(self.parse_expr()?);
                }
                self.expect(TokenKind::Semicolon)?;
                Ok(Declaration::Variable {
                    names,
                    typ,
                    default,
                    span: tok.span,
                })
            }
            TokenKind::Constant => {
                self.advance();
                let mut names = vec![self.expect_ident()?];
                while self.match_token(&TokenKind::Comma) {
                    names.push(self.expect_ident()?);
                }
                self.expect(TokenKind::Colon)?;
                let typ = self.parse_type_kind()?;
                self.expect(TokenKind::Assign)?;
                let value = self.parse_expr()?;
                self.expect(TokenKind::Semicolon)?;
                Ok(Declaration::Constant {
                    names,
                    typ,
                    value,
                    span: tok.span,
                })
            }
            TokenKind::Component => {
                self.advance();
                let name = self.expect_ident()?;
                self.expect(TokenKind::Is).ok(); // 'is' is optional in component decl
                let mut generics = Vec::new();
                let mut ports = Vec::new();
                while *self.peek_kind() != TokenKind::End && *self.peek_kind() != TokenKind::Eof {
                    if *self.peek_kind() == TokenKind::Generic {
                        generics = self.parse_generics()?;
                    } else if *self.peek_kind() == TokenKind::Port {
                        ports = self.parse_ports()?;
                    } else {
                        break;
                    }
                }
                self.expect(TokenKind::End)?;
                self.expect(TokenKind::Component)?;
                if let TokenKind::Ident(_) = self.peek_kind() {
                    self.advance();
                }
                self.expect(TokenKind::Semicolon)?;
                Ok(Declaration::Component {
                    name,
                    generics,
                    ports,
                    span: tok.span,
                })
            }
            TokenKind::Type => {
                self.advance();
                let name = self.expect_ident()?;
                self.expect(TokenKind::Is)?;
                let kind = if self.match_token(&TokenKind::LParen) {
                    // Enum type
                    let mut variants = vec![self.expect_ident()?];
                    while self.match_token(&TokenKind::Comma) {
                        variants.push(self.expect_ident()?);
                    }
                    self.expect(TokenKind::RParen)?;
                    TypeDefKind::Enum(variants)
                } else if self.match_token(&TokenKind::Range) {
                    let r = self.parse_range()?;
                    TypeDefKind::Integer(Some(r))
                } else if matches!(self.peek_kind(), TokenKind::Ident(s) if s == "array") {
                    self.advance(); // consume 'array'
                    self.expect(TokenKind::LParen)?;
                    let r = self.parse_range()?;
                    self.expect(TokenKind::RParen)?;
                    self.expect(TokenKind::Of)?;
                    let elem = self.parse_type_kind()?;
                    TypeDefKind::Array(r, Box::new(elem))
                } else {
                    self.parse_type_kind()?
                };
                self.expect(TokenKind::Semicolon)?;
                Ok(Declaration::Type {
                    name,
                    kind,
                    span: tok.span,
                })
            }
            _ => Err(VhdlError::ParseError {
                msg: format!("Expected declaration, found {:?}", tok.kind),
                span: tok.span,
            }),
        }
    }

    // --- Types ---
    fn parse_type_kind(&mut self) -> Result<TypeDefKind, VhdlError> {
        let tok = self.advance();
        match tok.kind {
            TokenKind::Ident(ref id) => match id.as_str() {
                "std_logic" => Ok(TypeDefKind::StdLogic),
                "std_logic_vector" => {
                    self.expect(TokenKind::LParen)?;
                    let r = self.parse_range()?;
                    self.expect(TokenKind::RParen)?;
                    Ok(TypeDefKind::StdLogicVector(r))
                }
                "bit" => Ok(TypeDefKind::Bit),
                "bit_vector" => {
                    self.expect(TokenKind::LParen)?;
                    let r = self.parse_range()?;
                    self.expect(TokenKind::RParen)?;
                    Ok(TypeDefKind::BitVector(r))
                }
                "integer" => {
                    let range = if self.match_token(&TokenKind::Range) {
                        Some(self.parse_range()?)
                    } else {
                        None
                    };
                    Ok(TypeDefKind::Integer(range))
                }
                "natural" => Ok(TypeDefKind::Natural),
                "positive" => Ok(TypeDefKind::Positive),
                "boolean" => Ok(TypeDefKind::Boolean),
                "time" => Ok(TypeDefKind::Time),
                custom => Ok(TypeDefKind::Alias(custom.to_string())),
            },
            _ => Err(VhdlError::ParseError {
                msg: format!("Expected type name, found {:?}", tok.kind),
                span: tok.span,
            }),
        }
    }

    fn parse_range(&mut self) -> Result<RangeExpr, VhdlError> {
        let left = self.parse_expr()?;
        let direction = if self.match_token(&TokenKind::Downto) {
            RangeDirection::Downto
        } else if self.match_token(&TokenKind::To) {
            RangeDirection::To
        } else {
            return Err(VhdlError::ParseError {
                msg: "Expected 'downto' or 'to' in range specification".to_string(),
                span: self.peek().span,
            });
        };
        let right = self.parse_expr()?;
        Ok(RangeExpr {
            left: Box::new(left),
            direction,
            right: Box::new(right),
        })
    }

    // --- Concurrent Statements ---
    fn parse_concurrent_stmt(&mut self) -> Result<ConcurrentStmt, VhdlError> {
        let start_tok = self.peek().clone();

        // Check if label: process or label: component_inst
        let mut label = None;
        if let TokenKind::Ident(ref name) = start_tok.kind {
            if self.tokens.get(self.pos + 1).map(|t| &t.kind) == Some(&TokenKind::Colon) {
                label = Some(name.clone());
                self.advance(); // consume ident
                self.advance(); // consume ':'
            }
        }

        // Process statement
        if *self.peek_kind() == TokenKind::Process {
            return self.parse_process(label, start_tok.span);
        }

        // Assert statement
        if *self.peek_kind() == TokenKind::Assert {
            return self.parse_concurrent_assert(start_tok.span);
        }

        // Component instantiation: label : comp_name [generic map (...)] port map (...)
        if let Some(lbl) = label {
            if let TokenKind::Ident(comp_name) = self.peek().kind.clone() {
                self.advance(); // consume component name
                return self.parse_component_inst(lbl, comp_name, start_tok.span);
            }
        }

        // Concurrent signal assignment: target <= expr [when cond else ...] [after time];
        let target = self.parse_target_expr()?;
        self.expect_signal_assign()?;
        let first_val = self.parse_expr()?;

        if self.match_token(&TokenKind::When) {
            // Conditional signal assignment
            let mut when_else = Vec::new();
            let first_cond = self.parse_expr()?;
            when_else.push((first_val, first_cond));

            while self.match_token(&TokenKind::Else) {
                let val = self.parse_expr()?;
                if self.match_token(&TokenKind::When) {
                    let cond = self.parse_expr()?;
                    when_else.push((val, cond));
                } else {
                    // Default value
                    let mut delay = None;
                    if self.match_token(&TokenKind::After) {
                        delay = Some(self.parse_expr()?);
                    }
                    self.expect(TokenKind::Semicolon)?;
                    return Ok(ConcurrentStmt::ConditionalSignalAssign {
                        target,
                        when_else,
                        default: val,
                        delay,
                        span: start_tok.span,
                    });
                }
            }
            return Err(VhdlError::ParseError {
                msg: "Expected final 'else' branch in conditional signal assignment".to_string(),
                span: self.peek().span,
            });
        }

        let mut delay = None;
        if self.match_token(&TokenKind::After) {
            delay = Some(self.parse_expr()?);
        }
        self.expect(TokenKind::Semicolon)?;

        Ok(ConcurrentStmt::SignalAssign {
            target,
            expr: first_val,
            delay,
            span: start_tok.span,
        })
    }

    fn expect_signal_assign(&mut self) -> Result<(), VhdlError> {
        let tok = self.advance();
        match tok.kind {
            TokenKind::SignalAssign | TokenKind::Lt => Ok(()),
            _ => Err(VhdlError::ParseError {
                msg: format!("Expected '<=', found {:?}", tok.kind),
                span: tok.span,
            }),
        }
    }

    fn parse_process(
        &mut self,
        label: Option<String>,
        span: Span,
    ) -> Result<ConcurrentStmt, VhdlError> {
        self.expect(TokenKind::Process)?;

        let mut sensitivity = None;
        if self.match_token(&TokenKind::LParen) {
            let mut sens = Vec::new();
            if *self.peek_kind() != TokenKind::RParen {
                sens.push(self.expect_ident()?);
                while self.match_token(&TokenKind::Comma) {
                    sens.push(self.expect_ident()?);
                }
            }
            self.expect(TokenKind::RParen)?;
            sensitivity = Some(sens);
        }

        self.match_token(&TokenKind::Is);

        let mut decls = Vec::new();
        while *self.peek_kind() != TokenKind::Begin && *self.peek_kind() != TokenKind::Eof {
            decls.push(self.parse_declaration()?);
        }

        self.expect(TokenKind::Begin)?;

        let mut body = Vec::new();
        while *self.peek_kind() != TokenKind::End && *self.peek_kind() != TokenKind::Eof {
            body.push(self.parse_seq_stmt()?);
        }

        self.expect(TokenKind::End)?;
        self.expect(TokenKind::Process)?;
        if let TokenKind::Ident(_) = self.peek_kind() {
            self.advance();
        }
        self.expect(TokenKind::Semicolon)?;

        Ok(ConcurrentStmt::Process {
            label,
            sensitivity,
            decls,
            body,
            span,
        })
    }

    fn parse_component_inst(
        &mut self,
        label: String,
        comp_name: String,
        span: Span,
    ) -> Result<ConcurrentStmt, VhdlError> {
        let mut generic_map = Vec::new();
        if self.match_token(&TokenKind::Generic) {
            self.expect(TokenKind::Map)?;
            generic_map = self.parse_association_list()?;
        }

        self.expect(TokenKind::Port)?;
        self.expect(TokenKind::Map)?;
        let port_map = self.parse_association_list()?;
        self.expect(TokenKind::Semicolon)?;

        Ok(ConcurrentStmt::ComponentInst {
            label,
            comp_name,
            generic_map,
            port_map,
            span,
        })
    }

    fn parse_association_list(&mut self) -> Result<Vec<(Option<String>, Expr)>, VhdlError> {
        self.expect(TokenKind::LParen)?;
        let mut list = Vec::new();

        loop {
            // Check if formal => actual or positional actual
            if let TokenKind::Ident(ref name) = self.peek().kind.clone() {
                if self.tokens.get(self.pos + 1).map(|t| &t.kind) == Some(&TokenKind::Arrow) {
                    self.advance(); // consume ident
                    self.advance(); // consume '=>'
                    let actual = self.parse_expr()?;
                    list.push((Some(name.clone()), actual));
                } else {
                    let actual = self.parse_expr()?;
                    list.push((None, actual));
                }
            } else {
                let actual = self.parse_expr()?;
                list.push((None, actual));
            }

            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }

        self.expect(TokenKind::RParen)?;
        Ok(list)
    }

    fn parse_concurrent_assert(&mut self, span: Span) -> Result<ConcurrentStmt, VhdlError> {
        self.expect(TokenKind::Assert)?;
        let cond = self.parse_expr()?;
        let mut report = None;
        if self.match_token(&TokenKind::Report) {
            if let TokenKind::StringLit(s) = self.peek().kind.clone() {
                self.advance();
                report = Some(s);
            }
        }
        let mut severity = None;
        if self.match_token(&TokenKind::Severity) {
            severity = Some(self.expect_ident()?);
        }
        self.expect(TokenKind::Semicolon)?;
        Ok(ConcurrentStmt::Assert {
            cond,
            report,
            severity,
            span,
        })
    }

    // --- Sequential Statements ---
    fn parse_seq_stmt(&mut self) -> Result<SeqStmt, VhdlError> {
        let tok = self.peek().clone();
        match tok.kind {
            TokenKind::If => self.parse_if(),
            TokenKind::Case => self.parse_case(),
            TokenKind::For => self.parse_for_loop(),
            TokenKind::While => self.parse_while_loop(),
            TokenKind::Wait => self.parse_wait(),
            TokenKind::Assert => self.parse_seq_assert(),
            TokenKind::Null => {
                self.advance();
                self.expect(TokenKind::Semicolon)?;
                Ok(SeqStmt::Null(tok.span))
            }
            _ => {
                // Target <= expr (signal assign) or Target := expr (variable assign)
                let target = self.parse_target_expr()?;
                let next_tok = self.advance();
                match next_tok.kind {
                    TokenKind::SignalAssign | TokenKind::Lt => {
                        let value = self.parse_expr()?;
                        let mut delay = None;
                        if self.match_token(&TokenKind::After) {
                            delay = Some(self.parse_expr()?);
                        }
                        self.expect(TokenKind::Semicolon)?;
                        Ok(SeqStmt::SignalAssign {
                            target,
                            value,
                            delay,
                            span: tok.span,
                        })
                    }
                    TokenKind::Assign => {
                        let value = self.parse_expr()?;
                        self.expect(TokenKind::Semicolon)?;
                        Ok(SeqStmt::VariableAssign {
                            target,
                            value,
                            span: tok.span,
                        })
                    }
                    _ => Err(VhdlError::ParseError {
                        msg: format!(
                            "Expected '<=' or ':=' after target in sequential statement, found {:?}",
                            next_tok.kind
                        ),
                        span: next_tok.span,
                    }),
                }
            }
        }
    }

    fn parse_target_expr(&mut self) -> Result<TargetExpr, VhdlError> {
        let name = self.expect_ident()?;
        if self.match_token(&TokenKind::LParen) {
            let left = self.parse_expr()?;
            if self.match_token(&TokenKind::Downto) {
                let right = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(TargetExpr::Slice(
                    name,
                    RangeExpr {
                        left: Box::new(left),
                        direction: RangeDirection::Downto,
                        right: Box::new(right),
                    },
                ))
            } else if self.match_token(&TokenKind::To) {
                let right = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(TargetExpr::Slice(
                    name,
                    RangeExpr {
                        left: Box::new(left),
                        direction: RangeDirection::To,
                        right: Box::new(right),
                    },
                ))
            } else {
                self.expect(TokenKind::RParen)?;
                Ok(TargetExpr::Index(name, Box::new(left)))
            }
        } else {
            Ok(TargetExpr::Ident(name))
        }
    }

    fn parse_if(&mut self) -> Result<SeqStmt, VhdlError> {
        let start_tok = self.expect(TokenKind::If)?;
        let cond = self.parse_expr()?;
        self.expect(TokenKind::Then)?;

        let mut then_branch = Vec::new();
        while !matches!(
            self.peek_kind(),
            TokenKind::Elsif | TokenKind::Else | TokenKind::End | TokenKind::Eof
        ) {
            then_branch.push(self.parse_seq_stmt()?);
        }

        let mut elsif_branches = Vec::new();
        while self.match_token(&TokenKind::Elsif) {
            let el_cond = self.parse_expr()?;
            self.expect(TokenKind::Then)?;
            let mut el_stmts = Vec::new();
            while !matches!(
                self.peek_kind(),
                TokenKind::Elsif | TokenKind::Else | TokenKind::End | TokenKind::Eof
            ) {
                el_stmts.push(self.parse_seq_stmt()?);
            }
            elsif_branches.push((el_cond, el_stmts));
        }

        let mut else_branch = None;
        if self.match_token(&TokenKind::Else) {
            let mut el_stmts = Vec::new();
            while !matches!(self.peek_kind(), TokenKind::End | TokenKind::Eof) {
                el_stmts.push(self.parse_seq_stmt()?);
            }
            else_branch = Some(el_stmts);
        }

        self.expect(TokenKind::End)?;
        self.expect(TokenKind::If)?;
        self.expect(TokenKind::Semicolon)?;

        Ok(SeqStmt::If {
            cond,
            then_branch,
            elsif_branches,
            else_branch,
            span: start_tok.span,
        })
    }

    fn parse_case(&mut self) -> Result<SeqStmt, VhdlError> {
        let start_tok = self.expect(TokenKind::Case)?;
        let expr = self.parse_expr()?;
        self.expect(TokenKind::Is)?;

        let mut arms = Vec::new();
        while *self.peek_kind() == TokenKind::When {
            let when_span = self.advance().span; // consume 'when'
            let mut choices = Vec::new();

            loop {
                if self.match_token(&TokenKind::Others) {
                    choices.push(Choice::Others);
                } else {
                    let c_expr = self.parse_expr()?;
                    if self.match_token(&TokenKind::Downto) {
                        let right = self.parse_expr()?;
                        choices.push(Choice::Range(RangeExpr {
                            left: Box::new(c_expr),
                            direction: RangeDirection::Downto,
                            right: Box::new(right),
                        }));
                    } else if self.match_token(&TokenKind::To) {
                        let right = self.parse_expr()?;
                        choices.push(Choice::Range(RangeExpr {
                            left: Box::new(c_expr),
                            direction: RangeDirection::To,
                            right: Box::new(right),
                        }));
                    } else {
                        choices.push(Choice::Expr(c_expr));
                    }
                }

                if !self.match_token(&TokenKind::Or) && !self.match_token(&TokenKind::Slash) {
                    // In VHDL, choices are separated by '|' (or '/' in some representations)
                    break;
                }
            }

            self.expect(TokenKind::Arrow)?;

            let mut stmts = Vec::new();
            while !matches!(
                self.peek_kind(),
                TokenKind::When | TokenKind::End | TokenKind::Eof
            ) {
                stmts.push(self.parse_seq_stmt()?);
            }

            arms.push(CaseArm {
                choices,
                stmts,
                span: when_span,
            });
        }

        self.expect(TokenKind::End)?;
        self.expect(TokenKind::Case)?;
        self.expect(TokenKind::Semicolon)?;

        Ok(SeqStmt::Case {
            expr,
            arms,
            span: start_tok.span,
        })
    }

    fn parse_for_loop(&mut self) -> Result<SeqStmt, VhdlError> {
        let start_tok = self.expect(TokenKind::For)?;
        let var_name = self.expect_ident()?;
        self.expect(TokenKind::In)?;
        let range = self.parse_range()?;
        self.expect(TokenKind::Loop)?;

        let mut body = Vec::new();
        while !matches!(self.peek_kind(), TokenKind::End | TokenKind::Eof) {
            body.push(self.parse_seq_stmt()?);
        }

        self.expect(TokenKind::End)?;
        self.expect(TokenKind::Loop)?;
        if let TokenKind::Ident(_) = self.peek_kind() {
            self.advance();
        }
        self.expect(TokenKind::Semicolon)?;

        Ok(SeqStmt::ForLoop {
            var_name,
            range,
            body,
            span: start_tok.span,
        })
    }

    fn parse_while_loop(&mut self) -> Result<SeqStmt, VhdlError> {
        let start_tok = self.expect(TokenKind::While)?;
        let cond = self.parse_expr()?;
        self.expect(TokenKind::Loop)?;

        let mut body = Vec::new();
        while !matches!(self.peek_kind(), TokenKind::End | TokenKind::Eof) {
            body.push(self.parse_seq_stmt()?);
        }

        self.expect(TokenKind::End)?;
        self.expect(TokenKind::Loop)?;
        if let TokenKind::Ident(_) = self.peek_kind() {
            self.advance();
        }
        self.expect(TokenKind::Semicolon)?;

        Ok(SeqStmt::WhileLoop {
            cond,
            body,
            span: start_tok.span,
        })
    }

    fn parse_wait(&mut self) -> Result<SeqStmt, VhdlError> {
        let start_tok = self.expect(TokenKind::Wait)?;

        if self.match_token(&TokenKind::For) {
            let duration = self.parse_expr()?;
            self.expect(TokenKind::Semicolon)?;
            return Ok(SeqStmt::WaitFor {
                duration,
                span: start_tok.span,
            });
        }

        if self.match_token(&TokenKind::Until) {
            let cond = self.parse_expr()?;
            self.expect(TokenKind::Semicolon)?;
            return Ok(SeqStmt::WaitUntil {
                cond,
                span: start_tok.span,
            });
        }

        if self.match_token(&TokenKind::On) {
            let mut signals = vec![self.expect_ident()?];
            while self.match_token(&TokenKind::Comma) {
                signals.push(self.expect_ident()?);
            }
            self.expect(TokenKind::Semicolon)?;
            return Ok(SeqStmt::WaitOn {
                signals,
                span: start_tok.span,
            });
        }

        self.expect(TokenKind::Semicolon)?;
        Ok(SeqStmt::Wait {
            span: start_tok.span,
        })
    }

    fn parse_seq_assert(&mut self) -> Result<SeqStmt, VhdlError> {
        let start_tok = self.expect(TokenKind::Assert)?;
        let cond = self.parse_expr()?;
        let mut report = None;
        if self.match_token(&TokenKind::Report) {
            if let TokenKind::StringLit(s) = self.peek().kind.clone() {
                self.advance();
                report = Some(s);
            }
        }
        let mut severity = None;
        if self.match_token(&TokenKind::Severity) {
            severity = Some(self.expect_ident()?);
        }
        self.expect(TokenKind::Semicolon)?;
        Ok(SeqStmt::Assert {
            cond,
            report,
            severity,
            span: start_tok.span,
        })
    }

    // --- Expressions (Pratt / Precedence Climbing) ---
    pub fn parse_expr(&mut self) -> Result<Expr, VhdlError> {
        self.parse_expr_logical()
    }

    // Logical operators: and, or, nand, nor, xor, xnor
    fn parse_expr_logical(&mut self) -> Result<Expr, VhdlError> {
        let mut lhs = self.parse_expr_relational()?;

        while matches!(
            self.peek_kind(),
            TokenKind::And
                | TokenKind::Or
                | TokenKind::Nand
                | TokenKind::Nor
                | TokenKind::Xor
                | TokenKind::Xnor
        ) {
            let tok = self.advance();
            let op = match tok.kind {
                TokenKind::And => BinaryOpKind::And,
                TokenKind::Or => BinaryOpKind::Or,
                TokenKind::Nand => BinaryOpKind::Nand,
                TokenKind::Nor => BinaryOpKind::Nor,
                TokenKind::Xor => BinaryOpKind::Xor,
                TokenKind::Xnor => BinaryOpKind::Xnor,
                _ => unreachable!(),
            };
            let rhs = self.parse_expr_relational()?;
            let span = lhs.span();
            lhs = Expr::BinaryOp(op, Box::new(lhs), Box::new(rhs), span);
        }

        Ok(lhs)
    }

    // Relational: =, /=, <, <=, >, >=
    fn parse_expr_relational(&mut self) -> Result<Expr, VhdlError> {
        let mut lhs = self.parse_expr_shift()?;

        while matches!(
            self.peek_kind(),
            TokenKind::Eq
                | TokenKind::Ne
                | TokenKind::Lt
                | TokenKind::SignalAssign // could be <= comparison
                | TokenKind::Gt
                | TokenKind::Ge
        ) {
            let tok = self.advance();
            let op = match tok.kind {
                TokenKind::Eq => BinaryOpKind::Eq,
                TokenKind::Ne => BinaryOpKind::Ne,
                TokenKind::Lt => BinaryOpKind::Lt,
                TokenKind::SignalAssign => BinaryOpKind::Le,
                TokenKind::Gt => BinaryOpKind::Gt,
                TokenKind::Ge => BinaryOpKind::Ge,
                _ => unreachable!(),
            };
            let rhs = self.parse_expr_shift()?;
            let span = lhs.span();
            lhs = Expr::BinaryOp(op, Box::new(lhs), Box::new(rhs), span);
        }

        Ok(lhs)
    }

    // Shift: sll, srl, sla, sra, rol, ror
    fn parse_expr_shift(&mut self) -> Result<Expr, VhdlError> {
        let mut lhs = self.parse_expr_additive()?;

        while matches!(
            self.peek_kind(),
            TokenKind::Sll
                | TokenKind::Srl
                | TokenKind::Sla
                | TokenKind::Sra
                | TokenKind::Rol
                | TokenKind::Ror
        ) {
            let tok = self.advance();
            let op = match tok.kind {
                TokenKind::Sll => BinaryOpKind::Sll,
                TokenKind::Srl => BinaryOpKind::Srl,
                TokenKind::Sla => BinaryOpKind::Sla,
                TokenKind::Sra => BinaryOpKind::Sra,
                TokenKind::Rol => BinaryOpKind::Rol,
                TokenKind::Ror => BinaryOpKind::Ror,
                _ => unreachable!(),
            };
            let rhs = self.parse_expr_additive()?;
            let span = lhs.span();
            lhs = Expr::BinaryOp(op, Box::new(lhs), Box::new(rhs), span);
        }

        Ok(lhs)
    }

    // Additive: +, -, & (concatenation)
    fn parse_expr_additive(&mut self) -> Result<Expr, VhdlError> {
        let mut lhs = self.parse_expr_multiplicative()?;

        while matches!(
            self.peek_kind(),
            TokenKind::Plus | TokenKind::Minus | TokenKind::Ampersand
        ) {
            let tok = self.advance();
            let op = match tok.kind {
                TokenKind::Plus => BinaryOpKind::Add,
                TokenKind::Minus => BinaryOpKind::Sub,
                TokenKind::Ampersand => BinaryOpKind::Concat,
                _ => unreachable!(),
            };
            let rhs = self.parse_expr_multiplicative()?;
            let span = lhs.span();
            lhs = Expr::BinaryOp(op, Box::new(lhs), Box::new(rhs), span);
        }

        Ok(lhs)
    }

    // Multiplicative: *, /, mod, rem
    fn parse_expr_multiplicative(&mut self) -> Result<Expr, VhdlError> {
        let mut lhs = self.parse_expr_unary()?;

        while matches!(
            self.peek_kind(),
            TokenKind::Star | TokenKind::Slash | TokenKind::Mod | TokenKind::Rem
        ) {
            let tok = self.advance();
            let op = match tok.kind {
                TokenKind::Star => BinaryOpKind::Mul,
                TokenKind::Slash => BinaryOpKind::Div,
                TokenKind::Mod => BinaryOpKind::Mod,
                TokenKind::Rem => BinaryOpKind::Rem,
                _ => unreachable!(),
            };
            let rhs = self.parse_expr_unary()?;
            let span = lhs.span();
            lhs = Expr::BinaryOp(op, Box::new(lhs), Box::new(rhs), span);
        }

        Ok(lhs)
    }

    // Unary: not, abs, +, -
    fn parse_expr_unary(&mut self) -> Result<Expr, VhdlError> {
        let tok = self.peek().clone();
        match tok.kind {
            TokenKind::Not => {
                self.advance();
                let sub = self.parse_expr_unary()?;
                Ok(Expr::UnaryOp(UnaryOpKind::Not, Box::new(sub), tok.span))
            }
            TokenKind::Abs => {
                self.advance();
                let sub = self.parse_expr_unary()?;
                Ok(Expr::UnaryOp(UnaryOpKind::Abs, Box::new(sub), tok.span))
            }
            TokenKind::Minus => {
                self.advance();
                let sub = self.parse_expr_unary()?;
                Ok(Expr::UnaryOp(UnaryOpKind::Neg, Box::new(sub), tok.span))
            }
            TokenKind::Plus => {
                self.advance();
                let sub = self.parse_expr_unary()?;
                Ok(Expr::UnaryOp(UnaryOpKind::Pos, Box::new(sub), tok.span))
            }
            _ => self.parse_expr_primary(),
        }
    }

    // Primary: literals, identifiers, index, slice, call, aggregate, parenthesized
    fn parse_expr_primary(&mut self) -> Result<Expr, VhdlError> {
        let tok = self.peek().clone();

        // Check if parenthesized expression or aggregate: (others => '0')
        if tok.kind == TokenKind::LParen {
            return self.parse_paren_or_aggregate();
        }

        match tok.kind {
            TokenKind::CharLit(c) => {
                self.advance();
                let lv = LogicVal::from_char(c).unwrap_or(LogicVal::U);
                Ok(Expr::LitLogic(lv, tok.span))
            }
            TokenKind::BitStringLit(bits) => {
                self.advance();
                Ok(Expr::LitVector(bits, tok.span))
            }
            TokenKind::StringLit(s) => {
                self.advance();
                // Check if string can be interpreted as vector of logic values (e.g. "0101")
                let mut is_logic_vec = !s.is_empty();
                let mut bits = Vec::new();
                for c in s.chars() {
                    if let Some(lv) = LogicVal::from_char(c) {
                        bits.push(lv);
                    } else {
                        is_logic_vec = false;
                        break;
                    }
                }
                if is_logic_vec {
                    Ok(Expr::LitVector(bits, tok.span))
                } else {
                    Ok(Expr::LitString(s, tok.span))
                }
            }
            TokenKind::IntLit(val) => {
                self.advance();
                // Check if followed by time unit: ns, ps, us, ms, sec, fs
                if let TokenKind::Ident(ref unit) = self.peek().kind {
                    let multiplier = match unit.as_str() {
                        "fs" => Some(1u64),
                        "ps" => Some(1_000u64),
                        "ns" => Some(1_000_000u64),
                        "us" => Some(1_000_000_000u64),
                        "ms" => Some(1_000_000_000_000u64),
                        "sec" | "s" => Some(1_000_000_000_000_000u64),
                        _ => None,
                    };
                    if let Some(mult) = multiplier {
                        self.advance(); // consume unit
                        let fs = (val as u64).saturating_mul(mult);
                        return Ok(Expr::LitTime(fs, tok.span));
                    }
                }
                Ok(Expr::LitInt(val, tok.span))
            }
            TokenKind::RealLit(val) => {
                self.advance();
                // Check if followed by time unit
                if let TokenKind::Ident(ref unit) = self.peek().kind {
                    let multiplier = match unit.as_str() {
                        "fs" => Some(1.0f64),
                        "ps" => Some(1_000.0f64),
                        "ns" => Some(1_000_000.0f64),
                        "us" => Some(1_000_000_000.0f64),
                        "ms" => Some(1_000_000_000_000.0f64),
                        "sec" | "s" => Some(1_000_000_000_000_000.0f64),
                        _ => None,
                    };
                    if let Some(mult) = multiplier {
                        self.advance();
                        let fs = (val * mult).round() as u64;
                        return Ok(Expr::LitTime(fs, tok.span));
                    }
                }
                Ok(Expr::LitInt(val.round() as i64, tok.span))
            }
            TokenKind::Ident(name) => {
                self.advance();
                let lower = name.to_ascii_lowercase();
                if lower == "true" {
                    return Ok(Expr::LitBool(true, tok.span));
                }
                if lower == "false" {
                    return Ok(Expr::LitBool(false, tok.span));
                }

                // Check for attribute 'event, 'stable, 'image, etc.
                if self.match_token(&TokenKind::Tick) {
                    let attr_name = self.expect_ident()?;
                    if attr_name == "event" {
                        // sig'event -> synthesized as function call
                        return Ok(Expr::FunctionCall(
                            format!("{lower}'event"),
                            vec![Expr::Ident(lower, tok.span)],
                            tok.span,
                        ));
                    }
                }

                // Check for function call or indexing/slicing: name ( args / index / slice )
                if self.match_token(&TokenKind::LParen) {
                    // Check if slice: left downto/to right
                    let first = self.parse_expr()?;
                    if self.match_token(&TokenKind::Downto) {
                        let right = self.parse_expr()?;
                        self.expect(TokenKind::RParen)?;
                        return Ok(Expr::Slice(
                            lower,
                            RangeExpr {
                                left: Box::new(first),
                                direction: RangeDirection::Downto,
                                right: Box::new(right),
                            },
                            tok.span,
                        ));
                    } else if self.match_token(&TokenKind::To) {
                        let right = self.parse_expr()?;
                        self.expect(TokenKind::RParen)?;
                        return Ok(Expr::Slice(
                            lower,
                            RangeExpr {
                                left: Box::new(first),
                                direction: RangeDirection::To,
                                right: Box::new(right),
                            },
                            tok.span,
                        ));
                    }

                    // Could be multiple arguments or index
                    let mut args = vec![first];
                    while self.match_token(&TokenKind::Comma) {
                        args.push(self.parse_expr()?);
                    }
                    self.expect(TokenKind::RParen)?;

                    if args.len() == 1 {
                        // Could be function call or indexing or type conversion
                        // We classify known type casts or function calls:
                        if lower == "rising_edge"
                            || lower == "falling_edge"
                            || lower == "to_integer"
                            || lower == "to_unsigned"
                            || lower == "to_signed"
                            || lower == "resize"
                            || lower == "not"
                        {
                            Ok(Expr::FunctionCall(lower, args, tok.span))
                        } else if lower == "std_logic_vector"
                            || lower == "unsigned"
                            || lower == "signed"
                        {
                            Ok(Expr::TypeCast(lower, Box::new(args.remove(0)), tok.span))
                        } else {
                            // Default to index (will resolve to call if function in scope during eval)
                            Ok(Expr::Index(lower, Box::new(args.remove(0)), tok.span))
                        }
                    } else {
                        Ok(Expr::FunctionCall(lower, args, tok.span))
                    }
                } else {
                    Ok(Expr::Ident(lower, tok.span))
                }
            }
            _ => Err(VhdlError::ParseError {
                msg: format!("Unexpected token in expression: {:?}", tok.kind),
                span: tok.span,
            }),
        }
    }

    fn parse_paren_or_aggregate(&mut self) -> Result<Expr, VhdlError> {
        let start_tok = self.expect(TokenKind::LParen)?;

        // Check if aggregate: (others => '0')
        if self.match_token(&TokenKind::Others) {
            self.expect(TokenKind::Arrow)?;
            let val = self.parse_expr()?;
            self.expect(TokenKind::RParen)?;
            return Ok(Expr::Aggregate(
                vec![AggregateChoice::Others(Box::new(val))],
                start_tok.span,
            ));
        }

        let first = self.parse_expr()?;

        // If followed by '=>', it's an aggregate like (0 => '1', others => '0')
        if self.match_token(&TokenKind::Arrow) {
            let val = self.parse_expr()?;
            let mut choices = vec![AggregateChoice::Index(Box::new(first), Box::new(val))];

            while self.match_token(&TokenKind::Comma) {
                if self.match_token(&TokenKind::Others) {
                    self.expect(TokenKind::Arrow)?;
                    let oth_val = self.parse_expr()?;
                    choices.push(AggregateChoice::Others(Box::new(oth_val)));
                } else {
                    let idx = self.parse_expr()?;
                    self.expect(TokenKind::Arrow)?;
                    let v = self.parse_expr()?;
                    choices.push(AggregateChoice::Index(Box::new(idx), Box::new(v)));
                }
            }
            self.expect(TokenKind::RParen)?;
            return Ok(Expr::Aggregate(choices, start_tok.span));
        }

        // Check if comma-separated list of values (e.g. ('0', '1', '0'))
        if self.match_token(&TokenKind::Comma) {
            let mut list = vec![AggregateChoice::Positional(Box::new(first))];
            loop {
                let e = self.parse_expr()?;
                list.push(AggregateChoice::Positional(Box::new(e)));
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
            self.expect(TokenKind::RParen)?;
            return Ok(Expr::Aggregate(list, start_tok.span));
        }

        self.expect(TokenKind::RParen)?;
        Ok(first)
    }
}
