use crate::definition::types::Type;
use std::{collections::HashMap, rc::Rc};

#[derive(Clone)]
pub struct GlobalVariable {
    pub name: String,
    pub type_: Rc<Type>,
}

#[derive(Clone)]
pub struct LocalVariable {
    scope_depth: usize,
    pub frame_offset: usize,
    pub type_: Rc<Type>,
}

pub enum Variable {
    GlobalVal(GlobalVariable),
    LocalVal(LocalVariable),
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
pub struct VariableDeclearations {
    global_vals: HashMap<String, Rc<Type>>,
    local_vals: HashMap<String, LocalVariable>,
    local_scopes: Vec<LocalScope>,
    local_frame_size: usize,
    local_scope_depth: usize,
    hidden_local: HashMap<String, Vec<LocalVariable>>,
}

impl VariableDeclearations {
    pub fn new() -> Self {
        VariableDeclearations {
            global_vals: HashMap::new(),
            local_vals: HashMap::new(),
            local_scopes: vec![],
            local_frame_size: 0,
            local_scope_depth: 0,
            hidden_local: HashMap::new(),
        }
    }

    // グローバル変数を宣言
    pub fn declear_global_val(&mut self, name: String, type_: Rc<Type>) {
        self.global_vals.insert(name, type_);
    }

    // ローカル変数を現在のスコープで宣言
    pub fn declear_local_val(&mut self, name: String, type_: Rc<Type>) -> Result<(), ()> {
        // すでに同じローカル変数名が登録されている場合はそのローカル変数をhidden_localに対比させる
        if let Some(same_name_val) = self.local_vals.remove(&name) {
            // 現在のスコープですでに宣言されている場合はエラー
            if self.local_scope_depth == same_name_val.scope_depth {
                return Err(());
            } else {
                // すでに同じ変数名が複数宣言され, 秘匿済みの場合
                if let Some(same_name_vals) = self.hidden_local.get_mut(&name) {
                    // 新たに追加するローカル変数のスコープを抜けたら再度追加できるように最後尾に追加しする
                    same_name_vals.push(same_name_val);
                } else {
                    let hidden_vec = vec![same_name_val];
                    self.hidden_local.insert(name.clone(), hidden_vec);
                }
            }
        }

        // ローカル変数をスタックに追加すると8バイトアライメントを超えてしまう場合は,
        // スタックフレームをアライメント境界まで増やしてからローカル変数を追加する
        // すでにアライメント境界のときは何もしない
        if self.local_frame_size % 8 != 0 {
            if self.local_frame_size / 8 != (self.local_frame_size + type_.size) / 8 {
                self.local_frame_size += 8 - (self.local_frame_size % 8);
            }
        }

        // ローカル変数を必要な情報を追加して登録
        let val_size = type_.size;
        self.local_scopes[self.local_scope_depth]
            .scope_val_names
            .push(name.clone());
        let local_val = LocalVariable {
            scope_depth: self.local_scope_depth,
            frame_offset: self.local_frame_size,
            type_,
        };
        self.local_vals.insert(name, local_val);
        self.local_frame_size += val_size;
        Ok(())
    }

    // 使用中のローカル変数スタックサイズを取得
    pub fn get_local_frame_size(&self) -> usize {
        self.local_frame_size
    }

    // 変数を取得
    pub fn get_variable(&self, name: &String) -> Option<Variable> {
        if let Some(local_val) = self.local_vals.get(name) {
            Some(Variable::LocalVal(local_val.clone()))
        } else if let Some(global_val_type) = self.global_vals.get(name) {
            let global_val = GlobalVariable {
                name: name.clone(),
                type_: global_val_type.clone(),
            };
            Some(Variable::GlobalVal(global_val))
        } else {
            None
        }
    }

    // 新しいローカル変数スコープを作成する
    pub fn create_local_scope(&mut self) {
        let new_scope = LocalScope {
            frame_offset: self.local_frame_size,
            scope_val_names: vec![],
        };
        self.local_scopes.push(new_scope);
        self.local_scope_depth = self.local_scopes.len();
    }

    // 現在の(=最も深い)ローカル変数スコープから抜ける
    pub fn exit_local_scope(&mut self) {
        if let Some(exit_scope) = self.local_scopes.pop() {
            // スタックフレームサイズをスコープ開始時に戻す
            self.local_frame_size = exit_scope.frame_offset;
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
        }
    }
}
