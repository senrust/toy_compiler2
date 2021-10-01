use std::collections::HashMap;
use crate::interpret_token::{Node, NodeKind, NodeInfo};

enum OperationKind {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Assign,  
}

enum ASTKind {
    Operation(OperationKind),
    FanctionCall(String),
}

struct AST {
    kind: ASTKind,
    left: Option<Box<AST>>,
    right: Option<Box<AST>>,
    vec: Option<Vec<AST>>,
}

impl AST {
    fn new(kind: ASTKind) -> Self {
        AST { 
            kind, 
            left: None, 
            right: None, 
            vec: None
        }
    }
}

struct Nodes {
    vec: Vec<Node>,
    cur: usize,
}

impl Nodes {
    fn new(node_vec: Vec<Node>) -> Self {
        Nodes {
            vec: node_vec,
            cur: 0,
        }
    }
}

/// 型定義 
/// 
/// menbers  
/// - name - 型名
/// - size - サイズ(byte)
/// - members - メンバーのオフセットと型

struct TypeDefinition {
    name: String, 
    size: usize,
    members: Option<Vec<(usize, TypeDefinition)>>,
}

/// Typedef情報 
/// 
/// menbers  
/// - name - typedef名
/// - isprimitive - プリミティブ型か?
/// - index - TypeDefinitionsのvecインデックス
struct TypedefInfo {
    name: String, 
    isprimitive: bool,
    index: usize,
}

/// 型情報 
/// 
/// menbers  
/// - primitives - プリミティブ型
/// - structs - 構造体
struct TypeDefinitions {
    primitives: Vec<TypeDefinition>,
    structs: Vec<TypeDefinition>,
    typedef: Vec<TypedefInfo>,
}

fn generate_primitive_typedefinition() -> Vec<TypeDefinition> {
    let mut primitive_definitions: Vec<TypeDefinition> = vec![];
    primitive_definitions.push(TypeDefinition{name: format!("char"), size: 1, members: None});
    primitive_definitions.push(TypeDefinition{name: format!("short"), size: 1, members: None});
    primitive_definitions.push(TypeDefinition{name: format!("int"), size: 1, members: None});
    primitive_definitions.push(TypeDefinition{name: format!("long"), size: 1, members: None});
    primitive_definitions.push(TypeDefinition{name: format!("float"), size: 1, members: None});
    primitive_definitions.push(TypeDefinition{name: format!("double"), size: 1, members: None});
    primitive_definitions
}

impl TypeDefinitions {
    fn new() -> Self {
        TypeDefinitions {
            primitives: generate_primitive_typedefinition(),
            structs: vec![],
            typedef: vec![],
        }
    }
}

/// 型ID
///
/// pointerでポインター型か区別し, primitiveとindexでTypeDefinitionsから情報を読み出せる
///
///
/// menbers  
/// - pointer - ポインター型かどうか
/// - primitive - プリミティブ型か
/// - index - TypeDefinitionsのvecインデックス
/// - unsigned - unsignedか(プリミティブ型のみ有効)
struct TypeInfo {
    pointer: Option<Box<TypeInfo>>,
    primitive: bool,
    index: usize,
    unsigned: bool,
}

/// 関数定義
///
/// members
/// - argtype - 引数
/// - rettype - 戻り地
struct FunctionDifinition {
    argtype: Vec<TypeInfo>,
    rettype: TypeInfo,
}

/// 関数定義集合
///
/// members
/// - vec - 定義されている関数
struct FunctionDifinitions {
    vec: HashMap<String, FunctionDifinition>,
}

impl FunctionDifinitions {
    fn new() -> Self {
        FunctionDifinitions {
            vec: HashMap::new(),
        }
    }
}

/// 変数定義
///
/// members
/// - global - グローバル変数情報
/// - local - ローカル変数情報
struct Variables {
    global: HashMap<String, TypeInfo>,
    local: HashMap<String, TypeInfo>,
}

impl Variables {
    fn new() -> Self {
        Variables {
            global: HashMap::new(),
            local: HashMap::new()
        }
    }

    fn clear_local_variables(&mut self) {
        self.local.clear();
    }
}

fn make_asts(nodes_vec: Vec<Node>) -> Vec<AST> {
    let asts: Vec<AST> = vec![];
    let mut nodes = Nodes::new(nodes_vec);
    asts
}