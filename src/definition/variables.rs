use crate::definition::types::Type;
use std::{collections::HashMap, rc::Rc};

pub struct Variable {
    name: String,
    offset: bool,
    frame: usize,
    type_: Rc<Type>,
}

pub struct VariableFrame {
    offset: usize,
    size: usize,
    variables: HashMap<String, Variable>,
}

pub struct Variables {
    global: HashMap<String, Variable>,
    local: HashMap<String, usize>,
    local_frame: Vec<VariableFrame>,
}

impl Variables {
    pub fn new() -> Self {
        Variables {
            global: HashMap::new(),
            local: HashMap::new(),
            local_frame: vec![],
        }
    }
}
