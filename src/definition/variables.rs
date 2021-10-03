use crate::definition::types::Type;
use std::{collections::HashMap, rc::Rc};

pub struct Variable {
    name: String,
    size: usize,
    type_: Rc<Type>,
}

pub struct BlockVariable {
    variables: HashMap<String, (usize, Variable)>,
}

pub struct Variables {
    global: HashMap<String, Variable>,
    local: HashMap<String, Variable>,
    local_block: Vec<Vec<String>>,
    local_frame_size: usize,
}

impl Variables {
    pub fn new() -> Self {
        Variables {
            global: HashMap::new(),
            local: HashMap::new(),
            local_block: vec![],
            local_frame_size: 0,
        }
    }
}
