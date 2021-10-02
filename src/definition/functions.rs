
use std::{collections::HashMap, rc::Rc};
use super::types::Type;

pub struct Function {
    size: usize,
    args: Vec<Rc<Type>>,
    ret: Rc<Type>,
}

struct Functions {
    dict: HashMap<String, Function>,
}