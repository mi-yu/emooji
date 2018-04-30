mod tokenizer;
use self::tokenizer::Tokenizer;
use std::io::prelude::*;
use std::error::Error;
use std::fs::File;

#[derive(Debug, PartialEq, Copy, Clone)]
enum VarType {
    BOOL,
    INT,
    STR
}

pub struct Compiler {
    tokenizer: Tokenizer,
    file: File
}

impl Compiler {
    pub fn new(program: String, file: File) -> Compiler {
        Compiler {
            tokenizer: Tokenizer::new(program),
            file: file
        }
    }

    pub fn gen_data(&mut self){
        let tokenizer = &mut self.tokenizer;
        let content = ".data\n\
                        \t\targc_: .quad 0\n\
                        \t\tFormat_ints: .byte '%', 'l', 'u', 10, 0\n\
                        \t\tFormat_strings: .byte '%', 's', 10, 0\n\
                        \t\tFuncTable: .quad 0\n\
                        \t\tFuncCall: .quad 0\n";

        let mut vars: Vec<(String, VarType)> = Vec::new();
        let mut tkn = tokenizer.get_token();

        while !(tkn.is("END")) {
            if tkn.is("NEW") {
                tkn = tokenizer.get_token();
                let vt: VarType;
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

        // println!("{:?}", vars);
        if let Err(why) = self.file.write_all(content.as_bytes()) {
            panic!("couldn't write to file: {}", why.description());
        }

        tokenizer.reset();
    }

    pub fn gen_code(&mut self){
        // let mut tokenizer = &self.tokenizer;
        let content = "\n\n.text\n\
                        .global main\n\
                        .extern printf\n\
                        .extern malloc\n\
                        main:\n\
                        \t\tmovq %rdi, argc_\n\
                        \t\tmovq $16000, %rdi\n\
                        \t\tcall malloc\n\
                        \t\tmovq %rax, FuncTable\n";



        // Write to .s file
        if let Err(why) = self.file.write_all(content.as_bytes()) {
            panic!("couldn't write to file: {}", why.description());
        }

        if let Err(why) = self.file.write_all("\t\tmovq $0, %rax\n\
                                            \t\tretq\n".as_bytes()) {
            panic!("couldn't write to file: {}", why.description());
        }

        let tokenizer = &mut self.tokenizer;
        tokenizer.start();
    }
}