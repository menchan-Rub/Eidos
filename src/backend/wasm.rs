use std::collections::HashMap;
use std::convert::TryFrom;

use log::{debug, info, error};
use wasmtime::{Engine, Module as WasmModule, Store, Linker};
use wasmtime_wasi::WasiCtx;
use walrus::{Module as WalrusModule, ModuleConfig, ValType, FunctionBuilder, LocalId, TypeId};
use walrus::{ir::*, InstrSeqBuilder, InstrSeq, ExportItem, GlobalId, FunctionId};

use crate::core::{Result, EidosError};
use crate::core::eir::{Module, Function, BlockId, Instruction, Operand, Literal, BinaryOp, UnaryOp};
use crate::core::types::{Type, TypeKind};

use super::codegen::{Backend, CodegenOptions, OutputFormat, Target};

/// WebAssemblyバックエンド
pub struct WasmBackend {
    /// 型キャッシュ
    type_cache: HashMap<Type, ValType>,
    /// グローバル変数マップ
    global_map: HashMap<String, GlobalId>,
    /// 関数マップ
    function_map: HashMap<String, FunctionId>,
    /// WALRUSモジュール
    module: Option<WalrusModule>,
    /// 型情報キャッシュ
    type_info_cache: HashMap<String, Type>,
}

impl WasmBackend {
    /// 新しいWebAssemblyバックエンドを作成
    pub fn new() -> Self {
        Self {
            type_cache: HashMap::new(),
            global_map: HashMap::new(),
            function_map: HashMap::new(),
            module: None,
            type_info_cache: HashMap::new(),
        }
    }
    
    /// EidosのEIR型をWASM型に変換
    fn convert_type(&mut self, ty: &Type) -> Result<ValType> {
        // キャッシュにあれば返す
        if let Some(wasm_type) = self.type_cache.get(ty) {
            return Ok(*wasm_type);
        }
        
        // 型に応じて変換
        let wasm_type = match &ty.kind {
            TypeKind::Unit => {
                // Unit型は空として表現（I32を代用）
                ValType::I32
            },
            TypeKind::Bool => {
                // Bool型はI32（0/1）
                ValType::I32
            },
            TypeKind::Int => {
                // Int型はI64
                ValType::I64
            },
            TypeKind::Float => {
                // Float型はF64
                ValType::F64
            },
            TypeKind::Char => {
                // Char型はI32（UTF-32）
                ValType::I32
            },
            TypeKind::String | TypeKind::Array(_) | TypeKind::Tuple(_) | 
            TypeKind::Function { .. } | TypeKind::Struct { .. } | TypeKind::Enum { .. } => {
                // 複合型はメモリ上のアドレスとして表現（I32）
                ValType::I32
            },
            _ => {
                return Err(EidosError::CodeGen(format!(
                    "未対応の型: {:?}", ty
                )));
            }
        };
        
        // キャッシュに登録
        self.type_cache.insert(ty.clone(), wasm_type);
        
        Ok(wasm_type)
    }
    
    /// リテラル値をWASM命令に変換
    fn build_literal(&self, builder: &mut InstrSeqBuilder, literal: &Literal) -> Result<()> {
        match literal {
            Literal::Int(value) => {
                // WebAssemblyでは64ビット整数をi64_constで表現
                builder.i64_const(*value);
                Ok(())
            },
            Literal::Float(value) => {
                // 64ビット浮動小数点数はf64_constで表現
                builder.f64_const(*value);
                Ok(())
            },
            Literal::Bool(value) => {
                // ブール値はi32_constで0または1として表現
                builder.i32_const(*value as i32);
                Ok(())
            },
            Literal::Char(value) => {
                // 文字はUTF-32コードポイントとしてi32_constで表現
                builder.i32_const(*value as i32);
                Ok(())
            },
            Literal::String(value) => {
                // 文字列リテラルの処理
                // 1. データセクションに文字列を追加
                let module = if let Some(ref module) = self.module {
                    module
                } else {
                    return Err(EidosError::CodeGen("モジュールが初期化されていません".to_string()));
                };
                
                // バイト列に変換（UTF-8エンコーディング）
                let bytes = value.as_bytes().to_vec();
                let bytes_len = bytes.len() as u32;
                
                // 文字列データをデータセクションに追加
                let offset = self.add_string_to_data_section(module, &bytes)?;
                
                // スタックにオフセットと長さをプッシュ
                // まずオフセット（メモリ上の開始位置）
                builder.i32_const(offset as i32);
                
                // 必要に応じて長さも別に処理できるよう、コメントに残す
                // builder.i32_const(bytes_len as i32);
                
                Ok(())
            },
            Literal::Unit => {
                // Unit型は0として表現（void返り値など）
                builder.i32_const(0);
                Ok(())
            },
            Literal::Array(elements) => {
                // 配列リテラルの処理
                // メモリ上に配列データを配置するためのヘルパー関数が必要
                
                // 1. 要素数を計算
                let elem_count = elements.len();
                
                // 空配列の場合は特別処理
                if elem_count == 0 {
                    // 空配列はnullポインタとして表現
                    builder.i32_const(0);
                    return Ok(());
                }
                
                // 2. 要素サイズを決定（要素の型に基づいて）
                let elem_size = match elements.first() {
                    Some(Literal::Int(_)) => 8,      // i64型
                    Some(Literal::Float(_)) => 8,    // f64型
                    Some(Literal::Bool(_)) => 1,     // bool型
                    Some(Literal::Char(_)) => 4,     // char型（UTF-32）
                    Some(Literal::String(_)) => 4,   // 文字列（ポインタ）
                    Some(Literal::Array(_)) => 4,    // ネストされた配列（ポインタ）
                    _ => 4,                          // デフォルト
                };
                
                // 3. 必要なメモリサイズを計算
                let total_size = elem_count * elem_size;
                
                // 4. メモリを確保（malloc呼び出し）
                // 必要なメモリサイズをスタックにプッシュ
                builder.i32_const(total_size as i32);
                
                // malloc関数のIDを取得または宣言
                let malloc_func = self.get_or_declare_malloc(module)?;
                
                // malloc関数を呼び出し、メモリを確保
                builder.call(malloc_func);
                
                // 5. スタックトップに確保されたメモリのポインタがある
                let local_ptr = builder.local(ValType::I32); // ポインタを一時的にローカル変数に保存
                builder.local_set(local_ptr);
                
                // 6. 各要素をメモリに格納
                for (i, elem) in elements.iter().enumerate() {
                    // ポインタ + オフセットをスタックにプッシュ
                    builder.local_get(local_ptr);
                    
                    // オフセットを計算して足す（i * elem_size）
                    if i > 0 {
                        builder.i32_const((i * elem_size) as i32);
                        builder.i32_add();
                    }
                    
                    // 要素の値を評価
                    self.build_literal(builder, elem)?;
                    
                    // メモリに書き込む（要素の型に応じて異なる命令を使用）
                    match elem {
                        Literal::Int(_) => builder.i64_store(elem_size.trailing_zeros() as u32, 0),
                        Literal::Float(_) => builder.f64_store(elem_size.trailing_zeros() as u32, 0),
                        Literal::Bool(_) => builder.i32_store8(0, 0), // 1バイトとして格納
                        Literal::Char(_) => builder.i32_store(2, 0),  // 4バイトとして格納
                        Literal::String(_) => builder.i32_store(2, 0), // ポインタとして格納
                        Literal::Array(_) => builder.i32_store(2, 0),  // ポインタとして格納
                        _ => builder.i32_store(2, 0),                 // デフォルトは4バイト
                    };
                }
                
                // 7. 確保されたメモリのポインタをスタックにプッシュ
                builder.local_get(local_ptr);
                
                Ok(())
            },
            _ => Err(EidosError::CodeGen(format!("未対応のリテラル型: {:?}", literal))),
        }
    }
    
    /// 文字列データをデータセクションに追加
    fn add_string_to_data_section(&self, module: &WalrusModule, bytes: &[u8]) -> Result<u32> {
        // データセクションに格納するためのメモリオフセットを計算
        // 適切なオフセット管理の実装
        
        // 既存のデータセグメントからオフセットを計算
        let mut next_offset = 0;
        
        // モジュール内の全データセグメントを検査
        for data in module.data.iter() {
            if let Some(Value::I32(offset)) = data.kind.get_absolute_offset() {
                let segment_end = offset as u32 + data.value.len() as u32;
                // 最大値を更新
                next_offset = std::cmp::max(next_offset, segment_end);
            }
        }
        
        // 4バイトアライメントに調整
        next_offset = (next_offset + 3) & !3;
        
        // メモリセクションを取得または作成
        let memory = match module.memories.iter().next() {
            Some(memory) => memory.id(),
            None => {
                // メモリがなければ追加
                let memory_id = module.memories.add_local(false, 1, None);
                module.exports.add("memory", memory_id);
                memory_id
            }
        };
        
        // データセクションに文字列を追加
        module.data.add(
            memory,
            walrus::ir::Value::I32(next_offset as i32),
            bytes.to_vec(),
        );
        
        Ok(next_offset)
    }
    
    /// 静的なメモリ領域を確保し、オフセットを返す
    fn allocate_static_memory(&self, module: &WalrusModule, size: u32) -> Result<u32> {
        // 既存のデータセグメントからオフセットを計算
        let mut next_offset = 0;
        
        // モジュール内の全データセグメントを検査
        for data in module.data.iter() {
            if let Some(Value::I32(offset)) = data.kind.get_absolute_offset() {
                let segment_end = offset as u32 + data.value.len() as u32;
                // 最大値を更新
                next_offset = std::cmp::max(next_offset, segment_end);
            }
        }
        
        // アライメント調整
        next_offset = (next_offset + 7) & !7; // 8バイトアライメント
        
        // 将来的なメモリアクセスのために領域を予約
        // （実際にはリザーブするだけで初期化はしない）
        
        Ok(next_offset)
    }
    
    /// 静的なメモリにリテラル値を初期化
    fn initialize_static_memory(&self, module: &WalrusModule, offset: u32, literal: &Literal) -> Result<()> {
        // メモリセクションを取得
        let memory = match module.memories.iter().next() {
            Some(memory) => memory.id(),
            None => {
                return Err(EidosError::CodeGen("メモリセクションが見つかりません".to_string()));
            }
        };
        
        // リテラル型に応じてバイト列に変換
        let bytes = match literal {
            Literal::Int(value) => {
                // i64をバイト列に変換
                value.to_le_bytes().to_vec()
            },
            Literal::Float(value) => {
                // f64をバイト列に変換
                value.to_le_bytes().to_vec()
            },
            Literal::Bool(value) => {
                // ブール値を1バイトとして保存（0または1）
                vec![*value as u8]
            },
            Literal::Char(value) => {
                // 文字をUTF-32コードポイントとして保存
                (*value as u32).to_le_bytes().to_vec()
            },
            Literal::String(value) => {
                // 文字列の場合、文字列データは別の場所に配置し、そのポインタを保存
                let string_bytes = value.as_bytes();
                let string_offset = self.add_string_to_data_section(module, string_bytes)?;
                
                // ポインタ（オフセット）を保存
                (string_offset as u32).to_le_bytes().to_vec()
            },
            Literal::Unit => {
                // Unit型は0として表現
                vec![0, 0, 0, 0]
            },
            Literal::Array(_) => {
                // 配列は再帰的に処理する必要がある（ここでは単純に0を返す）
                vec![0, 0, 0, 0]
            },
        };
        
        // データセクションに追加
        module.data.add(
            memory,
            walrus::ir::Value::I32(offset as i32),
            bytes,
        );
        
        Ok(())
    }
    
    /// 命令をWASMバイトコードに変換
    fn build_instruction(&self, 
                        module: &mut WalrusModule,
                        builder: &mut InstrSeqBuilder, 
                        instr: &Instruction,
                        locals: &HashMap<String, LocalId>) -> Result<()> {
        match instr {
            Instruction::BinaryOp { op, lhs, rhs, result } => {
                // オペランドをスタックにプッシュ
                self.build_operand(module, builder, lhs, locals)?;
                self.build_operand(module, builder, rhs, locals)?;
                
                // 演算子に応じた命令を追加
                match op {
                    BinaryOp::Add => builder.i64_add(),
                    BinaryOp::Sub => builder.i64_sub(),
                    BinaryOp::Mul => builder.i64_mul(),
                    BinaryOp::Div => builder.i64_div_s(),
                    BinaryOp::Rem => builder.i64_rem_s(),
                    BinaryOp::Eq => {
                        builder.i64_eq();
                        // boolをi32として表現
                        builder.i32_from_i64();
                    },
                    BinaryOp::Ne => {
                        builder.i64_ne();
                        // boolをi32として表現
                        builder.i32_from_i64();
                    },
                    BinaryOp::Lt => {
                        builder.i64_lt_s();
                        // boolをi32として表現
                        builder.i32_from_i64();
                    },
                    BinaryOp::Le => {
                        builder.i64_le_s();
                        // boolをi32として表現
                        builder.i32_from_i64();
                    },
                    BinaryOp::Gt => {
                        builder.i64_gt_s();
                        // boolをi32として表現
                        builder.i32_from_i64();
                    },
                    BinaryOp::Ge => {
                        builder.i64_ge_s();
                        // boolをi32として表現
                        builder.i32_from_i64();
                    },
                    BinaryOp::And => {
                        // i64の論理積
                        builder.i64_and();
                        // boolをi32として表現
                        builder.i32_from_i64();
                    },
                    BinaryOp::Or => {
                        // i64の論理和
                        builder.i64_or();
                        // boolをi32として表現
                        builder.i32_from_i64();
                    },
                    _ => {
                        return Err(EidosError::CodeGen(format!(
                            "未対応のバイナリ演算子: {:?}", op
                        )));
                    }
                }
                
                // 結果をローカル変数に格納
                if let Some(local_id) = locals.get(&result.to_string()) {
                    builder.local_set(*local_id);
                } else {
                    return Err(EidosError::CodeGen(format!(
                        "ローカル変数が見つかりません: {:?}", result
                    )));
                }
            },
            Instruction::UnaryOp { op, operand, result } => {
                // オペランドをスタックにプッシュ
                self.build_operand(module, builder, operand, locals)?;
                
                // 演算子に応じた命令を追加
                match op {
                    UnaryOp::Neg => {
                        // 符号反転
                        if operand.is_int_type() {
                            builder.i64_const(0);
                            builder.i64_sub();
                        } else if operand.is_float_type() {
                            builder.f64_neg();
                        } else {
                            return Err(EidosError::CodeGen("負数化は整数か浮動小数点数のみサポートされています".to_string()));
                        }
                    },
                    UnaryOp::Not => {
                        if operand.is_bool_type() {
                            // 論理否定（i32で表現）
                            builder.i32_const(1);
                            builder.i32_xor();
                        } else {
                            return Err(EidosError::CodeGen("論理否定はブール値のみサポートされています".to_string()));
                        }
                    },
                    _ => {
                        return Err(EidosError::CodeGen(format!(
                            "未対応の単項演算子: {:?}", op
                        )));
                    }
                }
                
                // 結果をローカル変数に格納
                if let Some(local_id) = locals.get(&result.to_string()) {
                    builder.local_set(*local_id);
                } else {
                    return Err(EidosError::CodeGen(format!(
                        "ローカル変数が見つかりません: {:?}", result
                    )));
                }
            },
            Instruction::Load { address, result } => {
                // アドレスをスタックにプッシュ
                self.build_operand(module, builder, address, locals)?;
                
                // メモリからロード（i64として）
                builder.i64_load(0, 0);
                
                // 結果をローカル変数に格納
                if let Some(local_id) = locals.get(&result.to_string()) {
                    builder.local_set(*local_id);
                } else {
                    return Err(EidosError::CodeGen(format!(
                        "ローカル変数が見つかりません: {:?}", result
                    )));
                }
            },
            Instruction::Store { address, value } => {
                // アドレスをスタックにプッシュ
                self.build_operand(module, builder, address, locals)?;
                
                // 値をスタックにプッシュ
                self.build_operand(module, builder, value, locals)?;
                
                // メモリにストア（i64として）
                builder.i64_store(0, 0);
            },
            Instruction::Call { func, args, result } => {
                // 引数をスタックにプッシュ
                for arg in args {
                    self.build_operand(module, builder, arg, locals)?;
                }
                
                // 関数の呼び出し
                match func {
                    Operand::FunctionRef(name) => {
                        if let Some(func_id) = self.function_map.get(name) {
                            builder.call(*func_id);
                        } else {
                            return Err(EidosError::CodeGen(format!(
                                "関数が見つかりません: {:?}", name
                            )));
                        }
                    },
                    Operand::Variable(name) => {
                        // 関数ポインタを介した間接呼び出し
                        // 1. 関数テーブルの存在を確認
                        let table_id = self.ensure_function_table(module)?;
                        
                        // 2. 関数ポインタをスタックにプッシュ（テーブルインデックス）
                        if let Some(local_id) = locals.get(name) {
                            builder.local_get(*local_id);
                        } else {
                            return Err(EidosError::CodeGen(format!(
                                "関数ポインタが見つかりません: {:?}", name
                            )));
                        }
                        
                        // 3. 関数型を推測（引数と戻り値の型から）
                        let param_types: Vec<ValType> = args.iter()
                            .map(|arg| self.get_operand_type(arg))
                            .collect();
                        
                        let return_type = if result.is_empty() {
                            vec![] // 戻り値なし
                        } else {
                            vec![ValType::I64] // デフォルトはi64（実際には型情報を参照すべき）
                        };
                        
                        // 4. 型インデックスを取得
                        let type_id = module.types.get_type_id(&param_types, &return_type)
                            .unwrap_or_else(|| {
                                // 見つからない場合は新しく追加
                                module.types.add(&param_types, &return_type)
                            });
                        
                        // 5. 間接呼び出し
                        builder.call_indirect(table_id, type_id);
                    },
                    _ => {
                        return Err(EidosError::CodeGen("関数参照または関数ポインタが必要です".to_string()));
                    }
                }
                
                // 戻り値がある場合は結果に格納
                if let Some(local_id) = locals.get(&result.to_string()) {
                    builder.local_set(*local_id);
                }
            },
            Instruction::Return { value } => {
                // 戻り値がある場合はスタックにプッシュ
                if let Some(val) = value {
                    self.build_operand(module, builder, val, locals)?;
                }
                // 戻り命令
                builder.return_();
            },
            Instruction::Branch { target } => {
                // ブロックへのジャンプ
                builder.br(*target);
            },
            Instruction::ConditionalBranch { condition, true_target, false_target } => {
                // 条件をスタックにプッシュ
                self.build_operand(module, builder, condition, locals)?;
                
                // 条件分岐
                builder.if_else(
                    Some(ValType::I32),
                    |then_builder| {
                        then_builder.br(*true_target);
                        Ok(())
                    },
                    |else_builder| {
                        else_builder.br(*false_target);
                        Ok(())
                    }
                )?;
            },
            Instruction::Alloca { ty, result } => {
                // WebAssemblyではスタック上の割り当てを直接サポートしていないため、
                // 線形メモリに割り当てる必要がある
                
                // メモリ割り当て関数を実装
                
                // 型サイズを計算
                let type_size = self.get_type_size(ty);
                
                // サイズをスタックにプッシュ
                builder.i32_const(type_size);
                
                // アロケーション関数を取得または宣言
                let alloc_func = if let Some(func_id) = self.function_map.get("__alloc") {
                    *func_id
                } else {
                    // __alloc関数がない場合はmallocを使用
                    if let Some(malloc_func) = self.function_map.get("malloc") {
                        *malloc_func
                    } else {
                        // malloc関数も見つからない場合は自動的に宣言して追加
                        if let Some(ref mut module) = module {
                            let malloc_type_id = module.types.add(&[ValType::I32], &[ValType::I32]);
                            let malloc_func_id = module.imports.add_function("env", "malloc", malloc_type_id);
                            self.function_map.insert("malloc".to_string(), malloc_func_id);
                            malloc_func_id
                        } else {
                            return Err(EidosError::CodeGen("メモリ割り当て関数を宣言できません".to_string()));
                        }
                    }
                };
                
                // アロケーション関数呼び出し
                builder.call(alloc_func);
                
                // アドレスをローカル変数に格納
                if let Some(local_id) = locals.get(&result.to_string()) {
                    builder.local_set(*local_id);
                } else {
                    return Err(EidosError::CodeGen(format!(
                        "ローカル変数が見つかりません: {:?}", result
                    )));
                }
            },
            Instruction::Load { ptr, result } => {
                // ポインタをスタックにプッシュ
                self.build_operand(module, builder, ptr, locals)?;
                
                // メモリからロード
                let val_type = self.get_operand_type(ptr);
                
                // アドレスアライメントヒントを追加（最適化のため）
                let alignment = match val_type {
                    ValType::I32 => 4, // 4バイトアライメント
                    ValType::I64 => 8, // 8バイトアライメント
                    ValType::F32 => 4, // 4バイトアライメント
                    ValType::F64 => 8, // 8バイトアライメント
                    _ => 1,            // デフォルトは1バイト
                };
                
                debug!("メモリロード: 型={:?}, アライメント={}", val_type, alignment);
                
                // メモリアクセス命令
                match val_type {
                    ValType::I32 => builder.i32_load(alignment.trailing_zeros() as u32, 0),
                    ValType::I64 => builder.i64_load(alignment.trailing_zeros() as u32, 0),
                    ValType::F32 => builder.f32_load(alignment.trailing_zeros() as u32, 0),
                    ValType::F64 => builder.f64_load(alignment.trailing_zeros() as u32, 0),
                    _ => return Err(EidosError::CodeGen(format!("未対応のロード型: {:?}", val_type))),
                }
                
                // 結果をローカル変数に格納
                if let Some(local_id) = locals.get(&result.to_string()) {
                    builder.local_set(*local_id);
                } else {
                    return Err(EidosError::CodeGen(format!(
                        "ローカル変数が見つかりません: {:?}", result
                    )));
                }
            },
            Instruction::Store { ptr, value } => {
                // ポインタと値をスタックにプッシュ
                self.build_operand(module, builder, ptr, locals)?;
                self.build_operand(module, builder, value, locals)?;
                
                // 値の型に基づいてストア命令を選択
                let val_type = self.get_operand_type(value);
                
                // アドレスアライメントヒントを追加（最適化のため）
                let alignment = match val_type {
                    ValType::I32 => 4, // 4バイトアライメント
                    ValType::I64 => 8, // 8バイトアライメント
                    ValType::F32 => 4, // 4バイトアライメント
                    ValType::F64 => 8, // 8バイトアライメント
                    _ => 1,            // デフォルトは1バイト
                };
                
                debug!("メモリストア: 型={:?}, アライメント={}", val_type, alignment);
                
                // メモリアクセス命令
                match val_type {
                    ValType::I32 => builder.i32_store(alignment.trailing_zeros() as u32, 0),
                    ValType::I64 => builder.i64_store(alignment.trailing_zeros() as u32, 0),
                    ValType::F32 => builder.f32_store(alignment.trailing_zeros() as u32, 0),
                    ValType::F64 => builder.f64_store(alignment.trailing_zeros() as u32, 0),
                    _ => return Err(EidosError::CodeGen(format!("未対応のストア型: {:?}", val_type))),
                }
            },
            Instruction::GetElementPtr { ptr, indices, result } => {
                // GEP命令はポインタ計算
                // ベースポインタをスタックにプッシュ
                self.build_operand(module, builder, ptr, locals)?;
                
                // ベースポインタのメモリレイアウト情報を取得
                let base_type = if let Some(base_type) = self.get_ptr_element_type(ptr) {
                    base_type
                } else {
                    return Err(EidosError::CodeGen(format!(
                        "ポインタのベース型を特定できません: {:?}", ptr
                    )));
                };
                
                debug!("GEP: ベース型={:?}, インデックス数={}", base_type, indices.len());
                
                // 各インデックスを計算して適用
                let mut current_offset = 0;
                let mut current_stride = self.get_type_size(&base_type);
                
                for (i, idx) in indices.iter().enumerate() {
                    match idx {
                        Operand::Literal(Literal::Int(value)) => {
                            // 定数インデックスの場合は直接オフセット計算
                            let offset = current_stride * (*value as i32);
                            current_offset += offset;
                            
                            debug!("GEP: 定数インデックス[{}]={}, オフセット={}, 累積={}", 
                                   i, value, offset, current_offset);
                            
                            // 次のストライドを計算（複合型の場合）
                            if i == 0 && indices.len() > 1 {
                                if let TypeKind::Array(elem_type) = &base_type.kind {
                                    current_stride = self.get_type_size(elem_type);
                                    debug!("GEP: 配列要素のストライド={}", current_stride);
                                } else if let TypeKind::Struct { fields, .. } = &base_type.kind {
                                    if *value >= 0 && (*value as usize) < fields.len() {
                                        let field = &fields[*value as usize];
                                        current_stride = self.get_type_size(&field.field_type);
                                        debug!("GEP: 構造体フィールドのストライド={}", current_stride);
                                    } else {
                                        return Err(EidosError::CodeGen(format!(
                                            "構造体インデックスが範囲外: {} (フィールド数: {})", value, fields.len()
                                        )));
                                    }
                                }
                            }
                        },
                        _ => {
                            // 変数インデックスの場合はランタイム計算
                            if i == 0 {
                                // 最初のインデックスはバイト単位のオフセットではなく要素インデックス
                                self.build_operand(module, builder, idx, locals)?;
                                
                                // インデックスをスケール（要素サイズを掛ける）
                                if current_stride != 1 {
                                    builder.i32_const(current_stride);
                                    builder.i32_mul();
                                }
                                
                                // これを現在のアドレスに加算
                                builder.i32_add();
                                
                                // 次の要素のサイズを計算
                                if let TypeKind::Array(elem_type) = &base_type.kind {
                                    current_stride = self.get_type_size(elem_type);
                                }
                            } else {
                                // 2番目以降のインデックスは現在のポインタからの相対計算
                                self.build_operand(module, builder, idx, locals)?;
                                
                                // インデックスをスケール
                                if current_stride != 1 {
                                    builder.i32_const(current_stride);
                                    builder.i32_mul();
                                }
                                
                                // これを現在のアドレスに加算
                                builder.i32_add();
                            }
                            
                            // 定数オフセットをクリア（変数インデックスで計算したため）
                            current_offset = 0;
                        }
                    }
                }
                
                // 最終的な定数オフセットを加算
                if current_offset != 0 {
                    builder.i32_const(current_offset);
                    builder.i32_add();
                }
                
                // 結果をローカル変数に格納
                if let Some(local_id) = locals.get(&result.to_string()) {
                    builder.local_set(*local_id);
                } else {
                    return Err(EidosError::CodeGen(format!(
                        "ローカル変数が見つかりません: {:?}", result
                    )));
                }
            },
            _ => {
                return Err(EidosError::CodeGen(format!(
                    "未対応の命令: {:?}", instr
                )));
            }
        }
        
        Ok(())
    }
    
    /// オペランドをWASMバイトコードに変換
    fn build_operand(&self, 
                    module: &mut WalrusModule,
                    builder: &mut InstrSeqBuilder, 
                    operand: &Operand,
                    locals: &HashMap<String, LocalId>) -> Result<()> {
        match operand {
            Operand::Literal(literal) => {
                self.build_literal(builder, literal)?;
            },
            Operand::Variable(name) => {
                if let Some(local_id) = locals.get(name) {
                    builder.local_get(*local_id);
                } else {
                    return Err(EidosError::CodeGen(format!(
                        "ローカル変数が見つかりません: {:?}", name
                    )));
                }
            },
            Operand::GlobalRef(name) => {
                if let Some(global_id) = self.global_map.get(name) {
                    builder.global_get(*global_id);
                } else {
                    return Err(EidosError::CodeGen(format!(
                        "グローバル変数が見つかりません: {:?}", name
                    )));
                }
            },
            Operand::FunctionRef(name) => {
                // 関数参照は関数テーブルを通じた間接呼び出しに変換
                if let Some(func_id) = self.function_map.get(name) {
                    // 関数テーブルが存在するか確認
                    let table_id = self.ensure_function_table(module)?;
                    
                    // 関数インデックスをテーブルに追加
                    let elem_idx = self.add_function_to_table(module, *func_id)?;
                    
                    // テーブルインデックスをスタックにプッシュ
                    builder.i32_const(elem_idx as i32);
                } else {
                    return Err(EidosError::CodeGen(format!(
                        "関数が見つかりません: {:?}", name
                    )));
                }
            },
            _ => {
                return Err(EidosError::CodeGen(format!(
                    "未対応のオペランド: {:?}", operand
                )));
            }
        }
        
        Ok(())
    }
    
    /// 関数テーブルが存在することを確認し、存在しなければ作成
    fn ensure_function_table(&self, module: &mut WalrusModule) -> Result<walrus::TableId> {
        // 既存のテーブルを探す
        for table in module.tables.iter() {
            return Ok(table.id());
        }
        
        // テーブルが存在しない場合は新しく作成
        let table_id = module.tables.add_local(
            walrus::ValType::Funcref,  // 関数参照型
            10,                         // 初期サイズ
            Some(1000),                // 最大サイズ
            None,                      // 初期値（なし）
        );
        
        // テーブルをエクスポート
        module.exports.add("__function_table", table_id);
        
        Ok(table_id)
    }
    
    /// 関数をテーブルに追加し、そのインデックスを返す
    fn add_function_to_table(&self, module: &mut WalrusModule, func_id: FunctionId) -> Result<u32> {
        // テーブル要素セクションを取得または作成
        let table_id = self.ensure_function_table(module)?;
        
        // この関数のインデックスを決定（現在のサイズ）
        let idx = module.elements.iter().count() as u32;
        
        // テーブル要素を追加
        module.elements.add(
            table_id,
            walrus::ElementKind::Active {
                table: table_id,
                offset: walrus::ir::Value::I32(idx as i32),
            },
            walrus::ValType::Funcref,
            vec![Some(func_id)],
        );
        
        Ok(idx)
    }
    
    /// オペランドのWASM型を取得
    fn get_operand_type(&self, operand: &Operand) -> ValType {
        match operand {
            Operand::Literal(Literal::Int(_)) => ValType::I64,
            Operand::Literal(Literal::Float(_)) => ValType::F64,
            Operand::Literal(Literal::Bool(_)) => ValType::I32,
            Operand::Literal(Literal::Char(_)) => ValType::I32,
            Operand::Literal(Literal::String(_)) => ValType::I32, // ポインタとして
            Operand::Literal(Literal::Unit) => ValType::I32,      // 0として
            _ => ValType::I32, // デフォルト
        }
    }
    
    /// 型のサイズを取得
    fn get_type_size(&self, ty: &Type) -> i32 {
        match &ty.kind {
            TypeKind::Int => 8,
            TypeKind::Float => 8,
            TypeKind::Bool => 1,
            TypeKind::Char => 4,
            TypeKind::String => 8, // ポインタとサイズとして
            TypeKind::Unit => 0,
            TypeKind::Array(_) => 8, // ポインタとして
            TypeKind::Tuple(_) => 8, // ポインタとして
            TypeKind::Function { .. } => 8, // ポインタとして
            TypeKind::Struct { .. } => 8, // ポインタとして
            TypeKind::Enum { .. } => 8, // タグとペイロードポインタ
            _ => 4, // デフォルト
        }
    }
    
    /// オペランドが整数型かどうか
    fn is_int_type(&self, operand: &Operand) -> bool {
        match operand {
            Operand::Literal(Literal::Int(_)) => true,
            _ => false,
        }
    }
    
    /// オペランドが浮動小数点型かどうか
    fn is_float_type(&self, operand: &Operand) -> bool {
        match operand {
            Operand::Literal(Literal::Float(_)) => true,
            _ => false,
        }
    }
    
    /// オペランドがブール型かどうか
    fn is_bool_type(&self, operand: &Operand) -> bool {
        match operand {
            Operand::Literal(Literal::Bool(_)) => true,
            _ => false,
        }
    }
    
    /// ポインタが指す要素の型を取得
    fn get_ptr_element_type(&self, ptr: &Operand) -> Option<Type> {
        match ptr {
            Operand::Variable(name) => {
                // 変数名から型情報を取得
                // 型情報データベースを使用
                // 型情報がキャッシュにあればそれを使用
                if let Some(type_info) = self.get_variable_type_info(name) {
                    if let TypeKind::Pointer(elem_type) = &type_info.kind {
                        return Some(elem_type.as_ref().clone());
                    }
                }
                
                // キャッシュにない場合は名前から推測
                if name.contains("array") || name.contains("vector") {
                    // 配列型と推測
                    Some(Type::array(Type::int()))
                } else if name.contains("struct") {
                    // 構造体型と推測
                    let mut fields = Vec::new();
                    for i in 0..3 {  // 仮に3フィールドと仮定
                        fields.push(crate::core::types::StructField {
                            name: format!("field{}", i),
                            field_type: Type::int(),
                        });
                    }
                    Some(Type::struct_type("anonymous", fields, false))
                } else {
                    // 整数型と推測
                    Some(Type::int())
                }
            },
            Operand::GlobalRef(name) => {
                // グローバル変数の型情報
                if let Some(type_info) = self.get_global_type_info(name) {
                    if let TypeKind::Pointer(elem_type) = &type_info.kind {
                        return Some(elem_type.as_ref().clone());
                    }
                    return Some(type_info);
                }
                Some(Type::int())  // デフォルト
            },
            Operand::FunctionRef(name) => {
                // 関数ポインタ
                if let Some(func_type) = self.get_function_type_info(name) {
                    return Some(func_type);
                }
                Some(Type::function(vec![], Type::unit()))
            },
            _ => None,
        }
    }
    
    /// 変数の型情報を取得
    fn get_variable_type_info(&self, name: &str) -> Option<Type> {
        // 型情報キャッシュを検索
        if let Some(ty) = self.type_info_cache.get(name) {
            return Some(ty.clone());
        }
        
        // キャッシュにない場合は変数名から型を推測
        let inferred_type = if name.ends_with("_i") || name.ends_with("_int") {
            Type::int()
        } else if name.ends_with("_f") || name.ends_with("_float") {
            Type::float()
        } else if name.ends_with("_b") || name.ends_with("_bool") {
            Type::bool()
        } else if name.ends_with("_c") || name.ends_with("_char") {
            Type::char()
        } else if name.ends_with("_s") || name.ends_with("_str") {
            Type::string()
        } else if name.contains("_arr") || name.contains("_array") {
            // 配列の場合、要素の型も推測
            let elem_type = if name.contains("_i_") || name.contains("_int_") {
                Type::int()
            } else if name.contains("_f_") || name.contains("_float_") {
                Type::float()
            } else if name.contains("_b_") || name.contains("_bool_") {
                Type::bool()
            } else {
                Type::unknown()
            };
            Type::array(elem_type)
        } else if name.contains("_ptr") {
            // ポインタの場合、指す型も推測
            let pointed_type = if name.contains("_i_") || name.contains("_int_") {
                Type::int()
            } else if name.contains("_f_") || name.contains("_float_") {
                Type::float()
            } else if name.contains("_b_") || name.contains("_bool_") {
                Type::bool()
            } else {
                Type::unknown()
            };
            Type::pointer(pointed_type)
        } else {
            // デフォルトは整数型
            Type::int()
        };
        
        // 推測した型をキャッシュに追加（mutableな参照のため、cloneしてから渡す）
        let mut type_info_cache = self.type_info_cache.clone();
        type_info_cache.insert(name.to_string(), inferred_type.clone());
        
        // このインスタンスに反映（unsafe操作を回避するための方法）
        std::mem::replace(&mut *(self as *mut Self).type_info_cache, type_info_cache);
        
        Some(inferred_type)
    }
    
    /// グローバル変数の型情報を取得
    fn get_global_type_info(&self, name: &str) -> Option<Type> {
        // 型情報キャッシュを検索
        if let Some(ty) = self.type_info_cache.get(name) {
            return Some(ty.clone());
        }
        
        // キャッシュにない場合は変数と同様の規則で推測
        self.get_variable_type_info(name)
    }
    
    /// 関数の型情報を取得
    fn get_function_type_info(&self, name: &str) -> Option<Type> {
        // 実際の実装では型情報データベースを参照します
        
        // 関数名から引数と戻り値の型を推測
        let mut param_types = Vec::new();
        let ret_type;
        
        // 戻り値型を推測
        if name.starts_with("get_") || name.contains("_to_") {
            if name.ends_with("_i") || name.ends_with("_int") {
                ret_type = Type::int();
            } else if name.ends_with("_f") || name.ends_with("_float") {
                ret_type = Type::float();
            } else if name.ends_with("_b") || name.ends_with("_bool") {
                ret_type = Type::bool();
            } else if name.ends_with("_s") || name.ends_with("_str") {
                ret_type = Type::string();
            } else {
                ret_type = Type::unknown();
            }
        } else if name.starts_with("is_") || name.starts_with("has_") || name.starts_with("check_") {
            ret_type = Type::bool();
        } else if name.starts_with("set_") || name.starts_with("create_") || name.starts_with("delete_") {
            ret_type = Type::unit();
        } else {
            ret_type = Type::unknown();
        }
        
        // 引数型は詳細な情報がないと難しいので、デフォルトは空リスト
        Some(Type::function(param_types, ret_type))
    }
    
    /// メモリ確保用のmalloc関数を取得または宣言
    fn get_or_declare_malloc(&self, module: &mut WalrusModule) -> Result<FunctionId> {
        if let Some(&func_id) = self.function_map.get("malloc") {
            return Ok(func_id);
        }
        
        // malloc関数が見つからない場合は自動的に宣言して追加
        let malloc_type_id = module.types.add(&[ValType::I32], &[ValType::I32]);
        let malloc_func_id = module.imports.add_function("env", "malloc", malloc_type_id);
        
        // 関数マップに追加（immutable borrowチェックを回避するため、cloneが必要）
        let mut function_map = self.function_map.clone();
        function_map.insert("malloc".to_string(), malloc_func_id);
        
        // このインスタンスに反映（unsafe操作を回避するための方法）
        std::mem::replace(&mut *(self as *mut Self).function_map, function_map);
        
        Ok(malloc_func_id)
    }
    
    /// メモリ解放用のfree関数を取得または宣言
    fn get_or_declare_free(&self, module: &mut WalrusModule) -> Result<FunctionId> {
        if let Some(&func_id) = self.function_map.get("free") {
            return Ok(func_id);
        }
        
        // free関数が見つからない場合は自動的に宣言して追加
        let free_type_id = module.types.add(&[ValType::I32], &[]);
        let free_func_id = module.imports.add_function("env", "free", free_type_id);
        
        // 関数マップに追加（immutable borrowチェックを回避するため、cloneが必要）
        let mut function_map = self.function_map.clone();
        function_map.insert("free".to_string(), free_func_id);
        
        // このインスタンスに反映（unsafe操作を回避するための方法）
        std::mem::replace(&mut *(self as *mut Self).function_map, function_map);
        
        Ok(free_func_id)
    }
    
    /// メモリ再割り当て用のrealloc関数を取得または宣言
    fn get_or_declare_realloc(&self, module: &mut WalrusModule) -> Result<FunctionId> {
        if let Some(&func_id) = self.function_map.get("realloc") {
            return Ok(func_id);
        }
        
        // realloc関数が見つからない場合は自動的に宣言して追加
        let realloc_type_id = module.types.add(&[ValType::I32, ValType::I32], &[ValType::I32]);
        let realloc_func_id = module.imports.add_function("env", "realloc", realloc_type_id);
        
        // 関数マップに追加
        let mut function_map = self.function_map.clone();
        function_map.insert("realloc".to_string(), realloc_func_id);
        
        // このインスタンスに反映
        std::mem::replace(&mut *(self as *mut Self).function_map, function_map);
        
        Ok(realloc_func_id)
    }
    
    /// 関数ポインタ（コールバック）を作成
    fn create_function_pointer(&self, module: &mut WalrusModule, func_id: FunctionId) -> Result<i32> {
        // 関数テーブルの存在を確認
        let table_id = self.ensure_function_table(module)?;
        
        // 関数をテーブルに追加し、インデックスを取得
        let func_index = self.add_function_to_table(module, func_id)?;
        
        Ok(func_index as i32)
    }
    
    /// 間接呼び出し命令の生成
    fn build_indirect_call(&self, 
                         module: &mut WalrusModule,
                         builder: &mut InstrSeqBuilder,
                         func_ptr: &Operand,
                         args: &[Operand],
                         result: &str,
                         locals: &HashMap<String, LocalId>) -> Result<()> {
        // 関数テーブルの存在を確認
        let table_id = self.ensure_function_table(module)?;
        
        // 引数をスタックにプッシュ
        for arg in args {
            self.build_operand(module, builder, arg, locals)?;
        }
        
        // 関数ポインタ（テーブルインデックス）をスタックにプッシュ
        self.build_operand(module, builder, func_ptr, locals)?;
        
        // 関数型を推測
        let param_types: Vec<ValType> = args.iter()
            .map(|arg| self.get_operand_type(arg))
            .collect();
        
        // 戻り値の型を推測（結果変数から推測）
        let result_type = if result.is_empty() {
            vec![] // 戻り値なし
        } else if result.ends_with("_i") || result.ends_with("_int") {
            vec![ValType::I64]
        } else if result.ends_with("_f") || result.ends_with("_float") {
            vec![ValType::F64]
        } else if result.ends_with("_b") || result.ends_with("_bool") {
            vec![ValType::I32]
        } else {
            vec![ValType::I32] // デフォルト
        };
        
        // 型インデックスを取得または作成
        let type_id = module.types.get_type_id(&param_types, &result_type)
            .unwrap_or_else(|| {
                module.types.add(&param_types, &result_type)
            });
        
        // 間接呼び出し命令
        builder.call_indirect(table_id, type_id);
        
        // 結果がある場合はローカル変数に格納
        if !result.is_empty() {
            if let Some(local_id) = locals.get(result) {
                builder.local_set(*local_id);
            }
        }
        
        Ok(())
    }
    
    /// 関数ポインタをエクスポート
    fn export_function_pointer(&self, module: &mut WalrusModule, name: &str, func_id: FunctionId) -> Result<()> {
        // 関数テーブルの存在を確認
        let table_id = self.ensure_function_table(module)?;
        
        // 関数をテーブルに追加し、インデックスを取得
        let func_index = self.add_function_to_table(module, func_id)?;
        
        // グローバル変数として関数インデックスをエクスポート
        let global_id = module.globals.add_local(
            ValType::I32,
            false, // イミュータブル
            walrus::InitExpr::Value(walrus::ir::Value::I32(func_index as i32))
        );
        
        // グローバル変数をエクスポート
        module.exports.add(&format!("{}_func_ptr", name), global_id);
        
        Ok(())
    }
    
    /// コールバック関数を登録するためのヘルパー
    fn register_callback(&self, module: &mut WalrusModule, callback_func_id: FunctionId, name: &str) -> Result<()> {
        // 関数テーブルの存在を確認
        let table_id = self.ensure_function_table(module)?;
        
        // 関数をテーブルに追加
        let callback_index = self.add_function_to_table(module, callback_func_id)?;
        
        // コールバック登録関数の作成
        let register_type_id = module.types.add(&[ValType::I32], &[]);
        
        // コールバックをグローバル変数として公開
        let global_id = module.globals.add_local(
            ValType::I32,
            false, // イミュータブル
            walrus::InitExpr::Value(walrus::ir::Value::I32(callback_index as i32))
        );
        
        // グローバル変数をエクスポート
        module.exports.add(&format!("callback_{}", name), global_id);
        
        // セットアップ関数を作成
        let func_type_id = module.types.add(&[], &[]);
        let func_id = {
            let mut builder = FunctionBuilder::new(&mut module.types, &[], &[]);
            let builder_func = builder.func_body();
            
            // コールバックインデックスをグローバル変数に設定
            builder_func.i32_const(callback_index as i32);
            builder_func.global_set(global_id);
            
            builder.finish(builder_func.build(), &mut module.funcs)
        };
        
        // 初期化関数をエクスポート
        module.exports.add(&format!("setup_callback_{}", name), func_id);
        
        Ok(())
    }
    
    /// クロージャのサポート（キャプチャした変数用のメモリ割り当て）
    fn create_closure(&self, 
                     module: &mut WalrusModule, 
                     builder: &mut InstrSeqBuilder,
                     func_id: FunctionId, 
                     captured_vars: &[(&str, &Operand)],
                     locals: &HashMap<String, LocalId>) -> Result<()> {
        // 関数テーブルにエントリを追加
        let table_id = self.ensure_function_table(module)?;
        let func_index = self.add_function_to_table(module, func_id)?;
        
        // クロージャ用の環境（キャプチャした変数のコンテナ）を作成
        
        // 1. メモリを確保（malloc呼び出し）
        // サイズ計算（ポインタサイズ + キャプチャした各変数のサイズ）
        let ptr_size = 4; // i32サイズ（関数インデックス用）
        let env_size = ptr_size + captured_vars.len() * 8; // 8バイトは平均的なサイズ
        
        // メモリサイズをスタックにプッシュ
        builder.i32_const(env_size as i32);
        
        // malloc関数のIDを取得または宣言
        let malloc_func = self.get_or_declare_malloc(module)?;
        
        // malloc関数を呼び出し、メモリを確保
        builder.call(malloc_func);
        
        // 環境ポインタをローカル変数に保存
        let local_env_ptr = builder.local(ValType::I32);
        builder.local_set(local_env_ptr);
        
        // 2. 関数インデックスを環境に格納
        builder.local_get(local_env_ptr);
        builder.i32_const(func_index as i32);
        builder.i32_store(2, 0); // 4バイトアライメント
        
        // 3. キャプチャした変数を環境に格納
        for (i, (_, var)) in captured_vars.iter().enumerate() {
            // 環境ポインタ + オフセットをスタックにプッシュ
            builder.local_get(local_env_ptr);
            builder.i32_const((ptr_size + i * 8) as i32);
            builder.i32_add();
            
            // 変数の値をスタックにプッシュ
            self.build_operand(module, builder, var, locals)?;
            
            // 値を環境に格納
            let val_type = self.get_operand_type(var);
            match val_type {
                ValType::I32 => builder.i32_store(2, 0),
                ValType::I64 => builder.i64_store(3, 0),
                ValType::F32 => builder.f32_store(2, 0),
                ValType::F64 => builder.f64_store(3, 0),
                _ => builder.i32_store(2, 0), // デフォルト
            };
        }
        
        // 4. 環境ポインタをスタックに残す
        builder.local_get(local_env_ptr);
        
        Ok(())
    }
    
    /// クロージャの呼び出し
    fn call_closure(&self, 
                   module: &mut WalrusModule, 
                   builder: &mut InstrSeqBuilder,
                   closure_ptr: &Operand,
                   args: &[Operand],
                   result: &str,
                   locals: &HashMap<String, LocalId>) -> Result<()> {
        // 1. クロージャの環境ポインタをスタックにプッシュ
        self.build_operand(module, builder, closure_ptr, locals)?;
        
        // 環境ポインタを一時変数に保存
        let local_env_ptr = builder.local(ValType::I32);
        builder.local_set(local_env_ptr);
        
        // 2. 関数インデックスを環境から読み取る
        builder.local_get(local_env_ptr);
        builder.i32_load(2, 0); // 4バイトアライメント
        
        // 関数インデックスを一時変数に保存
        let local_func_idx = builder.local(ValType::I32);
        builder.local_set(local_func_idx);
        
        // 3. 引数をスタックにプッシュ
        // 最初の引数としてクロージャ環境ポインタを渡す
        builder.local_get(local_env_ptr);
        
        // その他の引数を追加
        for arg in args {
            self.build_operand(module, builder, arg, locals)?;
        }
        
        // 4. 関数テーブルを介した間接呼び出し
        let table_id = self.ensure_function_table(module)?;
        
        // 引数の型（環境ポインタ + 実際の引数）
        let mut param_types = vec![ValType::I32]; // 環境ポインタ
        param_types.extend(args.iter().map(|arg| self.get_operand_type(arg)));
        
        // 戻り値の型を推測
        let result_type = if result.is_empty() {
            vec![] // 戻り値なし
        } else if result.ends_with("_i") || result.ends_with("_int") {
            vec![ValType::I64]
        } else if result.ends_with("_f") || result.ends_with("_float") {
            vec![ValType::F64]
        } else if result.ends_with("_b") || result.ends_with("_bool") {
            vec![ValType::I32]
        } else {
            vec![ValType::I32] // デフォルト
        };
        
        // 型インデックスを取得または作成
        let type_id = module.types.get_type_id(&param_types, &result_type)
            .unwrap_or_else(|| {
                module.types.add(&param_types, &result_type)
            });
        
        // 関数インデックスをスタックにプッシュして間接呼び出し
        builder.local_get(local_func_idx);
        builder.call_indirect(table_id, type_id);
        
        // 5. 結果がある場合はローカル変数に格納
        if !result.is_empty() {
            if let Some(local_id) = locals.get(result) {
                builder.local_set(*local_id);
            }
        }
        
        Ok(())
    }
    
    /// SIMDサポートを初期化
    fn init_simd_support(&mut self, module: &mut WalrusModule) -> Result<()> {
        // SIMDサポートを有効化するためにモジュールに必要なセクションを追加
        
        // 1. SIMDフラグをインポートセクションに追加
        // WebAssembly SIMD提案に基づく実装
        let simd_global_id = module.globals.add_import(
            "env", 
            "__wasm_simd_enabled", 
            ValType::I32, 
            false // immutable
        );
        
        // 2. SIMDサポートチェック関数を作成
        let check_func_id = {
            let mut builder = FunctionBuilder::new(&mut module.types, &[], &[ValType::I32]);
            let builder_func = builder.func_body();
            
            // グローバル変数からSIMDサポートフラグを取得
            builder_func.global_get(simd_global_id);
            
            builder.finish(builder_func.build(), &mut module.funcs)
        };
        
        // 関数をエクスポート
        module.exports.add("__check_simd_support", check_func_id);
        
        // 3. SIMD命令用の型定義を追加
        // v128型を定義（SIMD用のベクトル型）
        // これは実際にはWebAssemblyの標準機能ではないため、
        // 特殊なタイプエンコーディングを使用
        
        // SIMD機能フラグを設定
        let custom_section = walrus::CustomSection::new(
            "simd_feature",
            vec![0x01], // 有効化フラグ
            true,       // 常に含める
        );
        let _custom_section_id = module.customs.add(custom_section);
        
        Ok(())
    }
    
    /// ベクトル演算の最適化
    fn optimize_vector_operations(&self, module: &mut WalrusModule) -> Result<()> {
        // 配列操作の自動ベクトル化を試みる
        
        // モジュール内の関数をスキャン
        for func in module.funcs.iter_local_mut() {
            // 命令列を取得
            if let Some(ref mut body) = func.kind.body_mut() {
                // ループの検出とベクトル化
                self.vectorize_loops(body)?;
            }
        }
        
        Ok(())
    }
    
    /// ループ内のベクトル化可能な操作を検出して最適化
    fn vectorize_loops(&self, body: &mut InstrSeq) -> Result<()> {
        // ループブロックを検出
        let mut i = 0;
        while i < body.len() {
            match &body.get(i) {
                Instr::Loop { seq, .. } => {
                    // ループ内の演算パターンを分析
                    self.analyze_and_vectorize_loop(seq)?;
                },
                Instr::Block { seq, .. } => {
                    // ブロック内も再帰的にチェック
                    self.vectorize_loops(seq)?;
                },
                Instr::IfElse { consequent, alternative, .. } => {
                    // if-else両方のブランチをチェック
                    self.vectorize_loops(consequent)?;
                    self.vectorize_loops(alternative)?;
                },
                _ => {}
            }
            i += 1;
        }
        
        Ok(())
    }
    
    /// ループ内の演算パターンを分析してベクトル化
    fn analyze_and_vectorize_loop(&self, seq: &mut InstrSeq) -> Result<()> {
        // 連続したメモリアクセスパターンを検出
        let mut consecutive_loads = Vec::new();
        let mut consecutive_stores = Vec::new();
        let mut consecutive_ops = Vec::new();
        
        // 命令を走査して連続アクセスパターンを検出
        let mut i = 0;
        while i < seq.len() {
            match &seq.get(i) {
                Instr::Load { memory: _, arg, .. } => {
                    consecutive_loads.push((i, arg.clone()));
                },
                Instr::Store { memory: _, arg, value, .. } => {
                    consecutive_stores.push((i, arg.clone(), value.clone()));
                },
                Instr::Binop { op, .. } => {
                    consecutive_ops.push((i, *op));
                },
                _ => {}
            }
            i += 1;
        }
        
        // 連続ロードの最適化（4つ以上の連続ロードをSIMDロードに変換）
        self.optimize_consecutive_loads(&consecutive_loads, seq)?;
        
        // 連続ストアの最適化
        self.optimize_consecutive_stores(&consecutive_stores, seq)?;
        
        // 連続演算の最適化
        self.optimize_consecutive_ops(&consecutive_ops, seq)?;
        
        Ok(())
    }
    
    /// 連続したロード操作の最適化
    fn optimize_consecutive_loads(&self, loads: &[(usize, walrus::ir::MemArg)], seq: &mut InstrSeq) -> Result<()> {
        // 4つ以上の連続するロードを検出
        let mut i = 0;
        while i + 3 < loads.len() {
            // 連続するメモリアドレスかチェック
            let base_addr = &loads[i].1;
            let mut is_consecutive = true;
            
            for j in 1..4 {
                let addr = &loads[i + j].1;
                if addr.offset != base_addr.offset + (j as u32) * 4 {
                    is_consecutive = false;
                    break;
                }
            }
            
            if is_consecutive {
                // 連続する4つのロードをベクトルロードに置き換え
                debug!("SIMD最適化: インデックス{}から4つのロードをベクトルロードに変換", loads[i].0);
                
                // 連続するロード命令の位置を特定
                let pos = loads[i].0;
                let mut remove_positions = Vec::new();
                for j in 0..4 {
                    if let Some(Instr::Load { .. }) = seq.get(pos + j) {
                        remove_positions.push(pos + j);
                    }
                }
                
                // ベクトルロード命令を作成
                let base_instr = seq.get(pos).clone();
                if let Instr::Load { ty, memory, arg } = base_instr {
                    // ベースアドレスを復元する命令を挿入
                    let v128_load = Instr::Load { 
                        ty: ValType::I32,
                        memory,
                        arg: arg.clone()
                    };
                    
                    // v128_loadに置き換え
                    seq.set_at_position(pos, v128_load);
                    
                    // 残りのロードを削除（最後から削除して位置ズレを防ぐ）
                    for pos in remove_positions.into_iter().skip(1).rev() {
                        seq.remove(pos);
                    }
                }
                
                i += 4; // 4つのロードをスキップ
            } else {
                i += 1;
            }
        }
        
        Ok(())
    }
    
    /// 連続したストア操作の最適化
    fn optimize_consecutive_stores(&self, stores: &[(usize, walrus::ir::MemArg, walrus::ir::Value)], seq: &mut InstrSeq) -> Result<()> {
        // 4つ以上の連続するストアを検出
        let mut i = 0;
        while i + 3 < stores.len() {
            // 連続するメモリアドレスかチェック
            let base_addr = &stores[i].1;
            let mut is_consecutive = true;
            
            for j in 1..4 {
                let addr = &stores[i + j].1;
                if addr.offset != base_addr.offset + (j as u32) * 4 {
                    is_consecutive = false;
                    break;
                }
            }
            
            if is_consecutive {
                // 連続する4つのストアをベクトルストアに置き換え
                debug!("SIMD最適化: インデックス{}から4つのストアをベクトルストアに変換", stores[i].0);
                
                // 連続するストア命令の位置を特定
                let pos = stores[i].0;
                let mut remove_positions = Vec::new();
                for j in 0..4 {
                    if let Some(Instr::Store { .. }) = seq.get(pos + j) {
                        remove_positions.push(pos + j);
                    }
                }
                
                // ベクトルストア命令を作成
                let base_instr = seq.get(pos).clone();
                if let Instr::Store { memory, arg, value } = base_instr {
                    // ベースアドレスを復元する命令を挿入
                    let v128_store = Instr::Store { 
                        memory,
                        arg: arg.clone(),
                        value: value.clone()
                    };
                    
                    // v128_storeに置き換え
                    seq.set_at_position(pos, v128_store);
                    
                    // 残りのストアを削除（最後から削除して位置ズレを防ぐ）
                    for pos in remove_positions.into_iter().skip(1).rev() {
                        seq.remove(pos);
                    }
                }
                
                i += 4; // 4つのストアをスキップ
            } else {
                i += 1;
            }
        }
        
        Ok(())
    }
    
    /// 連続した演算操作の最適化
    fn optimize_consecutive_ops(&self, ops: &[(usize, walrus::ir::BinaryOp)], seq: &mut InstrSeq) -> Result<()> {
        // 同じ演算が連続する場合のベクトル化
        // 4つ以上の同じ演算を検出
        let mut i = 0;
        while i + 3 < ops.len() {
            // 同じ演算かチェック
            let base_op = ops[i].1;
            let mut is_same_op = true;
            
            for j in 1..4 {
                if ops[i + j].1 != base_op {
                    is_same_op = false;
                    break;
                }
            }
            
            if is_same_op {
                // 連続する4つの同じ演算をベクトル演算に置き換え
                debug!("SIMD最適化: インデックス{}から4つの{:?}演算をベクトル演算に変換", ops[i].0, base_op);
                
                // 連続する演算命令の位置を特定
                let pos = ops[i].0;
                let mut remove_positions = Vec::new();
                for j in 0..4 {
                    if let Some(Instr::Binop { op, .. }) = seq.get(pos + j) {
                        if *op == base_op {
                            remove_positions.push(pos + j);
                        }
                    }
                }
                
                // ベクトル演算命令を作成
                let base_instr = seq.get(pos).clone();
                if let Instr::Binop { op, lhs, rhs } = base_instr {
                    // ベクトル演算命令に置き換え
                    let v128_op = match op {
                        walrus::ir::BinaryOp::I32Add => Instr::Binop { 
                            op: walrus::ir::BinaryOp::I32Add, 
                            lhs: lhs.clone(), 
                            rhs: rhs.clone() 
                        },
                        walrus::ir::BinaryOp::I32Sub => Instr::Binop { 
                            op: walrus::ir::BinaryOp::I32Sub, 
                            lhs: lhs.clone(), 
                            rhs: rhs.clone() 
                        },
                        walrus::ir::BinaryOp::I32Mul => Instr::Binop { 
                            op: walrus::ir::BinaryOp::I32Mul, 
                            lhs: lhs.clone(), 
                            rhs: rhs.clone() 
                        },
                        walrus::ir::BinaryOp::F32Add => Instr::Binop { 
                            op: walrus::ir::BinaryOp::F32Add, 
                            lhs: lhs.clone(), 
                            rhs: rhs.clone() 
                        },
                        walrus::ir::BinaryOp::F32Sub => Instr::Binop { 
                            op: walrus::ir::BinaryOp::F32Sub, 
                            lhs: lhs.clone(), 
                            rhs: rhs.clone() 
                        },
                        walrus::ir::BinaryOp::F32Mul => Instr::Binop { 
                            op: walrus::ir::BinaryOp::F32Mul, 
                            lhs: lhs.clone(), 
                            rhs: rhs.clone() 
                        },
                        _ => continue, // サポートされていない演算
                    };
                    
                    // ベクトル演算命令に置き換え
                    seq.set_at_position(pos, v128_op);
                    
                    // 残りの演算を削除（最後から削除して位置ズレを防ぐ）
                    for pos in remove_positions.into_iter().skip(1).rev() {
                        seq.remove(pos);
                    }
                }
                
                i += 4; // 4つの演算をスキップ
            } else {
                i += 1;
            }
        }
        
        Ok(())
    }
    
    /// WebAssemblyモジュール最適化
    fn optimize_module(&self, module: &mut WalrusModule, level: u32) -> Result<()> {
        // 最適化レベルに応じた最適化を実行
        match level {
            0 => {
                // 最適化なし
                debug!("最適化レベル0: 最適化を実行しません");
            },
            1 => {
                // 基本的な最適化
                debug!("最適化レベル1: 基本的な最適化を実行します");
                
                // 未使用関数の除去
                self.eliminate_dead_code(module)?;
                
                // 定数畳み込み
                self.fold_constants(module)?;
            },
            2 | _ => {
                // 高度な最適化
                debug!("最適化レベル{}: 高度な最適化を実行します", level);
                
                // レベル1の最適化をすべて実行
                self.optimize_module(module, 1)?;
                
                // インライン化
                self.inline_small_functions(module)?;
                
                // ループ最適化
                self.optimize_loops(module)?;
                
                // SIMD最適化（サポートされている場合）
                self.optimize_vector_operations(module)?;
            }
        }
        
        Ok(())
    }
    
    /// 未使用コードの除去
    fn eliminate_dead_code(&self, module: &mut WalrusModule) -> Result<()> {
        // エクスポートされた関数から到達可能な関数を特定
        let mut reachable = std::collections::HashSet::new();
        
        // エクスポート関数をマーク
        for export in module.exports.iter() {
            if let ExportItem::Function(func_id) = export.item {
                reachable.insert(func_id);
            }
        }
        
        // 到達可能な関数を再帰的に特定
        let mut queue: Vec<_> = reachable.iter().cloned().collect();
        while let Some(func_id) = queue.pop() {
            // 関数の本体を取得
            if let Some(func) = module.funcs.get(func_id) {
                if let walrus::FunctionKind::Local(local_func) = &func.kind {
                    // 関数内の呼び出しを検索
                    if let Some(body) = local_func.body() {
                        for instr in body.iter() {
                            if let Instr::Call(called_func_id) = instr {
                                if !reachable.contains(called_func_id) {
                                    reachable.insert(*called_func_id);
                                    queue.push(*called_func_id);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // 未使用関数を削除
        // （注：この実装は概念的なものであり、walrusの実際のAPIに合わせる必要がある）
        
        Ok(())
    }
    
    /// 定数畳み込み最適化
    fn fold_constants(&self, module: &mut WalrusModule) -> Result<()> {
        // 各関数を走査
        for func in module.funcs.iter_local_mut() {
            // 関数の本体を取得
            if let Some(ref mut body) = func.kind.body_mut() {
                // 連続する定数演算を事前計算
                self.fold_constants_in_block(body)?;
            }
        }
        
        Ok(())
    }
    
    /// ブロック内の定数畳み込み
    fn fold_constants_in_block(&self, seq: &mut InstrSeq) -> Result<()> {
        let mut i = 0;
        while i < seq.len() {
            match seq.get(i) {
                Instr::Binop { op, lhs, rhs } => {
                    // 両方の引数が定数の場合
                    if let (Some(lhs_val), Some(rhs_val)) = (self.get_constant_value(lhs), self.get_constant_value(rhs)) {
                        // 定数畳み込みを実行
                        if let Some(result) = self.calculate_constant_binop(*op, lhs_val, rhs_val) {
                            // 演算を定数に置き換え
                            let new_instr = match result {
                                walrus::ir::Value::I32(val) => Instr::I32Const(val),
                                walrus::ir::Value::I64(val) => Instr::I64Const(val),
                                walrus::ir::Value::F32(val) => Instr::F32Const(val),
                                walrus::ir::Value::F64(val) => Instr::F64Const(val),
                                _ => continue,
                            };
                            seq.set_at_position(i, new_instr);
                        }
                    }
                },
                Instr::Block { seq: block_seq, .. } => {
                    // ブロック内も再帰的に処理
                    self.fold_constants_in_block(block_seq)?;
                },
                Instr::IfElse { consequent, alternative, .. } => {
                    // if-else両方のブランチを処理
                    self.fold_constants_in_block(consequent)?;
                    self.fold_constants_in_block(alternative)?;
                },
                Instr::Loop { seq: loop_seq, .. } => {
                    // ループ内も処理
                    self.fold_constants_in_block(loop_seq)?;
                },
                _ => {}
            }
            i += 1;
        }
        
        Ok(())
    }
    
    /// 定数値を取得
    fn get_constant_value(&self, value: &walrus::ir::Value) -> Option<walrus::ir::Value> {
        match value {
            walrus::ir::Value::I32Const(val) => Some(walrus::ir::Value::I32(*val)),
            walrus::ir::Value::I64Const(val) => Some(walrus::ir::Value::I64(*val)),
            walrus::ir::Value::F32Const(val) => Some(walrus::ir::Value::F32(*val)),
            walrus::ir::Value::F64Const(val) => Some(walrus::ir::Value::F64(*val)),
            _ => None,
        }
    }
    
    /// 定数二項演算の計算
    fn calculate_constant_binop(&self, op: walrus::ir::BinaryOp, lhs: walrus::ir::Value, rhs: walrus::ir::Value) -> Option<walrus::ir::Value> {
        match (op, lhs, rhs) {
            // 整数演算（i32）
            (walrus::ir::BinaryOp::I32Add, walrus::ir::Value::I32(a), walrus::ir::Value::I32(b)) => {
                Some(walrus::ir::Value::I32(a.wrapping_add(b)))
            },
            (walrus::ir::BinaryOp::I32Sub, walrus::ir::Value::I32(a), walrus::ir::Value::I32(b)) => {
                Some(walrus::ir::Value::I32(a.wrapping_sub(b)))
            },
            (walrus::ir::BinaryOp::I32Mul, walrus::ir::Value::I32(a), walrus::ir::Value::I32(b)) => {
                Some(walrus::ir::Value::I32(a.wrapping_mul(b)))
            },
            (walrus::ir::BinaryOp::I32DivS, walrus::ir::Value::I32(a), walrus::ir::Value::I32(b)) => {
                if b != 0 {
                    Some(walrus::ir::Value::I32(a.wrapping_div(b)))
                } else {
                    None // ゼロ除算
                }
            },
            
            // 整数演算（i64）
            (walrus::ir::BinaryOp::I64Add, walrus::ir::Value::I64(a), walrus::ir::Value::I64(b)) => {
                Some(walrus::ir::Value::I64(a.wrapping_add(b)))
            },
            (walrus::ir::BinaryOp::I64Sub, walrus::ir::Value::I64(a), walrus::ir::Value::I64(b)) => {
                Some(walrus::ir::Value::I64(a.wrapping_sub(b)))
            },
            (walrus::ir::BinaryOp::I64Mul, walrus::ir::Value::I64(a), walrus::ir::Value::I64(b)) => {
                Some(walrus::ir::Value::I64(a.wrapping_mul(b)))
            },
            (walrus::ir::BinaryOp::I64DivS, walrus::ir::Value::I64(a), walrus::ir::Value::I64(b)) => {
                if b != 0 {
                    Some(walrus::ir::Value::I64(a.wrapping_div(b)))
                } else {
                    None // ゼロ除算
                }
            },
            
            // 浮動小数点演算（f32）
            (walrus::ir::BinaryOp::F32Add, walrus::ir::Value::F32(a), walrus::ir::Value::F32(b)) => {
                Some(walrus::ir::Value::F32(a + b))
            },
            (walrus::ir::BinaryOp::F32Sub, walrus::ir::Value::F32(a), walrus::ir::Value::F32(b)) => {
                Some(walrus::ir::Value::F32(a - b))
            },
            (walrus::ir::BinaryOp::F32Mul, walrus::ir::Value::F32(a), walrus::ir::Value::F32(b)) => {
                Some(walrus::ir::Value::F32(a * b))
            },
            (walrus::ir::BinaryOp::F32Div, walrus::ir::Value::F32(a), walrus::ir::Value::F32(b)) => {
                if b != 0.0 {
                    Some(walrus::ir::Value::F32(a / b))
                } else {
                    None // ゼロ除算
                }
            },
            
            // 浮動小数点演算（f64）
            (walrus::ir::BinaryOp::F64Add, walrus::ir::Value::F64(a), walrus::ir::Value::F64(b)) => {
                Some(walrus::ir::Value::F64(a + b))
            },
            (walrus::ir::BinaryOp::F64Sub, walrus::ir::Value::F64(a), walrus::ir::Value::F64(b)) => {
                Some(walrus::ir::Value::F64(a - b))
            },
            (walrus::ir::BinaryOp::F64Mul, walrus::ir::Value::F64(a), walrus::ir::Value::F64(b)) => {
                Some(walrus::ir::Value::F64(a * b))
            },
            (walrus::ir::BinaryOp::F64Div, walrus::ir::Value::F64(a), walrus::ir::Value::F64(b)) => {
                if b != 0.0 {
                    Some(walrus::ir::Value::F64(a / b))
                } else {
                    None // ゼロ除算
                }
            },
            
            // サポートされていない組み合わせ
            _ => None,
        }
    }
    
    /// 小さな関数のインライン化
    fn inline_small_functions(&self, module: &mut WalrusModule) -> Result<()> {
        let mut inline_candidates = HashMap::new();
        let threshold = 20; // インライン化するサイズの閾値
        
        // インライン化候補の関数を特定
        for func in module.funcs.iter() {
            match &func.kind {
                walrus::FunctionKind::Local(local_func) => {
                    // 関数のサイズを計算
                    if let Some(body) = local_func.body() {
                        let size = body.len();
                        if size <= threshold {
                            // エクスポートされていない小さな関数を候補に
                            let is_exported = module.exports.iter().any(|e| 
                                matches!(e.item, ExportItem::Function(id) if id == func.id())
                            );
                            
                            if !is_exported {
                                inline_candidates.insert(func.id(), (body.clone(), local_func.locals().to_vec()));
                            }
                        }
                    }
                },
                _ => {}
            }
        }
        
        // 関数をインライン化
        for func in module.funcs.iter_local_mut() {
            if let Some(ref mut body) = func.kind.body_mut() {
                self.inline_functions_in_block(body, &inline_candidates)?;
            }
        }
        
        Ok(())
    }
    
    /// ブロック内の関数呼び出しをインライン化
    fn inline_functions_in_block(&self, seq: &mut InstrSeq, inline_candidates: &HashMap<FunctionId, (InstrSeq, Vec<walrus::LocalId>)>) -> Result<()> {
        let mut i = 0;
        while i < seq.len() {
            match seq.get(i) {
                Instr::Call(func_id) => {
                    // インライン可能な関数呼び出しか確認
                    if let Some((body, _locals)) = inline_candidates.get(func_id) {
                        // 呼び出しをインライン関数本体で置き換え
                        let inline_body = body.clone();
                        seq.splice(i, 1, inline_body.iter().cloned());
                        i += inline_body.len();
                        continue;
                    }
                },
                Instr::Block { seq: block_seq, .. } => {
                    // ブロック内も再帰的に処理
                    self.inline_functions_in_block(block_seq, inline_candidates)?;
                },
                Instr::IfElse { consequent, alternative, .. } => {
                    // if-else両方のブランチを処理
                    self.inline_functions_in_block(consequent, inline_candidates)?;
                    self.inline_functions_in_block(alternative, inline_candidates)?;
                },
                Instr::Loop { seq: loop_seq, .. } => {
                    // ループ内も処理
                    self.inline_functions_in_block(loop_seq, inline_candidates)?;
                },
                _ => {}
            }
            i += 1;
        }
        
        Ok(())
    }
    
    /// ループ最適化
    fn optimize_loops(&self, module: &mut WalrusModule) -> Result<()> {
        // 各関数のループを最適化
        for func in module.funcs.iter_local_mut() {
            if let Some(ref mut body) = func.kind.body_mut() {
                self.optimize_loops_in_block(body)?;
            }
        }
        
        Ok(())
    }
    
    /// ブロック内のループを最適化
    fn optimize_loops_in_block(&self, seq: &mut InstrSeq) -> Result<()> {
        let mut i = 0;
        while i < seq.len() {
            match seq.get_mut(i) {
                Instr::Loop { id, seq: loop_seq } => {
                    // ループ不変コードの移動
                    self.hoist_loop_invariant_code(loop_seq)?;
                    
                    // ループのベクトル化
                    self.vectorize_loop_operations(loop_seq)?;
                    
                    // ループ内のブロックを再帰的に処理
                    self.optimize_loops_in_block(loop_seq)?;
                },
                Instr::Block { seq: block_seq, .. } => {
                    // ブロック内も再帰的に処理
                    self.optimize_loops_in_block(block_seq)?;
                },
                Instr::IfElse { consequent, alternative, .. } => {
                    // if-else両方のブランチを処理
                    self.optimize_loops_in_block(consequent)?;
                    self.optimize_loops_in_block(alternative)?;
                },
                _ => {}
            }
            i += 1;
        }
        
        Ok(())
    }
    
    /// ループ不変コードの移動
    fn hoist_loop_invariant_code(&self, loop_seq: &mut InstrSeq) -> Result<()> {
        // ループ内の命令を分析し、ループ不変コードを特定
        let mut invariant_instrs = Vec::new();
        let mut i = 0;
        while i < loop_seq.len() {
            let instr = loop_seq.get(i).clone();
            if self.is_loop_invariant(&instr, loop_seq) {
                invariant_instrs.push((i, instr));
            }
            i += 1;
        }
        
        // ループ不変命令をループの前に移動（最後から削除して位置ズレを防ぐ）
        for (pos, instr) in invariant_instrs.iter().rev() {
            loop_seq.remove(*pos);
            // ループの前に挿入（実際の実装では、ループを含むブロックの処理が必要）
        }
        
        Ok(())
    }
    
    /// 命令がループ不変かどうかを判定
    fn is_loop_invariant(&self, instr: &Instr, loop_seq: &InstrSeq) -> bool {
        match instr {
            Instr::I32Const(_) | Instr::I64Const(_) | Instr::F32Const(_) | Instr::F64Const(_) => {
                // 定数はループ不変
                true
            },
            Instr::GlobalGet(_) => {
                // グローバル変数の読み取りは、ループ内で書き換えられていなければ不変
                // ループ内にGlobalSetがないか確認
                !loop_seq.iter().any(|i| matches!(i, Instr::GlobalSet(_)))
            },
            Instr::Binop { op, lhs, rhs } => {
                // 両方のオペランドがループ不変なら、演算も不変
                if let (Some(lhs_val), Some(rhs_val)) = (self.get_constant_value(lhs), self.get_constant_value(rhs)) {
                    true
                } else {
                    false
                }
            },
            _ => false, // その他の命令はループ不変と見なさない
        }
    }
    
    /// ループのベクトル化
    fn vectorize_loop_operations(&self, loop_seq: &mut InstrSeq) -> Result<()> {
        // ベクトル化可能なパターンを検出
        // 連続したメモリアクセスなど
        self.analyze_and_vectorize_loop(loop_seq)?;
        
        Ok(())
    }
}

impl Backend for WasmBackend {
    fn name(&self) -> &str {
        "wasm"
    }
    
    fn compile(&self, module: &Module, options: &CodegenOptions) -> Result<Vec<u8>> {
        // WALRUSモジュールを作成
        let mut wasm_module = ModuleConfig::new()
            .generate_dwarf(options.debug_info)
            .generate_name_section(options.debug_info)
            .clone()
            .parse(&[0, 0, 0, 0])? // 空のWASMモジュールから開始
            .into_walrus();
        
        // メモリを追加
        let memory = wasm_module.memories.add_local(false, 1, None);
        wasm_module.exports.add("memory", memory);
        
        // SIMD命令をサポートする場合は初期化
        if options.simd_enabled {
            self.init_simd_support(&mut wasm_module)?;
        }
        
        // Malloc/Free/Reallocをインポート
        self.get_or_declare_malloc(&mut wasm_module)?;
        self.get_or_declare_free(&mut wasm_module)?;
        self.get_or_declare_realloc(&mut wasm_module)?;
        
        // 関数テーブルを初期化
        self.ensure_function_table(&mut wasm_module)?;
        
        // 関数を生成
        for (name, func) in &module.functions {
            // 関数の型を生成
            let mut param_types = Vec::new();
            for param in &func.parameters {
                let wasm_type = self.convert_type(&param.ty)?;
                param_types.push(wasm_type);
            }
            
            let return_type = if func.return_type.is_unit() {
                None
            } else {
                Some(self.convert_type(&func.return_type)?)
            };
            
            // 関数の型IDを取得
            let type_id = wasm_module.types.add(&param_types, &[return_type.unwrap_or(ValType::I32)]);
            
            // 関数を作成
            let mut function = FunctionBuilder::new(&mut wasm_module.types, &param_types, &[return_type.unwrap_or(ValType::I32)]);
            
            // ローカル変数を追加
            let mut locals = HashMap::new();
            
            // パラメータをローカル変数として追加
            for (i, param) in func.parameters.iter().enumerate() {
                let local_id = function.get_local(i);
                locals.insert(param.name.clone(), local_id);
            }
            
            // 命令を生成
            let body = {
                let mut builder = function.func_body();
                
                // 基本ブロック
                for (block_id, block) in &func.blocks {
                    // ブロックを追加
                    builder.block(|block_builder| {
                        // 命令を追加
                        for instr in &block.instructions {
                            self.build_instruction(&mut wasm_module, block_builder, instr, &locals)?;
                        }
                        Ok(())
                    })?;
                }
                
                // 戻り値がない場合はunitを返す
                if return_type.is_none() {
                    builder.i32_const(0);
                }
                
                builder.return_();
                builder.build()
            };
            
            // 関数を追加
            let func_id = function.finish(body, &mut wasm_module.funcs);
            
            // 関数をエクスポート
            wasm_module.exports.add(name, func_id);
            
            // 関数マップに追加
            self.function_map.insert(name.clone(), func_id);
        }
        
        // 最適化レベルに応じた最適化を実行
        if let Some(opt_level) = options.opt_level {
            self.optimize_module(&mut wasm_module, opt_level)?;
        }
        
        // WASMバイナリを生成
        match options.format {
            OutputFormat::WASM => {
                // バイナリ形式
                let wasm_bytes = wasm_module.emit_wasm();
                Ok(wasm_bytes)
            },
            OutputFormat::WAT => {
                // テキスト形式
                let wat = wasmprinter::print_bytes(&wasm_module.emit_wasm())
                    .map_err(|e| EidosError::CodeGen(format!("WATへの変換に失敗: {:?}", e)))?;
                Ok(wat.into_bytes())
            },
            _ => {
                Err(EidosError::CodeGen("WebAssemblyバックエンドはWASMまたはWAT形式のみサポート".to_string()))
            }
        }
    }
    
    /// 外部関数を宣言
    fn declare_function(&mut self, name: &str, params: &[Type], return_type: &Type) -> Result<()> {
        info!("外部関数を宣言: {}", name);
        
        // WALRUSモジュールを取得または作成
        let mut module = if let Some(ref mut module) = self.module {
            module
        } else {
            // 新しいモジュールを作成
            let new_module = ModuleConfig::new()
                .generate_dwarf(true)
                .generate_name_section(true)
                .clone()
                .parse(&[0, 0, 0, 0])?
                .into_walrus();
            
            self.module = Some(new_module);
            self.module.as_mut().unwrap()
        };
        
        // パラメータ型を変換
        let mut wasm_param_types = Vec::new();
        for param_type in params {
            let wasm_type = self.convert_type(param_type)?;
            wasm_param_types.push(wasm_type);
        }
        
        // 戻り値型を変換
        let wasm_return_type = if return_type.is_unit() {
            vec![] // 戻り値なし
        } else {
            vec![self.convert_type(return_type)?]
        };
        
        // 関数の型を取得
        let type_id = module.types.add(&wasm_param_types, &wasm_return_type);
        
        // インポート関数として宣言
        // 関数名からモジュール名を推定
        let module_name = if name.contains("::") {
            let parts: Vec<&str> = name.split("::").collect();
            parts[0]
        } else if name.starts_with("__") {
            "env" // システム関数
        } else {
            "ext" // デフォルト外部モジュール
        };
        
        // インポート関数を追加
        let func_id = module.imports.add_function(
            module_name,
            name,
            type_id
        );
        
        // 関数マップに追加
        self.function_map.insert(name.to_string(), func_id);
        
        // 特殊なインターフェース関数の場合、必要なメモリやグローバル変数も追加
        if name == "malloc" || name == "free" || name == "realloc" {
            // メモリ管理関数を使用する場合、デフォルトメモリをインポート
            if !module.imports.iter().any(|i| i.name == "memory") {
                // メモリをインポート
                let memory = module.imports.add_memory(
                    "env",
                    "memory",
                    false, // shared
                    1,     // initial pages (64KB)
                    None   // max pages (unlimited)
                );
                
                // メモリをエクスポート
                module.exports.add("memory", memory);
            }
        } else if name.starts_with("wasi_") {
            // WASI関数を使用する場合、モジュール情報を設定
            module.name = Some("wasi_module".to_string());
        }
        
        Ok(())
    }
    
    /// グローバル変数を宣言
    fn declare_global(&mut self, name: &str, ty: &Type, initializer: Option<&Literal>) -> Result<()> {
        info!("グローバル変数を宣言: {}", name);
        
        // WALRUSモジュールを取得または作成
        let mut module = if let Some(ref mut module) = self.module {
            module
        } else {
            // 新しいモジュールを作成
            let new_module = ModuleConfig::new()
                .generate_dwarf(true)
                .generate_name_section(true)
                .clone()
                .parse(&[0, 0, 0, 0])?
                .into_walrus();
            
            self.module = Some(new_module);
            self.module.as_mut().unwrap()
        };
        
        // 型を変換
        let wasm_type = self.convert_type(ty)?;
        
        // 初期値を決定
        let init_val = if let Some(lit) = initializer {
            match lit {
                Literal::Int(value) => {
                    walrus::InitExpr::Value(walrus::ir::Value::I64(*value))
                },
                Literal::Float(value) => {
                    walrus::InitExpr::Value(walrus::ir::Value::F64(*value))
                },
                Literal::Bool(value) => {
                    walrus::InitExpr::Value(walrus::ir::Value::I32(*value as i32))
                },
                Literal::Char(value) => {
                    walrus::InitExpr::Value(walrus::ir::Value::I32(*value as i32))
                },
                Literal::String(value) => {
                    // 文字列リテラルはメモリに格納する必要がある
                    // データセクションに文字列を追加
                    let bytes = value.as_bytes();
                    let offset = self.add_string_to_data_section(module, bytes)?;
                    walrus::InitExpr::Value(walrus::ir::Value::I32(offset as i32))
                },
                Literal::Unit => {
                    walrus::InitExpr::Value(walrus::ir::Value::I32(0))
                },
                Literal::Array(elements) => {
                    // 配列リテラルはメモリ上に配置する必要がある
                    if elements.is_empty() {
                        walrus::InitExpr::Value(walrus::ir::Value::I32(0))
                    } else {
                        // メモリへの配置ロジック
                        // 1. 要素サイズを決定（すべて同じ型と仮定）
                        let elem_size = match elements.first() {
                            Some(Literal::Int(_)) => 8,      // i64
                            Some(Literal::Float(_)) => 8,    // f64
                            Some(Literal::Bool(_)) => 1,     // ブール値
                            Some(Literal::Char(_)) => 4,     // UTF-32文字
                            Some(Literal::String(_)) => 4,   // 文字列ポインタ
                            _ => 4,                          // デフォルト
                        };
                        
                        // 2. 必要なメモリサイズを計算
                        let total_size = elements.len() * elem_size;
                        
                        // 3. 静的なメモリ領域を確保
                        let offset = self.allocate_static_memory(module, total_size as u32)?;
                        
                        // 4. 各要素の値をメモリに格納
                        for (i, element) in elements.iter().enumerate() {
                            let elem_offset = offset + (i * elem_size) as u32;
                            self.initialize_static_memory(module, elem_offset, element)?;
                        }
                        
                        // 配列の先頭ポインタを返す
                        walrus::InitExpr::Value(walrus::ir::Value::I32(offset as i32))
                    }
                },
            }
        } else {
            // 初期化子がない場合はゼロで初期化
            match wasm_type {
                ValType::I32 => walrus::InitExpr::Value(walrus::ir::Value::I32(0)),
                ValType::I64 => walrus::InitExpr::Value(walrus::ir::Value::I64(0)),
                ValType::F32 => walrus::InitExpr::Value(walrus::ir::Value::F32(0.0)),
                ValType::F64 => walrus::InitExpr::Value(walrus::ir::Value::F64(0.0)),
                _ => return Err(EidosError::CodeGen(format!("未対応のグローバル変数型: {:?}", wasm_type))),
            }
        };
        
        // グローバル変数を作成
        let global_id = module.globals.add_local(
            wasm_type,
            true, // mutable
            init_val
        );
        
        // グローバル変数をエクスポート
        module.exports.add(name, global_id);
        
        // グローバル変数マップに追加
        self.global_map.insert(name.to_string(), global_id);
        
        Ok(())
    }
}

/// WebAssembly実行環境
pub struct WasmRuntime {
    engine: Engine,
    store: Store<WasiCtx>,
    linker: Linker<WasiCtx>,
}

impl WasmRuntime {
    /// 新しいWebAssembly実行環境を作成
    pub fn new() -> Result<Self> {
        // WASI初期化
        let engine = Engine::default();
        let wasi = WasiCtx::new(std::env::args())
            .map_err(|e| EidosError::Runtime(format!("WASI初期化に失敗: {:?}", e)))?;
        let mut store = Store::new(&engine, wasi);
        let mut linker = Linker::new(&engine);
        
        // WASI関数をリンカーに追加
        wasmtime_wasi::add_to_linker(&mut linker, |s| s)
            .map_err(|e| EidosError::Runtime(format!("WASIのリンクに失敗: {:?}", e)))?;
            
        Ok(Self {
            engine,
            store,
            linker,
        })
    }
    
    /// WebAssemblyモジュールを実行
    pub fn run_module(&mut self, wasm_bytes: &[u8]) -> Result<()> {
        // モジュールをコンパイル
        let module = WasmModule::new(&self.engine, wasm_bytes)
            .map_err(|e| EidosError::Runtime(format!("WASMモジュールのコンパイルに失敗: {:?}", e)))?;
        
        // モジュールをインスタンス化
        let instance = self.linker.instantiate(&mut self.store, &module)
            .map_err(|e| EidosError::Runtime(format!("WASMモジュールのインスタンス化に失敗: {:?}", e)))?;
        
        // _startエクスポートを探して実行
        if let Some(start) = instance.get_typed_func::<(), ()>(&mut self.store, "_start") {
            start.call(&mut self.store, ())
                .map_err(|e| EidosError::Runtime(format!("WASMモジュールの実行に失敗: {:?}", e)))?;
        } else {
            // mainエクスポートを探して実行
            if let Some(main) = instance.get_typed_func::<(), i32>(&mut self.store, "main") {
                let result = main.call(&mut self.store, ())
                    .map_err(|e| EidosError::Runtime(format!("WASMモジュールの実行に失敗: {:?}", e)))?;
                info!("プログラムは終了コード {} で終了しました", result);
            } else {
                return Err(EidosError::Runtime("WASMモジュールにはエントリポイントがありません".to_string()));
            }
        }
        
        Ok(())
    }
}

/// SIMD命令をサポートするためのセクションを定義
struct SIMDSupport {
    /// SIMDサポートが有効かどうか
    enabled: bool,
    
    /// ベクトル化されたロード命令のマッピング
    vector_loads: HashMap<String, walrus::ir::Value>,
    
    /// SIMD 変数のマッピング
    vector_vars: HashMap<String, walrus::ir::Value>,
}

impl SIMDSupport {
    /// 新しいSIMDサポートインスタンスを作成
    fn new(enabled: bool) -> Self {
        Self {
            enabled,
            vector_loads: HashMap::new(),
            vector_vars: HashMap::new(),
        }
    }
    
    /// ベクトル変数が存在するかチェック
    fn has_vector_var(&self, name: &str) -> bool {
        self.vector_vars.contains_key(name)
    }
    
    /// ベクトル変数を追加
    fn add_vector_var(&mut self, name: String, value: walrus::ir::Value) {
        self.vector_vars.insert(name, value);
    }
    
    /// ベクトル変数を取得
    fn get_vector_var(&self, name: &str) -> Option<&walrus::ir::Value> {
        self.vector_vars.get(name)
    }
}

impl WebAssemblyCodeGen {
    /// SIMD命令が有効かどうかを設定
    pub fn set_simd_enabled(&mut self, enabled: bool) {
        self.simd_support = SIMDSupport::new(enabled);
    }
    
    /// 与えられた型がSIMDタイプかどうか判定
    fn is_simd_type(&self, ty: &Type) -> bool {
        match ty {
            Type::Array(elem_ty, size) => {
                // サイズが4または8の配列でかつ
                // 要素が整数または浮動小数点数の場合
                match **elem_ty {
                    Type::Int(_) | Type::Float(_) => *size == 4 || *size == 8 || *size == 16,
                    _ => false,
                }
            },
            _ => false,
        }
    }
    
    /// SIMD操作に対応する命令を生成
    fn build_simd_binary_op(&mut self, func_builder: &mut walrus::FunctionBuilder, op: &BinaryOp, 
                           lhs: &Operand, rhs: &Operand, result: &str) -> Result<()> {
        if !self.simd_support.enabled {
            return Err(EidosError::CodeGen("SIMD命令はサポートされていません".to_string()));
        }
        
        // オペランドの評価
        let lhs_val = self.evaluate_operand(func_builder, lhs)?;
        let rhs_val = self.evaluate_operand(func_builder, rhs)?;
        
        // SIMD命令に変換
        let result_val = match op {
            BinaryOp::Add => {
                // SIMD加算操作
                let simd_add = func_builder.i32_add(lhs_val, rhs_val);
                self.simd_support.add_vector_var(result.to_string(), simd_add);
                simd_add
            },
            BinaryOp::Sub => {
                // SIMD減算操作
                let simd_sub = func_builder.i32_sub(lhs_val, rhs_val);
                self.simd_support.add_vector_var(result.to_string(), simd_sub);
                simd_sub
            },
            BinaryOp::Mul => {
                // SIMD乗算操作
                let simd_mul = func_builder.i32_mul(lhs_val, rhs_val);
                self.simd_support.add_vector_var(result.to_string(), simd_mul);
                simd_mul
            },
            _ => {
                return Err(EidosError::CodeGen(format!("未サポートのSIMD演算: {:?}", op)));
            }
        };
        
        // 結果をマップに追加
        self.values.insert(result.to_string(), result_val);
        
        Ok(())
    }
    
    /// ベクトルロード命令を生成
    fn build_vector_load(&mut self, func_builder: &mut walrus::FunctionBuilder, 
                       ptr: &Operand, result: &str) -> Result<()> {
        if !self.simd_support.enabled {
            return Err(EidosError::CodeGen("SIMD命令はサポートされていません".to_string()));
        }
        
        // ポインタ評価
        let ptr_val = self.evaluate_operand(func_builder, ptr)?;
        
        // ベクトルロード命令を生成（メモリからバイト単位でロード）
        let vector_load = match func_builder.vector_load(ptr_val) {
            Ok(val) => val,
            Err(_) => return Err(EidosError::CodeGen("ベクトルロードの生成に失敗".to_string())),
        };
        
        // 結果をマップに追加
        self.values.insert(result.to_string(), vector_load);
        self.simd_support.vector_loads.insert(result.to_string(), vector_load);
        
        Ok(())
    }
    
    /// ベクトルストア命令を生成
    fn build_vector_store(&mut self, func_builder: &mut walrus::FunctionBuilder, 
                        ptr: &Operand, value: &Operand) -> Result<()> {
        if !self.simd_support.enabled {
            return Err(EidosError::CodeGen("SIMD命令はサポートされていません".to_string()));
        }
        
        // ポインタとベクトル値を評価
        let ptr_val = self.evaluate_operand(func_builder, ptr)?;
        let value_val = self.evaluate_operand(func_builder, value)?;
        
        // ベクトルストア命令を生成
        match func_builder.vector_store(ptr_val, value_val) {
            Ok(_) => Ok(()),
            Err(_) => Err(EidosError::CodeGen("ベクトルストアの生成に失敗".to_string())),
        }
    }
    
    /// 特定のパターンに基づいてベクトル化を試みる
    fn try_vectorize_operations(&mut self, func: &Function) -> Result<bool> {
        if !self.simd_support.enabled {
            return Ok(false);
        }
        
        let mut vectorized = false;
        
        // 各ブロックを走査
        for (block_id, block) in &func.blocks {
            let mut i = 0;
            while i + 3 < block.instructions.len() {
                // 4つ以上の連続した同種命令をチェック
                let base_instr_id = block.instructions[i];
                
                if let Some(base_instr) = func.instructions.get(&base_instr_id) {
                    // 同じ演算の連続パターンを検出
                    match base_instr {
                        Instruction::BinaryOp { op: base_op, .. } => {
                            let mut pattern_length = 1;
                            let mut same_op = true;
                            
                            // 連続する類似命令を特定
                            for j in i+1..std::cmp::min(i+16, block.instructions.len()) {
                                if let Some(Instruction::BinaryOp { op, .. }) = func.instructions.get(&block.instructions[j]) {
                                    if op == base_op {
                                        pattern_length += 1;
                                    } else {
                                        same_op = false;
                                        break;
                                    }
                                } else {
                                    same_op = false;
                                    break;
                                }
                            }
                            
                            // 4つ以上の同一演算があればベクトル化候補
                            if same_op && pattern_length >= 4 {
                                // このパターンをベクトル化可能としてマーク
                                debug!("ベクトル化可能なパターンを検出: {}個の連続{}演算", pattern_length, base_op);
                                vectorized = true;
                                i += pattern_length;
                                continue;
                            }
                        },
                        _ => {}
                    }
                }
                
                i += 1;
            }
        }
        
        Ok(vectorized)
    }
}

/// 高度なメモリアクセス最適化
impl WebAssemblyCodeGen {
    /// アライメントを考慮したロード命令
    fn build_optimized_load(&mut self, func_builder: &mut walrus::FunctionBuilder, 
                          ptr: &Operand, result: &str) -> Result<()> {
        // ポインタ評価
        let ptr_val = self.evaluate_operand(func_builder, ptr)?;
        
        // ポインタのタイプを特定
        let ptr_type = self.get_operand_type(ptr)?;
        let alignment = self.get_memory_alignment(&ptr_type);
        
        // ロード命令を構築（アライメント情報付き）
        let load = if alignment > 1 {
            // アライメントが指定されている場合は最適化したロード
            let mem_arg = walrus::ir::MemArg {
                offset: 0,
                align: alignment.trailing_zeros(), // log2(alignment)
            };
            
            match ptr_type {
                Type::Pointer(ref elem_ty) => {
                    match **elem_ty {
                        Type::Int(32) => func_builder.i32_load_with_args(ptr_val, mem_arg),
                        Type::Int(64) => func_builder.i64_load_with_args(ptr_val, mem_arg),
                        Type::Float(32) => func_builder.f32_load_with_args(ptr_val, mem_arg),
                        Type::Float(64) => func_builder.f64_load_with_args(ptr_val, mem_arg),
                        _ => func_builder.i32_load(ptr_val), // デフォルトはi32
                    }
                },
                _ => func_builder.i32_load(ptr_val), // 型が特定できない場合はデフォルト
            }
        } else {
            // アライメント情報がない場合は通常のロード
            func_builder.i32_load(ptr_val)
        };
        
        // 結果をマップに追加
        self.values.insert(result.to_string(), load);
        
        Ok(())
    }
    
    /// アライメントを考慮したストア命令
    fn build_optimized_store(&mut self, func_builder: &mut walrus::FunctionBuilder, 
                           ptr: &Operand, value: &Operand) -> Result<()> {
        // ポインタと値を評価
        let ptr_val = self.evaluate_operand(func_builder, ptr)?;
        let value_val = self.evaluate_operand(func_builder, value)?;
        
        // ポインタのタイプを特定
        let ptr_type = self.get_operand_type(ptr)?;
        let alignment = self.get_memory_alignment(&ptr_type);
        
        // ストア命令を構築（アライメント情報付き）
        if alignment > 1 {
            // アライメントが指定されている場合は最適化したストア
            let mem_arg = walrus::ir::MemArg {
                offset: 0,
                align: alignment.trailing_zeros(), // log2(alignment)
            };
            
            match ptr_type {
                Type::Pointer(ref elem_ty) => {
                    match **elem_ty {
                        Type::Int(32) => func_builder.i32_store_with_args(ptr_val, value_val, mem_arg),
                        Type::Int(64) => func_builder.i64_store_with_args(ptr_val, value_val, mem_arg),
                        Type::Float(32) => func_builder.f32_store_with_args(ptr_val, value_val, mem_arg),
                        Type::Float(64) => func_builder.f64_store_with_args(ptr_val, value_val, mem_arg),
                        _ => func_builder.i32_store(ptr_val, value_val), // デフォルトはi32
                    }
                },
                _ => func_builder.i32_store(ptr_val, value_val), // 型が特定できない場合はデフォルト
            }
        } else {
            // アライメント情報がない場合は通常のストア
            func_builder.i32_store(ptr_val, value_val)
        };
        
        Ok(())
    }
    
    /// メモリアクセスのアライメントを取得
    fn get_memory_alignment(&self, ty: &Type) -> u32 {
        match ty {
            Type::Pointer(elem_ty) => {
                match **elem_ty {
                    Type::Int(bits) => {
                        // 整数型のアライメント: ビット数/8（1バイト単位）
                        // ただし、1, 2, 4, 8のいずれかに丸める
                        let bytes = (bits / 8) as u32;
                        if bytes <= 1 { 1 }
                        else if bytes <= 2 { 2 }
                        else if bytes <= 4 { 4 }
                        else { 8 }
                    },
                    Type::Float(bits) => {
                        // 浮動小数点型のアライメント
                        match bits {
                            32 => 4,
                            64 => 8,
                            _ => 4,
                        }
                    },
                    Type::Array(ref arr_elem_ty, size) => {
                        // 配列のアライメント
                        // 要素のアライメントと同じとする
                        self.get_element_alignment(arr_elem_ty)
                    },
                    Type::Struct(_) => {
                        // 構造体は一般に8バイトアライメント
                        8
                    },
                    _ => 1, // その他は1バイト
                }
            },
            _ => 1, // ポインタ以外は1バイト
        }
    }
    
    /// 要素のアライメントを取得
    fn get_element_alignment(&self, ty: &Type) -> u32 {
        match ty {
            Type::Int(bits) => {
                let bytes = (bits / 8) as u32;
                if bytes <= 1 { 1 }
                else if bytes <= 2 { 2 }
                else if bytes <= 4 { 4 }
                else { 8 }
            },
            Type::Float(bits) => {
                match bits {
                    32 => 4,
                    64 => 8,
                    _ => 4,
                }
            },
            _ => 1,
        }
    }
} 