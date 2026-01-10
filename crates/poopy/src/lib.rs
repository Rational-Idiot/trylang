mod binding;
mod env;
mod expr;
mod func;
mod statement;
mod utils;
mod val;

pub use env::Env;
pub use val::Val;

#[derive(Debug)]
pub struct Parse(statement::Statement);

impl Parse {
    pub fn eval(&self, env: &mut Env) -> Result<Val, String> {
        self.0.eval(env)
    }
}

pub fn parse(s: &str) -> Result<Parse, String> {
    let (s, stmt) = statement::Statement::new(s)?;

    if s.is_empty() {
        Ok(Parse(stmt))
    } else {
        Err("input wasn't fully consumes by parser".into())
    }
}
