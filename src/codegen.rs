use crate::ast;
use crate::ast::Operator;
use inkwell::AddressSpace;
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
                let int_value =
                    compile_expression(expr, &context, &builder, &variables, i64_type.into());
                builder.build_return(Some(&int_value)).unwrap();
            }
            ast::Statement::Declaration {
                is_mut,
                name,
                type_name,
                initializer,
            } => {
                let typ = get_llvm_type(type_name, &context);
                let init_value =
                    compile_expression(initializer, &context, &builder, &variables, typ);
                let ptr = builder.build_alloca(typ, &name).unwrap();
                let _ = builder.build_store(ptr, init_value);
                variables.insert(name.to_string(), (ptr, *is_mut, typ));
            }
            ast::Statement::Assignment { name, value } => {
                let Some((ptr, is_mut, type_name)) = variables.get(name) else {
                    panic!("Uninitialized Variable")
                };
                let new_val = compile_expression(value, &context, &builder, &variables, *type_name);

                if !*is_mut {
                    panic!("Cannot mutate constant")
                }
                _ = builder.build_store(*ptr, new_val);
            }
        }
    }

    return module.print_to_string().to_string();
}

fn compile_expression<'ctx>(
    expr: &ast::Expression,
    context: &'ctx Context,
    builder: &inkwell::builder::Builder<'ctx>,
    variables: &std::collections::HashMap<
        String,
        (
            inkwell::values::PointerValue<'ctx>,
            bool,
            inkwell::types::BasicTypeEnum<'ctx>,
        ),
    >,
    expected_type: inkwell::types::BasicTypeEnum<'ctx>,
) -> inkwell::values::BasicValueEnum<'ctx> {
    match expr {
        ast::Expression::Integer(val) => expected_type
            .into_int_type()
            .const_int(*val as u64, false)
            .into(),
        ast::Expression::Float(val) => expected_type
            .into_float_type()
            .const_float(*val as f64)
            .into(),
        ast::Expression::Binary(left, op, right) => {
            let lhs = compile_expression(left, context, builder, variables, expected_type);
            let rhs = compile_expression(right, context, builder, variables, expected_type);

            if expected_type.is_int_type() {
                let (lhs, rhs) = (lhs.into_int_value(), rhs.into_int_value());
                match op {
                    Operator::Add => builder.build_int_add(lhs, rhs, "tmpadd"),
                    Operator::Subtract => builder.build_int_sub(lhs, rhs, "tmpsub"),
                    Operator::Multiply => builder.build_int_mul(lhs, rhs, "tmpmul"),
                    Operator::Divide => builder.build_int_signed_div(lhs, rhs, "tmpdiv"),
                }
                .unwrap()
                .into()
            } else {
                let (lhs, rhs) = (lhs.into_float_value(), rhs.into_float_value());
                match op {
                    Operator::Add => builder.build_float_add(lhs, rhs, "tmpadd"),
                    Operator::Subtract => builder.build_float_sub(lhs, rhs, "tmpsub"),
                    Operator::Multiply => builder.build_float_mul(lhs, rhs, "tmpmul"),
                    Operator::Divide => builder.build_float_div(lhs, rhs, "tmpdiv"),
                }
                .unwrap()
                .into()
            }
        }
        ast::Expression::Identifier(name) => {
            let Some((ptr, _, type_name)) = variables.get(name) else {
                panic!("ERROR: Uninitialized Variable")
            };
            builder.build_load(*type_name, *ptr, name).unwrap()
        }
        _ => panic!(),
    }
}
fn get_llvm_type<'ctx>(
    ast_type: &ast::Type,
    context: &'ctx Context,
) -> inkwell::types::BasicTypeEnum<'ctx> {
    return match ast_type {
        ast::Type::I8 | ast::Type::U8 => context.i8_type().into(),
        ast::Type::I16 | ast::Type::U16 => context.i16_type().into(),
        ast::Type::I32 | ast::Type::U32 => context.i32_type().into(),
        ast::Type::I64 | ast::Type::U64 => context.i64_type().into(),
        ast::Type::F32 => context.f32_type().into(),
        ast::Type::F64 => context.f64_type().into(),
        ast::Type::Pointer(_) => context.ptr_type(inkwell::AddressSpace::from(0)).into(),
    };
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

    #[test]
    fn test_generate_i8() {
        let lexer: Lexer<'_> = Lexer::new("const x : i8 = 5;");
        let mut parser = Parser::new(lexer);
        let my_prog = parser.parse_program();

        let ir_string: String = generate_ir(&my_prog);
        println!("{}", ir_string);
        assert!(ir_string.contains("alloca i8"));
    }

    #[test]
    fn test_generate_f32() {
        let lexer: Lexer<'_> = Lexer::new("const pi : f64 = 3.14;");
        let mut parser = Parser::new(lexer);
        let my_prog = parser.parse_program();

        let ir_string: String = generate_ir(&my_prog);
        println!("{}", ir_string);
        assert!(ir_string.contains("alloca double"));
        assert!(ir_string.contains("store double 3.14"));
    }
}
