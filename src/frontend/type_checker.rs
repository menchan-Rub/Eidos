use std::collections::HashMap;

use crate::core::{Result, EidosError, SourceLocation};
use crate::core::ast::{ASTNode, Node, Program, TypeInfo, NodeId};
use crate::core::types::{Type, TypeEnvironment};
use crate::core::symbol::{SymbolTable, SymbolId};

/// 型チェッカー
pub struct TypeChecker {
    type_env: TypeEnvironment,
    node_types: HashMap<NodeId, Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            type_env: TypeEnvironment::new(),
            node_types: HashMap::new(),
        }
    }
    
    /// ASTの型チェックを実行
    pub fn check(&mut self, program: Program) -> Result<Program> {
        // 型チェックの開始をログに記録
        info!("型チェックを実行中: {} ノード", program.node_count());
        
        // 型チェックの進捗を追跡するためのカウンター
        let mut checked_nodes = 0;
        let total_nodes = program.node_count();
        
        // 型チェックの開始時間を記録
        let start_time = std::time::Instant::now();
        
        // 型チェックの詳細ログを有効化
        debug!("型チェックの詳細ログを有効化しました");
        
        // 型チェックの前処理
        trace!("型チェックの前処理を開始");
        // 型環境の初期化
        self.initialize_type_environment(&program)?;
        
        // 前処理：型宣言の収集と登録
        self.collect_type_declarations(&program)?;
        
        // 関数シグネチャの収集（相互再帰関数のサポートのため）
        self.collect_function_signatures(&program)?;
        
        // 各ノードを走査して型付け（ボトムアップ方式）
        for node_id in program.traverse_post_order() {
            let node = program.get_node(node_id).ok_or_else(|| {
                EidosError::Internal(format!("ノードが見つかりません: {:?}", node_id))
            })?;
            
            let inferred_type = self.infer_node_type(&program, node)?;
            self.node_types.insert(node_id, inferred_type);
        }
        
        // 型制約の検証
        self.verify_type_constraints(&program)?;
        
        // ジェネリック型の具体化
        self.resolve_generic_types(&program)?;
        
        // 型情報を含むプログラムを生成
        let mut typed_program = program.clone();
        
        // 各ノードに型情報を付与
        for (node_id, type_info) in &self.node_types {
            if let Some(node) = typed_program.get_node_mut(*node_id) {
                node.type_info = Some(TypeInfo {
                    type_id: type_info.id,
                    resolved_type: type_info.clone(),
                });
            }
        }
        
        // 最終検証：未解決の型変数がないことを確認
        self.verify_no_unresolved_types(&typed_program)?;
        
        info!("型チェック完了: {} ノードを型付けしました", self.node_types.len());
        Ok(typed_program)
    }
    
    /// 型環境の初期化
    fn initialize_type_environment(&mut self, program: &Program) -> Result<()> {
        // 標準ライブラリの型を登録
        self.register_stdlib_types();
        
        // プログラム固有の初期化
        self.type_env.enter_scope();
        Ok(())
    }
    
    /// 標準ライブラリの型を登録
    fn register_stdlib_types(&mut self) {
        // 基本型の登録
        self.type_env.register_primitive_types();
        
        // コレクション型の登録（Vector, HashMap, HashSet, LinkedList, Queue, Stack, PriorityQueue）
        self.type_env.register_collection_types();
        
        // その他の標準ライブラリ型の登録
        self.type_env.register_io_types();
        self.type_env.register_math_types();
        self.type_env.register_string_types();
    }
    
    /// 型宣言の収集と登録
    fn collect_type_declarations(&mut self, program: &Program) -> Result<()> {
        // 構造体、列挙型、型エイリアスなどの宣言を収集
        for node_id in program.traverse_pre_order() {
            let node = program.get_node(node_id).ok_or_else(|| {
                EidosError::Internal(format!("ノードが見つかりません: {:?}", node_id))
            })?;
            
            match &node.kind {
                Node::StructDecl(struct_decl) => {
                    self.register_struct_declaration(struct_decl, &node.location)?;
                },
                Node::EnumDecl(enum_decl) => {
                    self.register_enum_declaration(enum_decl, &node.location)?;
                },
                Node::TypeAlias(alias) => {
                    self.register_type_alias(alias, &node.location)?;
                },
                _ => {}
            }
        }
        Ok(())
    }
    
    /// 関数シグネチャの収集
    fn collect_function_signatures(&mut self, program: &Program) -> Result<()> {
        for node_id in program.traverse_pre_order() {
            let node = program.get_node(node_id).ok_or_else(|| {
                EidosError::Internal(format!("ノードが見つかりません: {:?}", node_id))
            })?;
            
            if let Node::FunctionDecl(func_decl) = &node.kind {
                let func_type = self.create_function_type(func_decl, program)?;
                self.type_env.register_function(&func_decl.name, func_type);
            }
        }
        Ok(())
    }
    
    /// 型制約の検証
    fn verify_type_constraints(&self, program: &Program) -> Result<()> {
        // 型制約（インターフェース実装、ジェネリック制約など）を検証
        for (node_id, node_type) in &self.node_types {
            let node = program.get_node(*node_id).ok_or_else(|| {
                EidosError::Internal(format!("ノードが見つかりません: {:?}", node_id))
            })?;
            
            // 型アノテーションと推論された型の互換性チェック
            if let Some(type_annotation) = &node.type_annotation {
                let annotated_type = self.type_env.resolve_type_annotation(type_annotation)?;
                if !self.type_env.is_compatible(node_type, &annotated_type) {
                    return Err(EidosError::Type {
                        message: format!("型アノテーションと実際の型が一致しません: 期待 {:?}, 実際 {:?}", 
                                        annotated_type, node_type),
                        location: node.location.clone(),
                    });
                }
            }
        }
        Ok(())
    }
    
    /// ジェネリック型の具体化
    fn resolve_generic_types(&mut self, program: &Program) -> Result<()> {
        // 型推論の結果に基づいてジェネリック型パラメータを具体的な型に置き換える
        let mut substitutions = HashMap::new();
        
        // ジェネリック型の制約を収集
        for (node_id, node_type) in &self.node_types {
            let node = program.get_node(*node_id).ok_or_else(|| {
                EidosError::Internal(format!("ノードが見つかりません: {:?}", node_id))
            })?;
            
            self.collect_generic_constraints(node, node_type, &mut substitutions)?;
        }
        
        // 収集した制約に基づいて型を具体化
        for (node_id, node_type) in self.node_types.iter_mut() {
            *node_type = self.type_env.substitute_generic_types(node_type, &substitutions)?;
        }
        
        Ok(())
    }
    
    /// 未解決の型変数がないことを確認
    fn verify_no_unresolved_types(&self, program: &Program) -> Result<()> {
        for (node_id, node_type) in &self.node_types {
            if node_type.has_unknown_types() {
                let node = program.get_node(*node_id).ok_or_else(|| {
                    EidosError::Internal(format!("ノードが見つかりません: {:?}", node_id))
                })?;
                
                return Err(EidosError::Type {
                    message: format!("型推論に失敗しました: 未解決の型 {:?}", node_type),
                    location: node.location.clone(),
                });
            }
        }
        Ok(())
    }
    /// ノードの型を推論
    fn infer_node_type(&mut self, program: &Program, node: &ASTNode) -> Result<Type> {
        match &node.kind {
            Node::Literal(lit) => {
                // リテラルの型は簡単に決定できる
                Ok(match lit {
                    crate::core::ast::Literal::Int(_) => Type::int(),
                    crate::core::ast::Literal::Float(_) => Type::float(),
                    crate::core::ast::Literal::Bool(_) => Type::bool(),
                    crate::core::ast::Literal::Char(_) => Type::char(),
                    crate::core::ast::Literal::String(_) => Type::string(),
                    crate::core::ast::Literal::Unit => Type::unit(),
                    crate::core::ast::Literal::Array(elements) => {
                        if elements.is_empty() {
                            // 空配列は型を特定できないため、不明として扱う
                            Type::array(Type::unknown())
                        } else {
                            // 最初の要素から配列の型を推論
                            let first_elem_id = elements[0];
                            let first_elem = program.get_node(first_elem_id).ok_or_else(|| {
                                EidosError::Internal(format!("ノードが見つかりません: {:?}", first_elem_id))
                            })?;
                            let elem_type = self.infer_node_type(program, first_elem)?;
                            
                            // 他の要素が同じ型かチェック
                            for elem_id in elements.iter().skip(1) {
                                let elem = program.get_node(*elem_id).ok_or_else(|| {
                                    EidosError::Internal(format!("ノードが見つかりません: {:?}", elem_id))
                                })?;
                                let this_type = self.infer_node_type(program, elem)?;
                                if !self.type_env.is_assignable(&this_type, &elem_type) {
                                    return Err(EidosError::Type {
                                        message: format!("配列要素の型が一致しません: 期待 {:?}, 実際 {:?}", elem_type, this_type),
                                        location: elem.location.clone(),
                                    });
                                }
                            }
                            
                            Type::array(elem_type)
                        }
                    }
                })
            },
            Node::Variable(var_name) => {
                // 変数の型はシンボルテーブルから取得
                if let Some(var_type) = self.type_env.get_variable_type(var_name) {
                    Ok(var_type.clone())
                } else {
                    Err(EidosError::Type {
                        message: format!("未定義の変数: {}", var_name),
                        location: node.location.clone(),
                    })
                }
            },
            Node::BinaryOp { op, left, right } => {
                // 左右のオペランドの型を取得
                let left_node = program.get_node(*left).ok_or_else(|| {
                    EidosError::Internal(format!("ノードが見つかりません: {:?}", left))
                })?;
                let right_node = program.get_node(*right).ok_or_else(|| {
                    EidosError::Internal(format!("ノードが見つかりません: {:?}", right))
                })?;
                
                let left_type = self.infer_node_type(program, left_node)?;
                let right_type = self.infer_node_type(program, right_node)?;
                
                // 演算子に基づいて型を決定
                match op {
                    // 算術演算子
                    crate::core::ast::BinaryOp::Add | 
                    crate::core::ast::BinaryOp::Sub | 
                    crate::core::ast::BinaryOp::Mul | 
                    crate::core::ast::BinaryOp::Div | 
                    crate::core::ast::BinaryOp::Mod => {
                        if left_type.is_numeric() && right_type.is_numeric() {
                            // 数値同士の演算
                            if left_type.is_float() || right_type.is_float() {
                                // どちらかがfloatならfloat
                                Ok(Type::float())
                            } else {
                                // それ以外はint
                                Ok(Type::int())
                            }
                        } else if op == &crate::core::ast::BinaryOp::Add && 
                                  (left_type.is_string() || right_type.is_string()) {
                            // 文字列の連結はstringを返す
                            Ok(Type::string())
                        } else {
                            Err(EidosError::Type {
                                message: format!("不適切な演算: {:?} {:?} {:?}", left_type, op, right_type),
                                location: node.location.clone(),
                            })
                        }
                    },
                    // 比較演算子
                    crate::core::ast::BinaryOp::Eq | 
                    crate::core::ast::BinaryOp::Ne | 
                    crate::core::ast::BinaryOp::Lt | 
                    crate::core::ast::BinaryOp::Le | 
                    crate::core::ast::BinaryOp::Gt | 
                    crate::core::ast::BinaryOp::Ge => {
                        // 比較演算子はboolを返す
                        if self.type_env.is_comparable(&left_type, &right_type) {
                            Ok(Type::bool())
                        } else {
                            Err(EidosError::Type {
                                message: format!("比較できない型: {:?} {:?} {:?}", left_type, op, right_type),
                                location: node.location.clone(),
                            })
                        }
                    },
                    // 論理演算子
                    crate::core::ast::BinaryOp::And | 
                    crate::core::ast::BinaryOp::Or => {
                        if left_type.is_bool() && right_type.is_bool() {
                            Ok(Type::bool())
                        } else {
                            Err(EidosError::Type {
                                message: format!("論理演算子はブール型のみサポート: {:?} {:?} {:?}", left_type, op, right_type),
                                location: node.location.clone(),
                            })
                        }
                    },
                    _ => Err(EidosError::Type {
                        message: format!("未サポートの演算子: {:?}", op),
                        location: node.location.clone(),
                    }),
                }
            },
            Node::UnaryOp { op, operand } => {
                let operand_node = program.get_node(*operand).ok_or_else(|| {
                    EidosError::Internal(format!("ノードが見つかりません: {:?}", operand))
                })?;
                
                let operand_type = self.infer_node_type(program, operand_node)?;
                
                match op {
                    crate::core::ast::UnaryOp::Neg => {
                        if operand_type.is_numeric() {
                            Ok(operand_type.clone())
                        } else {
                            Err(EidosError::Type {
                                message: format!("負数化は数値型のみサポート: {:?}", operand_type),
                                location: node.location.clone(),
                            })
                        }
                    },
                    crate::core::ast::UnaryOp::Not => {
                        if operand_type.is_bool() {
                            Ok(Type::bool())
                        } else {
                            Err(EidosError::Type {
                                message: format!("論理否定はブール型のみサポート: {:?}", operand_type),
                                location: node.location.clone(),
                            })
                        }
                    },
                    _ => Err(EidosError::Type {
                        message: format!("未サポートの単項演算子: {:?}", op),
                        location: node.location.clone(),
                    }),
                }
            },
            Node::If { condition, then_branch, else_branch } => {
                // 条件式はブール型であるべき
                let cond_node = program.get_node(*condition).ok_or_else(|| {
                    EidosError::Internal(format!("ノードが見つかりません: {:?}", condition))
                })?;
                let cond_type = self.infer_node_type(program, cond_node)?;
                
                if !cond_type.is_bool() {
                    return Err(EidosError::Type {
                        message: format!("条件式はブール型である必要があります: {:?}", cond_type),
                        location: node.location.clone(),
                    });
                }
                
                // then節とelse節の型
                let then_node = program.get_node(*then_branch).ok_or_else(|| {
                    EidosError::Internal(format!("ノードが見つかりません: {:?}", then_branch))
                })?;
                let then_type = self.infer_node_type(program, then_node)?;
                
                if let Some(else_branch) = else_branch {
                    let else_node = program.get_node(*else_branch).ok_or_else(|| {
                        EidosError::Internal(format!("ノードが見つかりません: {:?}", else_branch))
                    })?;
                    let else_type = self.infer_node_type(program, else_node)?;
                    
                    // then節とelse節の型は互換性がなければならない
                    if self.type_env.is_assignable(&then_type, &else_type) {
                        Ok(then_type)
                    } else if self.type_env.is_assignable(&else_type, &then_type) {
                        Ok(else_type)
                    } else {
                        Err(EidosError::Type {
                            message: format!("if-elseの各分岐の型が一致しません: {:?} と {:?}", then_type, else_type),
                            location: node.location.clone(),
                        })
                    }
                } else {
                    // else節がない場合はunit型
                    Ok(Type::unit())
                }
            },
            Node::Block { statements, expression } => {
                // ブロック内の各文を処理
                for stmt_id in statements {
                    let stmt = program.get_node(*stmt_id).ok_or_else(|| {
                        EidosError::Internal(format!("ノードが見つかりません: {:?}", stmt_id))
                    })?;
                    self.infer_node_type(program, stmt)?;
                }
                
                // 最後の式があればその型、なければunit型
                if let Some(expr_id) = expression {
                    let expr = program.get_node(*expr_id).ok_or_else(|| {
                        EidosError::Internal(format!("ノードが見つかりません: {:?}", expr_id))
                    })?;
                    self.infer_node_type(program, expr)
                } else {
                    Ok(Type::unit())
                }
            },
            Node::Let { name, type_annotation, initializer } => {
                // 初期化子の型を推論
                let init_type = if let Some(init_id) = initializer {
                    let init_node = program.get_node(*init_id).ok_or_else(|| {
                        EidosError::Internal(format!("ノードが見つかりません: {:?}", init_id))
                    })?;
                    self.infer_node_type(program, init_node)?
                } else {
                    Type::unknown()
                };
                
                // 型注釈があればそれを使用
                let var_type = if let Some(type_id) = type_annotation {
                    let type_info = self.type_env.get_type(*type_id).ok_or_else(|| {
                        EidosError::Type {
                            message: format!("不明な型: {:?}", type_id),
                            location: node.location.clone(),
                        }
                    })?;
                    
                    // 初期化子がある場合は型チェック
                    if initializer.is_some() && !self.type_env.is_assignable(&init_type, type_info) {
                        return Err(EidosError::Type {
                            message: format!("型の不一致: 期待 {:?}, 実際 {:?}", type_info, init_type),
                            location: node.location.clone(),
                        });
                    }
                    
                    type_info.clone()
                } else {
                    // 型注釈がなければ初期化子の型
                    if init_type.is_unknown() {
                        return Err(EidosError::Type {
                            message: "型注釈か初期化子が必要です".to_string(),
                            location: node.location.clone(),
                        });
                    }
                    init_type.clone()
                };
                
                // 変数を型環境に登録
                self.type_env.set_variable_type(name.clone(), var_type.clone());
                
                // let式はunit型を返す
                Ok(Type::unit())
            },
            Node::Assign { target, value } => {
                // 左辺の型を取得
                let target_node = program.get_node(*target).ok_or_else(|| {
                    EidosError::Internal(format!("ノードが見つかりません: {:?}", target))
                })?;
                
                let target_type = self.infer_node_type(program, target_node)?;
                
                // 右辺の型を取得
                let value_node = program.get_node(*value).ok_or_else(|| {
                    EidosError::Internal(format!("ノードが見つかりません: {:?}", value))
                })?;
                
                let value_type = self.infer_node_type(program, value_node)?;
                
                // 型の互換性をチェック
                if !self.type_env.is_assignable(&value_type, &target_type) {
                    return Err(EidosError::Type {
                        message: format!("型の不一致: 期待 {:?}, 実際 {:?}", target_type, value_type),
                        location: node.location.clone(),
                    });
                }
                
                // 代入式はunit型を返す
                Ok(Type::unit())
            },
            Node::Function { name, parameters, return_type, body } => {
                // パラメータの型リスト
                let mut param_types = Vec::new();
                
                // パラメータを型環境に登録
                for param in parameters {
                    let param_type = if let Some(type_id) = param.type_id {
                        self.type_env.get_type(type_id).ok_or_else(|| {
                            EidosError::Type {
                                message: format!("不明な型: {:?}", type_id),
                                location: node.location.clone(),
                            }
                        })?.clone()
                    } else {
                        return Err(EidosError::Type {
                            message: format!("パラメータ{}に型注釈が必要です", param.name),
                            location: node.location.clone(),
                        });
                    };
                    
                    self.type_env.set_variable_type(param.name.clone(), param_type.clone());
                    param_types.push(param_type);
                }
                
                // 戻り値の型
                let ret_type = if let Some(type_id) = return_type {
                    self.type_env.get_type(*type_id).ok_or_else(|| {
                        EidosError::Type {
                            message: format!("不明な型: {:?}", type_id),
                            location: node.location.clone(),
                        }
                    })?.clone()
                } else {
                    // 戻り値の型が指定されていない場合はunit
                    Type::unit()
                };
                
                // 関数本体の型を推論
                let body_node = program.get_node(*body).ok_or_else(|| {
                    EidosError::Internal(format!("ノードが見つかりません: {:?}", body))
                })?;
                
                let body_type = self.infer_node_type(program, body_node)?;
                
                // 本体の型と戻り値の型が一致するか確認
                if !self.type_env.is_assignable(&body_type, &ret_type) {
                    return Err(EidosError::Type {
                        message: format!("戻り値の型の不一致: 期待 {:?}, 実際 {:?}", ret_type, body_type),
                        location: body_node.location.clone(),
                    });
                }
                
                // 関数型を作成して返す
                let func_type = Type::function(param_types, ret_type.clone());
                
                // 関数を型環境に登録
                self.type_env.set_variable_type(name.clone(), func_type.clone());
                
                Ok(func_type)
            },
            Node::Call { callee, arguments } => {
                // 関数オブジェクトの型を取得
                let callee_node = program.get_node(*callee).ok_or_else(|| {
                    EidosError::Internal(format!("ノードが見つかりません: {:?}", callee))
                })?;
                
                let callee_type = self.infer_node_type(program, callee_node)?;
                
                // 関数型であることを確認
                if !callee_type.is_function() {
                    return Err(EidosError::Type {
                        message: format!("呼び出し可能でない値: {:?}", callee_type),
                        location: node.location.clone(),
                    });
                }
                
                // 引数の型を取得
                let mut arg_types = Vec::new();
                for arg_id in arguments {
                    let arg_node = program.get_node(*arg_id).ok_or_else(|| {
                        EidosError::Internal(format!("ノードが見つかりません: {:?}", arg_id))
                    })?;
                    
                    let arg_type = self.infer_node_type(program, arg_node)?;
                    arg_types.push(arg_type);
                }
                
                // 関数定義から引数型と戻り値型を取得
                let (param_types, return_type) = callee_type.function_signature().ok_or_else(|| {
                    EidosError::Internal("関数型から関数シグネチャを取得できません".to_string())
                })?;
                
                // 引数の数が一致するか確認
                if arg_types.len() != param_types.len() {
                    return Err(EidosError::Type {
                        message: format!("引数の数が一致しません: 期待 {}, 実際 {}", param_types.len(), arg_types.len()),
                        location: node.location.clone(),
                    });
                }
                
                // 各引数の型が一致するか確認
                for (i, (arg_type, param_type)) in arg_types.iter().zip(param_types.iter()).enumerate() {
                    if !self.type_env.is_assignable(arg_type, param_type) {
                        return Err(EidosError::Type {
                            message: format!("引数 #{} の型が一致しません: 期待 {:?}, 実際 {:?}", i + 1, param_type, arg_type),
                            location: node.location.clone(),
                        });
                    }
                }
                
                // 戻り値の型を返す
                Ok(return_type.clone())
            },
            // その他のノード型の実装
            _ => {
                // デフォルトでは不明な型を返す
                Ok(Type::unknown())
            }
        }
    }
}

// ここでは簡易的にログマクロを定義（実際の実装では、crate::log などを使う）
macro_rules! info {
    ($($arg:tt)*) => {
        // 実際の実装ではログを出力する
        use log::{info as log_info};
        log_info!("{}", format!($($arg)*));
    };
} 