use super::token::{Token, Type};
use super::position::Position;
use std::rc::Rc;

#[derive(Clone, PartialEq, Debug)]
struct CharWithPosition {
    position: Position,
    value: char,
}

const NON_SYMBOL_CHARS: &str = "'\"()[] {}:;";
const ILLEGAL_CHARS: &str = "[]{}:"; // reservered for future use (allowed in text)

pub struct Scanner {
    position: Position,
    lookahead: Vec<CharWithPosition>,
    comment: String,
    chars: Box<dyn Iterator<Item = char>>,
}

impl Scanner {
    pub fn named(initial_position: Position, chars: Box<dyn Iterator<Item = char>>) -> Self {
        let mut scanner = Scanner {
            position: initial_position,
            lookahead: Vec::with_capacity(2),
            comment: String::new(),
            chars: chars,
        };
        for _ in 1..=2 {
            match scanner.read_char() {
                Some(ch) => {
                    scanner.lookahead.push(ch);
                }
                None => {}
            }
        }
        // In first line of code, # is a comment character, allowing "#!/usr/bin/fpl"
        // as first line for starting an interpreter on Unix like operating systems.
        if !scanner.eof() && scanner.lookahead[0].value == '#' {
            scanner.skip_rest_of_line();
        }
        scanner
    }

    pub fn anonymous(chars: Box<dyn Iterator<Item = char>>) -> Self {
        Self::named(
            Position {
                name: Rc::new(String::from("<unknown>")),
                line: 1,
                column: 1,
            },
            chars,
        )
    }

    fn eof(&self) -> bool {
        self.lookahead.is_empty()
    }

    fn move_one_char(&mut self) -> Position {
        if self.eof() {
            self.position.clone()
        } else {
            let pos = self.lookahead.remove(0).position;
            match self.read_char() {
                Some(ch) => {
                    self.position = ch.position.clone();
                    self.lookahead.push(ch);
                }
                None => {}
            };
            pos
        }
    }

    fn read_char(&mut self) -> Option<CharWithPosition> {
        match self.chars.next() {
            Some(ch) => {
                let result = Some(CharWithPosition {
                    position: self.position.clone(),
                    value: ch,
                });
                self.position = match ch {
                    '\n' => Position {
                        name: self.position.name.clone(),
                        line: self.position.line + 1,
                        column: 1,
                    },
                    '\r' => Position {
                        name: self.position.name.clone(),
                        line: self.position.line,
                        column: 1,
                    },
                    _ => Position {
                        name: self.position.name.clone(),
                        line: self.position.line,
                        column: self.position.column + 1,
                    },
                };
                result
            }
            None => None,
        }
    }

    // only call this when you have checked !self.eof()
    fn next_char(&self) -> char {
        self.lookahead[0].value
    }

    fn next_char_is(&mut self, ch: char) -> bool {
        !self.eof() && self.next_char() == ch
    }

    fn next_char_is_whitespace(&mut self) -> bool {
        !self.eof() && self.next_char().is_whitespace()
    }

    fn next_char_is_symbol_char(&mut self) -> bool {
        !self.eof() && !NON_SYMBOL_CHARS.contains(self.next_char())
    }

    fn skip_rest_of_line(&mut self) -> String {
        let mut content = String::new();
        while !self.eof() && !self.next_char_is('\n') && !self.next_char_is('\r') {
            content.push(self.next_char());
            self.move_one_char();
        }
        content
    }

    fn skip_comment(&mut self) {
        while self.next_char_is(';') || self.next_char_is_whitespace() {
            if self.next_char_is(';') {
                if self.comment.len() > 0 {
                    self.comment.push('\n');
                }
                self.move_one_char(); // skip ';'
                let comment_line = String::from(self.skip_rest_of_line().trim());
                self.comment += comment_line.as_str();
            } else {
                while !self.eof() && self.next_char_is_whitespace() {
                    self.move_one_char();
                }
            }
        }
    }

    fn symbol(&mut self) -> Token {
        let mut symbol_text = String::new();
        let symbol_pos = self.position.clone();
        while self.next_char_is_symbol_char() {
            symbol_text.push(self.next_char());
            self.move_one_char();
        }
        let opt_comment = if self.comment.len() > 0 {
            Some(self.comment.clone())
        } else {
            None
        };
        Token::new_with_pos(Type::Symbol { value: symbol_text, comment: opt_comment }, symbol_pos)
    }

    fn text(&mut self) -> Token {
        self.move_one_char(); // skip leading "

        let mut text = String::new();
        // TODO: EOF in String testen (Claude meint, es ist falsch)
        while !self.eof() && self.next_char() != '"' {
            let ch = self.next_char();
			if ch == '\\' {
				self.move_one_char();
				if self.eof() {
					panic!("Unterminated \\ at end of input");
				}
                let esc = self.next_char();
                self.move_one_char();
                match esc {
                    '"' => text.push('"'),
                    'n' => text.push('\n'),
                    'r' => text.push('\r'),
                    't' => text.push('\t'),
                    'u' => text.push(self.read_hexadecimal_character(4)),
                    'v' => text.push(self.read_hexadecimal_character(8)),
                    _ => text.push(ch)
                }
			} else {
                text.push(self.next_char());
                self.move_one_char();
            }
            if self.eof() {
                panic!("Unterminated string at end of input");
            }
        }
        self.move_one_char(); // skip trailing "
        // self.position.clone() ist die Position am Ende des Textes.
        Token::new_with_pos(Type::Text { value: text }, self.position.clone())
    }
    
    fn read_hexadecimal_character(&mut self, number_of_hex_digits: usize) -> char {
        self.move_one_char(); // skip u or v
        let mut result: u32 = 0;
        for _ in 1..=number_of_hex_digits {
            result <<= 4;
            if self.eof() {
                panic!("Unterminated string at end of input")
            }
            let ch = self.lookahead[0].value.to_ascii_lowercase();
			result += self.hex_digit(ch);
            self.move_one_char();
        }
        match char::from_u32(result) {
            Some(ch) => ch,
            None => panic!("illegal unicode value sequence")
        }
    }

	fn hex_digit(&self, low: char) -> u32 {
        (match low {
            '0' ..= '9' => low as u8 - '0' as u8,
            'a' ..= 'f' => low as u8 - 'a' as u8 + 10,
            _ => panic!("illegal character in unicode hex sequence")
        }) as u32
    }
}

impl Iterator for Scanner {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_comment();
        if self.eof() {
            None
        } else {
            Some(match self.next_char() {
                '(' => {
                    Token::new_with_pos(Type::LeftParen, self.move_one_char())
                }
                ')' => {
                    Token::new_with_pos(Type::RightParen, self.move_one_char())
                }
                '\'' => {
                    Token::new_with_pos(Type::Quote,  self.move_one_char())
                }
                ch if !NON_SYMBOL_CHARS.contains(ch) => {
                    self.symbol()
                }
                '"' => {
                    self.text()
                }
                // TODO: Number, if (ch == '-' && nextIsNumberCharacter() || ch >= '0' && ch <= '9') {
                ch if ILLEGAL_CHARS.contains(ch) => {
                    panic!("illegal character")
                }
                _ => {
                    todo!()
                }
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(dead_code, unused)]

    fn scan_and_collect(source: &str) -> String {
        let chars: Vec<char> = source.chars().collect();
        Scanner::anonymous(Box::new(chars.into_iter()))
            .map(|token| token.to_string())
            .collect::<Vec<String>>()
            .join(",")
    }

    #[test]
    fn test_empty_source() {
        let mut sc = Scanner::anonymous(Box::new("".chars()));
        let next = sc.next();
        assert!(next.is_none());
    }

    #[test]
    fn test_left_paren_with_position_check() {
        let mut sc = Scanner::anonymous(Box::new("(".chars()));
        let next = sc.next().unwrap();
        let position = next.position.unwrap();
        assert_eq!(1, position.line);
        assert_eq!(1, position.column);
        assert_eq!(String::from("<unknown>"), *position.name);
    }

    #[test]
    fn test_left_paren() {
        assert_eq!(String::from("("), scan_and_collect("("));
    }

    #[test]
    fn test_right_paren() {
        assert_eq!(String::from(")"), scan_and_collect(")"));
    }

    #[test]
    fn test_quote() {
        assert_eq!(String::from(")"), scan_and_collect(")"));
    }

    #[test]
    fn test_symbol() {
        assert_eq!(String::from("foo,bar"), scan_and_collect("foo bar"));
    }

    #[test]
    fn test_symbol_with_comment() {
       // TODO
       assert_eq!(String::from("symbol"), scan_and_collect("symbol"));
    }

    #[test]
    fn test_simple_text() {
        assert_eq!(String::from("\"foo\""), scan_and_collect("\"foo\""));
    }

    /*

    @Test
    public void symbolAndWhitespace() throws Exception {
        try (Scanner sc = new Scanner("test", new StringReader("symbol   "))) {
            Token t = sc.next();
            assertNotNull(t);
            assertEquals(Id.SYMBOL, t.getId());
            assertEquals("symbol", t.toString());
        }
    }

    @Test
    public void symbolAndLeftParenthesis() throws Exception {
        try (Scanner sc = new Scanner("test", new StringReader("symbol("))) {
            Token t = sc.next();
            assertNotNull(t);
            assertEquals(Id.SYMBOL, t.getId());
            assertEquals("symbol", t.toString());
            t = sc.next();
            assertNotNull(t);
            assertEquals(Id.LEFT_PAREN, t.getId());
        }
    }

    @Test
    public void commentsAndSymbol() throws Exception {
        String COMMENT = "commentLine1" + NL + "; commentLine2" + NL + ";commentLine3";
        try (Scanner sc = new Scanner("test", new StringReader(";   " + COMMENT + NL + " symbol"))) {
            Token t = sc.next();
            assertNotNull(t);
            assertEquals(Id.SYMBOL, t.getId());
            assertEquals("symbol", t.toString());
            assertEquals(COMMENT.replace(";", "").replace(" ", ""), t.getComment());
        }
    }

    @Test
    public void commentAtEndOfFile() throws Exception {
        try (Scanner sc = new Scanner("test", new StringReader("symbol\n; xxx"))) {
            Token t = sc.next();
            assertNotNull(t);
            assertEquals(Id.SYMBOL, t.getId());
            assertEquals("symbol", t.toString());
            t = sc.next();
            assertEquals(Id.EOF, t.getId());
        }
    }

    @Test
    public void symbolEmptyCommentSymbol() throws Exception {
        try (Scanner sc = new Scanner("test", new StringReader("bla\n;\rblubber"))) {
            Token t = sc.next();
            assertNotNull(t);
            assertEquals(Id.SYMBOL, t.getId());
            assertEquals("bla", t.toString());
            String comment = t.getComment();
            assertEquals(0, comment.length());
            t = sc.next();
            assertNotNull(t);
            assertEquals(Id.SYMBOL, t.getId());
            assertEquals("blubber", t.toString());
        }
    }

    @Test
    public void symbolStartsWithMinus() throws Exception {
        try (Scanner sc = new Scanner("test", new StringReader("-a"))) {
            Token t = sc.next();
            assertEquals(Id.SYMBOL, t.getId());
            assertEquals("-a", t.toString());
            assertEquals(1, t.getPosition().getLine());
            t = sc.next();
            assertEquals(Id.EOF, t.getId());
        }
    }
    @Test
    public void firstLineWithHash() throws Exception {
        try (Scanner sc = new Scanner("test", new StringReader("#!/bin/fpl" + NL + "test"))) {
            Token t = sc.next();
            assertEquals(Id.SYMBOL, t.getId());
            assertEquals("test", t.toString());
            assertEquals(2, t.getPosition().getLine());
            t = sc.next();
            assertEquals(Id.EOF, t.getId());
        }
    }

    @Test
    public void parenthesisAndSymbol() throws Exception {
        try (Scanner sc = new Scanner("test", new StringReader("'( bla \n\r) ; sinnfrei\n\r;leer\n"))) {
            Token t = sc.next();
            assertNotNull(t);
            assertEquals(Id.QUOTE, t.getId());
            assertEquals("'", t.toString());
            Position p = t.getPosition();
            assertEquals("test", p.getName());
            assertEquals(1, p.getLine());
            assertEquals(2, p.getColumn());
            t = sc.next();
            assertNotNull(t);
            assertEquals(Id.LEFT_PAREN, t.getId());
            assertEquals("(", t.toString());
            t = sc.next();
            assertNotNull(t);
            assertEquals(Id.SYMBOL, t.getId());
            assertEquals("bla", t.getStringValue());
            assertEquals("Position[name=\"test\", line=1, column=5]", t.getPosition().toString());
            t = sc.next();
            assertNotNull(t);
            assertEquals(Id.RIGHT_PAREN, t.getId());
            assertEquals(")", t.toString());
            t = sc.next();
            assertEquals(Id.EOF, t.getId());
        }
    }

    @Test
    public void number() throws Exception {
        try (Scanner sc = new Scanner("test",
                new StringReader("123\t-456 ;comment \n1.23e4\n-31.4e-1\n2.78E+0\n3.14\n3E2\n-.5"))) {
            Token t = sc.next();
            assertNotNull(t);
            assertEquals(Id.INTEGER, t.getId());
            assertEquals(123, t.getIntegerValue());
            assertEquals("123", t.toString());
            t = sc.next();
            assertNotNull(t);
            assertEquals(Id.INTEGER, t.getId());
            assertEquals(-456, t.getIntegerValue());
            t = sc.next();
            assertNotNull(t);
            assertEquals(Id.DOUBLE, t.getId());
            assertEquals(1.23e4, t.getDoubleValue(), 0.001);
            assertEquals("12300.0", t.toString());
            t = sc.next();
            assertNotNull(t);
            assertEquals(Id.DOUBLE, t.getId());
            assertEquals(-31.4e-1, t.getDoubleValue(), 0.001);
            t = sc.next();
            assertNotNull(t);
            assertEquals(Id.DOUBLE, t.getId());
            assertEquals(2.78, t.getDoubleValue(), 0.001);
            t = sc.next();
            assertNotNull(t);
            assertEquals(Id.DOUBLE, t.getId());
            assertEquals(3.14, t.getDoubleValue(), 0.001);
            t = sc.next();
            assertNotNull(t);
            assertEquals(Id.DOUBLE, t.getId());
            assertEquals(300, t.getDoubleValue(), 0.001);
            t = sc.next();
            assertNotNull(t);
            assertEquals(Id.DOUBLE, t.getId());
            assertEquals(-0.5, t.getDoubleValue(), 0.001);
            Position p = t.getPosition();
            assertEquals("test", p.getName());
            assertEquals(7, p.getLine());
            assertEquals(2, p.getColumn());
        }
    }

    @Test
    public void string() throws Exception {
        try (Scanner sc = new Scanner("test", new StringReader("(\"a\\\"bc\ndef\\nhij\" \r\n\"a\\tb\\rc\\n\")"))) {
            Token t = sc.next();
            assertNotNull(t);
            assertEquals(Id.LEFT_PAREN, t.getId());
            t = sc.next();
            assertNotNull(t);
            assertEquals(Id.STRING, t.getId());
            assertEquals("a\"bc\ndef\nhij", t.getStringValue());
            t = sc.next();
            assertNotNull(t);
            assertEquals(Id.STRING, t.getId());
            assertEquals("a\tb\rc\n", t.getStringValue());
            assertEquals("\"a\tb\rc\n\"", t.toString());
            t = sc.next();
            assertNotNull(t);
            assertEquals(Id.RIGHT_PAREN, t.getId());
            assertEquals(Id.EOF, sc.next().getId());
        }
    }

    @Test
    public void jsonEscapes() throws Exception {
        try (Scanner sc = new Scanner("test", new StringReader("\"\\/\\f\\b\")"))) {
            Token t = sc.next();
            assertNotNull(t);
            assertEquals(Id.STRING, t.getId());
            assertEquals("/\f\b", t.getStringValue());
        }
    }

Out-of-Range Values (> U+10FFFF): The Unicode standard only defines code points up to 
U+10FFFF. Any 32-bit value higher than this is invalid.

    @Test
    public void hexEscape() throws Exception {
        try (Scanner sc = new Scanner("test", new StringReader("\"\\u12ab\")"))) {
            Token t = sc.next();
            assertNotNull(t);
            assertEquals(Id.STRING, t.getId());
            String s = t.getStringValue();
            assertEquals(1, s.length());
            char ch = s.charAt(0);
            assertEquals(0x12ab, ch);
        }
    }

    @Test
    public void shortHexSequence() throws Exception {
        try (Scanner sc = new Scanner("test", new StringReader("\"\\u12\""))) {
            sc.next();
            fail("missing exception");
        } catch (ParseException pe) {
            assertEquals("Illegal hex digit: \"", pe.getMessage());
    
    @Test
    public void unterminatedString() throws Exception {
        assertThrows(ParseException.class, () -> {
        try (Scanner sc = new Scanner("test", new StringReader("'( bla \") ; sinnfrei"))) {
                Token t = sc.next();
                while (t != null) {
                    t = sc.next();
                }
            }
        });
    }

    @Test
    public void badNumber() throws Exception {
        try (Scanner sc = new Scanner("test", new StringReader("123ef456"))) {
            try {
                sc.next();
            } catch (ParseException pe) {
                assertEquals("Bad number: 123ef456", pe.getMessage());
            }
        }
    }
    
    @Test
    public void badHexDigit() throws Exception {
            try (Scanner sc = new Scanner("test", new StringReader("\"\\u12z4\""))) {
                sc.next();
                fail("missing exception");
            } catch (ParseException pe) {
                assertEquals("Illegal hex digit: z", pe.getMessage());
            }
        }

    @Test
    public void badQuoting() throws Exception {
        try (Scanner sc = new Scanner("test", new StringReader("\"\\"))) {
    		sc.next();
	    	fail("missing exception");
	    } catch (ParseException pe) {
		    assertEquals("Unterminated \\ at end of input", pe.getMessage());
	    }
	}

	@Test
	public void endOfSourceInHexSequence() throws Exception {
		try (Scanner sc = new Scanner("test", new StringReader("\"\\u12"))) {
                sc.next();
                fail("missing exception");
            } catch (ParseException pe) {
                assertEquals("Unterminated string at end of input", pe.getMessage());
            }
        }

        @Test
        public void illegalSymbolCharacter() throws Exception {
            try (Scanner sc = new Scanner("test", new StringReader("{"))) {
                sc.next();
                fail("missing exception");
            } catch (ParseException pe) {
                assertEquals("Illegal character for symbol: {", pe.getMessage());
            }
        }

        @Test
        public void exceptionOnRead() throws Exception {
            try (Scanner sc = new Scanner("test", 1, 1, new OnReadExceptionReader())) {
                sc.next();
                fail("missing exception");
            } catch (ParseException pe) {
                assertEquals("bäm", pe.getMessage());
            }
        }
    */
}
