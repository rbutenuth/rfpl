use std::fmt::{self, Display, Formatter};

mod scanner;

#[derive(Debug, PartialEq)]
pub enum TokenValue {
    LeftParen,
    RightParen,
    Quote,
    Integer { value: i64 },
    Float { value: f64 },
    Symbol { value: String, comment: String },
    TokString { value: String },
    Eof
}

impl Display for TokenValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::LeftParen => write!(f, "("),
            Self::RightParen => write!(f, ")"),
            Self::Quote => write!(f, "'"),
            Self::Integer { value } => write!(f, "{}", value),
            Self::Float { value } => write!(f, "{}", value),
            Self::Symbol { value, comment } => 
                write!(f, "\n; {}\n{}", comment, value),
            Self::TokString { value} => write!(f, "\"{}\"", value),
            Self::Eof => write!(f, "<eof>"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitives() {
        assert_eq!(TokenValue::LeftParen.to_string(), "(");
        assert_eq!(TokenValue::RightParen.to_string(), ")");
        assert_eq!(TokenValue::Quote.to_string(), "'");
        assert_eq!(TokenValue::Eof.to_string(), "<eof>");
    }

    #[test]
    fn test_integer64() {
        let t = TokenValue::Integer{value: 42};
        assert_eq!(t.to_string(), "42");
    }

    #[test]
    fn test_float64() {
        let t = TokenValue::Float{value: 3.14};
        assert_eq!(t.to_string(), "3.14");
    }

    #[test]
    fn test_symbol() {
        let t = TokenValue::Symbol{
            value: String::from("i-am-a-symbol"), comment: String::from("I come with a comment!")
        };
        assert_eq!(t.to_string(), "\n; I come with a comment!\ni-am-a-symbol");
    }

    #[test]
    fn test_string() {
        let t = TokenValue::TokString { value: String::from("i-am-a-string")};
        assert_eq!(t.to_string(), "\"i-am-a-string\"");
    }

    #[test]
    fn test_equal() {
        let t1 = TokenValue::Integer{value: 1};
        let t2 = TokenValue::Integer{value: 1};
        assert_eq!(t1, t2);
    }

}