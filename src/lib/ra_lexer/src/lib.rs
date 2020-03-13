mod cursor;
use cursor::{Cursor, Position, is_end_of_line, is_whitespace, EOF_CHAR};
mod errors;
use errors::LexerError;



#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    SemiColon,
    Coma,
    Dot,
    Colon,
    Greater,
    GreaterOrEquals,
    Less,
    LessOrEequals,
    Equals,
    Plus,
    Minus,
    Exclamation,
    Slash,
    Comment,
    ForwardSlash,
    Ampersand,
    Pipe,
    OpenCurlyBrace,
    OpenSquareBrace,
    OpenParentheses,
    CloseCurlyBrace,
    CloseSquareBrace,
    CloseParentheses,
    At,
    HashPound,
    Asterisk,
    Percent,
    Underscore,
    Dollar,
    Power,
    Tilda,
    Question,
    Identifier,
    StringLiteral,
    Number(char, char),
    ContentBlock(Vec<Token>),
    Undetermined
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    kind: TokenKind,
    len: usize,
    content: String,
    position: (Position, Position),
    level: usize
}

impl Default for Token {
    fn default() -> Self {
        Self {
            len: 1,
            content: String::new(),
            position: (Position(0, 0), Position(0, 1)),
            kind: TokenKind::Undetermined,
            level: 0
        }
    }
}

pub fn tokenize(input: &str) -> impl Iterator<Item = Token> + '_ {
    let mut cursor = Cursor::new(input, Position(1, 0), 0);

    std::iter::from_fn(move || {
        if input.is_empty() {
            return None;
        }
        debug_assert!(!input.is_empty());
        let token = cursor.advance_token();
        Some(token)
    })
}


impl Cursor<'_> {
    fn advance_token(&mut self) -> Token {
        let mut start_position = self.position.clone();
        let first_char = self.bump().unwrap();
        
        if self.position.0 > start_position.0 {
            start_position = Position(self.position.0, self.position.1 - 1); // adjust position for after line change
        }

        match self.static_token(first_char, start_position) {
            Some(t) => t,
            None => {
                match first_char {
                    c if c.is_alphabetic() => self.identifier(c, start_position),
                    c if c.is_numeric() => self.number(c, start_position),
                    c if c == '\'' => self.string_literal(c, start_position),
                    c if c == '"' => self.string_literal(c, start_position),
                    _ => panic!(LexerError::UnexpectedCharacter)
                }
            }
        }
    }

    fn static_token(&mut self, c: char, start_position: Position) -> Option<Token> {
        let initial_len = self.len_consumed();

        let generate_token = |kind: TokenKind| -> Token  {
            Token {
                kind, 
                position: (start_position, self.position.clone()),
                level: self.level.clone(),
                len: self.len_consumed() - initial_len + 1,
                ..Default::default()
            }
        };

        let tok = match c {
            '/' => match self.first_ahead() {
                '/' => {
                    self.single_line_comment(start_position.clone())
                },
                '*' => {
                    self.multi_line_comment(start_position.clone())
                },
                _ => {
                    generate_token(TokenKind::Slash)
                },
            },
            '>' => match self.first_ahead() {
                '=' => {
                    self.two_characters_token(c, self.position.clone())
                },
                _ => generate_token(TokenKind::Greater)
            },
            '<' => match self.first_ahead() {
                '=' => {
                    self.two_characters_token(c, self.position.clone())
                },
                _ => generate_token(TokenKind::Less)
            },
            '`' => self.content_block(start_position.clone()),
            '!' => generate_token(TokenKind::Exclamation),
            '?' => generate_token(TokenKind::Question),
            '{' => generate_token(TokenKind::OpenCurlyBrace),
            '[' => generate_token(TokenKind::OpenSquareBrace),
            '(' => generate_token(TokenKind::OpenParentheses),
            '}' => generate_token(TokenKind::CloseCurlyBrace),
            ']' => generate_token(TokenKind::CloseSquareBrace),
            ')' => generate_token(TokenKind::CloseParentheses),
            ':' => generate_token(TokenKind::Colon),
            ',' => generate_token(TokenKind::Coma),
            '.' => generate_token(TokenKind::Dot),
            '+' => generate_token(TokenKind::Plus),
            '-' => generate_token(TokenKind::Minus),
            '=' => generate_token(TokenKind::Equals),
            ';' => generate_token(TokenKind::SemiColon),
            '&' => generate_token(TokenKind::Ampersand),
            '#' => generate_token(TokenKind::HashPound),
            '@' => generate_token(TokenKind::At),
            '\\' => generate_token(TokenKind::ForwardSlash),
            '|' => generate_token(TokenKind::Pipe),
            '_' => generate_token(TokenKind::Underscore),
            '%' => generate_token(TokenKind::Percent),
            '$' => generate_token(TokenKind::Dollar),
            '^' => generate_token(TokenKind::Power),
            '~' => generate_token(TokenKind::Tilda),
            _ => return None
        };

        Some(tok)
    }

    fn two_characters_token(&mut self, first_character: char, start_position: Position) -> Token {
        match first_character {
            '>' => {
                match self.bump().unwrap() {
                    '=' => Token{kind: TokenKind::GreaterOrEquals, len: 2, position: (start_position, self.position.clone()), ..Default::default()},
                    _ => panic!(LexerError::UnexpectedCharacter)
                }
            },
            '<' => {
                match self.bump().unwrap() {
                    '=' => Token{kind: TokenKind::LessOrEequals, len: 2, position: (start_position, self.position.clone()), ..Default::default()},
                    _ => panic!(LexerError::UnexpectedCharacter)
                }
            },
            _ => panic!(LexerError::UnexpectedCharacter)
        }
    }

    fn string_literal(&mut self, opening_quote: char, start_position: Position) -> Token {
        let mut string_literal = Token {
            kind: TokenKind::StringLiteral,
            ..Default::default()
        };

        let initial_len = self.len_consumed() - 1;

        loop {
            if self.position.0 > start_position.0 {
                panic!(LexerError::UnexpectedEndOfLine)   
            }

            match self.bump() {
                Some(c) => {
                    match c {
                        ch if ch == opening_quote => break,
                        ch => string_literal.content.push(ch)
                    }
                }
                None => panic!(LexerError::UnexpectedEndOfInput)
            }
        }

        string_literal.level = self.level.clone();
        string_literal.position = (start_position, self.position.clone());
        string_literal.len = self.len_consumed() - initial_len;

        string_literal
    }

    fn number(&mut self, c: char, start_position: Position) -> Token {
        let mut number = Token {
            kind: TokenKind::Undetermined,
            content: c.to_string(),
            ..Default::default()
        };

        let mut first_separator: TokenKind = TokenKind::Undetermined;
        let mut second_separator: TokenKind = TokenKind::Undetermined;
        let start_consumed = self.len_consumed() - 1; // add 1 for first token

        loop {
            let next_character = self.first_ahead();
            if is_end_of_line(next_character) || next_character == EOF_CHAR {
                break;
            }
            else {
                match self.static_token(next_character, start_position) {
                    Some(t) => {
                        match t.kind {
                            TokenKind::Dot | TokenKind::Coma => {
                                if first_separator == TokenKind::Undetermined{
                                    first_separator = t.kind;
                                }
                                else if first_separator != t.kind && second_separator == TokenKind::Undetermined {
                                    second_separator = t.kind;
                                }
                                else if second_separator == t.kind && first_separator != TokenKind::Undetermined {
                                    panic!(LexerError::UnexpectedCharacter(next_character))
                                }
                                number.content.push(self.bump().unwrap())
                            }
                            _ => break
                        }
                    },
                    None => {
                        match next_character {
                            c if c.is_numeric() => number.content.push(self.bump().unwrap()),
                            c if is_whitespace(c) => {
                                self.bump();
                                break
                            },
                            _ => panic!(LexerError::UnexpectedCharacter(next_character))
                        }
                    }
                }
            }
        }

        number.kind = TokenKind::Number(
            match second_separator {
                TokenKind::Coma => ',',
                TokenKind::Dot => '.',
                _ => {
                    match first_separator {
                        TokenKind::Coma => '.',
                        TokenKind::Dot => ',',
                        _ => ','
                    }
                }
            },
            match first_separator {
                TokenKind::Coma => ',',
                TokenKind::Dot => '.',
                _ => '.'
            }
        );

        number.len = self.len_consumed() - start_consumed;
        number.position = (start_position, self.position.clone());
        number.level = self.level.clone();

        number
    }

    fn identifier(&mut self, first_char: char, start_position: Position) -> Token {
        let mut identifier = Token {
            kind: TokenKind::Identifier,
            content: first_char.to_string(),
            ..Default::default()
        };
        let start_consumed = self.len_consumed() - 1; // add 1 for first token

        loop {
            let next_character = self.first_ahead();
            if is_end_of_line(next_character) || next_character == EOF_CHAR {
                break;
            }
            else {
                match self.static_token(next_character, start_position) {
                    Some(_) => break,
                    None => identifier.content.push(self.bump().unwrap())
                }
            }
        }

        identifier.position = (start_position, self.position.clone());
        identifier.len = self.len_consumed() - start_consumed;
        identifier.level = self.level.clone();

        identifier
    }

    fn content_block(&mut self, start_position: Position) -> Token {
        let mut content_block = Token{
            kind: TokenKind::ContentBlock(Default::default()),
            ..Default::default()
        };
        let initial_level = self.level;
        let initial_line = self.position.0;
        let mut buffer = String::new();
        let start_consumed = self.len_consumed() - 1; // add 1 for first token
        let mut block_closed = false;
        while let Some(ch) = self.bump() {
            if self.level < initial_level {
                panic!(LexerError::UnexepectedIndentLevel);
            }
            
            match ch {
                c if self.level == initial_level && c == '`' => {
                    block_closed = true;
                    break
                },
                c if self.level > initial_level => {
                    content_block.content.push(c);
                    content_block.len += 1;
                },
                c => buffer.push(c)
            }
        }

        if !block_closed {
            panic!(LexerError::UnexpectedEndOfInput);
        }

        if initial_line == self.position.0 {
            content_block.content.push_str(&buffer);
            content_block.len = buffer.len();
        }
        else {
            let mut buffer_cursor = Cursor::new(&buffer, Position(start_position.0, start_position.1 + 1), initial_level);
            let mut content_block_initialization_tokens: Vec<Token> = Vec::new();
            while !buffer_cursor.is_eof() {
                content_block_initialization_tokens.push(buffer_cursor.advance_token());
            }

            content_block.kind = TokenKind::ContentBlock(content_block_initialization_tokens);
        }

        content_block.position = (start_position, self.position.clone());
        content_block.len = self.len_consumed() - start_consumed;
        content_block.level = self.level.clone();

        content_block
    }

    fn single_line_comment (&mut self, start_position: Position) -> Token {
        self.bump();
        let mut comment = Token{
            kind: TokenKind::Comment,
            ..Default::default()
        };

        let start_consumed = self.len_consumed();

        while let Some(ch) = self.bump() {
            if self.position.0 == start_position.0 {
                comment.len += 1;
                comment.content.push(ch);
            }
            else {
                break;
            }
        };

        comment.position = (start_position, self.position.clone());
        comment.len = self.len_consumed() - start_consumed + 2; // add 2 for "//"
        comment.level = self.level.clone();
        
        comment
    }

    fn multi_line_comment (&mut self, start_position: Position) -> Token {
        self.bump();
        let mut comment = Token{
            kind: TokenKind::Comment,
            ..Default::default()
        };

        let start_consumed = self.len_consumed();

        let mut keep_lines = self.position.0;

        while let Some(ch) = self.bump() {
            if keep_lines < self.position.0 {
                keep_lines = self.position.0;
                comment.content.push('\n');
            }
            match ch {
                '*' => match self.first_ahead() {
                    '/' => {
                        self.bump();
                        break;
                    },
                    _ => {
                        comment.content.push(self.bump().unwrap())
                    }
                },
                c => {
                    comment.content.push(c)
                }
            }
        };

        comment.position = (start_position, self.position.clone());
        comment.len = self.len_consumed() - start_consumed + 2; // add 2 for "/*"
        comment.level = self.level.clone();

        comment
    }
}

#[cfg(test)]
mod tests {
    use super::{tokenize, Token, TokenKind, Position};
    // use cursor::{};

    #[test]
    fn it_should_create_iterator_of_tokens() {
        let mut stream = tokenize("abc");
        stream.next();
    }

    #[test]
    fn it_should_parse_single_line_comments() {
        let mut stream = tokenize("//abc");
        assert_eq!(stream.next().unwrap(), Token{
            kind: TokenKind::Comment,
            content: String::from("abc"), 
            position: (Position(1, 0), Position(1, 5)),
            len: 5,
            level: 0
        });
    }

    #[test]
    fn it_should_parse_multi_line_comments() {
        let mut stream = tokenize("/*abc\nbca*/");
        assert_eq!(stream.next().unwrap(), Token{
            kind: TokenKind::Comment,
            content: String::from("abc\nbca"), 
            position: (Position(1, 0), Position(2, 5)),
            len: 11,
            level: 0
        });
    }

    #[test]
    fn it_should_parse_identifiers() {
        let mut stream = tokenize("abc");
        assert_eq!(stream.next().unwrap(), Token{
            kind: TokenKind::Identifier,
            content: String::from("abc"), 
            position: (Position(1, 0), Position(1, 3)),
            len: 3,
            level: 0
        });
    }

    #[test]
    fn it_should_parse_numbers() {
        let mut stream = tokenize("123");
        assert_eq!(stream.next().unwrap(), Token{
            kind: TokenKind::Number(',', '.'),
            content: String::from("123"), 
            position: (Position(1, 0), Position(1, 3)),
            len: 3,
            level: 0
        });
    }

    #[test]
    fn it_should_parse_numbers_with_decimal_separator() {
        let mut stream = tokenize("123,321");
        assert_eq!(stream.next().unwrap(), Token{
            kind: TokenKind::Number('.', ','),
            content: String::from("123,321"), 
            position: (Position(1, 0), Position(1, 7)),
            len: 7,
            level: 0
        });
    }

    #[test]
    fn it_should_parse_numbers_with_another_decimal_separator() {
        let mut stream = tokenize("123.321");
        assert_eq!(stream.next().unwrap(), Token{
            kind: TokenKind::Number(',', '.'),
            content: String::from("123.321"), 
            position: (Position(1, 0), Position(1, 7)),
            len: 7,
            level: 0
        });
    }

    #[test]
    fn it_should_parse_numbers_with_decimal_and_thouthands_separator() {
        let mut stream = tokenize("123.321,456");
        assert_eq!(stream.next().unwrap(), Token{
            kind: TokenKind::Number(',', '.'),
            content: String::from("123.321,456"), 
            position: (Position(1, 0), Position(1, 11)),
            len: 11,
            level: 0
        });
    }

    #[test]
    fn it_should_parse_numbers_with_decimal_and_multiple_thouthands_separator() {
        let mut stream = tokenize("123.321.123,456");
        assert_eq!(stream.next().unwrap(), Token{
            kind: TokenKind::Number(',', '.'),
            content: String::from("123.321.123,456"), 
            position: (Position(1, 0), Position(1, 15)),
            len: 15,
            level: 0
        });
    }

    #[test]
    fn it_should_parse_numbers_with_another_decimal_and_multiple_thouthands_separator() {
        let mut stream = tokenize("123,321,123.456");
        assert_eq!(stream.next().unwrap(), Token{
            kind: TokenKind::Number('.', ','),
            content: String::from("123,321,123.456"), 
            position: (Position(1, 0), Position(1, 15)),
            len: 15,
            level: 0
        });
    }

    #[test]
    #[should_panic]
    fn it_should_paninc_when_encountering_multiple_decimal_separators() {
        let mut stream = tokenize("123.321.123,456,654");
        stream.next();
    }

    #[test]
    fn it_should_parse_content_blocks() {
        let mut stream = tokenize("`abc`");
        assert_eq!(stream.next().unwrap(), Token{
            kind: TokenKind::ContentBlock(vec![]),
            content: String::from("abc"), 
            position: (Position(1, 0), Position(1, 5)),
            len: 5,
            level: 0
        });
    }

    #[test]
    fn it_should_parse_content_blocks_with_initialization_tokens() {
        let mut stream = tokenize("`ln=en\n\tabc\n`");
        assert_eq!(stream.next().unwrap(), Token{
            kind: TokenKind::ContentBlock(vec![
                    Token{level: 0, kind: TokenKind::Identifier, content: String::from("ln"), position: (Position(1,1), Position(1,3)), len:2},
                    Token{level: 0 ,kind: TokenKind::Equals, content: String::new(), position: (Position(1,3), Position(1,4)), len:1},
                    Token{level: 0, kind: TokenKind::Identifier, content: String::from("en"), position: (Position(1,4), Position(1,6)), len:2},
                ]),
            content: String::from("abc"), 
            position: (Position(1, 0), Position(3, 1)),
            len: 13,
            level: 0
        });
    }

    #[test]
    #[should_panic]
    fn it_should_paninc_when_content_block_is_not_closed() {
        let mut stream = tokenize("`ln=en\n\tabc`");
        stream.next();
    }

    #[test]
    fn it_should_parse_tokens() {
        let mut stream = tokenize("!?&/");
        assert_eq!(stream.next().unwrap(), Token{
            kind: TokenKind::Exclamation,
            position: (Position(1, 0), Position(1, 1)),
            level: 0,
            ..Default::default()
        });
        assert_eq!(stream.next().unwrap(), Token{
            kind: TokenKind::Question,
            position: (Position(1, 1), Position(1, 2)),
            level: 0,
            ..Default::default()
        });
        assert_eq!(stream.next().unwrap(), Token{
            kind: TokenKind::Ampersand,
            position: (Position(1, 2), Position(1, 3)),
            level: 0,
            ..Default::default()
        });
        assert_eq!(stream.next().unwrap(), Token{
            kind: TokenKind::Slash,
            position: (Position(1, 3), Position(1, 4)),
            level: 0,
            ..Default::default()
        });
    }

    #[test]
    fn it_should_parse_multiple_lines() {
        let mut stream = tokenize("!?&/\n\tabc");
        stream.next();
        stream.next();
        stream.next();
        stream.next();
        assert_eq!(stream.next().unwrap(), Token{
            kind: TokenKind::Identifier,
            content: String::from("abc"), 
            position: (Position(2, 1), Position(2, 4)),
            len: 3,
            level: 1,
        })
    }

    #[test]
    fn it_should_parse_two_character_tokens() {
        let mut stream = tokenize("123 >= abc");
        stream.next();
        assert_eq!(stream.next().unwrap(), Token{
            kind: TokenKind::GreaterOrEquals,
            len: 2,
            position: (Position(1, 5), Position(1, 6)),
            ..Default::default()
        })
    }

    #[test]
    fn it_should_parse_string_literals() {
        let mut stream = tokenize("\"some\"");
        assert_eq!(stream.next().unwrap(), Token{
            kind: TokenKind::StringLiteral,
            content: String::from("some"),
            len: 6,
            position: (Position(1, 0), Position(1, 6)),
            ..Default::default()
        })
    }
}