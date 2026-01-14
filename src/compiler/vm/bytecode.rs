use crate::compiler::vm::{make_op, OpCode};
use crate::val::Val;
use crate::{Compile, Node, Operator};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bytecode {
    pub instructions: Vec<u8>,
    pub constants: Vec<Node>,
}

impl Bytecode {
    fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct Interpreter {
    bytecode: Bytecode,
}

impl Compile for Interpreter {
    type Output = Bytecode;

    fn from_ast(ast: Vec<Node>) -> Self::Output {
        let mut interpreter = Interpreter {
            bytecode: Bytecode::new(),
        };
        let len = ast.len();
        for (idx, node) in ast.into_iter().enumerate() {
            println!("compiling node {:?}", node);
            interpreter.interpret_node(node);
            // pop one element from the stack after each expression statement
            // to clean up, except for the last one so it can be inspected
            if idx < len - 1 {
                interpreter.add_instruction(OpCode::OpPop);
            }
        }
        interpreter.bytecode
    }
}

impl Interpreter {
    fn add_constant(&mut self, node: Node) -> u16 {
        self.bytecode.constants.push(node);
        (self.bytecode.constants.len() - 1) as u16 // cast to u16 because that is the size of our constant pool index
    }

    fn add_instruction(&mut self, op_code: OpCode) -> u16 {
        let position_of_new_instruction = self.bytecode.instructions.len() as u16;
        self.bytecode.instructions.extend(make_op(op_code));
        println!(
            "added instructions {:?} from opcode {:?}",
            self.bytecode.instructions,
            op_code.clone()
        );
        position_of_new_instruction
    }

    fn interpret_node(&mut self, node: Node) {
        match node {
            Node::Val(Val::Int(num)) => {
                let const_index = self.add_constant(Node::Val(Val::Int(num)));
                self.add_instruction(OpCode::OpConstant(const_index));
            }
            Node::Val(Val::Float(f)) => {
                let const_idx = self.add_constant(Node::Val(Val::Float(f)));
                self.add_instruction(OpCode::OpConstant(const_idx));
            }
            Node::UnaryExpr { op, child } => {
                self.interpret_node(*child);
                match op {
                    Operator::Plus => self.add_instruction(OpCode::OpPlus),
                    Operator::Minus => self.add_instruction(OpCode::OpMinus),
                    _ => unreachable!("Invalid Unary Operator {:#?}", op),
                };
            }
            Node::BinaryExpr { op, lhs, rhs } => {
                self.interpret_node(*lhs);
                self.interpret_node(*rhs);
                match op {
                    Operator::Plus => self.add_instruction(OpCode::OpAdd),
                    Operator::Minus => self.add_instruction(OpCode::OpSub),
                    Operator::Multiply => self.add_instruction(OpCode::OpMul),
                    Operator::Divide => self.add_instruction(OpCode::OpDiv),
                };
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unary_int_ops() {
        unary_template("+", OpCode::OpPlus, 1);
        unary_template("-", OpCode::OpMinus, -1);
    }

    fn unary_template(op: &str, opcode: OpCode, _expected: i32) {
        let input = format!("{}1;", op);
        let bytecode = Interpreter::from_source(&input);

        let expected_instructions = vec![OpCode::OpConstant(0), opcode]
            .into_iter()
            .flat_map(make_op)
            .collect();

        assert_eq!(
            Bytecode {
                instructions: expected_instructions,
                constants: vec![Node::Val(Val::Int(1))]
            },
            bytecode
        );
    }

    #[test]
    fn infix_float_ops() {
        let input = "6.7 + 2.25;";
        let bytecode = Interpreter::from_source(input);

        let expected_instructions =
            vec![OpCode::OpConstant(0), OpCode::OpConstant(1), OpCode::OpAdd]
                .into_iter()
                .flat_map(make_op)
                .collect();

        assert_eq!(
            Bytecode {
                instructions: expected_instructions,
                constants: vec![
                    Node::Val(Val::Float(6.7.into())),
                    Node::Val(Val::Float(2.25.into())),
                ]
            },
            bytecode
        );
    }
}
