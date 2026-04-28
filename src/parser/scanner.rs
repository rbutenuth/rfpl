use super::token::{Position, Token, Type};
use std::rc::Rc;

#[derive(Clone, PartialEq, Debug)]
struct CharWithPosition {
    position: Position,
    value: char,
}

const NON_SYMBOL_CHARS: &str = "'\"()[] {}:;";

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
        self.lookahead.len() == 0
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

    fn next_char_is(&mut self, ch: char) -> bool {
        !self.eof() && self.lookahead[0].value == ch
    }

    fn next_char_is_whitespace(&mut self) -> bool {
        !self.eof() && self.lookahead[0].value.is_whitespace()
    }

    fn next_char_is_symbol_char(&mut self) -> bool {
        !self.eof() && !NON_SYMBOL_CHARS.contains(self.lookahead[0].value)
    }

    fn skip_rest_of_line(&mut self) -> String {
        let mut content = String::new();
        while !self.eof() && !self.next_char_is('\n') && !self.next_char_is('\r') {
            content.push(self.lookahead[0].value);
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
        while self.next_char_is_symbol_char() {
            symbol_text.push(self.lookahead[0].value);
            self.move_one_char();
        }
        let opt_comment = if self.comment.len() > 0 {
            Some(self.comment.clone())
        } else {
            None
        };
        Token::new_with_pos(Type::Symbol { value: symbol_text, comment: opt_comment }, self.position.clone())
    }
/*
	private Token symbol(Position position) throws ParseException {
		StringBuilder sb = new StringBuilder();
		while (ch != -1 && !Character.isWhitespace(ch) && NON_SYMBOL_CHARS.indexOf(ch) == -1) {
			sb.append((char) ch);
			readChar();
		}
		Token t = new Token(position, Id.SYMBOL, sb.toString(), comment.toString());
		return t;
	}
 */

}

impl Iterator for Scanner {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_comment();
        if self.eof() {
            None
        } else {
            Some(match self.lookahead[0].value {
                '(' => {
                    Token::new_with_pos(Type::LeftParen, self.move_one_char())
                }
                ')' => {
                    self.move_one_char();
                    Token::new_with_pos(Type::RightParen, self.move_one_char())
                }
                '\'' => {
                    self.move_one_char();
                    Token::new_with_pos(Type::Quote,  self.move_one_char())
                }
                ch if !NON_SYMBOL_CHARS.contains(ch) => {
                    self.symbol()
                }
                _ => {
                    todo!()
                }
            })
        }
        /*
            if (ch == ')') {
                readChar();
                return new Token(position, Id.RIGHT_PAREN);
            } else if (ch == '\'') {
                readChar();
                return new Token(position, Id.QUOTE);
            } else if (ch == '-' && nextIsNumberCharacter() || ch >= '0' && ch <= '9') {
                return number(position);
            } else if (ch == '"') {
			return string(position);
		} else if (NON_SYMBOL_CHARS.indexOf(ch) != -1) {
			throw new ParseException(position, "Illegal character for symbol: " + (char) ch);
		}
*/
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
