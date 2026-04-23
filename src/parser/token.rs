use std::rc::Rc;
use std::fmt::{Result, Display, Formatter};


#[derive(Debug, PartialEq)]
pub enum Type {
    LeftParen,
    RightParen,
    Quote,
    Integer {
        value: i64,
    },
    Float {
        value: f64,
    },
    Symbol {
        value: String,
        comment: Option<String>,
    },
    Text {
        value: String,
    },
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::LeftParen => write!(f, "("),
            Self::RightParen => write!(f, ")"),
            Self::Quote => write!(f, "'"),
            Self::Integer { value } => write!(f, "{}", value),
            Self::Float { value } => write!(f, "{}", value),
            Self::Symbol { value, comment } => match comment {
                Some(c) => write!(f, "\n; {}\n{}", c, value),
                None => write!(f, "{}", value),
            },
            Self::Text { value } => write!(f, "\"{}\"", value),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Position {
    name: Rc<String>,
    line: u32,
    column: u32,
}

impl Position {
    fn new(name: &Rc<String>, line: u32, column: u32) -> Self {
        Position { name: name.clone(), line, column }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "name: {}, line: {}, column: {}", self.name, self.line, self.column)
    }
}

pub struct Token {
    t_type: Type,
    position: Option<Position>,
}

impl Token {
    pub fn new(t_type: Type) -> Self {
        Self {
            t_type: t_type,
            position: None,
        }
    }

    pub fn new_with_pos(t_type: Type, position: Position) -> Self {
        Self {
            t_type: t_type,
            position: Some(position),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitives() {
        assert_eq!(Type::LeftParen.to_string(), "(");
        assert_eq!(Type::RightParen.to_string(), ")");
        assert_eq!(Type::Quote.to_string(), "'");
    }

    #[test]
    fn test_integer64() {
        let t = Type::Integer { value: 42 };
        assert_eq!(t.to_string(), "42");
    }

    #[test]
    fn test_float64() {
        let t = Type::Float { value: 3.14 };
        assert_eq!(t.to_string(), "3.14");
    }

    #[test]
    fn test_symbol() {
        let t = Type::Symbol {
            value: String::from("i-am-a-symbol"),
            comment: None,
        };
        assert_eq!(t.to_string(), "i-am-a-symbol");
    }

    #[test]
    fn test_symbol_with_comment() {
        let t = Type::Symbol {
            value: String::from("i-am-a-symbol"),
            comment: Some(String::from("I come with a comment!")),
        };
        assert_eq!(t.to_string(), "\n; I come with a comment!\ni-am-a-symbol");
    }

    #[test]
    fn test_text() {
        let t = Type::Text {
            value: String::from("i-am-a-string"),
        };
        assert_eq!(t.to_string(), "\"i-am-a-string\"");
    }

    #[test]
    fn test_equal_type() {
        let t1 = Type::Integer { value: 1 };
        let t2 = Type::Integer { value: 1 };
        assert_eq!(t1, t2);
    }

    #[test]
    fn test_token_without_position() {
        let t = Token::new(Type::Integer { value: 42 });
        assert_eq!(Type::Integer { value: 42 }, t.t_type);
        assert!(t.position.is_none());
    }

    #[test]
    fn test_token_with_position() {
        let file = Rc::new(String::from("file.fpl"));
        let pos = Position::new(&file, 1, 10);
        let t = Token::new_with_pos(Type::Integer { value: 42 }, pos);
        assert_eq!(Type::Integer { value: 42 }, t.t_type);
        assert!(t.position.is_some());
        let pos2 = t.position.unwrap();
        assert_eq!(pos2.line, 1);
        assert_eq!(pos2.column, 10);
        assert_eq!(pos2.name, file);
        assert_eq!(pos2.to_string(), "name: file.fpl, line: 1, column: 10")
    }
}
