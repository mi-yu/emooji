use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::env;

mod tokenizer;
use tokenizer::Tokenizer;

fn main() {
    // parse arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Incorrect number of arguments.");
    }

    // read from .moo file
    let path = format!("{}{}", &args[1], ".moo");
    let path = Path::new(&path);
    let mut file = match File::open(&path) {
        Err(why) => panic!("Couldn't open .moo file: {}",
                            why.description()),
        Ok(file) => file,
    };
    let mut program_contents = String::with_capacity(500);
    match file.read_to_string(&mut program_contents) {
        Err(why) => panic!("Couldn't read .moo file: {}",
                            why.description()),
        Ok(_) => {},
    };

    // create Tokenizer
    let mut tokenizer = Tokenizer::new(program_contents);

    // create assembly file
    let path = format!("{}{}", &args[1], ".s");
    let path = Path::new(&path);
    let mut file = match File::create(&path) {
        Err(why) => panic!("Couldn't create .s file: {}",
                            why.description()),
        Ok(file) => file,
    };

    // start compilation
    gen_data(&mut file, &mut tokenizer);
    gen_code(&mut file, &mut tokenizer);
}

fn gen_data(file: &mut File, tokenizer: &mut Tokenizer){
    let mut content = ".data\n\
                    \t\targc_: .quad 0\n\
                    \t\tFormat: .byte '%', 'l', 'u', 10, 0\n\
                    \t\tFuncTable: .quad 0\n\
                    \t\tFuncCall: .quad 0\n";
    let mut vars: Vec<(String, VarType)> = Vec::new();
    let mut tkn = tokenizer.get_token();
    while !(tkn.is("END")) {
        if tkn.is("NEW") {
            tkn = tokenizer.get_token();
            let vt:VarType;
            if tkn.is("BOOL") {
                vt = VarType::BOOL;
            }
            else if tkn.is("INT") {
                vt = VarType::INT;
            }
            else if tkn.is("STR") {
                vt = VarType::STR;
            }
            else {
                panic!("Bad instantiation: found NEW keyword without type.");
            }
            tkn = tokenizer.get_token();
            if tkn.is("ID") {
                vars.push((tkn.value_str, vt));
            }
        }
        tkn = tokenizer.get_token();
    }
    if let Err(why) = file.write_all(content.as_bytes()) {
        panic!("couldn't write to file: {}", why.description());
    }
}

fn gen_code(file: &mut File, tokenizer: &mut Tokenizer){
    let content = "\n\n.text\n\
                    .global main\n\
                    .extern printf\n\
                    .extern malloc\n\
                    main:\n\
                    \t\tmovq %rdi, argc_\n\
                    \t\tmovq $16000, %rdi\n\
                    \t\tcall malloc\n\
                    \t\tmovq %rax, FuncTable\n";
    if let Err(why) = file.write_all(content.as_bytes()) {
        panic!("couldn't write to file: {}", why.description());
    }
    // load_token(code_text);
    // init funcs;
    // seq();
    if let Err(why) = file.write_all("\t\tretq\n".as_bytes()) {
        panic!("couldn't write to file: {}", why.description());
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum VarType {
    BOOL,
    INT,
    STR
}