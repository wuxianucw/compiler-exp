use std::{env, fs::read_to_string};

use parser::{ast::Stmt, grammar::StmtListParser};
use lalrpop_util::ParseError;

fn print_stmt(stmt: &Stmt, buf: &mut String) {
    match stmt {
        Stmt::If(_, s) => {
            buf.push_str(" if-then 分支语句，嵌套");
            print_stmt(s, buf);
        }
        Stmt::IfElse(_, s, r) => {
            buf.push_str(" if-then-else 分支语句，then 分支嵌套");
            print_stmt(s, buf);
            buf.push_str("，else 分支嵌套");
            print_stmt(r, buf);
        }
        Stmt::While(_, s) => {
            buf.push_str(" while-do 循环语句，嵌套");
            print_stmt(s, buf);
        }
        Stmt::Assign(_, _) => {
            buf.push_str("赋值语句");
        }
        Stmt::VarDecl(_, _) => {
            buf.push_str("变量声明语句");
        }
        Stmt::Block(_) => {
            buf.push_str("语句块");
        }
        Stmt::Expr(_) => {
            buf.push_str("表达式");
        }
    }
}

fn main() {
    let mut args = env::args();
    let path = args.nth(1).unwrap();
    let file_contents = read_to_string(path).unwrap();
    let parser = StmtListParser::new();

    for line in file_contents.lines() {
        let mut buf = String::new();
        match parser.parse(line) {
            Ok(ast) => {
                for stmt in ast {
                    print_stmt(&stmt, &mut buf);
                    println!("{}", buf.trim_start());
                }
            }
            Err(e) => {
                let location = match e {
                    ParseError::InvalidToken { location } => location,
                    ParseError::UnrecognizedEof { location, .. } => location,
                    ParseError::UnrecognizedToken { token, .. } => token.0,
                    ParseError::ExtraToken { token } => token.0,
                    ParseError::User { .. } => todo!(),
                };
                println!("语法错误，位置 {location}");
            }
        }
    }
}
