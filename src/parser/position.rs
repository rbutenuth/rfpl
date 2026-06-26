use std::{fmt::{Display, Formatter, Result}, rc::Rc};

#[derive(Clone, PartialEq, Debug)]
pub struct Position {
    pub name: Rc<String>,
    pub line: u32,
    pub column: u32,
}

impl Position {
    pub fn new(name: Rc<String>, line: u32, column: u32) -> Self {
        Position { name: name, line, column }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "name: {}, line: {}, column: {}", self.name, self.line, self.column)
    }
}

