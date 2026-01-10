use std::io::{self, Write};

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();
    let mut inp = String::new();
    let mut env = poopy::Env::default();

    loop {
        write!(stdout, "-> ")?;
        stdout.flush()?;

        stdin.read_line(&mut inp)?;

        match run(inp.trim(), &mut env) {
            Ok(Some(val)) => writeln!(stdout, "{}", val)?,
            Ok(None) => {}
            Err(msg) => writeln!(stderr, "{}", msg)?,
        }

        inp.clear();
    }
}

fn run(inp: &str, env: &mut poopy::Env) -> Result<Option<poopy::Val>, String> {
    let parse = poopy::parse(inp).map_err(|msg| format!("Parse Error: {}", msg))?;

    let eval = parse
        .eval(env)
        .map_err(|msg| format!("Evaluation Error: {}", msg))?;

    if eval == poopy::Val::Unit {
        Ok(None)
    } else {
        Ok(Some(eval))
    }
}
