use std::rc::Rc;

use crate::definition::{functions::*, number::*, types::*, variables::*};

pub struct Definitions {
    type_: TypesDefinitions,
    variable: VariableDeclearations,
    function: FunctionDefinitions,
}

impl Definitions {
    pub fn new() -> Self {
        let type_ = TypesDefinitions::new();
        let variable = VariableDeclearations::new();
        let function = FunctionDefinitions::new();
        Definitions {
            type_,
            variable,
            function,
        }
    }

    pub fn get_primitive_type(&self, num: &Number) -> Rc<Type> {
        self.type_.get_primitive_type(&num)
    }

    pub fn get_type(&self, name: &str) -> Result<Rc<Type>, ()> {
        self.type_.get_type(name)
    }

    pub fn define_type(&mut self, name: &str, type_: Type) -> Result<Rc<Type>, ()> {
        self.type_.define_type(name, type_)
    }

    pub fn get_function(&mut self, name: &str) -> Option<Rc<Function>> {
        self.function.get_function(name)
    }

    pub fn declear_function(&mut self, name: &str, function: Function) -> Result<Rc<Type>, ()> {
        if let Ok(func) = self.function.declear_function(name, function) {
            let func_type = Type::new_fucntion(func);
            Ok(Rc::new(func_type))
        } else {
            Err(())
        }
    }

    pub fn declear_global_val(&mut self, name: &str, type_: Rc<Type>) -> Result<Variable, ()> {
        self.variable.declear_global_val(name, type_)
    }

    pub fn declear_local_val(&mut self, name: &str, type_: Rc<Type>) -> Result<Variable, ()> {
        self.variable.declear_local_val(name, type_)
    }

    pub fn get_variable(&self, name: &str) -> Option<Variable> {
        self.variable.get_variable(name)
    }

    pub fn create_local_scope(&mut self) {
        self.variable.create_local_scope()
    }

    pub fn exit_local_scope(&mut self) {
        self.variable.exit_local_scope()
    }

    pub fn get_local_val_frame_size(&self) -> usize {
        self.variable.get_local_val_frame_size()
    }

    pub fn clear_local_val_scope(&mut self) {
        self.variable.clear_local_val_scope()
    }
}
