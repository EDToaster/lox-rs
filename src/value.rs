#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
}

impl Value {
    pub fn is_truthy(self) -> bool {
        match self {
            Value::Nil => false,
            Value::Bool(b) => b,
            Value::Number(_) => true,
        }
    }
}
