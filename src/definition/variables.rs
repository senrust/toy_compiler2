use std::{collections::HashMap, rc::Rc};
use crate::definition::types::Type;

pub struct Variable {
    name: String,
    local: bool,
    type_: Rc<Type>,
}

pub struct Variables {
    global: HashMap<String, Variable>,
    local: HashMap<String, Variable>,
}