mod tokenizer;
use self::tokenizer::Tokenizer;
use std::io::prelude::*;
use std::error::Error;
use std::fs::File;
use self::tokenizer::TokenType;
use self::tokenizer::Token;

#[derive(Debug, PartialEq, Copy, Clone)]
enum VarType {
    BOOL,
    INT,
    STR
}

pub struct Compiler {
    tokens: Vec<Token>,
    file: File,
    pos: usize
}

impl Compiler {
    pub fn new(program: String, file: File) -> Compiler {
        let mut tokenizer = Tokenizer::new(program);
        let tokens = tokenizer.tokenize();

        Compiler {
            tokens: tokens,
            file: file,
            pos: 0
        }
    }

    fn consume(&mut self) {
        self.pos += 1;
    }

    fn peek(&self) -> TokenType {
        self.tokens[self.pos].kind
    }

    fn current(&self) -> Token {
        self.tokens[self.pos].clone()
    }

    fn reset(&mut self) {
        self.pos = 0;
    }

    fn write(&mut self, data: &str) {
        if let Err(why) = self.file.write_all(data.as_bytes()) {
            panic!("couldn't write to file: {}", why.description());
        }
    }

    fn write_print_int(&mut self) {
        self.write("\t\tpush %rax\n\
                    \t\tmovq $0, %rax\n\
                    \t\tmovq $Format_ints, %rdi\n\
                    \t\tcall printf\n\
                    \t\tpop %rax\n");
    }

    pub fn gen_data(&mut self){
        let content = ".data\n\
                        \t\targc_: .quad 0\n\
                        \t\tFormat_ints: .byte '%', 'l', 'u', 10, 0\n\
                        \t\tFormat_strings: .byte '%', 's', 10, 0\n\
                        \t\tFuncTable: .quad 0\n\
                        \t\tFuncCall: .quad 0\n";

        let mut vars: Vec<(String, VarType)> = Vec::new();

        while self.peek() != TokenType::END {
            // println!("{:?}", self.current());
            if self.peek() == TokenType::NEW {
                self.consume();
                let vt: VarType;
                match self.peek() {
                    TokenType::BOOL => vt = VarType::BOOL,
                    TokenType::INT => vt = VarType::INT,
                    TokenType::STR => vt = VarType::STR,
                    _ => panic!("Bad instantiation: found NEW keyword without type.")
                };

                self.consume();
                if self.peek() == TokenType::ID {
                    vars.push((self.current().value_str, vt));
                }
            }
            self.consume();
        }

        // println!("{:?}", vars);
        self.write(content);

        self.reset();
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



        // Write initialization data to .s file
        self.write(content);

        // Write code to .s file
        self.program();

        // Wrap up .s file
        self.write("\t\tmovq $0, %rax\n\
                    \t\tretq\n");

    }

    fn program(&mut self) {
        while self.statement() {}
    }

    fn statement(&mut self) -> bool {
        match self.peek() {
            TokenType::PRINT => {
                self.consume();
                self.expression();
                self.write("# printing:\n");
                self.write("\t\tmovq %rax, %rsi\n");
                self.write_print_int();
                self.write("# finished printing\n");
            },
            _ => {
                self.consume();
                // println!("LSKDFJSLKDFJ");
                return false;
            }
        };
        return true;
    }

    fn expression(&mut self) {
        self.e3();
        while self.peek() == TokenType::EQEQ {
            self.consume();
            self.write("\t\tpush %rax\n");
            self.e3();
            self.write("\t\tpop %r15\n\
                        \t\tsubq %r15, %rax\n\
                        \t\tsete %al\n\
                        \t\tmovzbq %al, %rax\n");
        }
    }

    fn e3(&mut self) {
        self.e2();
        while self.peek() == TokenType::PLUS || self.peek() == TokenType::MINUS {
            let t_type = self.peek();
            self.consume();
            self.write("\t\tpush %rax\n");
            self.e2();

            match t_type {
                TokenType::PLUS => self.write("\t\tpop %r15\n\
                                                \t\taddq %r15, %rax\n"),
                TokenType::MINUS => self.write("\t\tmovq %rax, %r15\n\
                                                \t\tpop %rax\n\
                                                \t\tsubq %r15, %rax\n"),
                _ => panic!("Compile error at {}", self.pos)
            };
        }
    }

    fn e2(&mut self) {
        self.e1();
        if self.peek() == TokenType::MUL {
            while self.peek() == TokenType::MUL {
                self.consume();
                self.write("\t\tpush %rax\n");
                self.e1();
                self.write("\t\tpop %r15\n\
                            \t\tmul %r15\n");
            }
            
            self.write("\t\tmovq $0, %r15\n");
        }
    }

    fn e1(&mut self) {
        match self.peek() {
            TokenType::VAL => {
                let curr = self.current();
                if curr.value_str.len() == 0 {
                    self.write(format!("\t\tmovq ${}, %rax\n", curr.value_int).as_ref());
                } else {
                    println!("needs to be implemented");
                }

                self.consume();
            },
            _ => {
                println!("needs to be implemented");
            }
        }
    }
}