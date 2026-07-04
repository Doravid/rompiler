use crate::lexer::{Lexer, Token};
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

    for statement in &program.statements {
        match statement {
            ast::Statement::Return(ast::Expression::Number(val)) => {
                let int_value = i64_type.const_int(*val as u64, false);
                builder.build_return(Some(&int_value)).unwrap();
            }
            _ => {}
        }
    }

    return module.print_to_string().to_string();
}

#[test]
fn test_generate_return() {
    let lexer: Lexer<'_> = Lexer::new("return 5;");
    let mut parser = Parser::new(lexer);
    let my_prog = parser.parse_program();

    let ir_string: String = generate_ir(&my_prog);
    println!("{}", ir_string);
    assert!(ir_string.contains("ret i64 5"));
}
