#![allow(clippy::only_used_in_recursion)]

use ordered_float::OrderedFloat;

use crate::{val::Val, Compile, Node, Operator, Result};

// ANCHOR: interpreter
pub struct Interpreter;

impl Compile for Interpreter {
    type Output = Result<Val>;

    fn from_ast(ast: Vec<Node>) -> Self::Output {
        let mut ret = Val::Float(ordered_float::OrderedFloat(0f32));
        let evaluator = Eval::new();
        for node in ast {
            ret += evaluator.eval(&node);
        }
        Ok(ret)
    }
}
// ANCHOR_END: interpreter

// ANCHOR: interpreter_recursive
struct Eval;

impl Eval {
    pub fn new() -> Self {
        Self
    }
    // ANCHOR: interpreter_eval
    pub fn eval(&self, node: &Node) -> Val {
        match node {
            Node::Val(Val::Int(n)) => Val::Int(*n),
            Node::Val(Val::Float(f)) => Val::Float(*f),
            Node::UnaryExpr { op, child } => {
                let child = self.eval(child);
                match op {
                    Operator::Plus => child,
                    Operator::Minus => -child,
                    _ => {
                        panic!("Cannot apply {op} to the Val");
                    }
                }
            }
            Node::BinaryExpr { op, lhs, rhs } => {
                let lhs_ret = self.eval(lhs);
                let rhs_ret = self.eval(rhs);

                match op {
                    Operator::Plus => lhs_ret + rhs_ret,
                    Operator::Minus => lhs_ret - rhs_ret,
                    Operator::Multiply => lhs_ret * rhs_ret,
                    Operator::Divide => match (lhs_ret, rhs_ret) {
                        (Val::Int(l), Val::Int(r)) => Val::Float(OrderedFloat(l as f32 / r as f32)),
                        (Val::Float(l), Val::Float(r)) => Val::Float(l / r),
                        (Val::Float(l), Val::Int(r)) => Val::Float(l / OrderedFloat(r as f32)),
                        (Val::Int(l), Val::Float(r)) => Val::Float(OrderedFloat(l as f32) / r),
                    },
                }
            }
        }
    }
    // ANCHOR_END: interpreter_eval
}
// ANCHOR_END: interpreter_recursive

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basics() {
        let tests = [
            ("1", 1),
            ("1 + 2", 3),
            // ("(1 + 2)", 3), // Uncomment if parentheses are supported
            ("2 + (2 - 1)", 3),
            ("(2 + 3) - 1", 4),
            ("1 + ((2 + 3) - (2 + 3))", 1),
        ];

        for (src, expected) in tests {
            let val = Interpreter::from_source(src).unwrap();
            let result = match val {
                Val::Int(n) => n,
                Val::Float(f) => f.into_inner() as i32,
            };
            assert_eq!(result, expected, "Failed on input: {}", src);
        }
    }
}
