use crate::ast::{self, Expression, Type};
use crate::ast::{Operator, Statement};
use inkwell::context::Context;
use inkwell::types::BasicType;

pub fn generate_ir(program: &ast::Program) -> String {
    let context: Context = Context::create();
    let module: inkwell::module::Module<'_> = context.create_module("main");
    let builder: inkwell::builder::Builder<'_> = context.create_builder();

    let i64_type: inkwell::types::IntType<'_> = context.i64_type();

    // let mut variables:  = std::collections::HashMap::new();

    for func in &program.functions {
        let param_types: Vec<inkwell::types::BasicMetadataTypeEnum> = func
            .parameters
            .iter()
            .map(|(_, t)| get_llvm_type(t, &context).into())
            .collect();

        let ret_type = get_llvm_type(&func.return_type, &context);
        let fn_type = ret_type.fn_type(&param_types, false);
        module.add_function(&func.name, fn_type, None);
    }

    for func in &program.functions {
        let function = module.get_function(&func.name).unwrap();
        let basic_block = context.append_basic_block(function, "entry");
        builder.position_at_end(basic_block);

        let mut variables: std::collections::HashMap<
            String,
            (
                inkwell::values::PointerValue<'_>,
                bool,
                inkwell::types::BasicTypeEnum<'_>,
            ),
        > = std::collections::HashMap::new();

        let mut variables = std::collections::HashMap::new();

        for (i, (param_name, param_type)) in func.parameters.iter().enumerate() {
            let llvm_param_type = get_llvm_type(param_type, &context);
            let ptr = builder.build_alloca(llvm_param_type, param_name).unwrap();

            let arg_val = function.get_nth_param(i as u32).unwrap();
            _ = builder.build_store(ptr, arg_val);

            variables.insert(param_name.clone(), (ptr, false, llvm_param_type));
        }
        for statement in &func.body {
            match statement {
                Statement::Return(expr) => {
                    let int_value = compile_expression(
                        expr,
                        &context,
                        &builder,
                        &variables,
                        &module,
                        i64_type.into(),
                    );
                    builder.build_return(Some(&int_value)).unwrap();
                }
                Statement::Declaration {
                    is_mut,
                    name,
                    type_name,
                    initializer,
                } => {
                    let typ = get_llvm_type(type_name, &context);
                    let ptr = builder.build_alloca(typ, &name).unwrap();
                    if let Some(init_expr) = initializer {
                        let init_value = compile_expression(
                            init_expr, &context, &builder, &variables, &module, typ,
                        );
                        let _ = builder.build_store(ptr, init_value);
                    }
                    variables.insert(name.to_string(), (ptr, *is_mut, typ));
                }

                ast::Statement::Assignment { name, value } => {
                    let Some((ptr, is_mut, type_name)) = variables.get(name) else {
                        panic!("Uninitialized Variable")
                    };
                    let new_val = compile_expression(
                        value, &context, &builder, &variables, &module, *type_name,
                    );

                    if !*is_mut {
                        panic!("Cannot mutate constant")
                    }
                    _ = builder.build_store(*ptr, new_val);
                }

                Statement::IndexAssignment { name, index, value } => {
                    let Some((ptr, is_mut, type_name)) = variables.get(name) else {
                        panic!("Uninitialized Variable")
                    };
                    if !*is_mut {
                        panic!("Trying to modify a const array!");
                    }
                    let comp_index = compile_expression(
                        index,
                        &context,
                        &builder,
                        &variables,
                        &module,
                        context.i64_type().into(),
                    );
                    let elem_type = if type_name.is_array_type() {
                        type_name.into_array_type().get_element_type()
                    } else {
                        *type_name
                    };
                    let comp_value = compile_expression(
                        value, &context, &builder, &variables, &module, elem_type,
                    );

                    let zero = context.i64_type().const_zero();

                    let element_ptr = unsafe {
                        builder
                            .build_gep(
                                *type_name,
                                *ptr,
                                &[zero, comp_index.into_int_value()],
                                &format!("{name}_idx_ptr"),
                            )
                            .unwrap()
                    };
                    _ = builder.build_store(element_ptr, comp_value);
                }
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
    module: &inkwell::module::Module<'ctx>,
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
        ast::Expression::AddressOf(expr) => {
            if let ast::Expression::Identifier(name) = &**expr {
                let Some((ptr, _, _)) = variables.get(name) else {
                    panic!("Uninitialized Variable")
                };
                (*ptr).into()
            } else {
                panic!("Cannot have a pointer to something other than an identifier");
            }
        }
        ast::Expression::Dereference(expr) => {
            let pointer: inkwell::values::PointerValue<'_> = compile_expression(
                expr,
                context,
                builder,
                variables,
                module,
                inkwell::types::BasicTypeEnum::PointerType(
                    context.ptr_type(inkwell::AddressSpace::from(0)).into(),
                ),
            )
            .into_pointer_value();
            return builder.build_load(expected_type, pointer, "deref").unwrap();
        }
        ast::Expression::Binary(left, op, right) => {
            let lhs = compile_expression(left, context, builder, variables, module, expected_type);
            let rhs = compile_expression(right, context, builder, variables, module, expected_type);

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
        ast::Expression::Index(left, index) => {
            let Expression::Identifier(ref name) = **left else {
                panic!("ERROR: Expected identifier on left side of index expression");
            };

            let Some((ptr, _, type_name)) = variables.get(name) else {
                panic!("ERROR: Uninitialized Variable {name}");
            };

            let index_val = compile_expression(
                index,
                context,
                builder,
                variables,
                module,
                context.i64_type().into(),
            )
            .into_int_value();

            let zero = context.i64_type().const_zero();

            let element_ptr = unsafe {
                builder
                    .build_gep(
                        *type_name,
                        *ptr,
                        &[zero, index_val],
                        &format!("{name}_idx_ptr"),
                    )
                    .unwrap()
            };

            builder
                .build_load(expected_type, element_ptr, "elem_val")
                .unwrap()
        }
        ast::Expression::Call(name, args) => {
            if name == "syscall" {
                return compile_syscall(context, builder, variables, module, args);
            }
            let func = module.get_function(name).expect("Function not found");
            let mut vals: Vec<inkwell::values::BasicMetadataValueEnum> = vec![];
            for (i, arg) in args.iter().enumerate() {
                let param_type = func.get_nth_param(i as u32).unwrap().get_type();
                vals.push(
                    compile_expression(arg, context, builder, variables, module, param_type).into(),
                );
            }
            let call_site = builder.build_call(func, &vals, "tmpcall").unwrap();
            return call_site.try_as_basic_value().unwrap_basic();
        }
        _ => {
            panic!("");
        }
    }
}
fn compile_syscall<'ctx>(
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
    module: &inkwell::module::Module<'ctx>,
    args: &Vec<Expression>,
) -> inkwell::values::BasicValueEnum<'ctx> {
    let asm_fn_type = context.i64_type().fn_type(
        &[
            context.i64_type().into(),
            context.i64_type().into(),
            context.i64_type().into(),
            context.i64_type().into(),
        ],
        false,
    );

    let asm = context.create_inline_asm(
        asm_fn_type,
        "syscall".to_string(),
        "={rax},{rax},{rdi},{rsi},{rdx},~{rcx},~{r11},~{memory}".to_string(),
        true,
        false,
        None,
        false,
    );

    let mut compiled_args: Vec<inkwell::values::BasicMetadataValueEnum> = vec![];
    for arg in args {
        compiled_args.push(
            compile_expression(
                arg,
                context,
                builder,
                variables,
                module,
                context.i64_type().into(),
            )
            .into(),
        );
    }

    let call_site = builder
        .build_indirect_call(asm_fn_type, asm, &compiled_args, "syscall_ret")
        .unwrap();
    return call_site.try_as_basic_value().unwrap_basic();
}

fn get_llvm_type<'ctx>(
    ast_type: &ast::Type,
    context: &'ctx Context,
) -> inkwell::types::BasicTypeEnum<'ctx> {
    return match ast_type {
        Type::I8 | Type::U8 => context.i8_type().into(),
        Type::I16 | Type::U16 => context.i16_type().into(),
        Type::I32 | Type::U32 => context.i32_type().into(),
        Type::I64 | Type::U64 => context.i64_type().into(),
        Type::F32 => context.f32_type().into(),
        Type::F64 => context.f64_type().into(),
        Type::Pointer(_) => context.ptr_type(inkwell::AddressSpace::from(0)).into(),
        Type::Array(base_type, size) => get_llvm_type(base_type, context)
            .array_type(*size as u32)
            .into(),
    };
}

#[cfg(test)]
mod tests {
    use crate::{codegen::generate_ir, lexer::Lexer, parser::Parser};

    #[test]
    fn test_generate_return() {
        let lexer: Lexer<'_> = Lexer::new("func main() : i64 { return 5; }");
        let mut parser = Parser::new(lexer);
        let my_prog = parser.parse_program();

        let ir_string: String = generate_ir(&my_prog);
        println!("{}", ir_string);
        assert!(ir_string.contains("ret i64 5"));
    }
    #[test]
    fn test_generate_return_2() {
        let lexer: Lexer<'_> = Lexer::new("func main() : i64 { return 6 * 7 - 67;}");
        let mut parser = Parser::new(lexer);
        let my_prog = parser.parse_program();

        let ir_string: String = generate_ir(&my_prog);
        println!("{}", ir_string);
        assert!(ir_string.contains("ret i64 -25"));
    }

    #[test]
    fn test_generate_variables() {
        let lexer: Lexer<'_> = Lexer::new("func main() : i64 {const x : i64 = 5; return x;}");
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
        let lexer: Lexer<'_> =
            Lexer::new("func main() : i64 { var x : i64 = 5; x = 10; return x; } ");
        let mut parser = Parser::new(lexer);
        let my_prog = parser.parse_program();

        let ir_string: String = generate_ir(&my_prog);
        println!("{}", ir_string);
        assert!(ir_string.contains("store i64 10"));
        assert!(ir_string.contains("load i64"));
    }

    #[test]
    fn test_generate_self_assignment() {
        let lexer: Lexer<'_> =
            Lexer::new("func main() : i64 { var x : i64 = 5; x = x + 10; return x; }");
        let mut parser = Parser::new(lexer);
        let my_prog = parser.parse_program();

        let ir_string: String = generate_ir(&my_prog);
        println!("{}", ir_string);
        assert!(ir_string.contains("load i64"));
    }

    #[should_panic(expected = "Cannot mutate constant")]
    #[test]
    fn test_const_reassignment_fails() {
        let lexer: Lexer<'_> =
            Lexer::new("func main() : i64 {const x : i64 = 5; x = 10; return x;}");
        let mut parser = Parser::new(lexer);
        let my_prog = parser.parse_program();

        let ir_string: String = generate_ir(&my_prog);
        println!("{}", ir_string);
        assert!(ir_string.contains("load i64"));
    }

    #[test]
    fn test_generate_i8() {
        let lexer: Lexer<'_> = Lexer::new("func main() : i64 {const x : i8 = 5;}");
        let mut parser = Parser::new(lexer);
        let my_prog = parser.parse_program();

        let ir_string: String = generate_ir(&my_prog);
        println!("{}", ir_string);
        assert!(ir_string.contains("alloca i8"));
    }

    #[test]
    fn test_generate_f32() {
        let lexer: Lexer<'_> = Lexer::new("func main() : i64 {const pi : f64 = 3.14;}");
        let mut parser = Parser::new(lexer);
        let my_prog = parser.parse_program();

        let ir_string: String = generate_ir(&my_prog);
        println!("{}", ir_string);
        assert!(ir_string.contains("alloca double"));
        assert!(ir_string.contains("store double 3.14"));
    }

    #[test]
    fn test_generate_pointers() {
        let lexer: Lexer<'_> =
            Lexer::new("func main() : i64 {var x : i64 = 5; const p : *i64 = &x; return *p;}");
        let mut parser = Parser::new(lexer);
        let my_prog = parser.parse_program();

        let ir_string: String = generate_ir(&my_prog);
        println!("{}", ir_string);
        assert!(ir_string.contains("store ptr"));
        assert!(ir_string.contains("load ptr"));
        assert!(ir_string.contains("load i64"));
    }

    #[test]
    fn test_arrays() {
        let lexer: Lexer<'_> =
            Lexer::new("func main() : i64 {var arr: [5]i64; arr[2] = 10; return arr[2];}");
        let mut parser = Parser::new(lexer);
        let my_prog = parser.parse_program();

        let ir_string: String = generate_ir(&my_prog);
        println!("{}", ir_string);
        assert!(ir_string.contains("getelementptr"));
        assert!(ir_string.contains("store"));
        assert!(ir_string.contains("load"));
    }

    #[test]
    fn test_functions() {
        let lexer: Lexer<'_> = Lexer::new("func square(x: i64) : i64 { return x * x; }");
        let mut parser = Parser::new(lexer);
        let my_prog = parser.parse_program();

        let ir_string: String = generate_ir(&my_prog);
        println!("{}", ir_string);
        assert!(ir_string.contains("define i64 @square(i64 %0)"));
        assert!(ir_string.contains("alloca i64"));
        assert!(ir_string.contains("store i64 %0"));
        assert!(ir_string.contains("mul"));
    }

    #[test]
    fn test_function_call() {
        let lexer: Lexer<'_> = Lexer::new(
            "func add(x: i64, y: i64) : i64 { return x + y; } func main() : i64 { return add(5, 10); }",
        );
        let mut parser = Parser::new(lexer);
        let my_prog = parser.parse_program();

        let ir_string: String = generate_ir(&my_prog);
        println!("{}", ir_string);
        assert!(ir_string.contains("call i64 @add(i64 5, i64 10)"));
    }
    #[test]
    fn test_syscall() {
        let lexer: Lexer<'_> = Lexer::new(
            "func main() : i64 {
         var arr: [50]u8;
         arr[0] = 72;
         arr[1] = 101;
         arr[2] = 108;
         arr[3] = 108;
         arr[4] = 111;
         arr[5] = 10;
         return syscall(1, 2, &arr, 6); 
         }",
        );
        let mut parser = Parser::new(lexer);
        let my_prog = parser.parse_program();

        let ir_string: String = generate_ir(&my_prog);
        println!("{}", ir_string);
    }
}
