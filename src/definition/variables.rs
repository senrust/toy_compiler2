use crate::definition::types::Type;
use std::{collections::HashMap, ops::Deref, rc::Rc};

#[derive(Debug, PartialEq)]
pub struct GlobalVariable {
    pub name: String,
    pub type_: Type,
}

#[derive(Debug, PartialEq)]
pub struct LocalVariable {
    scope_depth: usize,
    pub name: String,
    pub frame_offset: usize,
    pub type_: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Variable {
    GlobalVal(Rc<GlobalVariable>),
    LocalVal(Rc<LocalVariable>),
}

impl Variable {
    pub fn get_type(&self) -> Type {
        match self {
            Variable::GlobalVal(global_val) => global_val.type_.clone(),
            Variable::LocalVal(local_val) => local_val.type_.clone(),
        }
    }

    pub fn get_array_elem_type(&self) -> Result<Type, ()> {
        if self.is_array_type() {
            match self {
                Variable::GlobalVal(global_val) => {
                    let (_array_len, elem_type) = global_val.type_.array.as_ref().unwrap();
                    Ok(elem_type.deref().clone())
                }
                Variable::LocalVal(local_val) => {
                    let (_array_len, elem_type) = local_val.type_.array.as_ref().unwrap();
                    Ok(elem_type.deref().clone())
                }
            }
        } else {
            Err(())
        }
    }

    pub fn is_pointer_type(&self) -> bool {
        match self {
            Variable::GlobalVal(global_val) => global_val.type_.is_pointer(),
            Variable::LocalVal(local_val) => local_val.type_.is_pointer(),
        }
    }

    pub fn is_array_type(&self) -> bool {
        match self {
            Variable::GlobalVal(global_val) => global_val.type_.is_array(),
            Variable::LocalVal(local_val) => local_val.type_.is_array(),
        }
    }
}

struct LocalScope {
    frame_offset: usize,          // スコープ開始時のスタックサイズ
    scope_val_names: Vec<String>, // スコープ内で宣言された変数
}

/// 変数宣言情報
///
/// member
/// - hidden_local - より深いスコープで同名のローカル変数が宣言された場合に,
/// 宣言済みのローカル変数を退避させるためのテーブル  
/// キーが変数名, 値が退避ローカル変数ベクトル(ベクトル後方ほど深いスコープで宣言された退避ローカル変数)
pub struct VariableDeclarations {
    global_vals: HashMap<String, Rc<GlobalVariable>>,
    local_vals: HashMap<String, Rc<LocalVariable>>,
    local_scopes: Vec<LocalScope>,
    current_frame_offset: usize,
    max_frame_offset: usize,
    local_scope_depth: usize,
    hidden_local: HashMap<String, Vec<Rc<LocalVariable>>>,
}

impl VariableDeclarations {
    pub fn new() -> Self {
        let mut val_declarations = VariableDeclarations {
            global_vals: HashMap::new(),
            local_vals: HashMap::new(),
            local_scopes: vec![],
            current_frame_offset: 8, // rbp分加わる
            max_frame_offset: 8,     // rbp分加わる
            local_scope_depth: 0,
            hidden_local: HashMap::new(),
        };
        let args_scope = LocalScope {
            frame_offset: val_declarations.current_frame_offset,
            scope_val_names: vec![],
        };
        val_declarations.local_scopes.push(args_scope);
        val_declarations
    }

    pub fn get_local_val_frame_size(&self) -> usize {
        self.max_frame_offset
    }

    // グローバル変数を宣言
    pub fn declare_global_val(&mut self, name: &str, type_: Type) -> Result<Variable, ()> {
        if self.global_vals.get(name).is_some() {
            return Err(());
        }
        let new_globalval = Rc::new(GlobalVariable {
            name: name.to_string(),
            type_,
        });
        self.global_vals.insert(name.to_string(), new_globalval);
        Ok(Variable::GlobalVal(
            self.global_vals.get(name).unwrap().clone(),
        ))
    }

    // ローカル変数を現在のスコープで宣言
    pub fn declare_local_val(&mut self, name: &str, type_: Type) -> Result<Variable, ()> {
        // すでに同じローカル変数名が登録されている場合はそのローカル変数をhidden_localに対比させる
        if let Some(same_name_val) = self.local_vals.remove(name) {
            // 現在のスコープですでに宣言されている場合はエラー
            if self.local_scope_depth == same_name_val.scope_depth {
                return Err(());
            } else {
                // すでに同じ変数名が複数宣言され, 秘匿済みの場合
                if let Some(same_name_vals) = self.hidden_local.get_mut(name) {
                    // 新たに追加するローカル変数のスコープを抜けたら再度追加できるように最後尾に追加しする
                    same_name_vals.push(same_name_val);
                } else {
                    let hidden_vec = vec![same_name_val];
                    self.hidden_local.insert(name.to_string(), hidden_vec);
                }
            }
        }

        // ローカル変数をスタックに追加すると8バイトアライメントを超えてしまう場合は,
        // スタックフレームをアライメント境界まで増やしてからローカル変数を追加する
        // すでにアライメント境界のときは何もしない
        if self.current_frame_offset % 8 != 0
            && self.current_frame_offset / 8 != (self.current_frame_offset + type_.size) / 8
        {
            self.current_frame_offset += 8 - (self.current_frame_offset % 8);
        }

        // ローカル変数を必要な情報を追加して登録
        let val_size = type_.size;

        self.local_scopes[self.local_scope_depth]
            .scope_val_names
            .push(name.to_string());
        let local_val = LocalVariable {
            scope_depth: self.local_scope_depth,
            name: name.to_string(),
            frame_offset: self.current_frame_offset,
            type_,
        };
        self.local_vals.insert(name.to_string(), Rc::new(local_val));
        self.current_frame_offset += val_size;
        self.max_frame_offset = std::cmp::max(self.max_frame_offset, self.current_frame_offset);
        Ok(Variable::LocalVal(
            self.local_vals.get(name).unwrap().clone(),
        ))
    }

    // 変数を取得
    pub fn get_variable(&self, name: &str) -> Option<Variable> {
        if let Some(local_val) = self.local_vals.get(name) {
            Some(Variable::LocalVal(local_val.clone()))
        } else {
            self.global_vals
                .get(name)
                .map(|global_val| Variable::GlobalVal(global_val.clone()))
        }
    }

    // ローカル変数スコープに入る
    pub fn enter_new_local_scope(&mut self) {
        self.local_scope_depth += 1;
        let new_scope = LocalScope {
            frame_offset: self.current_frame_offset,
            scope_val_names: vec![],
        };
        self.local_scopes.push(new_scope);
    }

    // 現在の(=最も深い)ローカル変数スコープから抜ける
    pub fn exit_current_local_scope(&mut self) {
        if let Some(exit_scope) = self.local_scopes.pop() {
            // スタックフレームサイズをスコープ開始時に戻す
            self.current_frame_offset = exit_scope.frame_offset;
            // 脱出するスコープに登録されているローカル変数をローカル変数テーブルから削除する
            for local_val in exit_scope.scope_val_names {
                self.local_vals.remove(&local_val);
                // 同じ変数名がより浅いスコープに登録されていた場合
                if let Some(same_name_hidden_locals) = self.hidden_local.get_mut(&local_val) {
                    if let Some(deepest_same_name_hidden_local) = same_name_hidden_locals.pop() {
                        // もし同じローカル変数がなくなった場合はハッシュテーブルのエントリを削除
                        if same_name_hidden_locals.is_empty() {
                            self.hidden_local.remove(&local_val);
                        }
                        // 最も深いスコープの変数名を登録しなおす
                        self.local_vals
                            .insert(local_val, deepest_same_name_hidden_local);
                    }
                }
            }
            self.local_scope_depth -= 1;
        }
    }

    pub fn clear_local_val_scope(&mut self) {
        self.local_vals.clear();
        self.local_scopes.clear();
        self.current_frame_offset = 8; // rbp分
        self.max_frame_offset = 8;
        self.local_scope_depth = 0;
        self.hidden_local.clear();

        let args_scope = LocalScope {
            frame_offset: self.current_frame_offset,
            scope_val_names: vec![],
        };
        self.local_scopes.push(args_scope);
    }
}
