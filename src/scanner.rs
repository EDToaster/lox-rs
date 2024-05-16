use std::{iter::Peekable, str::Chars};

use itertools::Itertools;

/// Scanner scans individual bytes
#[derive(Debug, Clone)]
struct Scanner<'a> {
    pub source: &'a str,
    pub source_iterator: Peekable<Chars<'a>>,
    pub start: usize,
    pub current: usize,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct TokenScanner<'a> {
    chars: Scanner<'a>,
}

impl<'a> Iterator for Scanner<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(c) = self.source_iterator.next() {
            self.current += 1;
            if c == '\n' {
                self.line += 1;
            }
            Some(c)
        } else {
            None
        }
    }
}

impl<'a> Scanner<'a> {
    // conditionally match the next char
    pub fn next_if_match(&mut self, c: char) -> bool {
        if let Some(&n) = self.source_iterator.peek() {
            if n == c {
                self.next();
                return true;
            }
        }
        false
    }

    pub fn make_lexeme(&mut self) -> &'a str {
        let ret = &self.source[self.start..self.current];
        self.start = self.current;
        ret
    }
}

impl<'a> Iterator for TokenScanner<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Skip whitespace
        self.take_whitespace();

        while let Some(c) = self.chars.next() {
            let tok = match c {
                c if c.is_ascii_digit() => self.take_numeric(),
                c if is_valid_identifier_first(c) => self.take_identifier_or_keyword(),
                '"' => self.take_string(),
                '(' => self.make_token(TokenType::LParen),
                ')' => self.make_token(TokenType::RParen),
                '{' => self.make_token(TokenType::LBrace),
                '}' => self.make_token(TokenType::RBrace),
                ';' => self.make_token(TokenType::Semi),
                ',' => self.make_token(TokenType::Comma),
                '.' => self.make_token(TokenType::Dot),
                '-' => self.make_token(TokenType::Minus),
                '+' => self.make_token(TokenType::Plus),
                '/' => {
                    if self.chars.next_if_match('/') {
                        self.take_until_newline();
                        continue;
                    } else {
                        self.make_token(TokenType::Slash)
                    }
                }
                '!' => {
                    let t = if self.chars.next_if_match('=') {
                        TokenType::BangEqual
                    } else {
                        TokenType::Bang
                    };
                    self.make_token(t)
                }

                '=' => {
                    let t = if self.chars.next_if_match('=') {
                        TokenType::EqualEqual
                    } else {
                        TokenType::Equal
                    };
                    self.make_token(t)
                }

                '<' => {
                    let t = if self.chars.next_if_match('=') {
                        TokenType::LessEqual
                    } else {
                        TokenType::Less
                    };
                    self.make_token(t)
                }

                '>' => {
                    let t = if self.chars.next_if_match('=') {
                        TokenType::GreaterEqual
                    } else {
                        TokenType::Greater
                    };
                    self.make_token(t)
                }
                '*' => self.make_token(TokenType::Star),
                _ => self.make_token(TokenType::Error),
            };
            return Some(tok);
        }

        None
    }
}

impl<'a> TokenScanner<'a> {
    pub fn from_source(source: &str) -> TokenScanner {
        let scanner = Scanner {
            source,
            source_iterator: source.chars().peekable(),
            start: 0,
            current: 0,
            line: 1,
        };

        TokenScanner { chars: scanner }
    }

    /// Consume all whitespace elements
    fn take_whitespace(&mut self) {
        let mut comment = false;
        self.chars
            .take_while_ref(|&c| {
                comment = true;
                c.is_whitespace()
            })
            .count();
        self.chars.make_lexeme();
    }

    fn take_until_newline(&mut self) {
        self.chars.take_while_ref(|&c| c != '\n').count();
        self.chars.next();
        self.chars.make_lexeme();
    }

    /// Continue taking string until "
    fn take_string(&mut self) -> Token<'a> {
        self.chars.take_while_ref(|&c| c != '"').count();
        self.chars.next();
        self.make_token(TokenType::String)
    }

    /// Continue taking numeric digits assuming the first digit is already consumed
    fn take_numeric(&mut self) -> Token<'a> {
        self.chars.take_while_ref(|&c| c.is_ascii_digit()).count();

        // allow fractional
        if self.chars.next_if_match('.') {
            self.chars.take_while_ref(|&c| c.is_ascii_digit()).count();
        }
        self.make_token(TokenType::Number)
    }

    /// Continue taking identifier assuming the first letter is already consumed
    fn take_identifier_or_keyword(&mut self) -> Token<'a> {
        self.chars
            .take_while_ref(|&c| is_valid_identifier_rest(c))
            .count();

        self.make_identifier_or_keyword()
    }

    fn make_identifier_or_keyword(&mut self) -> Token<'a> {
        let lexeme = self.chars.make_lexeme();
        let ttype = match lexeme {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "fun" => TokenType::Fun,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "var" => TokenType::Var,
            "while" => TokenType::While,
            _ => TokenType::Ident,
        };
        Token {
            lexeme,
            ttype,
            line: self.chars.line,
        }
    }

    fn make_token(&mut self, ttype: TokenType) -> Token<'a> {
        let lexeme = self.chars.make_lexeme();
        Token {
            lexeme,
            ttype,
            line: self.chars.line,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Token<'a> {
    pub lexeme: &'a str,
    pub ttype: TokenType,
    pub line: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TokenType {
    // One char
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semi,
    Slash,
    Star,

    // One or two char
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Ident,
    String,
    Number,

    // Keywords
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    // Misc
    Error,
}

fn is_valid_identifier_first(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_valid_identifier_rest(c: char) -> bool {
    is_valid_identifier_first(c) || c.is_ascii_digit()
}
