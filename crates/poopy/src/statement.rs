use crate::{
    binding::BindingDef,
    env::Env,
    expr::Expr,
    func::{self, FuncDef},
    val::Val,
};

#[derive(Debug, PartialEq)]
pub(crate) enum Statement {
    BindingDef(BindingDef),
    Expr(Expr),
    FuncDef(FuncDef),
}

impl Statement {
    pub(crate) fn new(s: &str) -> Result<(&str, Self), String> {
        BindingDef::new(s)
            .map(|(s, bind)| (s, Self::BindingDef(bind)))
            .or_else(|_| Expr::new(s).map(|(s, expr)| (s, Self::Expr(expr))))
            .or_else(|_| FuncDef::new(s).map(|(s, func)| (s, Self::FuncDef(func))))
    }

    pub(crate) fn eval(&self, env: &mut Env) -> Result<Val, String> {
        match self {
            Self::BindingDef(bind) => {
                bind.eval(env)?;
                Ok(Val::Unit)
            }
            Self::Expr(expr) => expr.eval(env),
            Self::FuncDef(func) => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::expr::{Number, binding_usage::BindingUsage};

    use super::*;

    #[test]
    fn parse_binding_def() {
        assert_eq!(
            Statement::new("let a = 10"),
            Ok((
                "",
                Statement::BindingDef(BindingDef {
                    name: "a".to_string(),
                    val: Expr::Number(Number(10)),
                })
            ))
        )
    }

    #[test]
    fn parse_func_def() {
        assert_eq!(
            Statement::new("fn iden x => x"),
            Ok((
                "",
                Statement::FuncDef(FuncDef {
                    name: "iden".to_string(),
                    params: vec!["x".to_string()],
                    body: Box::new(Statement::Expr(Expr::BindingUsage(BindingUsage {
                        name: "x".to_string(),
                    })))
                })
            ))
        )
    }
}
