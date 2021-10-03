use super::types::Type;
use std::{collections::HashMap, rc::Rc};

pub struct Function {
    size: usize,
    args: Vec<Rc<Type>>,
    ret: Rc<Type>,
}

pub struct Functions {
    dict: HashMap<String, Function>,
}

impl Functions {
    pub fn new() -> Self {
        Functions {
            dict: HashMap::new(),
        }
    }
}
