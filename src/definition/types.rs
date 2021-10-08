use super::functions::Function;
use crate::ast_maker::Ast;
use crate::definition::number::Number;
use std::{collections::HashMap, marker::PhantomData, rc::Rc};

#[derive(Debug)]
pub struct SturctMember {
    name: String,
    type_: Rc<Type>,
}

impl SturctMember {
    fn new(name: &str, type_: Rc<Type>) -> Self {
        SturctMember {
            name: name.to_string(),
            type_,
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum PrimitiveType {
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

#[derive(Debug)]
// 型定義
// 配列型でインデックスアクセスを行わない場合はポインタ型に変換されるようにする
pub struct Type {
    pub size: usize,
    pub primitive: Option<PrimitiveType>,
    pub pointer: Option<Rc<Type>>,
    pub array: Option<Rc<Type>>,
    pub struct_name: Option<String>,
    pub struct_members: Option<Rc<HashMap<String, (usize, SturctMember)>>>,
    pub function: Option<Rc<Function>>,
    _private: PhantomData<()>, // コンストラクタからのみ作成できるようにする
}

// 定義済みの関数と新たに宣言した関数が同じ引数型, 戻り値型の関数であるかの比較に使用するため,
// ポインタ型, 配列型, 関数型の指す先はすでに定義済みのプリミティブ型か構造体型に行き着く
impl PartialEq for Type {
    fn eq(&self, rhs: &Self) -> bool {
        if self.size != rhs.size {
            return false;
        }

        // お互いにプリミティブ型の場合
        if self.primitive.is_some() && rhs.primitive.is_some() {
            return self.primitive.as_ref().unwrap() == rhs.primitive.as_ref().unwrap();
        }

        // お互いにポインタ型の場合
        if self.pointer.is_some() && self.pointer.is_some() {
            return self.pointer.as_ref().unwrap() == rhs.pointer.as_ref().unwrap();
        }

        // お互いに配列型の場合
        if self.array.is_some() && self.array.is_some() {
            return self.array.as_ref().unwrap() == rhs.array.as_ref().unwrap();
        }

        // お互いに構造体型の場合
        // 同じ構造体名であれば良い
        // 無名構造体の比較は関数の引数チェックでは行われない
        if self.struct_name.is_some() && self.struct_name.is_some() {
            return self.struct_name.as_ref().unwrap() == rhs.struct_name.as_ref().unwrap();
        }

        // お互いに関数型の場合
        if self.function.is_some() && self.function.is_some() {
            return self.function.as_ref().unwrap() == rhs.function.as_ref().unwrap();
        }

        // それ以外の場合はfalse
        false
    }
}

impl Type {
    // primitveはTypesのコンストラクトでのみ呼べるようにする
    fn new_primitive(primitive: PrimitiveType, size: usize) -> Self {
        Type {
            size,
            primitive: Some(primitive),
            pointer: None,
            array: None,
            struct_name: None,
            struct_members: None,
            function: None,
            _private: PhantomData,
        }
    }

    pub fn new_pointer(type_: Rc<Type>) -> Self {
        Type {
            size: 8,
            primitive: None,
            pointer: Some(type_),
            array: None,
            struct_name: None,
            struct_members: None,
            function: None,
            _private: PhantomData,
        }
    }

    // サイズは配列全体のサイズ
    // 配列は右辺値になったときはポインタ型として振る舞うようにする
    // (これで良いかはわからない)
    pub fn new_array(count: usize, type_: Rc<Type>) -> Self {
        Type {
            size: count * type_.size,
            primitive: None,
            pointer: None,
            array: Some(type_),
            struct_name: None,
            struct_members: None,
            function: None,
            _private: PhantomData,
        }
    }

    // 無名構造体は空文字列を渡す
    pub fn new_stuct(name: &str, members: Vec<SturctMember>) -> Self {
        let mut offset: usize = 0;
        let mut member_vec: HashMap<String, (usize, SturctMember)> = HashMap::new();
        for member in members {
            let member_size = member.type_.size;
            // このメンバーを加えることでアライメント境界を超える場合はオフセットをアライメント境界まで動かす
            // すでにアライメント境界のときは何もしない
            if offset % 8 != 0 && offset / 8 != (offset + member_size) / 8 {
                offset += 8 - (offset % 8);
            }

            member_vec.insert(member.name.clone(), (offset, member));
            offset += member_size;
        }
        Type {
            size: offset,
            primitive: None,
            pointer: None,
            array: None,
            struct_name: Some(name.to_string()),
            struct_members: Some(Rc::new(member_vec)),
            function: None,
            _private: PhantomData,
        }
    }

    // サイズ8バイトで定義する
    // asigneeが関数型のときに正しい関数ポインタ定義かチェックする
    // (これで良いかはわからない)
    pub fn new_fucntion(function: Rc<Function>) -> Self {
        Type {
            size: 8,
            primitive: None,
            pointer: None,
            array: None,
            struct_name: None,
            struct_members: None,
            function: Some(function),
            _private: PhantomData,
        }
    }

    pub fn is_pointer(&self) -> bool {
        self.pointer.is_some()
    }

    pub fn deref_pointer(&self) -> Option<Rc<Type>> {
        self.pointer.as_ref().cloned()
    }
}

pub struct TypesDefinitions {
    dict: HashMap<String, Rc<Type>>,
}

impl TypesDefinitions {
    pub fn new() -> Self {
        let mut types = TypesDefinitions {
            dict: HashMap::new(),
        };
        let type_void = Type::new_primitive(PrimitiveType::Void, 0);
        types.dict.insert("void".to_string(), Rc::new(type_void));

        let type_u8 = Type::new_primitive(PrimitiveType::U8, 1);
        types
            .dict
            .insert("unsigned char".to_string(), Rc::new(type_u8));
        let type_u16 = Type::new_primitive(PrimitiveType::U16, 2);
        types
            .dict
            .insert("unsigned short".to_string(), Rc::new(type_u16));
        let type_u32 = Type::new_primitive(PrimitiveType::U32, 4);
        types
            .dict
            .insert("unsigned int".to_string(), Rc::new(type_u32));
        let type_u64 = Type::new_primitive(PrimitiveType::U64, 8);
        types
            .dict
            .insert("unsigned long".to_string(), Rc::new(type_u64));

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

    pub fn get_primitive_type(&self, num_type: &Number) -> Rc<Type> {
        match num_type {
            Number::U64(_) => self.dict["long"].clone(),
            Number::F64(_) => self.dict["double"].clone(),
        }
    }

    pub fn get_type(&self, name: &str) -> Result<Rc<Type>, ()> {
        if self.dict.contains_key(name) {
            Ok(self.dict.get(name).unwrap().clone())
        } else {
            Err(())
        }
    }

    pub fn define_type(&mut self, name: &str, type_: Type) -> Result<Rc<Type>, ()> {
        if self.dict.contains_key(name) {
            Err(())
        } else {
            let defined_type = Rc::new(type_);
            self.dict.insert(name.to_string(), defined_type.clone());
            Ok(defined_type)
        }
    }
}

pub fn evaluate_binary_operation_type(left: &Ast, _right: &Ast) -> Result<Rc<Type>, ()> {
    Ok(left.type_.clone())
}
