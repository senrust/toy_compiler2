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
    local_block: HashMap<String, usize>,
    local_block_vec: Vec<BlockVariable>,
    local_frame_size: usize,
}

impl Variables {
    pub fn new() -> Self {
        Variables {
            global: HashMap::new(),
            local_block: HashMap::new(),
            local_block_vec: vec![],
            local_frame_size: 0,
        }
    }
}
