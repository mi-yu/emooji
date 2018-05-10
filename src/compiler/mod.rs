mod tokenizer;
use self::tokenizer::Tokenizer;
use std::io::prelude::*;
use std::error::Error;
use std::fs::File;
use self::tokenizer::TokenType;
use self::tokenizer::VarType;
use self::tokenizer::Token;
use std::collections::HashSet;
use std::collections::HashMap;

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
        // println!("{:?}", self.tokens[self.pos]);
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

    fn annotate_type(&mut self, var_type: VarType) {
        self.tokens[self.pos].var_type = var_type;
    }

    fn change_type_to_funid(&mut self) {
        self.tokens[self.pos].kind = TokenType::FUNID;
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

        let mut vars: HashMap<String, VarType> = HashMap::new();
        let mut funcs: HashSet<String> = HashSet::new();

        while self.peek() != TokenType::END {
            if self.peek() == TokenType::NEW {
                self.consume();
                let var_type = match self.peek() {
                    TokenType::BOOL => VarType::BOOL,
                    TokenType::INT => VarType::INT,
                    TokenType::STR => VarType::STR,
                    _ => panic!("Bad instantiation. Found NEW keyword without type: {}", 
                        self.debug_str()),
                };

                self.consume();
                match self.peek() {
                    TokenType::ID => {
                        let id = self.current().value_str;
                        content.push_str(&format!("\t\t{}: .quad 0\n", id));
                        self.annotate_type(var_type);
                        vars.insert(id, var_type);
                    },
                    _ => panic!("Bad instantiation. No variable name provided: {}", 
                        self.debug_str()),
                }
            }
            else if self.peek() == TokenType::FUN {
                self.consume();
                match self.peek() {
                    TokenType::ID => {
                        let id = self.current().value_str;
                        content.push_str(&format!("\t\t{}: .quad 0\n", id));
                        self.change_type_to_funid();
                        funcs.insert(id);
                    },
                    _ => panic!("Bad function declaration. No function name provided: {}", 
                        self.debug_str()),
                }
            }
            else {
                self.consume();
            }
        }

        self.write(&content);

        self.reset();

        // annotate all id types
        while self.peek() != TokenType::END {
            if self.peek() == TokenType::ID {
                let id = self.current().value_str;
                match vars.get(&id) {
                    Some(var_type) => self.annotate_type(*var_type),
                    None => {
                        if funcs.contains(&id) {
                            self.change_type_to_funid();
                        }
                        else {
                            panic!("Variable or function never declared: {}", self.debug_str());
                        }
                    }
                }
            }
            else {
                self.consume();
            }
        }

        self.reset();
    }

    pub fn check_syntax(&mut self) {
        self.check_syntax_seq();
        while self.peek() != TokenType::END {
            self.check_statement_syntax();
        }
        self.reset();
    }

    fn check_syntax_seq(&mut self) {
        while self.peek() != TokenType::RBRACE {
            self.check_statement_syntax();
        }
    }

    fn check_statement_syntax(&mut self) {
        match self.peek() {
            TokenType::LBRACE => {
                self.consume();
                self.check_syntax_seq();
                if self.peek() != TokenType::RBRACE {
                    panic!("Missing a closing brace: {}", self.debug_str());
                }
                self.consume();
            },
            TokenType::ID => {
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
            },
            TokenType::IF => {
                self.consume();            
                let expr_type = self.check_expr_syntax();
                if !(Token::can_convert_to(expr_type, VarType::BOOL)) {
                    panic!("Condition must evaluate to boolean: {}")
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
                        if !(Token::can_convert_to(expr_type, VarType::BOOL)) {
                            panic!("Condition must evaluate to boolean: {}")
                        }
                        self.check_statement_syntax();
                    }
                    else {
                        ended = true;
                        self.check_statement_syntax();
                    }
                }
            },
            TokenType::WHILE => {
                self.consume();
                let expr_type = self.check_expr_syntax();
                if !(Token::can_convert_to(expr_type, VarType::BOOL)) {
                    panic!("Condition must evaluate to boolean: {}")
                }
                self.check_statement_syntax();
            },
            TokenType::PRINT => {
                self.consume();
                self.check_expr_syntax();

                // enforce end punctuation
                if self.peek() != TokenType::LEND {
                    panic!("Missing line end punctuation: {}", 
                        self.debug_str());
                }
                self.consume();
            },
            // TokenType::SWAP => {
            //     self.consume();

            //     // check if args enclosed by parens
            //     let parens;
            //     if self.peek() != TokenType::LPAREN {
            //         parens = true;
            //         self.consume();
            //     }
            //     else {
            //         parens = false;
            //     }
            //     self.check_expr_syntax();
            //     self.check_expr_syntax();
                    // THIS IS NOT DONE
            //     if parens {
            //         if self.peek() != TokenType::RPAREN {
            //             panic!("Missing closing parenthesis: {}", self.debug_str());
            //         }
            //         self.consume();
            //     }
                
            //     // enforce end punctuation
            //     if self.peek() != TokenType::LEND {
            //         panic!("Missing line end punctuation: {}", 
            //             self.debug_str());
            //     }
            //     self.consume();
            // },
            TokenType::FUN => {
                self.consume();

            }
            _ => {
                panic!("Unexpected token: {}", self.debug_str());
            }
        }
    }

    fn check_expr_syntax(&mut self) -> VarType {
        let mut vt1 = self.check_e3_syntax();
        while self.peek() == TokenType::EQEQ {
            self.consume();
            let vt2 = self.check_e3_syntax();
            if !(vt1 == vt2){
                panic!("Cannot check equality of mismatched types: {}", self.debug_str());
            }
            vt1 = VarType::BOOL;
        }
        vt1
    }

    fn check_e3_syntax(&mut self) -> VarType {
        let mut vt1 = self.check_e2_syntax();
        while self.peek() == TokenType::PLUS || self.peek() == TokenType::MINUS {
            let op_type = self.peek();
            self.consume();
            let vt2 = self.check_e2_syntax();

            match op_type {
                TokenType::PLUS => {
                    if (vt1==VarType::STR) || (vt2==VarType::STR) {
                        vt1 = VarType::STR;
                    }
                    else if (vt1==VarType::INT) || (vt2==VarType::INT) {
                        vt1 = VarType::INT;
                    }
                },
                TokenType::MINUS => {
                    if (vt1==VarType::STR) || (vt2==VarType::STR) {
                        panic!("Subtraction not defined for strings: {}", self.debug_str());
                    }
                    else {
                        vt1 = VarType::INT;
                    }
                },
                _ => panic!("Internal error: {}", self.debug_str()),
            };
        }
        vt1
    }

    fn check_e2_syntax(&mut self) -> VarType {
        let mut vt1 = self.check_e1_syntax();
        while (self.peek() == TokenType::MUL) || (self.peek() == TokenType::DIV) {
            let op_type = self.peek();
            self.consume();
            let vt2 = self.check_e1_syntax();

            match op_type {
                TokenType::MUL => {
                    if (vt1==VarType::STR) || (vt2==VarType::STR) {
                        panic!("Multiplication not defined for strings: {}", self.debug_str());
                    }
                    else if (vt1==VarType::INT) || (vt2==VarType::INT) {
                        vt1 = VarType::INT;
                    }
                },
                TokenType::DIV => {
                    if (vt1==VarType::STR) || (vt2==VarType::STR) {
                        panic!("Division not defined for strings: {}", self.debug_str());
                    }
                    else {
                        vt1 = VarType::INT;
                    }
                },
                _ => panic!("Internal error: {}", self.debug_str()),
            }
        }
        vt1
    }

    fn check_e1_syntax(&mut self) -> VarType {
        let vt1;
        match self.peek() {
            TokenType::VAL => {
                vt1 = self.current().var_type;
                self.consume();
            },
            TokenType::LPAREN => {
                self.consume();
                vt1 = self.check_expr_syntax();
                if self.peek() != TokenType::RPAREN {
                    panic!("Missing closing parenthesis: {}", self.debug_str());
                }
                self.consume();
            },
            TokenType::ID => {
                vt1 = self.current().var_type;
                self.consume();
            },
            _ => {
                panic!("Value not found: {}", self.debug_str());
            }
        }
        vt1
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
            TokenType::CALL => {
                self.consume();
                let id = self.current().value_str;
                self.write(&format!("\t\tcall *{}\n", id));
                self.consume();
            },
            TokenType::FUN => {
                let curr_pos = self.pos;
                self.consume();
                // if self.peek() != TokenType::ID {
                //     panic!("function def must be followed by ID");
                // }
                let id = self.current().value_str;
                self.consume();

                self.write(&format!("\t\tlea func_{}, %rax\n", curr_pos));
                self.write(&format!("\t\tmovq %rax, {}\n", id));
                self.write(&format!("\t\tjmp finish_define_func_{}\n", curr_pos));
                self.write(&format!("func_{}:\n", curr_pos));
                self.statement();
                self.write("\t\tret\n");
                self.write(&format!("finish_define_func_{}:\n", curr_pos));
            },
            TokenType::LBRACE => {
                self.consume();
                self.program();
                // if self.peek() != TokenType::RBRACE {
                //     panic!("mismatched braces at {:?}", self.current());
                // }
                self.consume();
            },
            TokenType::IF => {
                self.consume();
                self.expression();
                self.write("\t\tcmp $0, %rax\n");
                let curr_pos = self.pos;
                self.write(&format!("\t\t je if_{}\n", curr_pos));
                self.statement();
                self.write(&format!("\t\t jmp done_if_{}\n", curr_pos));
                self.write(&format!("if_{}:\n", curr_pos));

                if self.peek() == TokenType::ELSE {
                    self.consume();
                    self.statement();
                }

                self.write(&format!("done_if_{}:\n", curr_pos));
            },
            TokenType::WHILE => {
                self.consume();
                let curr_pos = self.pos;
                self.write(&format!("while_{}:\n", curr_pos));
                self.expression();
                self.write("\t\tcmp $0, %rax\n");
                self.write(&format!("\t\tje while_done_{}\n", curr_pos));
                self.statement();
                self.write(&format!("\t\tjmp while_{}\n", curr_pos));
                self.write(&format!("while_done_{}:\n", curr_pos));
            }
            TokenType::ID => {
                let id = self.current().value_str;
                self.consume();

                // if self.peek() != TokenType::EQ {
                //     panic!("needs EQ after ID, {:?}", self.current());
                // }
                self.consume();

                self.expression();
                // assign value to variable
                self.write(&format!("\t\tmovq %rax, {}\n", id));
            },
            TokenType::PRINT => {
                self.consume();

                self.expression();
                self.write("# printing:\n");
                self.write("\t\tmovq %rax, %rsi\n");
                self.write_print_int();
                self.write("# finished printing\n");
            },
            TokenType::END | TokenType::RBRACE => {
                return false;
            },
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
                self.write(&format!("\t\tmovq ${}, %rax\n", curr.value_int));
                self.consume();
            },
            TokenType::LPAREN => {
                self.consume();
                self.expression();
                self.consume();
            },
            TokenType::ID => {
                let id = self.current().value_str;
                self.write(&format!("\t\tmovq {}, %rax\n", id));
                self.consume();
            },
            _ => {
                println!("needs to be implemented");
                self.consume();
            }
        }
    }
}