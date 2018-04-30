mod tokenizer_util;
use std::char;
use self::tokenizer_util::is_emoji;
use self::tokenizer_util::is_keycap;
use self::tokenizer_util::get_keycap_val;

mod token_types;
use self::token_types::TokenType;

#[derive(Debug)]
pub struct Token {
    pub kind: TokenType,
    pub value_int: u64,
    pub value_str: String
}

impl Token {
    pub fn new() -> Token {
        Token {
            kind: TokenType::NONE,
            value_int: 0,
            value_str: String::from("")
        }
    }

    pub fn is(&self, test_kind: &str) -> bool{
        match &test_kind.to_uppercase()[..] {
            "ELSE" => return self.kind == TokenType::ELSE,
            "IF" => return self.kind == TokenType::IF,
            "RPAREN" => return self.kind == TokenType::RPAREN,
            "LPAREN" => return self.kind == TokenType::LPAREN,
            "RAND" => return self.kind == TokenType::RAND,
            "NOT" => return self.kind == TokenType::NOT,
            "EQ" => return self.kind == TokenType::EQ,
            "EQEQ" => return self.kind == TokenType::EQEQ,
            "MUL" => return self.kind == TokenType::MUL,
            "DIV" => return self.kind == TokenType::DIV,
            "PLUS" => return self.kind == TokenType::PLUS,
            "MINUS" => return self.kind == TokenType::MINUS,
            "SWAP" => return self.kind == TokenType::SWAP,
            "NEW" => return self.kind == TokenType::NEW,
            "TRUE" => return self.kind == TokenType::TRUE,
            "FALSE" => return self.kind == TokenType::FALSE,
            "BOOL" => return self.kind == TokenType::BOOL,
            "INT" => return self.kind == TokenType::INT,
            "STR" => return self.kind == TokenType::STR,
            "LEND" => return self.kind == TokenType::LEND,
            "WHILE" => return self.kind == TokenType::WHILE,
            "PRINT" => return self.kind == TokenType::PRINT,
            "CALL" => return self.kind == TokenType::CALL,
            "ID" => return self.kind == TokenType::ID,
            "VAL" => return self.kind == TokenType::VAL,
            "END" => return self.kind == TokenType::END,
            _ => panic!("Not a valid type: {}", test_kind),
        }
    }
}

#[derive(Debug, PartialEq)]
enum TokenizerState {
    DEFINING,
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

    pub fn start(&mut self) {
        let mut tkn = Token::new();

        println!("{:?}\n {}", &self.program, self.program.len());

        while tkn.kind != TokenType::END {
            tkn = self.get_token();
            println!("{:?}, state: {:?}, pos: {}, actual char: {:?}", tkn, self.state, self.pos, self.program[self.pos] as u8);
        }
    }

    pub fn get_token(&mut self) -> Token{
        let mut tkn = Token {
            kind: TokenType::NONE,
            value_int: 0,
            value_str: String::from("")
        };

        let mut pos = self.pos;
        pos = self.fast_forward_whitespace(pos);

        if pos < self.program.len() {
            let prog = &self.program;
            match prog[pos] {
                '❌' => tkn.kind = TokenType::ELSE,
                '❓' => tkn.kind = TokenType::IF,
                '🌛' => tkn.kind = TokenType::RPAREN,
                '🌜' => tkn.kind = TokenType::LPAREN,
                '🎲' => tkn.kind = TokenType::RAND,
                '🚫' => tkn.kind = TokenType::NOT,
                '⬅' => tkn.kind = TokenType::EQ,
                '↔' => tkn.kind = TokenType::EQEQ,
                '✖' => tkn.kind = TokenType::MUL,
                '➗' => tkn.kind = TokenType::DIV,
                '➕' => tkn.kind = TokenType::PLUS,
                '➖' => tkn.kind = TokenType::MINUS,
                '🔀' => tkn.kind = TokenType::SWAP,
                '🆕' => tkn.kind = TokenType::NEW,
                '👍' => tkn.kind = TokenType::TRUE,
                '👎' => tkn.kind = TokenType::FALSE,
                '☯' => tkn.kind = TokenType::BOOL,
                '🔢' => tkn.kind = TokenType::INT,
                '🔤' => tkn.kind = TokenType::STR,
                '🔚' => tkn.kind = TokenType::LEND,
                '🔁' => tkn.kind = TokenType::WHILE,
                '📄' => tkn.kind = TokenType::PRINT,
                '📞' => tkn.kind = TokenType::CALL,
                _ => {
                    // Handle variable names
                    if self.state == TokenizerState::DEFINING {
                        tkn.kind = TokenType::ID;
                        let (end_pos, id) = self.create_id(pos);
                        pos = end_pos;
                        tkn.value_str = id;
                        self.state = TokenizerState::NONE;
                    } else { // Handle value creation
                        tkn.kind = TokenType::VAL;

                        // Make make digit (each keycap is 3 codepoints wide)
                        if pos + 3 < prog.len() && is_keycap(&prog[pos..pos+3]) {
                            while pos + 3 < prog.len() && is_keycap(&prog[pos..pos+3]) {
                                tkn.value_int *= 10;
                                tkn.value_int += get_keycap_val(prog[pos]);
                                pos += 3;
                            }
                            pos -= 1;
                        } else { // Handle make string
                            while pos < prog.len() && !prog[pos].is_whitespace() {
                                tkn.value_str.push(prog[pos]);
                                pos += 1;
                            }
                            pos -= 1;
                        }
                    }
                }
            };

            self.pos = pos + 1;
        } else {
            tkn.kind = TokenType::END;
        }

        self.normalize_pos_and_state(tkn.kind);

        tkn
    }

    fn normalize_pos_and_state(&mut self, kind: TokenType) {
        if kind == TokenType::EQ || kind == TokenType::EQEQ || kind == TokenType::MUL || kind == TokenType::BOOL {
            self.pos += 1;
        }

        if kind == TokenType::BOOL || kind == TokenType::INT || kind == TokenType::STR {
            self.state = TokenizerState::DEFINING;
        }
    }

    fn fast_forward_whitespace(&mut self, start: usize) -> usize {
        let mut i = start;
        let prog = &self.program;
        while i < prog.len() && prog[i].is_whitespace() {
            i += 1;
        }

        return i;
    }

    fn create_id(&self, mut start: usize) -> (usize, String) {
        let mut id = String::new();
        while {
            id.push(self.program[start]);
            start += 1;

            is_emoji(self.program[start])
        } {}

        (start, id)
    }
}