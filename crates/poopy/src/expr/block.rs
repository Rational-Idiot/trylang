use crate::env::Env;
use crate::statement::Statement;
use crate::utils;
use crate::val::Val;

#[derive(Debug, PartialEq)]
pub(crate) struct Block {
    pub(crate) stmts: Vec<Statement>,
}

impl Block {
    pub(crate) fn new(s: &str) -> Result<(&str, Self), String> {
        let s = utils::tag("{", s)?;
        let (s, _) = utils::extract_whitespace(s);

        let (s, stmts) = utils::sequence(Statement::new, s)?;

        let (s, _) = utils::extract_whitespace(s);
        let s = utils::tag("}", s)?;

        Ok((s, Block { stmts }))
    }

    pub(crate) fn eval(&self, env: &mut Env) -> Result<Val, String> {
        if self.stmts.is_empty() {
            return Ok(Val::Unit);
        }

        let mut env = env.create_child();

        let rem = &self.stmts[..self.stmts.len() - 1];

        for stmt in rem {
            stmt.eval(&mut env)?;
        }

        self.stmts.last().unwrap().eval(&mut env)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Expr, Number};
    use super::*;
    use crate::binding::BindingDef;
    use crate::env::Env;
    use crate::expr::Op;
    use crate::expr::binding_usage::BindingUsage;
    use crate::val::Val;

    #[test]
    fn parse_empty_block() {
        assert_eq!(Block::new("{}"), Ok(("", Block { stmts: Vec::new() })));
    }

    #[test]
    fn parse_empty_block_with_whitespace() {
        assert_eq!(Block::new("{   }"), Ok(("", Block { stmts: Vec::new() })));
    }

    #[test]
    fn parse_block_with_one_stmt() {
        assert_eq!(
            Block::new("{ 5 }"),
            Ok((
                "",
                Block {
                    stmts: vec![Statement::Expr(Expr::Number(Number(5)))],
                },
            )),
        );
    }

    #[test]
    fn parse_block_with_multiple_stmts() {
        assert_eq!(
            Block::new(
                "{
    let a = 10
    let b = a
    b
}",
            ),
            Ok((
                "",
                Block {
                    stmts: vec![
                        Statement::BindingDef(BindingDef {
                            name: "a".to_string(),
                            val: Expr::Number(Number(10)),
                        }),
                        Statement::BindingDef(BindingDef {
                            name: "b".to_string(),
                            val: Expr::BindingUsage(BindingUsage {
                                name: "a".to_string(),
                            }),
                        }),
                        Statement::Expr(Expr::BindingUsage(BindingUsage {
                            name: "b".to_string(),
                        })),
                    ],
                },
            )),
        );
    }

    #[test]
    fn eval_block() {
        assert_eq!(
            Expr::Block(Block {
                stmts: vec![Statement::Expr(Expr::Number(Number(10)))],
            })
            .eval(&mut Env::default()),
            Ok(Val::Number(10)),
        );
    }

    #[test]
    fn eval_empty_block() {
        assert_eq!(
            Block { stmts: Vec::new() }.eval(&mut Env::default()),
            Ok(Val::Unit),
        );
    }

    #[test]
    fn eval_block_with_one_expr() {
        assert_eq!(
            Block {
                stmts: vec![Statement::Expr(Expr::Number(Number(25)))],
            }
            .eval(&mut Env::default()),
            Ok(Val::Number(25)),
        );
    }

    #[test]
    fn eval_block_with_binding_def_and_usage() {
        assert_eq!(
            Block {
                stmts: vec![
                    Statement::BindingDef(BindingDef {
                        name: "one".to_string(),
                        val: Expr::Number(Number(1)),
                    }),
                    Statement::Expr(Expr::BindingUsage(BindingUsage {
                        name: "one".to_string(),
                    })),
                ],
            }
            .eval(&mut Env::default()),
            Ok(Val::Number(1)),
        );
    }

    #[test]
    fn eval_block_with_multiple_binding_defs() {
        assert_eq!(
            Block {
                stmts: vec![
                    Statement::BindingDef(BindingDef {
                        name: "foo".to_string(),
                        val: Expr::Number(Number(5)),
                    }),
                    Statement::BindingDef(BindingDef {
                        name: "bar".to_string(),
                        val: Expr::Number(Number(4)),
                    }),
                    Statement::BindingDef(BindingDef {
                        name: "baz".to_string(),
                        val: Expr::Number(Number(3)),
                    }),
                ],
            }
            .eval(&mut Env::default()),
            Ok(Val::Unit),
        );
    }

    #[test]
    fn eval_block_with_multiple_exprs() {
        assert_eq!(
            Block {
                stmts: vec![
                    Statement::Expr(Expr::Number(Number(100))),
                    Statement::Expr(Expr::Number(Number(30))),
                    Statement::Expr(Expr::Operation {
                        lhs: Number(10),
                        rhs: Number(7),
                        op: Op::Sub,
                    }),
                ],
            }
            .eval(&mut Env::default()),
            Ok(Val::Number(3)),
        );
    }

    #[test]
    fn eval_block_using_bindings_from_parent_env() {
        let mut env = Env::default();
        env.store("foo".to_string(), Val::Number(2));

        assert_eq!(
            Block {
                stmts: vec![
                    Statement::BindingDef(BindingDef {
                        name: "baz".to_string(),
                        val: Expr::BindingUsage(BindingUsage {
                            name: "foo".to_string(),
                        }),
                    }),
                    Statement::Expr(Expr::BindingUsage(BindingUsage {
                        name: "baz".to_string(),
                    })),
                ],
            }
            .eval(&mut env),
            Ok(Val::Number(2)),
        );
    }
}
