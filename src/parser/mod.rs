use std::fmt::{self, Display, Formatter};

mod scanner;

#[derive(Debug, PartialEq)]
pub enum Type {
    LeftParen,
    RightParen,
    Quote,
    Integer { value: i64 },
    Float { value: f64 },
    Symbol { value: String, comment: Option<String> },
    Text { value: String },
    Eof
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::LeftParen => write!(f, "("),
            Self::RightParen => write!(f, ")"),
            Self::Quote => write!(f, "'"),
            Self::Integer { value } => write!(f, "{}", value),
            Self::Float { value } => write!(f, "{}", value),
            Self::Symbol { value, comment } => 
                match comment {
                    Some(c) => write!(f, "\n; {}\n{}", c, value),
                    None => write!(f, "{}", value)
                },
            Self::Text { value} => write!(f, "\"{}\"", value),
            Self::Eof => write!(f, "<eof>"),
        }
    }
}

const UNKNOWN: Position = Position { name: "<unknown>", line: 1, column: 1 };
const INTERNAL: Position = Position { name: "<internal>", line: 1, column: 1 };

struct Position<'a> {
    name: &'a str,
    line: u32,
    column: u32,
}
struct Token<'a> {
    t_type: Type,
    position:Position<'a>,
}

impl<'a> Token<'a> {
    fn new(t_type: Type, position: Position<'a>) -> Self {
        Self { t_type, position }
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
        assert_eq!(Type::Eof.to_string(), "<eof>");
    }

    #[test]
    fn test_integer64() {
        let t = Type::Integer{value: 42};
        assert_eq!(t.to_string(), "42");
    }

    #[test]
    fn test_float64() {
        let t = Type::Float{value: 3.14};
        assert_eq!(t.to_string(), "3.14");
    }

    #[test]
    fn test_symbol() {
        let t = Type::Symbol{
            value: String::from("i-am-a-symbol"), comment: None
        };
        assert_eq!(t.to_string(), "i-am-a-symbol");
    }

    #[test]
    fn test_symbol_with_comment() {
        let t = Type::Symbol{
            value: String::from("i-am-a-symbol"), comment: Some(String::from("I come with a comment!"))
        };
        assert_eq!(t.to_string(), "\n; I come with a comment!\ni-am-a-symbol");
    }

    #[test]
    fn test_text() {
        let t = Type::Text { value: String::from("i-am-a-string")};
        assert_eq!(t.to_string(), "\"i-am-a-string\"");
    }

    #[test]
    fn test_equal_type() {
        let t1 = Type::Integer{value: 1};
        let t2 = Type::Integer{value: 1};
        assert_eq!(t1, t2);
    }

}