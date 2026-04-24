use std::rc::Rc;
use super::token::{Token, Position, Type};


pub struct Scanner {
    // private static final String NON_SYMBOL_CHARS = "'\"()[] {}:;";
    // private static final String NL = System.lineSeparator();
    name: Rc<String>,
    line: u32,
    column: u32,
    current_char: char,
    next_char: char,
    comment: String,
    start: bool,
    chars_queued: u32, // no more than 2 (current_char and next_char)
    chars: Box<dyn Iterator<Item = char>>,
}

impl Scanner {
    pub fn named(
        name: Rc<String>,
        line: u32,
        column: u32,
        chars: Box<dyn Iterator<Item = char>>,
    ) -> Self {
        let mut scanner = Scanner {
            name: name.clone(),
            line: line,
            column: column,
            current_char: '\0',
            next_char: '\0',
            comment: String::new(),
            start: true,
            chars_queued: 2,
            chars: chars,
        };
        scanner.read_char();
        // In first line of code, # is a comment character, allowing "#!/usr/bin/fpl"
        // as first line for starting an interpreter on Unix like operating systems.
        if scanner.current_char == '#' {
            scanner.skip_rest_of_line();
        }
        scanner
    }

    pub fn anonymous(chars: Box<dyn Iterator<Item = char>>) -> Self {
        Self::named(Rc::new(String::from("<unknown>")), 1, 1, chars)
    }

    fn read_char(&mut self) {
        if self.start {
            self.current_char = self.read_with_eof_handling();
            self.next_char = self.read_with_eof_handling();
        } else {
            self.start = false;
            self.current_char = self.next_char;
            self.current_char = self.read_with_eof_handling();
        }
        match self.current_char {
            '\n' => {
                self.line += 1;
                self.column = 1;
            }
            '\r' => {
                self.column = 1;
            }
            _ => {
                self.column += 1;
            }
        }
    }

    fn read_with_eof_handling(&mut self) -> char {
        match self.chars.next() {
            Some(ch) => {
                ch
            },
            None => {
                if self.chars_queued > 0 {
                    self.chars_queued -= 1;
                }
                '\0'
            }
        }
    }

    fn eof(&mut self) -> bool {
        self.chars_queued == 0
    }

    fn skip_rest_of_line(&mut self) -> String {
        let mut content = String::new();
        while self.current_char != '\n' && self.current_char != '\r' && !self.eof() {
            content.push(self.current_char);
            self.read_char();
        }
        content
    }

    fn skip_comment(&mut self) {
        while self.current_char == ';' || self.current_char.is_whitespace() {
            if self.current_char == ';' {
                if self.comment.len() > 0 {
                    self.comment.push('\n');
                }
                self.read_char(); // skip ';'
                let comment_line = String::from(self.skip_rest_of_line().trim());
                self.comment += comment_line.as_str();
            } else {
                while !self.eof() && self.current_char.is_whitespace() {
                    self.read_char();
                }
            }
        }
    }
}

impl Iterator for Scanner {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        println!("current_char: {}", self.current_char);
        if self.eof() {
            None
        } else {
            self.skip_comment();
            let _ = Position::new(self.name.clone(), self.line, self.column);
            if self.current_char == '(' {
    			self.read_char();
	    		Some(Token::new(Type::LeftParen))
            } else {
                todo!()
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
		} else {
			return symbol(position);
		}
*/
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
    fn test_lp() {
        let mut sc = Scanner::anonymous(Box::new("(".chars()));
        let next = sc.next();
    }

    #[test]
    fn test_left_paren() {
        assert_eq!(String::from("("), scan_and_collect("("));
    }

    // #[test]
    // fn test_symbol() {
    //     assert_eq!(String::from("symbol"), scan_and_collect("symbol"));
    // }
    /*

    @Test
    public void symbol() throws Exception {
        try (Scanner sc = new Scanner("test", new StringReader("symbol"))) {
            Token t = sc.next();
            assertNotNull(t);
            assertEquals(Id.SYMBOL, t.getId());
            assertEquals("symbol", t.toString());
        }
    }

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
		}
	}

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
