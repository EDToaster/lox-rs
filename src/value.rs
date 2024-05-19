use std::{fmt::Display, rc::Rc};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    // TODO: String interning
    Str(Rc<str>),
}

impl<'a> Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Number(n) => write!(f, "{n}"),
            Value::Str(s) => write!(f, "{s}"),
        }
    }
}

// #[derive(Debug, Clone)]
// pub enum Constant {
//     Nil,
//     Bool(bool),
//     Number(f64),
//     Str(String),
// }

impl Value {
    pub fn is_truthy(self) -> bool {
        match self {
            Value::Nil => false,
            Value::Bool(b) => b,
            _ => true,
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Number(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::Str(value.into())
    }
}

// impl From<&'a Constant> for Value {
//     fn from(value: &'a Constant) -> Self {
//         match value {
//             Constant::Nil => Value::Nil,
//             Constant::Bool(b) => Value::Bool(*b),
//             Constant::Number(n) => Value::Number(*n),
//             Constant::Str(s) => Value::Str(s.as_str()),
//         }
//     }
// }
