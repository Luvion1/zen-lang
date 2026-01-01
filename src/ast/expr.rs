use crate::token::*;

#[derive(Debug, Clone)]
pub enum Expr {
    IntegerLiteral {
        value: String,
        token: Token,
    },
    FloatLiteral {
        value: f64,
        token: Token,
    },
    StringLiteral {
        value: String,
        token: Token,
    },
    CharLiteral {
        value: char,
        token: Token,
    },
    BooleanLiteral {
        value: bool,
        token: Token,
    },
    Identifier {
        name: String,
        token: Token,
    },
    BinaryOp {
        left: Box<Expr>,
        op: Token,
        right: Box<Expr>,
    },
    UnaryOp {
        op: Token,
        operand: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        token: Token,
    },
    OwnershipTransfer {
        expr: Box<Expr>,
        token: Token,
    },
}
