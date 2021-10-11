use super::types::Type;
use std::{collections::HashMap, ops::Deref};

#[derive(Debug, Clone)]
pub struct Function {
    pub args: Option<Vec<Type>>,
    pub ret: Option<Type>,
}

impl Function {
    pub fn new(args: Option<Vec<Type>>, ret: Option<Type>) -> Self {
        Function { args, ret }
    }
}

impl PartialEq for Function {
    fn eq(&self, rhs: &Self) -> bool {
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

        if self.ret.is_some()
            && rhs.ret.is_some()
            && self.ret.as_ref().unwrap() != rhs.ret.as_ref().unwrap()
        {
            return false;
        }
        true
    }
}

pub struct FunctionDefinitions {
    pub dict: HashMap<String, Function>,
}

impl FunctionDefinitions {
    pub fn new() -> Self {
        FunctionDefinitions {
            dict: HashMap::new(),
        }
    }

    pub fn get_function(&mut self, name: &str) -> Option<Function> {
        self.dict.get(name).cloned()
    }

    pub fn declar_function(&mut self, name: &str, function: Function) -> Result<Function, ()> {
        // 同じ関数の複数回宣言は許可する
        if let Some(exist_function) = self.dict.get(name) {
            if *exist_function == function {
                Ok(exist_function.clone())
            } else {
                Err(())
            }
        } else {
            self.dict.insert(name.to_string(), function);
            Ok(self.dict.get(name).unwrap().clone())
        }
    }
}
