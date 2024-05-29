use std::{env, fs::File, io::BufReader};

use anyhow::Context;
use lexer::Lexer;

fn main() -> anyhow::Result<()> {
    let mut args = env::args();
    let path = args.nth(1).context("no input file")?;
    let file = File::open(path).context("failed to open file")?;
    let lexer = Lexer::new(BufReader::new(file))?;

    for token in lexer {
        println!("{}", token?);
    }

    Ok(())
}
