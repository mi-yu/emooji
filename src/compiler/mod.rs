mod tokenizer;
use self::tokenizer::Tokenizer;
use std::io::prelude::*;
use std::error::Error;
use std::fs::File;
use self::tokenizer::TokenType;
use self::tokenizer::VarType;
use self::tokenizer::Token;

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

    fn debug_str(&self) -> String {
        let mut content = format!("token at pos {}: ", self.pos);
        let mut count = 0;
        while count < 5 && self.pos+count < self.tokens.len(){
            content.push_str(&self.tokens[self.pos + count].to_string());
            count += 1;
        }
        content
    }

    fn annotate_type(&mut self, var_type: VarType, var_count: i32) {
        self.tokens[self.pos].var_type = var_type;
        self.tokens[self.pos].var_num = var_count;
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

    pub fn gen_data(&mut self) {
        let mut content = String::from(".data\n\
                        \t\targc_: .quad 0\n\
                        \t\tFormat_ints: .byte '%', 'l', 'u', 10, 0\n\
                        \t\tFormat_strings: .byte '%', 's', 10, 0\n\
                        \t\tFuncTable: .quad 0\n\
                        \t\tFuncCall: .quad 0\n");

        // let mut vars: Vec<String> = Vec::new();
        let mut var_count = 0;

        while self.peek() != TokenType::END {
            if self.peek() == TokenType::NEW {
                self.consume();
                let var_type = match self.peek() {
                    TokenType::BOOL => VarType::BOOL,
                    TokenType::INT => VarType::INT,
                    TokenType::STR => VarType::STR,
                    TokenType::FUN => VarType::FUN,
                    _ => panic!("Bad instantiation. Found NEW keyword without type: {}", 
                        self.debug_str()),
                };

                self.consume();
                match self.peek() {
                    TokenType::ID => {
                        self.annotate_type(var_type, var_count);
                        // vars.push(self.current().value_str);
                        if var_type == VarType::STR {
                            self.consume();
                            self.consume();
                            content.push_str(&format!("\t\tvar{}: .string \"{}\"\n", var_count, self.current().value_str));
                        } else {
                            content.push_str(&format!("\t\tvar{}: .quad 0\n", var_count));
                            self.consume();
                            self.consume();
                        }
                        // annotate values as well
                        self.annotate_type(var_type, var_count);
                        var_count += 1;
                    },
                    _ => panic!("Bad instantiation. No variable name provided: {}", 
                        self.debug_str()),
                }
            }
            self.consume();
        }

        self.write(&content);

        self.reset();
    }

    pub fn check_syntax(&mut self) {
        self.check_syntex_seq()
        while self.peek() != TokenType::END {
            check_statement_syntax();
        }
        self.reset();
    }

    fn check_syntex_seq(&mut self) {
        while self.peek() != TokenType::RBRACE {
            self.check_statement_syntax();
        }
    }

    fn check_statement_syntax(&mut self) {
        if self.peek() == TokenType::LBRACE {
            self.consume();
            self.check_syntax_seq();
            if self.peek() != TokenType::RBRACE {
                panic!("Missing a closing brace: {}", self.debug_str());
            }
            self.consume();
        }
        else if self.peek() == TokenType::ID {
            // consume var name token
            let var_type = self.current().var_type;
            self.consume();

            // consume EQ token
            if self.peek() != TokenType::EQ {
                panic!("Bad instantiation. Must assign value to new variable: {}", 
                    self.debug_str());
            }
            self.consume();

            // check expression validity and check if assignment types match
            let expr_type = self.check_expr_syntax();
            if !(Token::can_convert_to(expr_type, var_type)){
                panic!("Illegal assignment. Cannot convert to {}: {}",
                    match var_type {
                        VarType::BOOL => "bool",
                        VarType::INT => "int",
                        VarType::STR => "string",
                        _ => ""
                    },
                    self.debug_str());
            }

            // enforce end punctuation
            if self.peek() != TokenType::LEND {
                panic!("Missing line end punctuation: {}", 
                    self.debug_str());
            }
            self.consume();
        }
        else if self.peek() == TokenType::IF {
            self.consume();            
            let expr_type = self.check_expr_syntax();
            if !(Token::can_convert_to(expr_type, VarType::BOOL) {
                panic!("Condition must evaluate to boolean: {}",
            }
            self.check_statement_syntax();
            let mut ended = false;
            while self.peek() == TokenType::ELSE {
                self.consume();
                if ended {
                    panic!("Misplaced 'else': {}", self.debug_str());
                }
                if self.peek() == TokenType::IF {
                    self.consume();
                    let expr_type = self.check_expr_syntax();
                    if !(Token::can_convert_to(expr_type, VarType::BOOL) {
                        panic!("Condition must evaluate to boolean: {}",
                    }
                    self.check_statement_syntax();
                }
                else {
                    ended = true;
                    self.check_statement_syntax();
                }
            }
        }
        else if self.peek() == TokenType::WHILE {
            self.consume();
        }
        else if self.peek() == TokenType::PRINT {
            self.consume();
        }
        else if self.peek() == TokenType::RAND {
            self.consume();
        }
        else if self.peek() == TokenType::SWAP {
            self.consume();
        }
        else if self.peek() == TokenType::FUN {
            self.consume();
        }
    }

    fn check_expr_syntax(&mut self) -> VarType {
        self.consume();
        VarType::BOOL
    }

    pub fn gen_code(&mut self) {
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
        println!("{:?}", self.current());
        match self.peek() {
            TokenType::ID => {
                let var_id = self.current().var_num;
                self.consume();

                // TODO: handle function calls

                if self.peek() != TokenType::EQ {
                    panic!("needs EQ after ID");
                }
                self.consume();

                self.expression();
                // assign value to variable
                self.write(&format!("\t\tmovq %rax, var{}\n", var_id));
            },
            TokenType::PRINT => {
                self.consume();

                self.expression();
                self.write("# printing:\n");
                self.write("\t\tmovq %rax, %rsi\n");
                self.write_print_int();
                self.write("# finished printing\n");
            },
            TokenType::END => {
                return false;
            }
            _ => {
                self.consume();
                // println!("LSKDFJSLKDFJ");
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
                // println!("{:?}", curr);
                if curr.var_type == VarType::INT || curr.var_type == VarType::BOOL {
                    self.write(&format!("\t\tmovq ${}, %rax\n", curr.value_int));
                } else {
                    self.write(&format!("\t\tmovq var{}, %rax\n", curr.var_num));
                }

                self.consume();
            },
            _ => {
                println!("needs to be implemented");
                self.consume();
            }
        }
    }
}