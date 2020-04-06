pub mod cursor;
pub mod errors;
pub mod token;

use token::{Token, TokenKind};
use cursor::{Cursor, Position, is_end_of_line, is_whitespace, EOF_CHAR};
use errors::LexerError;


pub fn tokenize(input: &str) -> impl Iterator<Item = Token> + '_ {
    tokenize_cursor(Cursor::new(input, Position(1, 0), 0, 0))
}

fn tokenize_cursor(mut cursor: Cursor<'_>) -> impl Iterator<Item = Token> + '_ {
    std::iter::from_fn(move || {
        if cursor.is_eof() {
            return None;
        }
        match cursor.advance_token() {
            Ok(t) => Some(t),
            Err(e) => {
                println!("{:?}", cursor.position);
                panic!("{:?}", e);
            }
        }
    })
}


impl Cursor<'_> {
    fn advance_token(&mut self) -> Result<Token, LexerError> {
        let mut start_position = self.position.clone();
        let first_char = match self.bump() {
            Some(ch) => ch,
            None => return Err(LexerError::UnexpectedEndOfInput)
        };
 
        if self.position.0 > start_position.0 {
            start_position = Position(self.position.0, self.position.1 - 1); // adjust position for after line change
        }

        let initial_len = self.len_consumed();

        let generate_some_token = |kind: TokenKind| -> Result<Token, LexerError>  {
            Ok(Token {
                kind, 
                position: (start_position, self.position.clone()),
                level: self.level.clone(),
                len: self.len_consumed() - initial_len + 1,
                ..Default::default()
            })
        };

        match first_char {
            '/' => match self.first_ahead() {
                '/' => self.single_line_comment(start_position.clone()),
                '*' => self.multi_line_comment(start_position.clone()),
                _ => generate_some_token(TokenKind::Slash),
            },
            '>' => match self.first_ahead() {
                '=' => self.two_characters_token(first_char, self.position.clone()),
                _ => generate_some_token(TokenKind::Greater)
            },
            '<' => match self.first_ahead() {
                '=' => self.two_characters_token(first_char, self.position.clone()),
                _ => generate_some_token(TokenKind::Less)
            },
            '`' => self.content_block(start_position.clone()),
            '!' => generate_some_token(TokenKind::Exclamation),
            '?' => generate_some_token(TokenKind::Question),
            '{' => generate_some_token(TokenKind::OpenCurlyBrace),
            '[' => generate_some_token(TokenKind::OpenSquareBrace),
            '(' => generate_some_token(TokenKind::OpenParentheses),
            '}' => generate_some_token(TokenKind::CloseCurlyBrace),
            ']' => generate_some_token(TokenKind::CloseSquareBrace),
            ')' => generate_some_token(TokenKind::CloseParentheses),
            ':' => generate_some_token(TokenKind::Colon),
            ',' => generate_some_token(TokenKind::Coma),
            '.' => generate_some_token(TokenKind::Dot),
            '+' => generate_some_token(TokenKind::Plus),
            '-' => generate_some_token(TokenKind::Minus),
            '=' => generate_some_token(TokenKind::Equals),
            ';' => generate_some_token(TokenKind::SemiColon),
            '&' => generate_some_token(TokenKind::Ampersand),
            '#' => generate_some_token(TokenKind::HashPound),
            '@' => generate_some_token(TokenKind::At),
            '\\' => generate_some_token(TokenKind::ForwardSlash),
            '|' => generate_some_token(TokenKind::Pipe),
            '%' => generate_some_token(TokenKind::Percent),
            '$' => generate_some_token(TokenKind::Dollar),
            '^' => generate_some_token(TokenKind::Power),
            '~' => generate_some_token(TokenKind::Tilde),
            c if c.is_alphabetic() || c == '_' => self.identifier(c, start_position),
            c if c.is_numeric() => self.number(c, start_position),
            c if c == '\'' => self.string_literal(c, start_position),
            c if c == '"' => self.string_literal(c, start_position),
            c if is_whitespace(c) || is_end_of_line(c) => self.advance_token(),
            c => Err(LexerError::UnexpectedCharacter(c))
        }
    }

    fn two_characters_token(&mut self, first_character: char, start_position: Position) -> Result<Token, LexerError> {
        match first_character {
            '>' => {
                match self.bump().unwrap() {
                    '=' => Ok(Token{kind: TokenKind::GreaterOrEquals, len: 2, position: (start_position, self.position.clone()), ..Default::default()}),
                    c => Err(LexerError::UnexpectedCharacter(c))
                }
            },
            '<' => {
                match self.bump().unwrap() {
                    '=' => Ok(Token{kind: TokenKind::LessOrEquals, len: 2, position: (start_position, self.position.clone()), ..Default::default()}),
                    c => Err(LexerError::UnexpectedCharacter(c))
                }
            },
            c => Err(LexerError::UnexpectedCharacter(c))
        }
    }

    fn string_literal(&mut self, opening_quote: char, start_position: Position) -> Result<Token, LexerError> {
        let mut string_literal = Token {
            kind: TokenKind::StringLiteral,
            ..Default::default()
        };

        let initial_len = self.len_consumed() - 1;

        loop {
            if self.position.0 > start_position.0 {
                return Err(LexerError::UnexpectedEndOfLine);
            }

            match self.bump() {
                Some(c) => {
                    match c {
                        ch if ch == opening_quote => break,
                        ch => string_literal.content.push(ch)
                    }
                }
                None => return Err(LexerError::UnexpectedEndOfInput)
            }
        }

        string_literal.level = self.level.clone();
        string_literal.position = (start_position, self.position.clone());
        string_literal.len = self.len_consumed() - initial_len;

        Ok(string_literal)
    }

    fn number(&mut self, c: char, start_position: Position) -> Result<Token, LexerError> {
        let mut number = Token {
            content: c.to_string(),
            ..Default::default()
        };

        let mut first_separator: char = ' ';
        let mut second_separator: char = ' ';
        let start_consumed = self.len_consumed() - 1; // add 1 for first token

        loop {
            let next_character = self.first_ahead();
            if is_end_of_line(next_character) || next_character == EOF_CHAR {
                break;
            }
            else {
                match next_character {
                    '.'|',' => {
                        if !self.second_ahead().is_numeric() {
                            break;
                        }
                        if first_separator == ' ' {
                            first_separator = next_character;
                        }
                        else if first_separator != next_character && second_separator == ' ' {
                            second_separator = next_character;
                        }
                        else if second_separator == next_character && first_separator != ' ' {
                            return Err(LexerError::UnexpectedCharacter(next_character))
                        }
                        number.content.push(self.bump().unwrap())
                    }
                    c if c.is_numeric() => number.content.push(self.bump().unwrap()),
                    _ => break
                }
            }
        }

        number.kind = TokenKind::Number(
            match second_separator {
                ',' => ',',
                '.' => '.',
                _ => {
                    match first_separator {
                        ',' => '.',
                        '.' => ',',
                        _ => ','
                    }
                }
            },
            match first_separator {
                ',' => ',',
                '.' => '.',
                _ => '.'
            }
        );

        number.len = self.len_consumed() - start_consumed;
        number.position = (start_position, self.position.clone());
        number.level = self.level.clone();

        Ok(number)
    }

    fn identifier(&mut self, first_char: char, start_position: Position) -> Result<Token, LexerError> {
        let mut identifier = Token {
            kind: TokenKind::Identifier,
            content: first_char.to_string(),
            ..Default::default()
        };
        let start_consumed = self.len_consumed() - 1; // add 1 for first token

        loop {
            let next_character = self.first_ahead();
            match next_character {
                c if c.is_alphabetic() || c.is_numeric() || c == '_' => identifier.content.push(self.bump().unwrap()),
                _ => break
            }
        }

        identifier.position = (start_position, self.position.clone());
        identifier.len = self.len_consumed() - start_consumed;
        identifier.level = self.level.clone();

        Ok(identifier)
    }

    fn content_block(&mut self, start_position: Position) -> Result<Token, LexerError> {
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
                if is_end_of_line(ch) {
                    content_block.content.push(ch);
                    content_block.len += 1;
                }
                else {
                    return Err(LexerError::UnexpectedIndentLevel);
                }
            }
            else if self.level == initial_level && ch == '`' {
                block_closed = true;
                self.end_reading_continuous_block();
                break;
            }
            else if self.level > initial_level {
                self.start_reading_continuous_block();
                content_block.content.push(ch);
                content_block.len += 1;
            }
            else {
                match ch {
                    c if is_end_of_line(c) => self.end_reading_continuous_block(),
                    c => buffer.push(c)
                }
            }
        }

        if !block_closed {
            return Err(LexerError::UnexpectedEndOfInput);
        }

        if initial_line == self.position.0 {
            content_block.content.push_str(&buffer);
            content_block.len = buffer.len();
        }
        else {
            let mut buffer_tokens = tokenize_cursor(Cursor::new(&buffer, Position(start_position.0, start_position.1 + 1), initial_level, self.indent_width));
            let mut content_block_initialization_tokens: Vec<Token> = Vec::new();
            while let Some(t) = buffer_tokens.next() {
                content_block_initialization_tokens.push(t);
            }

            content_block.kind = TokenKind::ContentBlock(content_block_initialization_tokens);
        }

        content_block.position = (start_position, self.position.clone());
        content_block.len = self.len_consumed() - start_consumed;
        content_block.level = self.level.clone();

        Ok(content_block)
    }

    fn single_line_comment (&mut self, start_position: Position) -> Result<Token, LexerError> {
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
        
        Ok(comment)
    }

    fn multi_line_comment (&mut self, start_position: Position) -> Result<Token, LexerError> {
        self.bump();
        let mut comment = Token{
            kind: TokenKind::Comment,
            level: self.level.clone(),
            ..Default::default()
        };

        let start_consumed = self.len_consumed();
        self.start_reading_continuous_block();
        while let Some(ch) = self.bump() {

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
        self.end_reading_continuous_block();

        comment.position = (start_position, self.position.clone());
        comment.len = self.len_consumed() - start_consumed + 2; // add 2 for "/*"

        Ok(comment)
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
        let mut stream = tokenize("/*abc\nSOME*/");
        assert_eq!(stream.next().unwrap(), Token{
            kind: TokenKind::Comment,
            content: String::from("abc\nSOME"), 
            position: (Position(1, 0), Position(2, 6)),
            len: 12,
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
    fn it_should_parse_numbers_with_decimal_and_thousands_separator() {
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
    fn it_should_parse_numbers_with_decimal_and_multiple_thousands_separator() {
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
    fn it_should_parse_numbers_with_another_decimal_and_multiple_thousands_separators() {
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
    fn it_should_panic_when_encountering_multiple_decimal_separators() {
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
    fn it_should_keep_inner_indents_when_parsing_content() {
        let mut stream = tokenize("`\n\tabc\n\t\tabc\n`");
        assert_eq!(stream.next().unwrap(), Token{
            kind: TokenKind::ContentBlock(vec![]),
            content: String::from("abc\n\tabc"), 
            position: (Position(1, 0), Position(4, 1)),
            len: 14,
            level: 0
        });
    }

    #[test]
    #[should_panic]
    fn it_should_panic_when_content_block_is_not_closed() {
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
