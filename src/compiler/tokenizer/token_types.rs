#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TokenType {
    BOOL,
    CALL,
    DIV,
    ELSE,
    END,
    EQ,
    EQEQ,
    FALSE,
    FUN,
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
    SWAP,
    TRUE,
    VAL,
    WHILE,
}

// impl TokenType {
//     pub fn as_char(&self) -> char {
//         match *self {        
//             TokenType::ELSE => '❌',
//             TokenType::IF => '❓',
//             TokenType::RPAREN => '🌛',
//             TokenType::LPAREN => '🌜',
//             TokenType::RAND => '🎲',
//             TokenType::EQ => '⬅',
//             _ => 'z'
//         }
//     }
// }