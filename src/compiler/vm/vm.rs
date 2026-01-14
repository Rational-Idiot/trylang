use ordered_float::OrderedFloat;

use crate::compiler::vm::bytecode::Interpreter as BytecodeInterpreter;
use crate::compiler::vm::opcode::*;
use crate::compiler::vm::Bytecode;
use crate::val::Val;
use crate::{Compile, Node, Result};

pub struct VM {
    bytecode: Bytecode,
    stack: Vec<Node>,
}

impl VM {
    pub fn new(bytecode: Bytecode) -> Self {
        Self {
            bytecode,
            stack: Vec::new(),
        }
    }
    pub fn run(&mut self) {
        let mut ip = 0; // instruction pointer
        while ip < self.bytecode.instructions.len() {
            let inst_addr = ip;
            ip += 1;

            match self.bytecode.instructions[inst_addr] {
                0x01 => {
                    //OpConst
                    let const_idx = convert_two_u8s_to_usize(
                        self.bytecode.instructions[ip],
                        self.bytecode.instructions[ip + 1],
                    );
                    ip += 2;
                    self.push(self.bytecode.constants[const_idx].clone());
                }
                0x02 => {
                    //OpPop
                    self.pop();
                }
                0x03 => {
                    // OpAdd
                    match (self.pop(), self.pop()) {
                        (Node::Val(Val::Int(rhs)), Node::Val(Val::Int(lhs))) => {
                            self.push(Node::Val(Val::Int(lhs + rhs)))
                        }
                        (Node::Val(Val::Float(rhs)), Node::Val(Val::Float(lhs))) => {
                            self.push(Node::Val(Val::Float(lhs + rhs)))
                        }
                        (Node::Val(Val::Int(rhs)), Node::Val(Val::Float(lhs))) => {
                            self.push(Node::Val(Val::Float(lhs + OrderedFloat(rhs as f32))))
                        }
                        (Node::Val(Val::Float(rhs)), Node::Val(Val::Int(lhs))) => {
                            self.push(Node::Val(Val::Float(OrderedFloat(lhs as f32) + rhs)))
                        }
                        _ => panic!("Unknown types to OpAdd"),
                    }
                }
                0x04 => {
                    // OpSub
                    match (self.pop(), self.pop()) {
                        (Node::Val(Val::Int(rhs)), Node::Val(Val::Int(lhs))) => {
                            self.push(Node::Val(Val::Int(lhs - rhs)))
                        }
                        (Node::Val(Val::Float(rhs)), Node::Val(Val::Float(lhs))) => {
                            self.push(Node::Val(Val::Float(lhs - rhs)))
                        }
                        (Node::Val(Val::Int(rhs)), Node::Val(Val::Float(lhs))) => {
                            self.push(Node::Val(Val::Float(lhs - OrderedFloat(rhs as f32))))
                        }
                        (Node::Val(Val::Float(rhs)), Node::Val(Val::Int(lhs))) => {
                            self.push(Node::Val(Val::Float(OrderedFloat(lhs as f32) - rhs)))
                        }
                        _ => panic!("Unknown types to OpSub"),
                    }
                }
                0x05 => {
                    // OpMul
                    match (self.pop(), self.pop()) {
                        (Node::Val(Val::Int(rhs)), Node::Val(Val::Int(lhs))) => {
                            self.push(Node::Val(Val::Int(lhs * rhs)))
                        }
                        (Node::Val(Val::Float(rhs)), Node::Val(Val::Float(lhs))) => {
                            self.push(Node::Val(Val::Float(lhs * rhs)))
                        }
                        (Node::Val(Val::Int(rhs)), Node::Val(Val::Float(lhs))) => {
                            self.push(Node::Val(Val::Float(lhs * OrderedFloat(rhs as f32))))
                        }
                        (Node::Val(Val::Float(rhs)), Node::Val(Val::Int(lhs))) => {
                            self.push(Node::Val(Val::Float(OrderedFloat(lhs as f32) * rhs)))
                        }
                        _ => panic!("Unknown types to OpMul"),
                    }
                }
                0x06 => {
                    // OpDiv
                    match (self.pop(), self.pop()) {
                        (Node::Val(Val::Int(rhs)), Node::Val(Val::Int(lhs))) => {
                            self.push(Node::Val(Val::Int(lhs / rhs)))
                        }
                        (Node::Val(Val::Float(rhs)), Node::Val(Val::Float(lhs))) => {
                            self.push(Node::Val(Val::Float(lhs / rhs)))
                        }
                        (Node::Val(Val::Int(rhs)), Node::Val(Val::Float(lhs))) => {
                            self.push(Node::Val(Val::Float(lhs / OrderedFloat(rhs as f32))))
                        }
                        (Node::Val(Val::Float(rhs)), Node::Val(Val::Int(lhs))) => {
                            self.push(Node::Val(Val::Float(OrderedFloat(lhs as f32) / rhs)))
                        }
                        _ => panic!("Unknown types to OpDiv"),
                    }
                }
                0x0A => {
                    // OpPlus
                    match self.pop() {
                        Node::Val(Val::Int(num)) => self.push(Node::Val(Val::Int(num))),
                        Node::Val(Val::Float(num)) => self.push(Node::Val(Val::Float(num))),
                        _ => panic!("Unknown arg type to OpPlus"),
                    }
                }
                0x0B => {
                    // OpMinus
                    match self.pop() {
                        Node::Val(Val::Int(num)) => self.push(Node::Val(Val::Int(-num))),
                        Node::Val(Val::Float(num)) => self.push(Node::Val(Val::Float(-num))),
                        _ => panic!("Unknown arg type to OpMinus"),
                    }
                }
                _ => panic!("Unknown instruction"),
            }
        }
    }

    pub fn push(&mut self, node: Node) {
        self.stack.push(node);
    }

    pub fn pop(&mut self) -> Node {
        // ignoring stack underflow
        let node = self.stack.pop().expect("Stack Underflow");
        node
    }
    pub fn pop_last(&self) -> &Node {
        &self.stack.last().expect("Empty Stack")
    }

    pub fn peek(&self) -> Option<Val> {
        self.stack.last().map(|node| match node {
            Node::Val(v) => v.clone(),
            _ => panic!("Top of the stack is not a Val"),
        })
    }
}

impl Compile for VM {
    type Output = Result<i32>;

    fn from_ast(ast: Vec<Node>) -> Self::Output {
        let bytecode = BytecodeInterpreter::from_ast(ast);
        let mut vm = VM::new(bytecode);
        vm.run();
        match vm.pop_last() {
            Node::Val(Val::Int(n)) => Ok(*n),
            _ => Err(anyhow::anyhow!("Expected integer result")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::vm::bytecode::Interpreter;
    use crate::Compile;

    fn assert_peek(source: &str, expected: Node) {
        let byte_code = Interpreter::from_source(source);
        println!("byte code: {:?}", byte_code);
        let mut vm = VM::new(byte_code);
        vm.run();

        let expected_val = match expected {
            Node::Val(v) => v,
            _ => panic!("Expected a Node::Val"),
        };

        assert_eq!(
            Some(expected_val),
            vm.peek(),
            "Top of stack did not match expected value"
        );
    }

    #[test]
    fn unary_ints() {
        assert_peek("+1", Node::Val(Val::Int(1)));
        assert_peek("-2", Node::Val(Val::Int(-2)));
        assert_peek("--3", Node::Val(Val::Int(3)));
    }

    #[test]
    fn unary_floats() {
        assert_peek("+1.5", Node::Val(Val::Float(1.5.into())));
        assert_peek("-2.25", Node::Val(Val::Float((-2.25).into())));
    }

    #[test]
    fn binary_int_ops() {
        assert_peek("1 + 2;", Node::Val(Val::Int(3)));
        assert_peek("5 - 3;", Node::Val(Val::Int(2)));
        assert_peek("4 * 3;", Node::Val(Val::Int(12)));
        assert_peek("8 / 2;", Node::Val(Val::Int(4)));
    }

    #[test]
    fn binary_float_ops() {
        assert_peek("1.5 + 2.25;", Node::Val(Val::Float(3.75.into())));
        assert_peek("5.0 - 1.5;", Node::Val(Val::Float(3.5.into())));
        assert_peek("2.0 * 3.5;", Node::Val(Val::Float(7.0.into())));
        assert_peek("7.0 / 2.0;", Node::Val(Val::Float(3.5.into())));
    }

    #[test]
    fn mixed_numeric_ops() {
        assert_peek("1 + 2.5;", Node::Val(Val::Float(3.5.into())));
        assert_peek("5.0 - 2;", Node::Val(Val::Float(3.0.into())));
        assert_peek("2 * 1.5;", Node::Val(Val::Float(3.0.into())));
        assert_peek("5 / 2.0;", Node::Val(Val::Float(2.5.into())));
    }

    #[test]
    fn operator_precedence() {
        assert_peek("1 + 2 * 3;", Node::Val(Val::Int(7)));
        assert_peek("(1 + 2) * 3;", Node::Val(Val::Int(9)));
        assert_peek("10 - 4 / 2;", Node::Val(Val::Int(8)));
    }

    #[test]
    fn chained_expressions() {
        assert_peek("1 + 2 + 3;", Node::Val(Val::Int(6)));
        assert_peek("10 - 3 - 2;", Node::Val(Val::Int(5)));
        assert_peek("2 * 3 * 4;", Node::Val(Val::Int(24)));
    }

    #[test]
    fn test_debug_binary() {
        let input = "1 + 2;";
        let ast = crate::parser::parse(input).unwrap();
        println!("AST: {:#?}", ast);

        let bytecode = Interpreter::from_source(input);
        println!("Bytecode: {:#?}", bytecode);
    }
}
