use std::ptr::null;

use crate::ast::{Operator, Statement};
use crate::lexer::Lexer;
use crate::{ast, parser::Parser};
use inkwell::context::Context;

pub fn generate_ir(program: &ast::Program) -> String {
    let context: Context = Context::create();
    let module: inkwell::module::Module<'_> = context.create_module("main");
    let builder: inkwell::builder::Builder<'_> = context.create_builder();

    let i64_type: inkwell::types::IntType<'_> = context.i64_type();
    let fn_type: inkwell::types::FunctionType<'_> = i64_type.fn_type(&[], false);
    let function: inkwell::values::FunctionValue<'_> = module.add_function("main", fn_type, None);

    let basic_block: inkwell::basic_block::BasicBlock<'_> =
        context.append_basic_block(function, "entry");
    builder.position_at_end(basic_block);

    let mut variables = std::collections::HashMap::new();

    for statement in &program.statements {
        match statement {
            ast::Statement::Return(expr) => {
                let int_value = compile_expression(expr, &context, &builder, &variables);
                builder.build_return(Some(&int_value)).unwrap();
            }
            ast::Statement::Declaration {
                is_mut,
                name,
                type_name: _,
                initializer,
            } => {
                let init_value = compile_expression(initializer, &context, &builder, &variables);
                let ptr = builder.build_alloca(i64_type, &name).unwrap();
                let _ = builder.build_store(ptr, init_value);
                variables.insert(name.to_string(), (ptr, *is_mut));
            }
            ast::Statement::Assignment { name, value } => {
                let new_val = compile_expression(value, &context, &builder, &variables);
                let Some((ptr, is_mut)) = variables.get(name) else {
                    panic!("Uninitialized Variable")
                };
                if !*is_mut {
                    panic!("Cannot mutate constant")
                }
                _ = builder.build_store(*ptr, new_val);
            }

            _ => break,
        }
    }

    return module.print_to_string().to_string();
}

fn compile_expression<'ctx>(
    expr: &ast::Expression,
    context: &'ctx Context,
    builder: &inkwell::builder::Builder<'ctx>,
    variables: &std::collections::HashMap<String, (inkwell::values::PointerValue<'ctx>, bool)>,
) -> inkwell::values::IntValue<'ctx> {
    match expr {
        ast::Expression::Number(val) => context.i64_type().const_int(*val as u64, false),
        ast::Expression::Binary(left, op, right) => {
            let lhs = compile_expression(left, context, builder, variables);
            let rhs = compile_expression(right, context, builder, variables);
            match op {
                Operator::Add => builder.build_int_add(lhs, rhs, "tmpadd").unwrap(),
                Operator::Subtract => builder.build_int_sub(lhs, rhs, "tmpadd").unwrap(),
                Operator::Multiply => builder.build_int_mul(lhs, rhs, "tmpadd").unwrap(),
                Operator::Divide => builder.build_int_signed_div(lhs, rhs, "tmpadd").unwrap(),
            }
        }
        ast::Expression::Identifier(name) => {
            let Some(ptr) = variables.get(name) else {
                panic!("ERROR: Uninitialized Variable")
            };
            builder
                .build_load(context.i64_type(), ptr.0, name)
                .unwrap()
                .into_int_value()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{codegen::generate_ir, lexer::Lexer, parser::Parser};

    #[test]
    fn test_generate_return() {
        let lexer: Lexer<'_> = Lexer::new("return 5;");
        let mut parser = Parser::new(lexer);
        let my_prog = parser.parse_program();

        let ir_string: String = generate_ir(&my_prog);
        println!("{}", ir_string);
        assert!(ir_string.contains("ret i64 5"));
    }
    #[test]
    fn test_generate_return_2() {
        let lexer: Lexer<'_> = Lexer::new("return 6 * 7 - 67;");
        let mut parser = Parser::new(lexer);
        let my_prog = parser.parse_program();

        let ir_string: String = generate_ir(&my_prog);
        println!("{}", ir_string);
        assert!(ir_string.contains("ret i64 -25"));
    }

    #[test]
    fn test_generate_variables() {
        let lexer: Lexer<'_> = Lexer::new("const x : i64 = 5; return x;");
        let mut parser = Parser::new(lexer);
        let my_prog = parser.parse_program();

        let ir_string: String = generate_ir(&my_prog);
        println!("{}", ir_string);
        assert!(ir_string.contains("alloca i64"));
        assert!(ir_string.contains("store i64 5"));
        assert!(ir_string.contains("load i64"));
    }

    #[test]
    fn test_generate_assignment() {
        let lexer: Lexer<'_> = Lexer::new("var x : i64 = 5; x = 10; return x;");
        let mut parser = Parser::new(lexer);
        let my_prog = parser.parse_program();

        let ir_string: String = generate_ir(&my_prog);
        println!("{}", ir_string);
        assert!(ir_string.contains("store i64 10"));
        assert!(ir_string.contains("load i64"));
    }

    #[test]
    fn test_generate_self_assignment() {
        let lexer: Lexer<'_> = Lexer::new("var x : i64 = 5; x = x + 10; return x;");
        let mut parser = Parser::new(lexer);
        let my_prog = parser.parse_program();

        let ir_string: String = generate_ir(&my_prog);
        println!("{}", ir_string);
        assert!(ir_string.contains("load i64"));
    }

    #[should_panic(expected = "Cannot mutate constant")]
    #[test]
    fn test_const_reassignment_fails() {
        let lexer: Lexer<'_> = Lexer::new("const x : i64 = 5; x = 10; return x;");
        let mut parser = Parser::new(lexer);
        let my_prog = parser.parse_program();

        let ir_string: String = generate_ir(&my_prog);
        println!("{}", ir_string);
        assert!(ir_string.contains("load i64"));
    }
}
