use crate::token::*;

#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Text(String),
    Variable(String),
    Expression(String), // For function calls like add(result, result)
}

impl StringPart {
    pub fn is_static(&self) -> bool {
        matches!(self, StringPart::Text(_))
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            StringPart::Text(s) => Some(s),
            _ => None,
        }
    }
}

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
    InterpolatedString {
        parts: Vec<StringPart>,
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
    Borrow {
        expr: Box<Expr>,
        is_mutable: bool,
        token: Token,
    },
    FieldAccess {
        object: Box<Expr>,
        field: String,
        token: Token,
    },
    ArrayAccess {
        array: Box<Expr>,
        index: Box<Expr>,
        token: Token,
    },
    StructLiteral {
        struct_name: String,
        fields: Vec<(String, Expr)>,
        token: Token,
    },
    ModuleAccess {
        module: String,
        item: String,
        token: Token,
    },
}
