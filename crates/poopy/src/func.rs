use crate::{
    statement::Statement,
    utils::{self, extract_whitespace},
};

#[derive(Debug, PartialEq)]
pub(crate) struct FuncDef {
    pub(crate) name: String,
    pub(crate) params: Vec<String>,
    pub(crate) body: Box<Statement>,
}

impl FuncDef {
    pub(crate) fn new(s: &str) -> Result<(&str, Self), String> {
        let s = utils::tag("fn", s)?;
        let (s, _) = utils::extract_whitespace_atleast_one(s)?;

        let (s, name) = utils::extract_identifier(s)?;
        let (s, _) = utils::extract_whitespace(s);

        // let (s, params) = utils::sequence(|s| utils::extract_identifier(s), s)?;

        let s = utils::tag("=>", s)?;
        let (s, _) = utils::extract_whitespace(s);

        let (s, body) = Statement::new(s)?;

        Ok((
            s,
            Self {
                name: name.to_string(),
                params: Vec::new(),
                body: Box::new(body),
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        expr::{Expr, block::Block},
        func::FuncDef,
        statement::Statement,
    };

    #[test]
    fn parse_empty_func() {
        assert_eq!(
            FuncDef::new("fn non => {}"),
            Ok((
                "",
                FuncDef {
                    name: "non".to_string(),
                    params: Vec::new(),
                    body: Box::new(Statement::Expr(Expr::Block(Block { stmts: Vec::new() })))
                }
            ))
        )
    }
}
