use std::{collections::HashMap, env, fmt::Display, fs::read_to_string, process::exit};

use compiler::{
    ast::{Expr, Opcode, Stmt, Type},
    grammar::StmtListParser,
};
use lalrpop_util::ParseError;

type Quad = (&'static str, QuadItem, QuadItem, QuadItem);

#[derive(Debug, Clone)]
enum QuadItem {
    None,
    Num(u32),
    Id(String),
    TFList(usize, usize),
}

impl QuadItem {
    fn to_num(&self) -> u32 {
        if let QuadItem::Num(x) = self {
            *x
        } else {
            panic!()
        }
    }
}

impl From<u32> for QuadItem {
    fn from(value: u32) -> Self {
        QuadItem::Num(value)
    }
}

impl From<usize> for QuadItem {
    fn from(value: usize) -> Self {
        QuadItem::Num(value as u32)
    }
}

impl From<String> for QuadItem {
    fn from(value: String) -> Self {
        QuadItem::Id(value)
    }
}

impl From<()> for QuadItem {
    fn from(_value: ()) -> Self {
        QuadItem::None
    }
}

impl Display for QuadItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuadItem::None => "_".fmt(f),
            QuadItem::Num(num) => num.fmt(f),
            QuadItem::Id(id) => id.fmt(f),
            QuadItem::TFList(_, _) => "<err>".fmt(f),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct State {
    sym_table: HashMap<String, Type>,
    quads: Vec<Quad>,
    tmp_var_id: usize,
}

impl State {
    const QUAD_ID_OFFSET: usize = 100;

    fn request_tmp_var(&mut self) -> QuadItem {
        let id = self.tmp_var_id;
        self.tmp_var_id += 1;
        QuadItem::Id(format!("${id}"))
    }

    fn quad_id(&self) -> usize {
        self.quads.len() + Self::QUAD_ID_OFFSET
    }

    fn emit<A, B, R>(&mut self, op: &'static str, a: A, b: B, r: R) -> usize
    where
        A: Into<QuadItem>,
        B: Into<QuadItem>,
        R: Into<QuadItem>,
    {
        let qid = self.quad_id();
        self.quads.push((op, a.into(), b.into(), r.into()));
        qid
    }

    fn fill_list(&mut self, head: usize, value: u32) {
        if head < Self::QUAD_ID_OFFSET {
            return;
        }
        let idx = head - Self::QUAD_ID_OFFSET;
        if let QuadItem::Num(next) = self.quads[idx].3 {
            self.quads[idx].3 = QuadItem::Num(value);
            self.fill_list(next as usize, value);
        }
    }

    fn merge_list(&mut self, head1: usize, head2: usize) -> usize {
        let mut curr2 = head2;
        let mut tail2 = head2;
        while curr2 >= Self::QUAD_ID_OFFSET {
            tail2 = curr2;
            curr2 = self.quads[curr2 - Self::QUAD_ID_OFFSET].3.to_num() as usize;
        }
        if tail2 >= Self::QUAD_ID_OFFSET {
            self.quads[tail2 - Self::QUAD_ID_OFFSET].3 = QuadItem::Num(head1 as u32);
            head2
        } else {
            head1
        }
    }
}

fn visit_stmt(current: &Stmt, state: &mut State) {
    match current {
        Stmt::If(e, ts) => {
            let (val, _) = visit_expr(e, state);
            if let QuadItem::TFList(t, f) = val {
                state.fill_list(t, state.quad_id() as u32);
                for s in ts {
                    visit_stmt(s, state);
                }
                let next = state.emit("j", (), (), f);
                state.fill_list(next, state.quad_id() as u32);
            } else {
                panic!()
            }
        }
        Stmt::IfElse(e, ts, fs) => {
            let (val, _) = visit_expr(e, state);
            if let QuadItem::TFList(t, f) = val {
                state.fill_list(t, state.quad_id() as u32);
                for s in ts {
                    visit_stmt(s, state);
                }
                let next = state.emit("j", (), (), 0usize);
                state.fill_list(f, state.quad_id() as u32);
                for s in fs {
                    visit_stmt(s, state);
                }
                state.fill_list(next, state.quad_id() as u32);
            } else {
                panic!()
            }
        }
        Stmt::While(e, ts) => {
            let (val, entry) = visit_expr(e, state);
            if let QuadItem::TFList(t, f) = val {
                state.fill_list(t, state.quad_id() as u32);
                for s in ts {
                    visit_stmt(s, state);
                }
                state.emit("j", (), (), entry);
                state.fill_list(f, state.quad_id() as u32);
            } else {
                panic!()
            }
        }
        Stmt::Assign((var, left, right), e) => {
            if !state.sym_table.contains_key(var) {
                println!("语义错误：变量 {var} 未定义\n位置：{left}~{right}");
                exit(1);
            }
            match visit_expr(e, state).0 {
                QuadItem::Num(num) => {
                    state.emit(":=", num, (), var.clone());
                }
                QuadItem::Id(id) => {
                    state.emit(":=", id, (), var.clone());
                }
                QuadItem::TFList(t, f) => {
                    let tqid = state.emit(":=", 1u32, (), var.clone());
                    let fqid = state.emit(":=", 0u32, (), var.clone());
                    state.fill_list(t, tqid as u32);
                    state.fill_list(f, fqid as u32);
                }
                _ => panic!(),
            }
        }
        Stmt::VarDecl(vars, ty) => {
            for var in vars {
                state.sym_table.insert(var.clone(), *ty);
            }
        }
        Stmt::Block(ss) => {
            for s in ss {
                visit_stmt(s, state);
            }
        }
        Stmt::Expr(e) => {
            visit_expr(e, state);
        }
    }
}

fn visit_expr(current: &Expr, state: &mut State) -> (QuadItem, usize) {
    match current {
        Expr::Id(id, left, right) => {
            if !state.sym_table.contains_key(id) {
                println!("语义错误：变量 {id} 未定义\n位置：{left}~{right}");
                exit(1);
            }
            (QuadItem::Id(id.clone()), state.quad_id())
        }
        Expr::Num(num) => (QuadItem::Num(*num), state.quad_id()),
        Expr::Op(opcode, lhs, rhs) => {
            let (left_val, left_entry) = visit_expr(lhs, state);
            let (right_val, right_entry) = visit_expr(rhs, state);
            match opcode {
                Opcode::GreaterThan => {
                    let true_list = state.emit("j>", left_val, right_val, 0usize);
                    let false_list = state.emit("j", (), (), 0usize);
                    (QuadItem::TFList(true_list, false_list), left_entry)
                }
                Opcode::Equal => {
                    let true_list = state.emit("j=", left_val, right_val, 0usize);
                    let false_list = state.emit("j", (), (), 0usize);
                    (QuadItem::TFList(true_list, false_list), left_entry)
                }
                Opcode::LessThan => {
                    let true_list = state.emit("j<", left_val, right_val, 0usize);
                    let false_list = state.emit("j", (), (), 0usize);
                    (QuadItem::TFList(true_list, false_list), left_entry)
                }
                Opcode::Assign => {
                    state.emit(":=", right_val, (), left_val);
                    (QuadItem::None, left_entry)
                }
                Opcode::Add => {
                    let tmp = state.request_tmp_var();
                    state.emit("+", left_val, right_val, tmp.clone());
                    (tmp, left_entry)
                }
                Opcode::Sub => {
                    let tmp = state.request_tmp_var();
                    state.emit("-", left_val, right_val, tmp.clone());
                    (tmp, left_entry)
                }
                Opcode::Mul => {
                    let tmp = state.request_tmp_var();
                    state.emit("*", left_val, right_val, tmp.clone());
                    (tmp, left_entry)
                }
                Opcode::And => {
                    if let QuadItem::TFList(t, f) = left_val {
                        state.fill_list(t, right_entry as u32);
                        if let QuadItem::TFList(t2, f2) = right_val {
                            (QuadItem::TFList(t2, state.merge_list(f, f2)), left_entry)
                        } else {
                            panic!()
                        }
                    } else {
                        panic!()
                    }
                }
                Opcode::Or => {
                    if let QuadItem::TFList(t, f) = left_val {
                        state.fill_list(f, right_entry as u32);
                        if let QuadItem::TFList(t2, f2) = right_val {
                            (QuadItem::TFList(state.merge_list(t, t2), f2), left_entry)
                        } else {
                            panic!()
                        }
                    } else {
                        panic!()
                    }
                }
            }
        }
        Expr::Not(expr) => {
            let (val, entry) = visit_expr(expr, state);
            if let QuadItem::TFList(t, f) = val {
                (QuadItem::TFList(f, t), entry)
            } else {
                panic!()
            }
        }
    }
}

fn main() {
    let mut args = env::args();
    let path = args.nth(1).unwrap();
    let file_contents = read_to_string(path).unwrap();
    let parser = StmtListParser::new();
    let mut state = State::default();

    match parser.parse(&file_contents) {
        Ok(ast) => {
            for stmt in &ast {
                visit_stmt(stmt, &mut state);
            }
            println!("符号表：");
            for (k, v) in &state.sym_table {
                if !k.starts_with('$') {
                    println!(
                        "{k:>10} {:>5}",
                        match v {
                            Type::Bool => "bool",
                            Type::Int => "int",
                        }
                    )
                }
            }
            println!("四元式序列：");
            for (i, (op, a, b, r)) in state.quads.iter().enumerate() {
                println!("{}:  ({op:<3}, {a:>4}, {b:>4}, {r:>4})", i + 100);
            }
            println!("{}:  (nop,    _,    _,    _)", state.quad_id());
        }
        Err(e) => {
            match e {
                ParseError::InvalidToken { location } => {
                    println!("语法错误：无效的 token\n位置：{location}");
                }
                ParseError::UnrecognizedEof { location, expected } => {
                    println!(
                        "语法错误：预料之外的 EOF，语法不完整，期望 {}\n位置：{location}",
                        expected.join(", ")
                    );
                }
                ParseError::UnrecognizedToken { token, expected } => {
                    println!(
                        "语法错误：预料之外的 \"{}\"，期望 {}\n位置：{}~{}",
                        token.1 .1,
                        expected.join(", "),
                        token.0,
                        token.2
                    );
                }
                ParseError::ExtraToken { token } => {
                    println!(
                        "语法错误：预料之外的 \"{}\"，此处不应有此 token\n位置：{}~{}",
                        token.1 .1, token.0, token.2
                    );
                }
                _ => unreachable!(),
            }
            exit(1);
        }
    }
}
