use super::token::{Token, Type};

pub struct Scanner {
    // private static final String NON_SYMBOL_CHARS = "'\"()[] {}:;";
    // private static final String NL = System.lineSeparator();
    name: String,
    line: u32,
    column: u32,
    eof: bool,
    current_char: char,
    next_char: char,
    comment: String,
    start: bool,
    chars: Box<dyn Iterator<Item = char>>,
}

impl Scanner {
    pub fn named(
        name: String,
        line: u32,
        column: u32,
        chars: Box<dyn Iterator<Item = char>>,
    ) -> Self {
        let mut scanner = Scanner {
            name: name,
            line: line,
            column: column,
            eof: false,
            current_char: '\0',
            next_char: '\0',
            comment: String::new(),
            start: true,
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
        Self::named(String::from("<unknown>"), 1, 1, chars)
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
            },
            '\r' => {
                self.column = 1;
            },
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
                self.eof = true;
                '\0'
            }
        }
    }

	fn skip_rest_of_line(&mut self) -> String {
        let mut content = String::new();
        while self.current_char != '\n' && self.current_char != '\r' && !self.eof {
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
                while !self.eof && self.current_char.is_whitespace() {
                    self.read_char();
                }
            }
        }
    }

}

impl Iterator for Scanner {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.eof {
            None
        } else {
            self.skip_comment();
            todo!();
            //Some(Token::new(Type::Quote))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xxx() {
        let mut sc = Scanner::anonymous(Box::new("".chars()));
        let next = sc.next();
        assert!(next.is_none());
    }
}
