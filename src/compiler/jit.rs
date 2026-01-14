use crate::{Compile, Node, Operator, Result, val::Val};
use inkwell::{
    OptimizationLevel,
    builder::Builder,
    context::Context,
    execution_engine::JitFunction,
    types::{FloatType, IntType},
    values::{AnyValue, FloatValue, IntValue},
};

type JitFuncFloat = unsafe extern "C" fn() -> f64;

pub struct Jit;

impl Compile for Jit {
    type Output = Result<i32>;

    fn from_ast(ast: Vec<Node>) -> Self::Output {
        let context = Context::create();
        let module = context.create_module("calculator");
        let builder = context.create_builder();
        let execution_engine = module
            .create_jit_execution_engine(OptimizationLevel::None)
            .unwrap();

        // Check if we need float or int
        let needs_float = ast.iter().any(|node| contains_float(node));

        if needs_float {
            let f64_type = context.f64_type();
            let fn_type = f64_type.fn_type(&[], false);
            let function = module.add_function("jit", fn_type, None);
            let basic_block = context.append_basic_block(function, "entry");

            builder.position_at_end(basic_block);

            for node in ast {
                let recursive_builder = RecursiveBuilder::new(&context, &builder);
                let return_value = recursive_builder.build_float(&node);
                let _ = builder.build_return(Some(&return_value));
            }

            println!(
                "Generated LLVM IR: {}",
                function.print_to_string().to_string()
            );

            unsafe {
                let jit_function: JitFunction<JitFuncFloat> =
                    execution_engine.get_function("jit").unwrap();
                Ok(jit_function.call() as i32)
            }
        } else {
            let i32_type = context.i32_type();
            let fn_type = i32_type.fn_type(&[], false);
            let function = module.add_function("jit", fn_type, None);
            let basic_block = context.append_basic_block(function, "entry");

            builder.position_at_end(basic_block);

            for node in ast {
                let recursive_builder = RecursiveBuilder::new(&context, &builder);
                let return_value = recursive_builder.build_int(&node);
                let _ = builder.build_return(Some(&return_value));
            }

            println!(
                "Generated LLVM IR: {}",
                function.print_to_string().to_string()
            );

            unsafe {
                let jit_function: JitFunction<unsafe extern "C" fn() -> i32> =
                    execution_engine.get_function("jit").unwrap();
                Ok(jit_function.call())
            }
        }
    }
}

fn contains_float(node: &Node) -> bool {
    match node {
        Node::Val(Val::Float(_)) => true,
        Node::Val(Val::Int(_)) => false,
        Node::UnaryExpr { child, .. } => contains_float(child),
        Node::BinaryExpr { lhs, rhs, .. } => contains_float(lhs) || contains_float(rhs),
    }
}

struct RecursiveBuilder<'a> {
    context: &'a Context,
    i32_type: IntType<'a>,
    f64_type: FloatType<'a>,
    builder: &'a Builder<'a>,
}

impl<'a> RecursiveBuilder<'a> {
    pub fn new(context: &'a Context, builder: &'a Builder<'a>) -> Self {
        Self {
            context,
            i32_type: context.i32_type(),
            f64_type: context.f64_type(),
            builder,
        }
    }

    pub fn build_int(&self, ast: &Node) -> IntValue<'a> {
        match ast {
            Node::Val(Val::Int(n)) => self.i32_type.const_int(*n as u64, true),
            Node::Val(Val::Float(f)) => self.i32_type.const_int(f.0 as i32 as u64, true),
            Node::UnaryExpr { op, child } => {
                let child = self.build_int(child);
                match op {
                    Operator::Minus => child.const_neg(),
                    Operator::Plus => child,
                    _ => panic!("Unsupported unary operator in JIT: {:?}", op),
                }
            }
            Node::BinaryExpr { op, lhs, rhs } => {
                let left = self.build_int(lhs);
                let right = self.build_int(rhs);
                match op {
                    Operator::Plus => self
                        .builder
                        .build_int_add(left, right, "plus_temp")
                        .unwrap(),
                    Operator::Minus => self
                        .builder
                        .build_int_sub(left, right, "minus_temp")
                        .unwrap(),
                    Operator::Multiply => {
                        self.builder.build_int_mul(left, right, "mul_temp").unwrap()
                    }
                    Operator::Divide => self
                        .builder
                        .build_int_signed_div(left, right, "div_temp")
                        .unwrap(),
                }
            }
        }
    }

    pub fn build_float(&self, ast: &Node) -> FloatValue<'a> {
        match ast {
            Node::Val(Val::Int(n)) => self.f64_type.const_float(*n as f64),
            Node::Val(Val::Float(f)) => self.f64_type.const_float(f.0 as f64),
            Node::UnaryExpr { op, child } => {
                let child = self.build_float(child);
                match op {
                    Operator::Minus => {
                        // Multiply by -1.0 to negate
                        let neg_one = self.f64_type.const_float(-1.0);
                        self.builder
                            .build_float_mul(child, neg_one, "neg_temp")
                            .unwrap()
                    }
                    Operator::Plus => child,
                    _ => panic!("Unsupported unary operator in JIT: {:?}", op),
                }
            }
            Node::BinaryExpr { op, lhs, rhs } => {
                let left = self.build_float(lhs);
                let right = self.build_float(rhs);
                match op {
                    Operator::Plus => self
                        .builder
                        .build_float_add(left, right, "plus_temp")
                        .unwrap(),
                    Operator::Minus => self
                        .builder
                        .build_float_sub(left, right, "minus_temp")
                        .unwrap(),
                    Operator::Multiply => self
                        .builder
                        .build_float_mul(left, right, "mul_temp")
                        .unwrap(),
                    Operator::Divide => self
                        .builder
                        .build_float_div(left, right, "div_temp")
                        .unwrap(),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basics() {
        assert_eq!(Jit::from_source("1 + 2").unwrap(), 3);
        assert_eq!(Jit::from_source("2 + (2 - 1)").unwrap(), 3);
        assert_eq!(Jit::from_source("(2 + 3) - 1").unwrap(), 4);
        assert_eq!(Jit::from_source("1 + ((2 + 3) - (2 + 3))").unwrap(), 1);
        assert_eq!(Jit::from_source("(1 + 2)").unwrap(), 3);
    }

    #[test]
    fn precedence() {
        assert_eq!(Jit::from_source("2 + 3 * 4").unwrap(), 14);
        assert_eq!(Jit::from_source("10 - 4 / 2").unwrap(), 8);
        assert_eq!(Jit::from_source("2 + 3 - 1").unwrap(), 4);
    }

    #[test]
    fn unary() {
        assert_eq!(Jit::from_source("-5").unwrap(), -5);
        assert_eq!(Jit::from_source("+3").unwrap(), 3);
        assert_eq!(Jit::from_source("-1 + 2").unwrap(), 1);
    }

    #[test]
    fn float_ops() {
        // Note: Results are truncated to i32
        assert_eq!(Jit::from_source("3.14 * 2.0").unwrap(), 6);
        assert_eq!(Jit::from_source("7.0 / 2.0").unwrap(), 3);
        assert_eq!(Jit::from_source("3.14 * 2.0 + 1.5 / 3.0").unwrap(), 6);
    }
}

