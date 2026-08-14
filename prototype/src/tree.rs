use std::collections::VecDeque;
use crate::tree::Expr::{Add1, False, Int, Neg, True};

pub fn _true() -> Expr {
    True
}

pub fn _false() -> Expr {
    False
}

pub fn int(i: i32) -> Expr {
    Int(i)
}

pub fn add1(e: Expr) -> Expr {
    Add1(Box::new(e))
}

pub fn eq(left: Expr, right: Expr) -> Expr {
    Expr::Eq(Box::new(left), Box::new(right))
}

pub fn neg(e: Expr) -> Expr {
    Neg(Box::new(e))
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Expr {
    True,
    False,
    Int(i32),
    Add1(Box<Expr>),
    Eq(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>)
}

impl Expr {
    pub fn evaluate(&self) -> Option<Value> {
        let mut continuation = ContStack::Eval(&self, VecDeque::new());
        loop {
            println!("{continuation:?}");
            match continuation {
                ContStack::Eval(expr, mut stack) => {
                    continuation = match expr {
                        Expr::True => ContStack::Cont(stack, Value::Bool(true)),
                        Expr::False => ContStack::Cont(stack, Value::Bool(false)),
                        Expr::Int(i) => ContStack::Cont(stack, Value::Int(*i)),
                        Expr::Add1(e) => {
                            stack.push_back(ContFrame::Add1);
                            ContStack::Eval(e, stack)
                        }
                        Expr::Eq(e1, e2) => {
                            stack.push_back(ContFrame::EqSnd(e2));
                            ContStack::Eval(e1, stack)
                        }
                        Expr::Neg(e) => {
                            stack.push_back(ContFrame::Neg);
                            ContStack::Eval(e, stack)
                        }
                    };
                },
                ContStack::Cont(mut stack, value) => {
                    if let Some(frame) = stack.pop_back() {
                        continuation = match frame {
                            ContFrame::Add1 => ContStack::Cont(stack, Value::Int(value.as_int()? + 1)),
                            ContFrame::EqSnd(e) => {
                                stack.push_back(ContFrame::Eq(value));
                                ContStack::Eval(e, stack)
                            },
                            ContFrame::Eq(j) => ContStack::Cont(stack, Value::Bool(value.as_int()? == j.as_int()?)),
                            ContFrame::Neg => ContStack::Cont(stack, Value::Bool(! (value.as_bool()?)))
                        };
                    } else {
                        return Some(value)
                    }
                }
            }
        }
    }

    pub fn reduce(&self) -> Result<Value, Expr> {
        match self {
            True => Ok(Value::Bool(true)),
            False => Ok(Value::Bool(false)),
            Int(i) => Ok(Value::Int(*i)),
            Add1(e) => {
                match e.reduce() {
                    Ok(v) => {
                        if let Some(i) = v.as_int() {
                            Ok(Value::Int(i + 1))
                        } else {
                            Err(add1(v.as_expr()))
                        }
                    }
                    Err(e) => Err(add1(e))
                }
            }
            Expr::Eq(l, r) => {
                let left_result = match l.reduce() {
                    Ok(Value::Int(i)) => i,
                    Ok(v) => return Err(eq(v.as_expr(), r.as_ref().clone())),
                    Err(e) => return Err(eq(e, r.as_ref().clone()))
                };
                let right_result = match r.reduce() {
                    Ok(Value::Int(i)) => i,
                    Ok(v) => return Err(eq(int(left_result), v.as_expr())),
                    Err(e) => return Err(eq(int(left_result), e))
                };
                Ok(Value::Bool(left_result == right_result))
            }
            Neg(e) => {
                match e.reduce() {
                    Ok(Value::Bool(b)) => Ok(Value::Bool(!b)),
                    Ok(v) => Err(neg(v.as_expr())),
                    Err(e) => Err(neg(e))
                }
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Value {
    Bool(bool),
    Int(i32)
}

impl Value {
    pub fn as_bool(&self) -> Option<bool> {
        if let Value::Bool(b) = self {
            Some(*b)
        } else {
            None
        }
    }

    pub fn as_int(&self) -> Option<i32> {
        if let Value::Int(i) = self {
            Some(*i)
        } else {
            None
        }
    }

    pub fn as_expr(&self) -> Expr {
        match self {
            Value::Bool(b) => {
                if *b {
                    _true()
                } else {
                    _false()
                }
            }
            Value::Int(i) => int(*i)
        }
    }
}

#[derive(Debug)]
pub enum ContFrame<'a> {
    Add1,
    EqSnd(&'a Expr),
    Eq(Value),
    Neg
}

#[derive(Debug)]
pub enum ContStack<'a> {
    Eval(&'a Expr, VecDeque<ContFrame<'a>>),
    Cont(VecDeque<ContFrame<'a>>, Value)
}

#[cfg(test)]
mod test {
    use crate::tree::Value;
    use super::*;

    #[test]
    fn test_evaluate() {
        let expr = neg(eq(int(1), add1(int(0))));
        let result = expr.evaluate();
        assert_eq!(Some(Value::Bool(false)), result);
    }

    #[test]
    fn test_reduce() {
        let good_expr = neg(eq(int(1), add1(int(0))));
        let result = good_expr.reduce();
        assert_eq!(Ok(Value::Bool(false)), result);

        let bad_expr = neg(eq(add1(add1(int(2))), _false()));
        let result = bad_expr.reduce();
        assert_eq!(Err(neg(eq(int(4), _false()))), result);
    }
}
