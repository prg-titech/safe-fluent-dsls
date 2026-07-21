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

#[derive(Debug, Clone)]
pub enum Value {
    Variable(String),
    Int(i32),
    String(String),
    Chain(Box<Chain>)
}