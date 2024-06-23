#[derive(Debug, Clone, Copy)]
pub enum Type {
    Integer,
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
    Id(String),
    Num(u32),
    Op(Opcode, Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    If(Box<Expr>, Box<Stmt>),
    IfElse(Box<Expr>, Box<Stmt>, Box<Stmt>),
    While(Box<Expr>, Box<Stmt>),
    Assign(String, Box<Expr>),
    VarDecl(Vec<String>, Type),
    Block(Vec<Stmt>),
    Expr(Box<Expr>),
}
