mod diagnostics;

use diagnostics::{
    InvalidOrUnexpectedToken, LegacyDecimalEscape, LegacyOctalLiteral, UnexpectedNumber,
    UnexpectedToken,
};
use miette::{Error, SourceOffset, SourceSpan};
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn size(&self) -> usize {
        self.end - self.start
    }
}

impl From<Span> for SourceSpan {
    fn from(val: Span) -> Self {
        Self::new(
            SourceOffset::from(val.start),
            SourceOffset::from(val.size()),
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Eof,
    Arrow, // =>
    Number { value: f64 },
    String { value: String, raw: String },
    Word(WordKind),
    SingleLineComment,
    MultiLineComment,
    Backquote,  // `
    LBrace,     // {
    LParen,     // (
    RBrace,     // }
    RParen,     // )
    LBracket,   // [
    RBracket,   // ]
    Comma,      // ,
    Dot,        // .
    DotDotDot,  // ...
    Bang,       // !
    Semicolon,  // ;
    Colon,      // :
    Question,   // ?
    Tilde,      // ~
    PlusPlus,   // ++
    MinusMinus, // --
    AssignOp(AssignOp),
    BinaryOp(BinaryOp),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AssignOp {
    Assign,                   // =
    AddAssign,                // +=
    SubAssign,                // -=
    MulAssign,                // *=
    DivAssign,                // /=
    ModAssign,                // %=
    BitOrAssign,              // |=
    BitXorAssign,             // ^=
    BitAndAssign,             // &=
    ZeroFillRightShiftAssign, // >>>=
    RightShiftAssign,         // >>=
    LeftShiftAssign,          // <<=
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Lt,                 // <
    Le,                 // <=
    Gt,                 // >
    Ge,                 // >=
    LShift,             // <<
    RShift,             // >>
    ZeroFillRightShift, // >>>
    Eq,                 // ==
    EqEq,               // ===
    Ne,                 // !=
    NeNe,               // !==
    Add,                // +
    Sub,                // -
    Mul,                // *
    Div,                // /
    Mod,                // %
    BitOr,              // |
    BitXor,             // ^
    BitAnd,             // &
    LogicalOr,          // ||
    LogicalAnd,         // &&
}

#[derive(Debug, Clone, PartialEq)]
pub enum WordKind {
    Keyword(Keyword),
    Identifier(String),
    True,
    False,
    Null,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Keyword {
    Break,
    Case,
    Catch,
    Class,
    Const,
    Continue,
    Debugger,
    Default,
    Delete,
    Do,
    Else,
    Export,
    Extends,
    Finally,
    For,
    Function,
    If,
    Import,
    In,
    Instanceof,
    New,
    Return,
    Let,
    Super,
    Switch,
    This,
    Throw,
    Try,
    Typeof,
    Var,
    Void,
    While,
    With,
    Yield,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Punctuators {}

pub struct Lexer<'a> {
    source: &'a str,
    chars: Chars<'a>,
    last_pos: usize,
    pub(crate) errors: Vec<Error>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars(),
            last_pos: 0,
            errors: vec![],
        }
    }

    pub fn lex(mut self) -> (Vec<Token>, Vec<Error>) {
        let mut tokens = vec![];
        loop {
            let token = self.read_next_token();
            if token.kind == TokenKind::Eof {
                break;
            }
            tokens.push(token);
        }

        (tokens, self.errors.into_iter().collect())
    }

    fn cur(&mut self) -> Option<char> {
        self.chars.clone().next()
    }

    fn peek(&self) -> Option<char> {
        self.chars.clone().nth(1)
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.cur() {
            if c.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }
    }

    fn read_next_kind(&mut self) -> TokenKind {
        match self.chars.next() {
            Some(c) => match c {
                '\'' => {
                    let (value, raw) = self.read_string_literal('\'');
                    self.last_pos = self.offset();
                    TokenKind::String { value, raw }
                }
                '\"' => {
                    let (value, raw) = self.read_string_literal('\"');
                    self.last_pos = self.offset();
                    TokenKind::String { value, raw }
                }
                ')' => {
                    self.last_pos = self.offset();
                    TokenKind::RParen
                }
                '(' => {
                    self.last_pos = self.offset();
                    TokenKind::LParen
                }
                '{' => {
                    self.last_pos = self.offset();
                    TokenKind::LBrace
                }
                '}' => {
                    self.last_pos = self.offset();
                    TokenKind::RBrace
                }
                '[' => {
                    self.last_pos = self.offset();
                    TokenKind::LBracket
                }
                ']' => {
                    self.last_pos = self.offset();
                    TokenKind::RBracket
                }
                ':' => {
                    self.last_pos = self.offset();
                    TokenKind::Colon
                }
                '!' => {
                    self.last_pos = self.offset();
                    match self.peek() {
                        Some('=') => {
                            self.chars.next();
                            self.last_pos = self.offset();
                            match self.peek() {
                                Some('=') => {
                                    self.chars.next();
                                    self.last_pos = self.offset();
                                    TokenKind::BinaryOp(BinaryOp::NeNe)
                                }
                                _ => TokenKind::BinaryOp(BinaryOp::Ne),
                            }
                        }
                        _ => TokenKind::Bang,
                    }
                }
                '?' => {
                    self.last_pos = self.offset();
                    TokenKind::Question
                }
                ';' => {
                    self.last_pos = self.offset();
                    TokenKind::Semicolon
                }
                ',' => {
                    self.last_pos = self.offset();
                    TokenKind::Comma
                }
                '+' => {
                    self.last_pos = self.offset();
                    match self.peek() {
                        Some('=') => {
                            self.chars.next();
                            self.last_pos = self.offset();
                            TokenKind::AssignOp(AssignOp::AddAssign)
                        }
                        Some('+') => {
                            self.chars.next();
                            self.last_pos = self.offset();
                            TokenKind::PlusPlus
                        }
                        _ => TokenKind::BinaryOp(BinaryOp::Add),
                    }
                }
                '-' => {
                    self.last_pos = self.offset();
                    match self.peek() {
                        Some('-') => {
                            self.chars.next();
                            TokenKind::MinusMinus
                        }
                        Some('=') => {
                            self.chars.next();
                            TokenKind::AssignOp(AssignOp::SubAssign)
                        }
                        _ => TokenKind::BinaryOp(BinaryOp::Sub),
                    }
                }
                '*' => {
                    self.last_pos = self.offset();
                    match self.peek() {
                        Some('=') => {
                            self.chars.next();
                            TokenKind::AssignOp(AssignOp::MulAssign)
                        }
                        _ => TokenKind::BinaryOp(BinaryOp::Mul),
                    }
                }
                '/' => match self.cur() {
                    Some('/') => {
                        self.chars.next();
                        self.chars.next();
                        for c in self.chars.by_ref() {
                            if is_line_terminator(c) {
                                break;
                            }
                        }
                        self.last_pos = self.offset();
                        TokenKind::SingleLineComment
                    }
                    Some('*') => {
                        self.chars.next();
                        self.chars.next();
                        while let Some(c) = self.chars.next() {
                            if c == '*' && matches!(self.peek(), Some('/')) {
                                self.chars.next();
                                break;
                            }
                        }
                        self.last_pos = self.offset();
                        TokenKind::MultiLineComment
                    }
                    Some('=') => {
                        self.chars.next();
                        self.last_pos = self.offset();
                        TokenKind::AssignOp(AssignOp::DivAssign)
                    }
                    _ => {
                        self.last_pos = self.offset();
                        TokenKind::BinaryOp(BinaryOp::Div)
                    }
                },
                '%' => {
                    self.last_pos = self.offset();
                    match self.peek() {
                        Some('=') => {
                            self.chars.next();
                            TokenKind::AssignOp(AssignOp::ModAssign)
                        }
                        _ => TokenKind::BinaryOp(BinaryOp::Mod),
                    }
                }
                '=' => {
                    self.last_pos = self.offset();
                    match self.cur() {
                        Some('>') => {
                            self.chars.next();
                            self.last_pos = self.offset();
                            TokenKind::Arrow
                        }
                        _ => match self.peek() {
                            Some('=') => {
                                self.chars.next();
                                self.last_pos = self.offset();
                                match self.peek() {
                                    Some('=') => {
                                        self.chars.next();
                                        self.last_pos = self.offset();
                                        TokenKind::BinaryOp(BinaryOp::Eq)
                                    }
                                    _ => TokenKind::BinaryOp(BinaryOp::EqEq),
                                }
                            }
                            _ => TokenKind::AssignOp(AssignOp::Assign),
                        },
                    }
                }
                '>' => {
                    self.last_pos = self.offset();
                    match self.cur() {
                        Some('=') => {
                            self.chars.next();
                            self.last_pos = self.offset();
                            TokenKind::BinaryOp(BinaryOp::Ge)
                        }
                        _ => match self.peek() {
                            Some('>') => {
                                self.chars.next();
                                self.last_pos = self.offset();
                                match self.peek() {
                                    Some('>') => {
                                        self.chars.next();
                                        self.last_pos = self.offset();
                                        match self.peek() {
                                            Some('=') => {
                                                self.chars.next();
                                                self.last_pos = self.offset();
                                                TokenKind::AssignOp(
                                                    AssignOp::ZeroFillRightShiftAssign,
                                                )
                                            }
                                            _ => TokenKind::BinaryOp(BinaryOp::ZeroFillRightShift),
                                        }
                                    }
                                    Some('=') => {
                                        self.chars.next();
                                        self.last_pos = self.offset();
                                        TokenKind::AssignOp(AssignOp::RightShiftAssign)
                                    }
                                    _ => TokenKind::BinaryOp(BinaryOp::RShift),
                                }
                            }
                            _ => TokenKind::BinaryOp(BinaryOp::Gt),
                        },
                    }
                }
                '<' => {
                    self.last_pos = self.offset();
                    match self.cur() {
                        Some('=') => {
                            self.chars.next();
                            self.last_pos = self.offset();
                            TokenKind::BinaryOp(BinaryOp::Le)
                        }
                        _ => match self.peek() {
                            Some('<') => {
                                self.chars.next();
                                self.last_pos = self.offset();
                                TokenKind::BinaryOp(BinaryOp::LShift)
                            }
                            _ => TokenKind::BinaryOp(BinaryOp::Lt),
                        },
                    }
                }
                '&' => {
                    self.last_pos = self.offset();
                    match self.cur() {
                        Some('&') => {
                            self.chars.next();
                            self.last_pos = self.offset();
                            TokenKind::BinaryOp(BinaryOp::LogicalAnd)
                        }
                        Some('=') => {
                            self.chars.next();
                            self.last_pos = self.offset();
                            TokenKind::AssignOp(AssignOp::BitAndAssign)
                        }
                        _ => TokenKind::BinaryOp(BinaryOp::BitAnd),
                    }
                }
                '|' => {
                    self.last_pos = self.offset();
                    match self.cur() {
                        Some('|') => {
                            self.chars.next();
                            self.last_pos = self.offset();
                            TokenKind::BinaryOp(BinaryOp::LogicalOr)
                        }
                        Some('=') => {
                            self.chars.next();
                            self.last_pos = self.offset();
                            TokenKind::AssignOp(AssignOp::BitOrAssign)
                        }
                        _ => TokenKind::BinaryOp(BinaryOp::BitOr),
                    }
                }
                '^' => {
                    self.last_pos = self.offset();
                    match self.peek() {
                        Some('=') => {
                            self.chars.next();
                            self.last_pos = self.offset();
                            TokenKind::AssignOp(AssignOp::BitXorAssign)
                        }
                        _ => TokenKind::BinaryOp(BinaryOp::BitXor),
                    }
                }
                '~' => {
                    self.last_pos = self.offset();
                    TokenKind::Tilde
                }
                '0'..='9' => {
                    if c == '0' {
                        let value = match self.cur() {
                            Some('b') | Some('B') => self.read_binary_number(),
                            Some('o') | Some('O') => self.read_octal_number(),
                            Some('x') | Some('X') => self.read_hex_number(),
                            Some(c) if c.is_whitespace() => 0.0,
                            Some(c) if ('8'..='9').contains(&c) => {
                                let start = self.offset();
                                self.chars.next();
                                let value = self.read_number(c);
                                self.errors.push(
                                    LegacyDecimalEscape(Span {
                                        start: start - 1,
                                        end: self.offset(),
                                    })
                                    .into(),
                                );
                                value
                            }
                            Some(c) if ('0'..='7').contains(&c) => {
                                let start = self.offset();
                                let value = self.read_octal_number();
                                self.errors.push(
                                    LegacyOctalLiteral(Span {
                                        start: start - 1,
                                        end: self.offset(),
                                    })
                                    .into(),
                                );
                                value
                            }
                            Some('.') => match self.peek() {
                                Some(c) if c.is_ascii_digit() => self.read_number('0'),
                                _ => {
                                    self.errors.push(
                                        InvalidOrUnexpectedToken(
                                            c,
                                            Span {
                                                start: self.last_pos,
                                                end: self.offset(),
                                            },
                                        )
                                        .into(),
                                    );
                                    0.0
                                }
                            },
                            _ => {
                                self.errors.push(
                                    InvalidOrUnexpectedToken(
                                        c,
                                        Span {
                                            start: self.last_pos,
                                            end: self.offset(),
                                        },
                                    )
                                    .into(),
                                );
                                0.0
                            }
                        };
                        self.last_pos = self.offset();
                        TokenKind::Number { value }
                    } else {
                        let value = self.read_number(c);
                        self.last_pos = self.offset();
                        TokenKind::Number { value }
                    }
                }
                '.' => {
                    self.last_pos = self.offset();
                    match self.peek() {
                        Some(c) if c.is_ascii_digit() => {
                            let value = self.read_number('.');
                            TokenKind::Number { value }
                        }
                        _ => TokenKind::Dot,
                    }
                }
                c => {
                    if is_ident_start(c) {
                        self.read_identifier(c)
                    } else {
                        self.last_pos = self.offset();
                        panic!("{}", c)
                    }
                }
            },
            None => {
                self.last_pos = self.offset();
                TokenKind::Eof
            }
        }
    }

    fn read_next_token(&mut self) -> Token {
        self.skip_whitespace();
        let start = self.offset();
        let kind = self.read_next_kind();
        let end = self.last_pos;
        Token {
            kind,
            span: Span { start, end },
        }
    }

    fn offset(&self) -> usize {
        // treat as unicode, not utf-8
        self.source.chars().count() - self.chars.clone().count()
    }

    fn read_identifier(&mut self, head: char) -> TokenKind {
        let mut ident = String::from(head);
        while let Some(c) = self.chars.next() {
            if is_ident_part(c) {
                ident.push(c);
                self.last_pos = self.offset();
            } else if c.is_whitespace() {
                break;
            } else {
                self.errors.push(
                    UnexpectedToken(
                        c,
                        Span {
                            start: self.last_pos,
                            end: self.offset(),
                        },
                    )
                    .into(),
                );
            }
        }

        match ident.as_str() {
            "break" => TokenKind::Word(WordKind::Keyword(Keyword::Break)),
            "case" => TokenKind::Word(WordKind::Keyword(Keyword::Case)),
            "catch" => TokenKind::Word(WordKind::Keyword(Keyword::Catch)),
            "class" => TokenKind::Word(WordKind::Keyword(Keyword::Class)),
            "continue" => TokenKind::Word(WordKind::Keyword(Keyword::Continue)),
            "const" => TokenKind::Word(WordKind::Keyword(Keyword::Const)),
            "debugger" => TokenKind::Word(WordKind::Keyword(Keyword::Debugger)),
            "default" => TokenKind::Word(WordKind::Keyword(Keyword::Default)),
            "delete" => TokenKind::Word(WordKind::Keyword(Keyword::Delete)),
            "do" => TokenKind::Word(WordKind::Keyword(Keyword::Do)),
            "else" => TokenKind::Word(WordKind::Keyword(Keyword::Else)),
            "export" => TokenKind::Word(WordKind::Keyword(Keyword::Export)),
            "extends" => TokenKind::Word(WordKind::Keyword(Keyword::Extends)),
            "finally" => TokenKind::Word(WordKind::Keyword(Keyword::Finally)),
            "for" => TokenKind::Word(WordKind::Keyword(Keyword::For)),
            "function" => TokenKind::Word(WordKind::Keyword(Keyword::Function)),
            "if" => TokenKind::Word(WordKind::Keyword(Keyword::If)),
            "import" => TokenKind::Word(WordKind::Keyword(Keyword::Import)),
            "in" => TokenKind::Word(WordKind::Keyword(Keyword::In)),
            "instanceof" => TokenKind::Word(WordKind::Keyword(Keyword::Instanceof)),
            "new" => TokenKind::Word(WordKind::Keyword(Keyword::New)),
            "return" => TokenKind::Word(WordKind::Keyword(Keyword::Return)),
            "let" => TokenKind::Word(WordKind::Keyword(Keyword::Let)),
            "super" => TokenKind::Word(WordKind::Keyword(Keyword::Super)),
            "switch" => TokenKind::Word(WordKind::Keyword(Keyword::Switch)),
            "this" => TokenKind::Word(WordKind::Keyword(Keyword::This)),
            "throw" => TokenKind::Word(WordKind::Keyword(Keyword::Throw)),
            "try" => TokenKind::Word(WordKind::Keyword(Keyword::Try)),
            "typeof" => TokenKind::Word(WordKind::Keyword(Keyword::Typeof)),
            "var" => TokenKind::Word(WordKind::Keyword(Keyword::Var)),
            "void" => TokenKind::Word(WordKind::Keyword(Keyword::Void)),
            "while" => TokenKind::Word(WordKind::Keyword(Keyword::While)),
            "with" => TokenKind::Word(WordKind::Keyword(Keyword::With)),
            "yield" => TokenKind::Word(WordKind::Keyword(Keyword::Yield)),
            _ => TokenKind::Word(WordKind::Identifier(ident)),
        }
    }

    fn read_number(&mut self, head: char) -> f64 {
        let mut number = String::from(head);
        if let Some(c) = self.cur() {
            if !c.is_ascii_digit() && c != 'e' && c != '.' {
                return number
                    .parse::<f64>()
                    .expect("failed to parse number as f64");
            }
        }
        while let Some(c) = self.chars.next() {
            if c.is_ascii_digit() {
                number.push(c);
                self.last_pos = self.offset();
            } else if c.is_whitespace() {
                break;
            } else if c == '.' {
                number.push(c);
                self.last_pos = self.offset();
                match self.peek() {
                    Some(c) => {
                        if !c.is_ascii_digit() && c != 'e' && c != 'E' {
                            self.errors.push(
                                InvalidOrUnexpectedToken(
                                    c,
                                    Span {
                                        start: self.last_pos,
                                        end: self.offset(),
                                    },
                                )
                                .into(),
                            );
                        }
                    }
                    _ => {
                        self.errors.push(
                            InvalidOrUnexpectedToken(
                                c,
                                Span {
                                    start: self.last_pos,
                                    end: self.offset(),
                                },
                            )
                            .into(),
                        );
                    }
                };
            } else if c == 'e' || c == 'E' {
                number.push(c);
                self.last_pos = self.offset();
                match self.chars.next() {
                    Some(c) if matches!(c, '+') | matches!(c, '-') => {
                        number.push(c);
                        self.last_pos = self.offset();
                        match self.peek() {
                            Some(c) if !c.is_ascii_digit() => {
                                self.errors.push(
                                    InvalidOrUnexpectedToken(
                                        c,
                                        Span {
                                            start: self.last_pos,
                                            end: self.offset(),
                                        },
                                    )
                                    .into(),
                                );
                            }
                            None | Some(_) => {}
                        }
                    }
                    Some(c) if c.is_ascii_digit() => {
                        number.push(c);
                        self.last_pos = self.offset();
                    }
                    _ => {
                        self.errors.push(
                            InvalidOrUnexpectedToken(
                                c,
                                Span {
                                    start: self.last_pos,
                                    end: self.offset(),
                                },
                            )
                            .into(),
                        );
                    }
                };
            } else {
                self.errors.push(
                    InvalidOrUnexpectedToken(
                        c,
                        Span {
                            start: self.last_pos,
                            end: self.offset(),
                        },
                    )
                    .into(),
                );
            }
        }
        number
            .parse::<f64>()
            .expect("failed to parse number as f64")
    }

    fn read_binary_number(&mut self) -> f64 {
        self.chars.next();
        let mut number = String::new();
        while let Some(c) = self.chars.next() {
            if c.is_ascii_digit() {
                number.push(c);
                self.last_pos = self.offset();
            } else if c.is_whitespace() {
                break;
            } else if c == '.' {
                self.errors.push(
                    UnexpectedNumber(
                        c,
                        Span {
                            start: self.last_pos,
                            end: self.offset(),
                        },
                    )
                    .into(),
                )
            } else {
                self.errors.push(
                    InvalidOrUnexpectedToken(
                        c,
                        Span {
                            start: self.last_pos,
                            end: self.offset(),
                        },
                    )
                    .into(),
                );
            }
        }
        isize::from_str_radix(&number, 2).expect("failed to parse number as binary") as f64
    }

    fn read_octal_number(&mut self) -> f64 {
        self.chars.next();
        let mut number = String::new();
        while let Some(c) = self.chars.next() {
            if c.is_ascii_digit() && ('0'..'7').contains(&c) {
                number.push(c);
                self.last_pos = self.offset();
            } else if c.is_whitespace() {
                break;
            } else if c == '.' {
                self.errors.push(
                    UnexpectedNumber(
                        c,
                        Span {
                            start: self.last_pos,
                            end: self.offset(),
                        },
                    )
                    .into(),
                )
            } else {
                self.errors.push(
                    InvalidOrUnexpectedToken(
                        c,
                        Span {
                            start: self.last_pos,
                            end: self.offset(),
                        },
                    )
                    .into(),
                );
            }
        }
        isize::from_str_radix(&number, 8).expect("failed to parse number as octal") as f64
    }

    fn read_hex_number(&mut self) -> f64 {
        self.chars.next();
        let mut number = String::new();
        while let Some(c) = self.chars.next() {
            if c.is_ascii_digit() || ('a'..='e').contains(&c) || ('A'..='E').contains(&c) {
                number.push(c);
                self.last_pos = self.offset();
            } else if c.is_whitespace() {
                break;
            } else if c == '.' {
                self.errors.push(
                    UnexpectedNumber(
                        c,
                        Span {
                            start: self.last_pos,
                            end: self.offset(),
                        },
                    )
                    .into(),
                )
            } else {
                self.errors.push(
                    InvalidOrUnexpectedToken(
                        c,
                        Span {
                            start: self.last_pos,
                            end: self.offset(),
                        },
                    )
                    .into(),
                );
            }
        }
        isize::from_str_radix(&number, 16).expect("failed to parse number as hex") as f64
    }

    fn read_string_literal(&mut self, start_quote: char) -> (String, String) {
        let mut string = String::new();
        let mut raw = String::from(start_quote);
        while let Some(c) = self.chars.next() {
            if c == '\\' {
                if let Some(next_char) = self.chars.next() {
                    raw.push(c);
                    match next_char {
                        'n' => {
                            string.push('\n');
                            raw.push('n')
                        }
                        'r' => {
                            string.push('\r');
                            raw.push('r')
                        }
                        't' => {
                            string.push('\t');
                            raw.push('t')
                        }
                        '\\' => {
                            string.push('\\');
                            raw.push('\\')
                        }
                        '\'' => {
                            string.push('\'');
                            raw.push('\'');
                        }
                        '"' => {
                            string.push('"');
                            raw.push('"');
                        }
                        _ => {
                            string.push(next_char);
                            raw.push(next_char);
                        }
                    }
                }
            } else if c == start_quote {
                raw.push(c);
                break;
            } else {
                string.push(c);
                raw.push(c);
            }
            self.last_pos = self.offset();
        }
        (string, raw)
    }
}

// FIXME: support unicode like 'let ユニコード = 10'
fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_lowercase() || ch.is_ascii_uppercase() || ch == '$' || ch == '_'
}

fn is_ident_part(ch: char) -> bool {
    ch.is_ascii_lowercase()
        || ch.is_ascii_uppercase()
        || ch == '$'
        || ch == '_'
        || ch.is_ascii_digit()
        || ch == '\u{200c}'
}

fn is_line_terminator(ch: char) -> bool {
    ch == '\n' || ch == '\r' || ch == '\u{2028}' || ch == '\u{2029}'
}

pub fn lex(source: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(source);
    let mut tokens = vec![];
    loop {
        let token = lexer.read_next_token();
        if token.kind == TokenKind::Eof {
            break;
        }
        tokens.push(token);
    }
    tokens
}

pub fn lex_error(source: &str) -> Vec<Error> {
    let mut lexer = Lexer::new(source);
    let mut tokens = vec![];
    loop {
        let token = lexer.read_next_token();
        if token.kind == TokenKind::Eof {
            break;
        }
        tokens.push(token);
    }
    lexer.errors
}

pub fn run_lexer(source: &str) -> Result<Vec<Token>, Vec<miette::Report>> {
    let mut l = Lexer::new(source);
    let mut tokens = vec![];
    loop {
        let token = l.read_next_token();
        if token.kind == TokenKind::Eof {
            break;
        }
        tokens.push(token)
    }
    if l.errors.is_empty() {
        Ok(tokens)
    } else {
        Err(l.errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_add() {
        assert_eq!(
            lex("1+2"),
            vec![
                Token {
                    span: Span { start: 0, end: 1 },
                    kind: TokenKind::Number { value: 1_f64 },
                },
                Token {
                    span: Span { start: 1, end: 2 },
                    kind: TokenKind::BinaryOp(BinaryOp::Add),
                },
                Token {
                    span: Span { start: 2, end: 3 },
                    kind: TokenKind::Number { value: 2_f64 },
                },
            ]
        );
    }
    #[test]
    fn variable_decl() {
        assert_eq!(
            lex("const foo = 1 + 1"),
            vec![
                Token {
                    span: Span { start: 0, end: 5 },
                    kind: TokenKind::Word(WordKind::Keyword(Keyword::Const)),
                },
                Token {
                    span: Span { start: 6, end: 9 },
                    kind: TokenKind::Word(WordKind::Identifier("foo".to_string())),
                },
                Token {
                    span: Span { start: 10, end: 11 },
                    kind: TokenKind::AssignOp(AssignOp::Assign),
                },
                Token {
                    span: Span { start: 12, end: 13 },
                    kind: TokenKind::Number { value: 1_f64 },
                },
                Token {
                    span: Span { start: 14, end: 15 },
                    kind: TokenKind::BinaryOp(BinaryOp::Add)
                },
                Token {
                    span: Span { start: 16, end: 17 },
                    kind: TokenKind::Number { value: 1_f64 },
                },
            ]
        );
    }

    #[test]
    fn sigleline_commnet() {
        assert_eq!(
            lex("// this is single line comment
const foo = 1 + 1
"),
            vec![
                Token {
                    span: Span { start: 0, end: 31 },
                    kind: TokenKind::SingleLineComment,
                },
                Token {
                    span: Span { start: 31, end: 36 },
                    kind: TokenKind::Word(WordKind::Keyword(Keyword::Const)),
                },
                Token {
                    span: Span { start: 37, end: 40 },
                    kind: TokenKind::Word(WordKind::Identifier("foo".to_string())),
                },
                Token {
                    span: Span { start: 41, end: 42 },
                    kind: TokenKind::AssignOp(AssignOp::Assign),
                },
                Token {
                    span: Span { start: 43, end: 44 },
                    kind: TokenKind::Number { value: 1_f64 },
                },
                Token {
                    span: Span { start: 45, end: 46 },
                    kind: TokenKind::BinaryOp(BinaryOp::Add),
                },
                Token {
                    span: Span { start: 47, end: 48 },
                    kind: TokenKind::Number { value: 1_f64 },
                }
            ]
        );
    }

    #[test]
    fn multiline_commnet() {
        assert_eq!(
            lex("/* This is
multiline
comment
*/"),
            vec![Token {
                span: Span { start: 0, end: 31 },
                kind: TokenKind::MultiLineComment,
            }]
        );
    }

    #[test]
    fn identifier_names() {
        assert_eq!(
            lex("myVariable"),
            vec![Token {
                span: Span { start: 0, end: 10 },
                kind: TokenKind::Word(WordKind::Identifier("myVariable".to_string())),
            }]
        );
        assert_eq!(
            lex("_myVariable"),
            vec![Token {
                span: Span { start: 0, end: 11 },
                kind: TokenKind::Word(WordKind::Identifier("_myVariable".to_string())),
            }]
        );
        assert_eq!(
            lex("$myVariable"),
            vec![Token {
                span: Span { start: 0, end: 11 },
                kind: TokenKind::Word(WordKind::Identifier("$myVariable".to_string())),
            }]
        );
        assert_eq!(
            lex("\u{006D}yVariable"),
            vec![Token {
                span: Span { start: 0, end: 10 },
                kind: TokenKind::Word(WordKind::Identifier("myVariable".to_string())),
            }]
        );
        // FIXME: `unicode-xid` crate is not resolved
        // assert_eq!(
        //     lex("Åaaaaaaaaa"),
        //     vec![Token {
        //         span: Span { start: 0, end: 11 },
        //         kind: TokenKind::Word(WordKind::Identifier("Åaaaaaaaaa".to_string())),
        //     }]
        // );

        assert_eq!(
            lex("my$Variable"),
            vec![Token {
                span: Span { start: 0, end: 11 },
                kind: TokenKind::Word(WordKind::Identifier("my$Variable".to_string())),
            },]
        );
        assert_eq!(
            lex("my_variable"),
            vec![Token {
                span: Span { start: 0, end: 11 },
                kind: TokenKind::Word(WordKind::Identifier("my_variable".to_string())),
            },]
        );
        assert_eq!(
            lex("my\u{0056}ariable"),
            vec![Token {
                span: Span { start: 0, end: 10 },
                kind: TokenKind::Word(WordKind::Identifier("my\u{0056}ariable".to_string())),
            }]
        );
        assert_eq!(
            lex("myVariable\u{200C}"),
            vec![Token {
                span: Span { start: 0, end: 11 },
                kind: TokenKind::Word(WordKind::Identifier("myVariable\u{200C}".to_string())),
            }]
        );

        // error cases
        // 123Var
        // var //reserved
        // my-Variable // - is not allowed
        assert_eq!(lex_error("my-Variable").len(), 1);
    }

    #[test]
    fn reserved_keyword() {
        assert_eq!(
            lex("var"),
            vec![Token {
                span: Span { start: 0, end: 3 },
                kind: TokenKind::Word(WordKind::Keyword(Keyword::Var)),
            }]
        );
        assert_eq!(
            lex("if"),
            vec![Token {
                span: Span { start: 0, end: 2 },
                kind: TokenKind::Word(WordKind::Keyword(Keyword::If)),
            }]
        );
        assert_eq!(
            lex("else"),
            vec![Token {
                span: Span { start: 0, end: 4 },
                kind: TokenKind::Word(WordKind::Keyword(Keyword::Else)),
            }]
        );
    }

    #[test]
    fn numeric_literals() {
        assert_eq!(
            lex("0"),
            vec![Token {
                span: Span { start: 0, end: 1 },
                kind: TokenKind::Number { value: 0_f64 },
            }]
        );
        assert_eq!(
            lex("123"),
            vec![Token {
                span: Span { start: 0, end: 3 },
                kind: TokenKind::Number { value: 123_f64 },
            }]
        );

        // DecimalIntegerLiteral ExponentPart
        assert_eq!(
            lex("124e4"),
            vec![Token {
                span: Span { start: 0, end: 5 },
                kind: TokenKind::Number { value: 124e4_f64 },
            }]
        );

        // DecimalIntegerLiteral . DecimalDigitsopt ExponentPartopt
        assert_eq!(
            lex("125.456"),
            vec![Token {
                span: Span { start: 0, end: 7 },
                kind: TokenKind::Number { value: 125.456_f64 },
            }]
        );
        assert_eq!(
            lex("127e-4"),
            vec![Token {
                span: Span { start: 0, end: 6 },
                kind: TokenKind::Number { value: 127e-4_f64 },
            }]
        );
        assert_eq!(
            lex("128e+4"),
            vec![Token {
                span: Span { start: 0, end: 6 },
                kind: TokenKind::Number { value: 128e+4_f64 },
            }]
        );

        // // // DecimalDigits ExponentPartopt
        assert_eq!(
            lex(".456"),
            vec![Token {
                span: Span { start: 0, end: 4 },
                kind: TokenKind::Number { value: 0.456_f64 },
            }]
        );

        // BinaryIntegerLiteral
        assert_eq!(
            lex("0b1010"),
            vec![Token {
                span: Span { start: 0, end: 6 },
                kind: TokenKind::Number { value: 10_f64 },
            }]
        );

        // OctalIntegerLiteral
        assert_eq!(
            lex("0o123"),
            vec![Token {
                span: Span { start: 0, end: 5 },
                kind: TokenKind::Number { value: 83_f64 },
            }]
        );

        // HexIntegerLiteral
        assert_eq!(
            lex("0x123"),
            vec![Token {
                span: Span { start: 0, end: 5 },
                kind: TokenKind::Number { value: 291_f64 },
            }]
        );
    }

    #[test]
    fn string_literals() {
        assert_eq!(
            lex("let str1 = 'This is a simple string.'"),
            vec![
                Token {
                    span: Span { start: 0, end: 3 },
                    kind: TokenKind::Word(WordKind::Keyword(Keyword::Let)),
                },
                Token {
                    span: Span { start: 4, end: 8 },
                    kind: TokenKind::Word(WordKind::Identifier("str1".to_string())),
                },
                Token {
                    span: Span { start: 9, end: 10 },
                    kind: TokenKind::AssignOp(AssignOp::Assign),
                },
                Token {
                    span: Span { start: 11, end: 37 },
                    kind: TokenKind::String {
                        value: "This is a simple string.".to_string(),
                        raw: "'This is a simple string.'".to_string()
                    },
                }
            ]
        );

        assert_eq!(
            lex(r#"let str2 = "This is a simple string.""#),
            vec![
                Token {
                    span: Span { start: 0, end: 3 },
                    kind: TokenKind::Word(WordKind::Keyword(Keyword::Let)),
                },
                Token {
                    span: Span { start: 4, end: 8 },
                    kind: TokenKind::Word(WordKind::Identifier("str2".to_string())),
                },
                Token {
                    span: Span { start: 9, end: 10 },
                    kind: TokenKind::AssignOp(AssignOp::Assign),
                },
                Token {
                    span: Span { start: 11, end: 37 },
                    kind: TokenKind::String {
                        value: "This is a simple string.".to_string(),
                        raw: "\"This is a simple string.\"".to_string()
                    },
                }
            ]
        )
    }

    #[test]
    fn escape_string_literal() {
        assert_eq!(
            lex(r"'say \'Hello\''"),
            vec![Token {
                span: Span { start: 0, end: 15 },
                kind: TokenKind::String {
                    value: "say 'Hello'".to_string(),
                    raw: r"'say \'Hello\''".to_string()
                },
            }]
        );
        assert_eq!(
            lex(r#"'say \"Hello\"'"#),
            vec![Token {
                span: Span { start: 0, end: 15 },
                kind: TokenKind::String {
                    value: r#"say "Hello""#.to_string(),
                    raw: r#"'say \"Hello\"'"#.to_string()
                },
            }]
        );
        assert_eq!(
            lex(r#""Line1\nLine2""#),
            vec![Token {
                span: Span { start: 0, end: 14 },
                kind: TokenKind::String {
                    value: "Line1\nLine2".to_string(),
                    raw: r#""Line1\nLine2""#.to_string()
                },
            }]
        );
    }

    #[test]
    fn ex() {
        assert_eq!(
            lex("/="),
            vec![Token {
                span: Span { start: 0, end: 2 },
                kind: TokenKind::AssignOp(AssignOp::DivAssign),
            }]
        );
    }

    // #[test]
    // fn template_literal() {
    //     assert_eq!(
    //         lex("`Hello World!`"),
    //         vec![
    //             Token {
    //                 kind: TokenKind::Backquote,
    //                 span: Span { start: 0, end: 1 },
    //             },
    //             Token {
    //                 kind: TokenKind::Word(WordKind::Identifier("Hello World!".to_string())),
    //                 span: Span { start: 1, end: 14 },
    //             },
    //             Token {
    //                 kind: TokenKind::Backquote,
    //                 span: Span { start: 14, end: 15 },
    //             },
    //         ]
    //     );
    //     assert_eq!(
    //         lex(r"`Hello ${name}!`"),
    //         vec![Token {
    //             span: Span { start: 0, end: 17 },
    //             kind: TokenKind::String {
    //                 value: r"Hello, ${name}!".to_string(),
    //                 raw: r"Hello, ${name}!".to_string()
    //             },
    //         }]
    //     );
    // }
}
