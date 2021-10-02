use std::{collections::HashMap, rc::Rc};
use super::functions::Function;
use crate::definition::number::Number;
use crate::make_ast::AST;

pub struct TypeMember {
    offset: usize,
    name: String,
    type_: Rc<Type>, 
}

enum PrimitiveType {
    Void,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    F32,
    F64,
}

pub struct Type {
    pointer: Option<Rc<Type>>,
    name: Option<String>,
    size: usize,
    primitive: Option<PrimitiveType>,
    member: Option<Vec<TypeMember>>,
    function: Option<Function>,
}

impl Type {
    fn new_primitive(primitive: PrimitiveType, size: usize) -> Self {
        Type {
            pointer: None,
            name: None,
            size,
            primitive: Some(primitive),
            member: None,
            function: None,
        }
    }
}

pub struct Types {
    dict: HashMap<String, Rc<Type>>,
}

impl Types {
    pub fn new() -> Self {
        let mut types = Types { dict: HashMap::new() };
        let type_void = Type::new_primitive(PrimitiveType::Void, 0);
        types.dict.insert("void".to_string(), Rc::new(type_void));

        let type_u8 = Type::new_primitive(PrimitiveType::U8, 1);
        types.dict.insert("unsigned char".to_string(), Rc::new(type_u8));
        let type_u16 = Type::new_primitive(PrimitiveType::U16, 2);
        types.dict.insert("unsigned short".to_string(), Rc::new(type_u16));
        let type_u32 = Type::new_primitive(PrimitiveType::U32, 4);
        types.dict.insert("unsigned int".to_string(), Rc::new(type_u32));
        let type_u64 = Type::new_primitive(PrimitiveType::U64, 8);
        types.dict.insert("unsigned long".to_string(), Rc::new(type_u64));
        
        let type_i8 = Type::new_primitive(PrimitiveType::I8, 1);
        types.dict.insert("char".to_string(), Rc::new(type_i8));
        let type_i16 = Type::new_primitive(PrimitiveType::I16, 2);
        types.dict.insert("short".to_string(), Rc::new(type_i16));
        let type_i32 = Type::new_primitive(PrimitiveType::I32, 4);
        types.dict.insert("int".to_string(), Rc::new(type_i32));
        let type_i64 = Type::new_primitive(PrimitiveType::I64, 8);
        types.dict.insert("long".to_string(), Rc::new(type_i64));

        let type_f32 = Type::new_primitive(PrimitiveType::F32, 4);
        types.dict.insert("float".to_string(), Rc::new(type_f32));
        let type_f64 = Type::new_primitive(PrimitiveType::F64, 8);
        types.dict.insert("double".to_string(), Rc::new(type_f64));

        types
    }

    pub fn get_iimidiate_type(&self, num_type: &Number) -> Rc<Type> {
        match num_type {
            Number::U64(_) => {
                self.dict["long"].clone()
            }
            Number::F64(_) => {
                self.dict["double"].clone()
            }
        }
    }
}

pub fn evaluate_binary_operation_type(left: &AST, right: &AST) -> Rc<Type> {
    left.type_.clone()
}