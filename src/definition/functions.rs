use super::types::Type;
use std::{collections::HashMap, ops::Deref, rc::Rc};

#[derive(Debug)]
pub struct Function {
    name: String,
    args: Option<Vec<Rc<Type>>>,
    ret: Option<Rc<Type>>,
}

impl Function {
    pub fn new(name: &str, args: Option<Vec<Rc<Type>>>, ret: Option<Rc<Type>>) -> Self {
        Function {
            name: name.to_string(),
            args,
            ret,
        }
    }
}

impl PartialEq for Function {
    fn eq(&self, rhs: &Self) -> bool {
        if self.name != rhs.name {
            return false;
        }

        // 引数の数が異なる
        if self.args.is_some() && rhs.args.is_some() {
            let self_args = self.args.as_ref().unwrap();
            let rhs_args = rhs.args.as_ref().unwrap();
            if self_args.len() != rhs_args.len() {
                return false;
            }

            for (self_arg, rhs_arg) in self_args.iter().zip(rhs_args.iter()) {
                if *self_arg.deref() != *rhs_arg.deref() {
                    return false;
                }
            }
        }

        if self.ret.is_some() && rhs.ret.is_some() {
            if self.ret.as_ref().unwrap() != rhs.ret.as_ref().unwrap() {
                return false;
            }
        }
        true
    }
}

pub struct FunctionDefinitions {
    pub dict: HashMap<String, Rc<Function>>,
}

impl FunctionDefinitions {
    pub fn new() -> Self {
        FunctionDefinitions {
            dict: HashMap::new(),
        }
    }

    pub fn get_function(&mut self, name: &str) -> Option<Rc<Function>> {
        if let Some(function) = self.dict.get(name) {
            Some(function.clone())
        } else {
            None
        }
    }

    pub fn declear_function(&mut self, name: &str, function: Function) -> Result<Rc<Function>, ()> {
        // 同じ関数の複数回宣言は許可する
        if let Some(exist_function) = self.dict.get(name) {
            if *exist_function.deref() == function {
                Ok(exist_function.clone())
            } else {
                Err(())
            }
        } else {
            self.dict.insert(name.to_string(), Rc::new(function));
            Ok(self.dict.get(name).unwrap().clone())
        }
    }
}
