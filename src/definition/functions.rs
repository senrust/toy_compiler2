use super::types::Type;
use std::{collections::HashMap, rc::Rc};

#[derive(Clone, Debug)]
pub struct Function {
    args: Rc<Vec<Rc<Type>>>,
    ret: Rc<Type>,
}

impl Function {
    fn new(args: Vec<Rc<Type>>, ret: Rc<Type>) -> Self {
        Function {
            args: Rc::new(args),
            ret,
        }
    }
}

impl PartialEq for Function {
    fn eq(&self, rhs: &Self) -> bool {
        // 引数の数が異なる
        if self.args.len() != rhs.args.len() {
            return false;
        }

        for (self_arg, rhs_arg) in self.args.iter().zip(rhs.args.iter()) {
            if self_arg != rhs_arg {
                return false;
            }
        }

        if self.ret != rhs.ret {
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

    pub fn get_function(&mut self, name: &String) -> Option<Function> {
        if let Some(function) = self.dict.get(name) {
            Some(function.clone())
        } else {
            None
        }
    }

    pub fn declear_function(&mut self, name: &String, function: Function) -> Result<(), ()> {
        // 同じ関数の複数回宣言は許可する
        if let Some(exist_function) = self.dict.get(name) {
            if *exist_function == function {
                Ok(())
            } else {
                Err(())
            }
        } else {
            self.dict.insert(name.clone(), function);
            Ok(())
        }
    }
}
