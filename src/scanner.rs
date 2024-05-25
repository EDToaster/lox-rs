use std::{iter::Peekable, str::Chars};

use itertools::Itertools;

use crate::compiler::report_error;

/// Scanner scans individual bytes
#[derive(Debug, Clone)]
struct Scanner<'a> {
    pub source: &'a str,
    pub source_iterator: Peekable<Chars<'a>>,
    pub start: usize,
    pub current: usize,
    pub line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScannerState {
    General,
    StrInterp,
}

#[derive(Debug, Clone)]
pub struct TokenScanner<'a> {
    chars: Scanner<'a>,

    // Keeps track of current state of the scanner.0
    // "${" pushes StrInterp on the stack, "{" pushes General on the stack
    // when "}" is encountered, pop off the stack. If the top of the stack
    // was StrInterp, set force_str.
    state: Vec<ScannerState>,

    // Force the next token to be a Str, or StrInter
    force_str: bool,
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
    pub fn next_ignore_whitespace(&mut self) -> Option<char> {
        self.take_while_ref(|&c| c.is_whitespace()).count();
        self.make_lexeme();
        self.next()
    }

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
        self.make_lexeme_strip(0)
    }

    pub fn make_lexeme_strip(&mut self, end_strip: usize) -> &'a str {
        let ret = &self.source[self.start..(self.current - end_strip)];
        self.start = self.current;
        ret
    }
}

impl<'a> Iterator for TokenScanner<'a> {
    type Item = Token<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.force_str {
                self.force_str = false;
                return Some(self.take_string());
            }

            let c = self.chars.next_ignore_whitespace();
            if c.is_none() {
                break;
            }
            let c = c.unwrap();

            let tok = match c {
                c if c.is_ascii_digit() => self.take_numeric(),
                c if is_valid_identifier_first(c) => self.take_identifier_or_keyword(),
                '"' => {
                    // Ignore the "
                    self.chars.make_lexeme();
                    self.take_string()
                }
                '(' => self.make_token(TokenType::LParen),
                ')' => self.make_token(TokenType::RParen),
                '{' => {
                    self.state.push(ScannerState::General);
                    self.make_token(TokenType::LBrace)
                }
                '}' => {
                    // TODO: handle this properly!
                    // For example if the program looks like
                    //   print "hi"; }
                    let top = self.state.pop().expect("");
                    self.force_str = top == ScannerState::StrInterp;
                    self.make_token(TokenType::RBrace)
                }
                ';' => self.make_token(TokenType::Semi),
                ',' => self.make_token(TokenType::Comma),
                '.' => self.make_token(TokenType::Dot),
                '-' => self.make_token(TokenType::Minus),
                '+' => self.make_token(TokenType::Plus),
                '|' => self.make_token(TokenType::Bar),
                '*' => self.make_token(TokenType::Star),
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
                    } else if self.chars.next_if_match('>') {
                        TokenType::FatArrow
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
                '?' => {
                    let t = if self.chars.next_if_match(':') {
                        TokenType::QuestionColon
                    } else {
                        TokenType::Question
                    };
                    self.make_token(t)
                }
                ':' => self.make_token(TokenType::Colon),
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

        TokenScanner {
            chars: scanner,
            force_str: false,
            state: vec![],
        }
    }

    fn take_until_newline(&mut self) {
        self.chars.take_while_ref(|&c| c != '\n').count();
        self.chars.next();
        self.chars.make_lexeme();
    }

    /// Continue taking string until " or ${
    fn take_string(&mut self) -> Token<'a> {
        let mut dollar = false;

        while let Some(t) = self.chars.next() {
            if t == '"' {
                return self.make_token_strip(TokenType::Str, 1);
            } else if t == '$' {
                dollar = true;
            } else if dollar && t == '{' {
                self.state.push(ScannerState::StrInterp);
                return self.make_token_strip(TokenType::StrInterp, 2);
            } else {
                dollar = false;
            }
        }

        // unclosed string!
        let t = self.make_token(TokenType::Str);
        report_error(&t, "Unterminated string!");
        t
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
            "val" => TokenType::Val,
            "while" => TokenType::While,
            "match" => TokenType::Match,
            _ => TokenType::Ident,
        };
        Token {
            lexeme,
            ttype,
            line: self.chars.line,
        }
    }

    fn make_token(&mut self, ttype: TokenType) -> Token<'a> {
        self.make_token_strip(ttype, 0)
    }

    fn make_token_strip(&mut self, ttype: TokenType, end_strip: usize) -> Token<'a> {
        let lexeme = self.chars.make_lexeme_strip(end_strip);
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
    Bar,

    // One or two char
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    FatArrow,

    Question,
    Colon,
    // TODO: add support for ?, :, and ?: (true ? 1 : 0) and (nil ?: 0)
    //       where the statements after ?, :, and ?: are lazily evaluated.
    QuestionColon,

    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Ident,
    Str,
    StrInterp,
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
    Val,
    While,
    Match,

    // Misc
    Error,
}

fn is_valid_identifier_first(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_valid_identifier_rest(c: char) -> bool {
    is_valid_identifier_first(c) || c.is_ascii_digit()
}
