use std::env;
use std::fs;

pub mod ast;
pub mod codegen;
pub mod lexer;
pub mod parser;

fn main() {
    let mut args = env::args().skip(1);
    let mut input_file = String::new();
    let mut output_file = String::from("output.ll");

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-i" => input_file = args.next().unwrap_or_default(),
            "-o" => output_file = args.next().unwrap_or_default(),
            _ => {
                if input_file.is_empty() {
                    input_file = arg;
                }
            }
        }
    }

    if input_file.is_empty() {
        print!("Usage: rompiler -i <input_file> -o <output_file>\n");
        std::process::exit(1);
    }

    let source = fs::read_to_string(input_file).unwrap();
    let lexer = lexer::Lexer::new(&source);
    let mut parser = parser::Parser::new(lexer);
    let program = parser.parse_program();
    let ir = codegen::generate_ir(&program);

    fs::write(output_file, ir).unwrap();
}
