#[derive(Debug, Clone, Copy)]
pub enum Type {
    Int,
    Bool,
}

#[derive(Debug, Clone, Copy)]
pub enum Opcode {
    GreaterThan,
    Equal,
    LessThan,
    Assign,
    Add,
    Sub,
    Mul,
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Id(String, usize, usize),
    Num(u32),
    Op(Opcode, Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    If(Box<Expr>, Vec<Stmt>),
    IfElse(Box<Expr>, Vec<Stmt>, Vec<Stmt>),
    While(Box<Expr>, Vec<Stmt>),
    Assign((String, usize, usize), Box<Expr>),
    VarDecl(Vec<String>, Type),
    Block(Vec<Stmt>),
    Expr(Box<Expr>),
}
