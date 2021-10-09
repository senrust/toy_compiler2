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

    pub fn get_primitive_type(&self, num: &Number) -> Type {
        self.type_.get_primitive_type(num)
    }

    pub fn get_type(&self, name: &str) -> Option<Type> {
        self.type_.get_type(name)
    }

    pub fn define_type(&mut self, name: &str, type_: Type) -> Result<Type, ()> {
        self.type_.define_type(name, type_)
    }

    pub fn get_function(&mut self, name: &str) -> Option<Function> {
        self.function.get_function(name)
    }

    pub fn declear_function(&mut self, name: &str, function: Function) -> Result<Type, ()> {
        if let Ok(_definedfunc) = self.function.declear_function(name, function.clone()) {
            if let Some(func_type) = self.type_.get_type(name) {
                Ok(func_type)
            } else {
                let func_type = Type::new_fucntion(function);
                Ok(self.define_type(name, func_type).unwrap())
            }
        } else {
            Err(())
        }
    }

    pub fn declear_global_val(&mut self, name: &str, type_: Type) -> Result<Variable, ()> {
        self.variable.declear_global_val(name, type_)
    }

    pub fn declear_local_val(&mut self, name: &str, type_: Type) -> Result<Variable, ()> {
        self.variable.declear_local_val(name, type_)
    }

    pub fn get_variable(&self, name: &str) -> Option<Variable> {
        self.variable.get_variable(name)
    }

    pub fn initialize_local_scope(&mut self) {
        self.variable.clear_local_val_scope()
    }

    pub fn enter_new_local_scope(&mut self) {
        self.variable.enter_new_local_scope()
    }

    pub fn exit_current_local_scope(&mut self) {
        self.variable.exit_current_local_scope()
    }

    pub fn get_local_val_frame_size(&self) -> usize {
        self.variable.get_local_val_frame_size()
    }

    pub fn clear_local_val_scope(&mut self) {
        self.variable.clear_local_val_scope()
    }
}
