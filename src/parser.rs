#![allow(clippy::upper_case_acronyms, clippy::result_large_err)]

use ordered_float::OrderedFloat;
use pest::{self, Parser};

use crate::ast::{Node, Operator};
use crate::val::Val;

#[derive(pest_derive::Parser)]
#[grammar = "grammar.pest"]
struct CalcParser;

pub fn parse(source: &str) -> std::result::Result<Vec<Node>, pest::error::Error<Rule>> {
    let mut ast = vec![];
    let pairs = CalcParser::parse(Rule::Program, source)?;
    for pair in pairs {
        if let Rule::Expr = pair.as_rule() {
            ast.push(build_ast_from_expr(pair));
        }
    }
    Ok(ast)
}

fn build_ast_from_expr(pair: pest::iterators::Pair<Rule>) -> Node {
    let mut pairs = pair.into_inner();
    let mut lhs = build_ast_from_term(pairs.next().unwrap());

    while let Some(op) = pairs.next() {
        let rhs = build_ast_from_term(pairs.next().unwrap());
        lhs = parse_binary_expr(op, lhs, rhs);
    }
    lhs
}

fn build_ast_from_term(pair: pest::iterators::Pair<Rule>) -> Node {
    let mut pairs = pair.into_inner();
    let mut lhs = build_ast_from_factor(pairs.next().unwrap());

    while let Some(op) = pairs.next() {
        let rhs = build_ast_from_factor(pairs.next().unwrap());
        lhs = parse_binary_expr(op, lhs, rhs);
    }
    lhs
}

fn build_ast_from_factor(pair: pest::iterators::Pair<Rule>) -> Node {
    match pair.as_rule() {
        Rule::Factor => {
            let inner = pair.into_inner().next().unwrap();
            build_ast_from_factor(inner)
        }
        Rule::UnaryExpr => {
            let mut inner = pair.into_inner();
            let op_pair = inner.next().unwrap();
            let child = inner.next().unwrap();
            let child_node = build_ast_from_factor(child);

            let op_node = Node::UnaryExpr {
                op: match op_pair.as_str() {
                    "+" => Operator::Plus,
                    "-" => Operator::Minus,
                    _ => unreachable!(),
                },
                child: Box::new(child_node),
            };
            op_node
        }
        Rule::Primary => {
            let inner = pair.into_inner().next().unwrap();
            build_ast_from_primary(inner)
        }
        rule => build_ast_from_primary(pair),
    }
}

fn build_ast_from_primary(pair: pest::iterators::Pair<Rule>) -> Node {
    match pair.as_rule() {
        Rule::Int => {
            let int: i32 = pair.as_str().parse().unwrap();
            Node::Val(Val::Int(int))
        }
        Rule::Float => {
            let num: f32 = pair.as_str().parse().expect("Invalid Float Parsing");
            Node::Val(Val::Float(OrderedFloat(num)))
        }
        Rule::Expr => build_ast_from_expr(pair),
        unknown => panic!("Unknown primary: {:?}", unknown),
    }
}

fn parse_unary_expr(pair: pest::iterators::Pair<Rule>, child: Node) -> Node {
    Node::UnaryExpr {
        op: match pair.as_str() {
            "+" => Operator::Plus,
            "-" => Operator::Minus,
            _ => unreachable!(),
        },
        child: Box::new(child),
    }
}

fn parse_binary_expr(pair: pest::iterators::Pair<Rule>, lhs: Node, rhs: Node) -> Node {
    Node::BinaryExpr {
        op: match pair.as_str() {
            "+" => Operator::Plus,
            "-" => Operator::Minus,
            "*" => Operator::Multiply,
            "/" => Operator::Divide,
            _ => unreachable!("Unrecognised Operator"),
        },
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_integer() {
        let result = parse("42").unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            Node::Val(Val::Int(n)) => assert_eq!(*n, 42),
            _ => panic!("Expected Int"),
        }
    }

    #[test]
    fn test_simple_float() {
        let result = parse("3.14").unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            Node::Val(Val::Float(n)) => assert_eq!(n.0, 3.14),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_addition() {
        let result = parse("1 + 2").unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            Node::BinaryExpr { op, lhs, rhs } => {
                assert_eq!(*op, Operator::Plus);
                assert!(matches!(**lhs, Node::Val(Val::Int(1))));
                assert!(matches!(**rhs, Node::Val(Val::Int(2))));
            }
            _ => panic!("Expected BinaryExpr"),
        }
    }

    #[test]
    fn test_subtraction() {
        let result = parse("10 - 3").unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            Node::BinaryExpr { op, .. } => assert_eq!(*op, Operator::Minus),
            _ => panic!("Expected BinaryExpr"),
        }
    }

    #[test]
    fn test_multiplication() {
        let result = parse("4 * 5").unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            Node::BinaryExpr { op, .. } => assert_eq!(*op, Operator::Multiply),
            _ => panic!("Expected BinaryExpr"),
        }
    }

    #[test]
    fn test_division() {
        let result = parse("20 / 4").unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            Node::BinaryExpr { op, .. } => assert_eq!(*op, Operator::Divide),
            _ => panic!("Expected BinaryExpr"),
        }
    }

    #[test]
    fn test_unary_minus() {
        let result = parse("-5").unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            Node::UnaryExpr { op, child } => {
                assert_eq!(*op, Operator::Minus);
                assert!(matches!(**child, Node::Val(Val::Int(5))));
            }
            _ => panic!("Expected UnaryExpr"),
        }
    }

    #[test]
    fn test_unary_plus() {
        let result = parse("+3").unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            Node::UnaryExpr { op, .. } => assert_eq!(*op, Operator::Plus),
            _ => panic!("Expected UnaryExpr"),
        }
    }

    #[test]
    fn test_precedence_multiply_before_add() {
        // 2 + 3 * 4 should parse as 2 + (3 * 4)
        let result = parse("2 + 3 * 4").unwrap();
        match &result[0] {
            Node::BinaryExpr { op, lhs, rhs } => {
                assert_eq!(*op, Operator::Plus);
                assert!(matches!(**lhs, Node::Val(Val::Int(2))));
                match &**rhs {
                    Node::BinaryExpr { op, lhs, rhs } => {
                        assert_eq!(*op, Operator::Multiply);
                        assert!(matches!(**lhs, Node::Val(Val::Int(3))));
                        assert!(matches!(**rhs, Node::Val(Val::Int(4))));
                    }
                    _ => panic!("Expected nested multiplication"),
                }
            }
            _ => panic!("Expected BinaryExpr"),
        }
    }

    #[test]
    fn test_precedence_divide_before_subtract() {
        // 10 - 6 / 2 should parse as 10 - (6 / 2)
        let result = parse("10 - 6 / 2").unwrap();
        match &result[0] {
            Node::BinaryExpr { op, lhs, rhs } => {
                assert_eq!(*op, Operator::Minus);
                assert!(matches!(**lhs, Node::Val(Val::Int(10))));
                match &**rhs {
                    Node::BinaryExpr { op, .. } => {
                        assert_eq!(*op, Operator::Divide);
                    }
                    _ => panic!("Expected nested division"),
                }
            }
            _ => panic!("Expected BinaryExpr"),
        }
    }

    #[test]
    fn test_left_associativity_addition() {
        // 1 + 2 + 3 should parse as (1 + 2) + 3
        let result = parse("1 + 2 + 3").unwrap();
        match &result[0] {
            Node::BinaryExpr { op, lhs, rhs } => {
                assert_eq!(*op, Operator::Plus);
                assert!(matches!(**rhs, Node::Val(Val::Int(3))));
                match &**lhs {
                    Node::BinaryExpr { op, lhs, rhs } => {
                        assert_eq!(*op, Operator::Plus);
                        assert!(matches!(**lhs, Node::Val(Val::Int(1))));
                        assert!(matches!(**rhs, Node::Val(Val::Int(2))));
                    }
                    _ => panic!("Expected nested addition"),
                }
            }
            _ => panic!("Expected BinaryExpr"),
        }
    }

    #[test]
    fn test_left_associativity_multiplication() {
        // 2 * 3 * 4 should parse as (2 * 3) * 4
        let result = parse("2 * 3 * 4").unwrap();
        match &result[0] {
            Node::BinaryExpr { op, lhs, rhs } => {
                assert_eq!(*op, Operator::Multiply);
                assert!(matches!(**rhs, Node::Val(Val::Int(4))));
                match &**lhs {
                    Node::BinaryExpr { op, .. } => {
                        assert_eq!(*op, Operator::Multiply);
                    }
                    _ => panic!("Expected nested multiplication"),
                }
            }
            _ => panic!("Expected BinaryExpr"),
        }
    }

    #[test]
    fn test_parentheses_override_precedence() {
        // (2 + 3) * 4 should parse as (2 + 3) * 4
        let result = parse("(2 + 3) * 4").unwrap();
        match &result[0] {
            Node::BinaryExpr { op, lhs, rhs } => {
                assert_eq!(*op, Operator::Multiply);
                assert!(matches!(**rhs, Node::Val(Val::Int(4))));
                match &**lhs {
                    Node::BinaryExpr { op, lhs, rhs } => {
                        assert_eq!(*op, Operator::Plus);
                        assert!(matches!(**lhs, Node::Val(Val::Int(2))));
                        assert!(matches!(**rhs, Node::Val(Val::Int(3))));
                    }
                    _ => panic!("Expected nested addition"),
                }
            }
            _ => panic!("Expected BinaryExpr"),
        }
    }

    #[test]
    fn test_nested_parentheses() {
        let result = parse("((1 + 2))").unwrap();
        match &result[0] {
            Node::BinaryExpr { op, .. } => assert_eq!(*op, Operator::Plus),
            _ => panic!("Expected BinaryExpr"),
        }
    }

    #[test]
    fn test_unary_with_binary() {
        // -1 + 2
        let result = parse("-1 + 2").unwrap();
        match &result[0] {
            Node::BinaryExpr { op, lhs, rhs } => {
                assert_eq!(*op, Operator::Plus);
                assert!(matches!(**lhs, Node::UnaryExpr { .. }));
                assert!(matches!(**rhs, Node::Val(Val::Int(2))));
            }
            _ => panic!("Expected BinaryExpr"),
        }
    }

    #[test]
    fn test_unary_in_middle() {
        // 5 + -3
        let result = parse("5 + -3").unwrap();
        match &result[0] {
            Node::BinaryExpr { op, lhs, rhs } => {
                assert_eq!(*op, Operator::Plus);
                assert!(matches!(**lhs, Node::Val(Val::Int(5))));
                assert!(matches!(**rhs, Node::UnaryExpr { .. }));
            }
            _ => panic!("Expected BinaryExpr"),
        }
    }

    #[test]
    fn test_float_operations() {
        let result = parse("3.14 * 2.0 + 1.5 / 3.0").unwrap();
        assert_eq!(result.len(), 1);
        // Just verify it parses without panicking
        assert!(matches!(result[0], Node::BinaryExpr { .. }));
    }

    #[test]
    fn test_complex_expression() {
        let result = parse("10 + 2 * (6 - 4) / 2").unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Node::BinaryExpr { .. }));
    }

    #[test]
    fn test_whitespace_handling() {
        let result1 = parse("1+2").unwrap();
        let result2 = parse("1 + 2").unwrap();
        let result3 = parse("1  +  2").unwrap();

        // All should parse to the same structure
        assert!(matches!(result1[0], Node::BinaryExpr { .. }));
        assert!(matches!(result2[0], Node::BinaryExpr { .. }));
        assert!(matches!(result3[0], Node::BinaryExpr { .. }));
    }

    #[test]
    fn test_invalid_syntax() {
        assert!(parse("1 +").is_err());
        assert!(parse("1 2").is_err());
        assert!(parse("(1 + 2").is_err());
        assert!(parse("1 + 2)").is_err());
    }

    #[test]
    fn test_empty_string() {
        assert!(parse("").is_err());
    }

    #[test]
    fn test_multiple_operators() {
        let result = parse("1 + 2 - 3 * 4 / 5").unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Node::BinaryExpr { .. }));
    }
}
