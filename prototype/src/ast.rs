use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Chain {
    inner: Vec<Call>
}

impl Chain {
    pub fn new(inner: Vec<Call>) -> Self {
        Chain {
            inner
        }
    }

    pub fn put_first(&mut self, call: Call) {
        self.inner.insert(0, call);
    }
}

impl Display for Chain {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner.iter().map(|c| c.to_string()).collect::<Vec<String>>().join("."))
    }
}

#[derive(Debug, Clone)]
pub struct Call {
    method: String,
    arguments: Vec<Value>
}

impl Call {
    pub fn new(method: String, arguments: Vec<Value>) -> Self {
        Call {
            method,
            arguments
        }
    }
}

impl Display for Call {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,
           "{}({})",
           self.method,
           self.arguments.iter()
                   .map(|v| v.to_string())
                   .collect::<Vec<String>>()
                   .join(", ")
        )
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Variable(String),
    Int(i32),
    String(String),
    Chain(Box<Chain>)
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Variable(s) => write!(f, "{s}"),
            Value::Int(n) => write!(f, "{n}"),
            Value::String(s) => write!(f, r#""{s}""#),
            Value::Chain(c) => write!(f, "{c}")
        }
    }
}