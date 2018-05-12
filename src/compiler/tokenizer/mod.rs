mod tokenizer_util;
use std::char;
use self::tokenizer_util::is_emoji;
use self::tokenizer_util::is_keycap;
use self::tokenizer_util::get_keycap_val;
use self::tokenizer_util::is_keyword;
use self::tokenizer_util::is_variant_selector;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TokenType {
    BOOL,
    CALL,
    DELIM,
    DIV,
    ELSE,
    END,
    EQ,
    EQEQ,
    FUN,
    FUNID,
    ID,
    IF,
    INT,
    LBRACE,
    LEND,
    LPAREN,
    MUL,
    MINUS,
    NEW,
    NONE,
    NOT,
    PLUS,
    PRINT,
    RAND,
    RBRACE,
    RPAREN,
    STR,
    VAL,
    WHILE,
    QUOTE,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum VarType {
    BOOL,
    INT,
    STR,
    NONE,
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenType,
    pub value_int: u64,
    pub value_str: String,
    pub var_type: VarType,
    pub arg_types: Vec<VarType>
}

impl Clone for Token {
    fn clone(&self) -> Token {
        Token {
            kind: self.kind,
            value_int: self.value_int,
            value_str: self.value_str.clone(),
            var_type: self.var_type,
            arg_types: self.arg_types.clone()
        }
    }
}

impl Token {
    pub fn new() -> Token {
        Token {
            kind: TokenType::NONE,
            value_int: 0,
            value_str: String::from(""),
            var_type: VarType::NONE,
            arg_types: Vec::new()
        }
    }

    pub fn to_string(&self) -> String {
        self.value_str.clone()
    }

    pub fn can_convert_to(from_type: VarType, to_type: VarType) -> bool {
        if to_type == VarType::NONE {
            panic!("Internal error: cannot convert to NONE.");
        }
        match from_type {
            VarType::BOOL => true,
            VarType::INT => to_type != VarType::BOOL,
            VarType::STR => to_type == VarType::STR,
            _ => panic!("Internal error: cannot convert from NONE."),
        }
    }

    pub fn copy_arg_types(&mut self, args: &Vec<VarType>) {
        for vt in args {
            self.arg_types.push(*vt);
        }
    }
}

#[derive(Debug, PartialEq)]
enum TokenizerState {
    DEFINING,
    DEFINED,
    NONE
}

pub struct Tokenizer {
    program: Vec<char>,
    state: TokenizerState,
    pos: usize
}

impl Tokenizer {
    pub fn new(prog: String) -> Tokenizer {
        Tokenizer {
            program: prog.chars().collect(),
            state: TokenizerState::NONE,
            pos: 0
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tkn = Token::new();
        let mut tokens = Vec::new();
        // println!("{:?}\n {}", &self.program, self.program.len());

        while tkn.kind != TokenType::END {
            // println!("{:?}", tkn);
            tkn = self.get_token();
            tokens.push(tkn.clone())
            // println!("{:?}, state: {:?}, pos: {}, actual char: {:?}", tkn, self.state, self.pos, self.program[self.pos] as u8);
        }
        // println!("{:?}", tokens);
        return tokens;
    }

    pub fn get_token(&mut self) -> Token{
        let mut tkn = Token::new();

        let mut pos = self.pos;
        pos = self.fast_forward_whitespace(pos);

        if pos < self.program.len() {
            let prog = &self.program;
            // println!("{}", prog[pos]);
            tkn.kind = match prog[pos] {
                '❌' => TokenType::ELSE,
                '❓' => TokenType::IF,
                '🌛' => TokenType::RPAREN,
                '🌜' => TokenType::LPAREN,
                '🌘' => TokenType::LBRACE,
                '🌒' => TokenType::RBRACE,
                '🎲' => TokenType::RAND,
                '🚫' => TokenType::NOT,
                '⬅'  => TokenType::EQ,
                '↔'  => TokenType::EQEQ,
                '✖' => TokenType::MUL,
                '➗' => TokenType::DIV,
                '➕' => TokenType::PLUS,
                '➖' => TokenType::MINUS,
                // '🔀' => TokenType::SWAP,
                '🆕' => TokenType::NEW,
                '☯' => TokenType::BOOL,
                '🔢' => TokenType::INT,
                '🔤' => TokenType::STR,
                '◾' => TokenType::DELIM,
                '🔚' => TokenType::LEND,
                '🔁' => TokenType::WHILE,
                '📄' => TokenType::PRINT,
                '📞' => TokenType::CALL,
                '🤪' => TokenType::FUN,
                '👍' => {
                    tkn.value_int = 1;
                    tkn.var_type = VarType::BOOL;
                    TokenType::VAL
                    },
                '👎' => {
                    tkn.value_int = 0;
                    tkn.var_type = VarType::BOOL;
                    TokenType::VAL
                    },
                _ => {
                    // Handle variable names
                    if self.state == TokenizerState::DEFINING || self.state == TokenizerState::DEFINED {
                        let (end_pos, id) = self.create_id(pos);
                        pos = end_pos;
                        tkn.value_str = id;
                        self.state = TokenizerState::NONE;
                        TokenType::ID
                    } else { // Handle value creation
                        // Make make digit (each keycap is 3 codepoints wide)
                        if pos + 3 < prog.len() && is_keycap(&prog[pos..pos+3]) {
                            while pos + 3 < prog.len() && is_keycap(&prog[pos..pos+3]) {
                                tkn.value_int *= 10;
                                tkn.value_int += get_keycap_val(prog[pos]);
                                pos += 3;
                            }
                            pos -= 1;
                            tkn.var_type = VarType::INT;
                            TokenType::VAL
                        } else if prog[pos] == '💬' { // Handle make string
                            // println!("{:?}, state: {:?}, pos: {}, actual char: {:?}", tkn, self.state, self.pos, self.program[self.pos]);
                            pos += 1;
                            while pos < prog.len() && is_emoji(prog[pos]) && prog[pos] != '💬' {
                                tkn.value_str.push(prog[pos]);
                                pos += 1;
                            }
                            tkn.var_type = VarType::STR;
                            TokenType::VAL
                        } else {
                            while pos < prog.len() && is_emoji(prog[pos]) && !is_keyword(prog[pos]) && prog[pos] != '🔚' {
                                tkn.value_str.push(prog[pos]);
                                pos += 1;
                            }
                            pos -= 1;
                            TokenType::ID
                        }
                    }
                }
            };

            self.pos = pos + 1;
        } else {
            tkn.kind = TokenType::END;
        }

        self.normalize_pos_and_state(tkn.kind);
        // println!("{:?}, {}", tkn, self.pos);
        tkn
    }

    fn normalize_pos_and_state(&mut self, kind: TokenType) {
        if kind == TokenType::EQ || kind == TokenType::EQEQ || kind == TokenType::MUL || kind == TokenType::BOOL {
            self.pos += 1;
        }

        if kind == TokenType::BOOL || kind == TokenType::INT || kind == TokenType::STR {
            self.state = TokenizerState::DEFINING;
        }

        if self.state == TokenizerState::DEFINED {
            self.state = TokenizerState::NONE;
        }
        
        if kind == TokenType::VAL {
            self.state = TokenizerState::DEFINED;
        }

    }

    fn fast_forward_whitespace(&mut self, start: usize) -> usize {
        let mut i = start;
        let prog = &self.program;
        while i < prog.len() && (prog[i].is_whitespace() || is_variant_selector(prog[i])) {
            i += 1;
        }

        return i;
    }

    fn create_id(&self, mut start: usize) -> (usize, String) {
        let mut id = String::new();
        while is_emoji(self.program[start]) && self.program[start] != '🔚' {
            id.push(self.program[start]);
            start +=1;
        }

        (start - 1, id)
    }
}