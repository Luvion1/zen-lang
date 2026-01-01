use crate::ast::expr::*;
use crate::token::*;

#[derive(Debug, Clone)]
pub struct ElseIfBranch {
    pub condition: Expr,
    pub body: Vec<Stmt>,
    pub token: Token,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    VariableDecl {
        name: String,
        type_annotation: Option<String>,
        initializer: Option<Expr>,
        is_mutable: bool,
        token: Token,
    },
    Assignment {
        target: Expr,
        value: Expr,
        token: Token,
    },
    FunctionDecl {
        name: String,
        params: Vec<(String, String)>,
        return_type: String,
        body: Vec<Stmt>,
        token: Token,
    },
    Return {
        value: Option<Expr>,
        token: Token,
    },
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_if_branches: Vec<ElseIfBranch>,
        else_branch: Option<Vec<Stmt>>,
        token: Token,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
        token: Token,
    },
    For {
        init: Option<Box<Stmt>>,
        condition: Option<Expr>,
        increment: Option<Expr>,
        body: Vec<Stmt>,
        token: Token,
    },
    Match {
        value: Expr,
        arms: Vec<(Expr, Vec<Stmt>)>,
        default: Option<Vec<Stmt>>,
        token: Token,
    },
    ExprStmt {
        expr: Expr,
    },
    Block {
        statements: Vec<Stmt>,
    },
}
