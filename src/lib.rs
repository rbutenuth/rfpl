use std::{error::Error, fmt };
//use std::{error::Error, fmt, sync::Arc};

use list::FplList;

mod list;
mod parser;


#[derive(Debug)]
pub enum Value {
    Nil,
    Integer(i64),
    Float(f64),
    List(FplList),
    Symbol(), // TODO: implement
    String(), // TODO: implement
    Map(), // TODO: implement
    Object(), // TODO: implement
    Function(), // TODO: implement
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Self::Nil => Self::Nil,
            Self::Integer(i) => Self::Integer(*i),
            Self::Float(f) => Self::Float(*f),
            Self::Symbol() => Self::Symbol(),
            Self::String() => Self::String(),
            Self::List(list) => Self::List(list.clone()),
            Self::Map() => Self::Map(),
            Self::Object() => Self::Object(),
            Self::Function() => Self::Function(),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Integer(s), Value::Integer(o)) => s == o,
            _ => false,
        }
    }
}


#[derive(Debug)]
pub struct FplError{
    message: String,
    // TODO: Add information about source position and source (RUST for cause in Java)
}

impl fmt::Display for FplError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TODO {}", self.message)
    }
}

impl Error for FplError {
    // TODO: implement chaining
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl FplError {
    pub fn new(message: String) -> FplError {
        FplError{ message: message }
    }
}
