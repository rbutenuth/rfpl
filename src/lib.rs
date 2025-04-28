use std::sync::Arc;

use list::FplList;

mod list;


#[derive(Debug)]
pub enum Value {
    Nil,
    Integer(i64),
    Float(f64),
    List(FplList),
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
            Self::String() => Self::String(),
            Self::List(list) => Self::List(list.clone()),
            Self::Map() => Self::Map(),
            Self::Object() => Self::Object(),
            Self::Function() => Self::Function(),
        }
    }
}