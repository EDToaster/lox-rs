#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    Obj,
}

impl Value {
    pub fn is_truthy(self) -> bool {
        match self {
            Value::Nil => false,
            Value::Bool(b) => b,
            _ => true,
        }
    }
}
