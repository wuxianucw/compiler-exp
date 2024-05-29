use lexer::Lexer;
use std::fmt::Write;

#[test]
fn it_works() {
    let program = include_bytes!("./00_program.txt");
    let lexer = Lexer::new(&program[..]).unwrap();
    let mut result = String::new();
    lexer.for_each(|x| writeln!(result, "{}", x.unwrap()).unwrap());
    assert_eq!(result, include_str!("./00_expected_result.txt"));
}
