use crate::{env::Env, expr::Expr, utils, val::Val};

#[derive(Debug, PartialEq)]
pub(crate) struct BindingDef {
    pub(crate) name: String,
    pub(crate) val: Expr,
}

impl BindingDef {
    pub(crate) fn new(s: &str) -> Result<(&str, Self), String> {
        let s = utils::tag("let", s)?;
        let (s, _) = utils::extract_whitespace_atleast_one(s)?;

        let (s, name) = utils::extract_identifier(s)?;
        let (s, _) = utils::extract_whitespace(s);

        let s = utils::tag("=", s)?;
        let (s, _) = utils::extract_whitespace(s);

        let (s, val) = Expr::new(s)?;

        Ok((
            s,
            Self {
                name: name.to_string(),
                val,
            },
        ))
    }

    pub(crate) fn eval(&self, env: &mut Env) -> Result<Val, String> {
        let evaled = self.val.eval(env)?;
        env.store(self.name.clone(), evaled);
        Ok(Val::Unit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        expr::{Number, Op},
        statement::Statement,
    };

    #[test]
    fn parse_binding_def() {
        assert_eq!(
            BindingDef::new("let a = 10 / 2"),
            Ok((
                "",
                BindingDef {
                    name: "a".to_string(),
                    val: Expr::Operation {
                        lhs: Number(10),
                        rhs: Number(2),
                        op: Op::Div,
                    },
                },
            )),
        );
    }

    #[test]
    fn cannot_parse_binding_def_without_space_after_let() {
        assert_eq!(
            BindingDef::new("letaaa=1+2"),
            Err("expected a space".to_string()),
        );
    }

    #[test]
    fn eval_binding() {
        assert_eq!(
            Statement::BindingDef(BindingDef {
                name: "lmao".into(),
                val: Expr::Number(Number(69)),
            })
            .eval(&mut Env::default()),
            Ok(Val::Unit)
        );
    }

    #[test]
    fn eval_expr() {
        assert_eq!(
            Statement::Expr(Expr::Number(Number(420))).eval(&mut Env::default()),
            Ok(Val::Number(420))
        );
    }
}
