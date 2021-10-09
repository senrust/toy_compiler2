use super::types::DefinedType;
use std::{collections::HashMap, ops::Deref, rc::Rc};

#[derive(Debug)]
pub struct Function {
    name: String,
    args: Option<Vec<DefinedType>>,
    ret: Option<DefinedType>,
}

impl Function {
    pub fn new(name: &str, args: Option<Vec<DefinedType>>, ret: Option<DefinedType>) -> Self {
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

        if self.ret.is_some()
            && rhs.ret.is_some()
            && self.ret.as_ref().unwrap() != rhs.ret.as_ref().unwrap()
        {
            return false;
        }
        true
    }
}

// 関数定義はRc<Function>型で使用するが,
// ユーザー側でFunction型を作成し, Rc化すると循環参照が発生する可能性がある
// そのため関数型はFunctionDefinitionsに登録し, その戻り値のDefinedFunction型の使用を強制させる
#[derive(Debug, PartialEq, Clone)]
pub struct DefinedFunction(Rc<Function>);

impl DefinedFunction {
    fn new(func: Function) -> Self {
        Self(Rc::new(func))
    }
}

pub struct FunctionDefinitions {
    pub dict: HashMap<String, DefinedFunction>,
}

impl FunctionDefinitions {
    pub fn new() -> Self {
        FunctionDefinitions {
            dict: HashMap::new(),
        }
    }

    pub fn get_function(&mut self, name: &str) -> Option<DefinedFunction> {
        self.dict.get(name).cloned()
    }

    pub fn declear_function(
        &mut self,
        name: &str,
        function: Function,
    ) -> Result<DefinedFunction, ()> {
        // 同じ関数の複数回宣言は許可する
        if let Some(exist_function) = self.dict.get(name) {
            if *exist_function.deref().0 == function {
                Ok(exist_function.clone())
            } else {
                Err(())
            }
        } else {
            self.dict
                .insert(name.to_string(), DefinedFunction::new(function));
            Ok(self.dict.get(name).unwrap().clone())
        }
    }
}
