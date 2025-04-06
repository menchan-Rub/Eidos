use std::collections::HashMap;
use std::fmt::Write;
use std::sync::Arc;

use inkwell::context::Context;
use inkwell::module::Module as LLVMModule;
use inkwell::builder::Builder;
use inkwell::values::{FunctionValue, BasicValueEnum, BasicValue, PointerValue};
use inkwell::types::{BasicTypeEnum, BasicType};
use inkwell::targets::{Target, TargetMachine, InitializationConfig, RelocMode, CodeModel, FileType};
use inkwell::OptimizationLevel as LLVMOptLevel;
use log::{debug, info, error};

use crate::core::{Result, EidosError};
use crate::core::eir::{Module, Function, FunctionId, BlockId, Instruction, Operand, Literal, BinaryOp, UnaryOp};
use crate::core::types::{Type, TypeKind};

use super::codegen::{Backend, CodegenOptions, OutputFormat, Target as CodegenTarget};

/// LLVM バックエンド
pub struct LLVMBackend {
    /// LLVM コンテキスト
    context: Context,
    /// 型のキャッシュ
    type_cache: HashMap<Type, BasicTypeEnum<'static>>,
    /// 関数のマップ
    function_map: HashMap<String, FunctionValue<'static>>,
    /// グローバル変数のマップ
    global_map: HashMap<String, PointerValue<'static>>,
}

impl LLVMBackend {
    /// 新しいLLVMバックエンドを作成
    pub fn new() -> Self {
        // LLVMターゲットの初期化
        let config = InitializationConfig::default();
        Target::initialize_all(&config);
        
        Self {
            context: Context::create(),
            type_cache: HashMap::new(),
            function_map: HashMap::new(),
            global_map: HashMap::new(),
        }
    }
    
    /// EidosのEIR型をLLVM型に変換
    fn convert_type(&mut self, ty: &Type) -> Result<BasicTypeEnum<'static>> {
        // キャッシュにあれば返す
        if let Some(llvm_type) = self.type_cache.get(ty) {
            return Ok(*llvm_type);
        }
        
        // 型に応じて変換
        let llvm_type = match &ty.kind {
            TypeKind::Unit => {
                // Unit型はvoid型（構造体として表現）
                let struct_type = self.context.struct_type(&[], false);
                struct_type.into()
            },
            TypeKind::Bool => {
                // Bool型はi1
                self.context.bool_type().into()
            },
            TypeKind::Int => {
                // Int型はi64
                self.context.i64_type().into()
            },
            TypeKind::Float => {
                // Float型はdouble
                self.context.f64_type().into()
            },
            TypeKind::Char => {
                // Char型はi32（UTF-32）
                self.context.i32_type().into()
            },
            TypeKind::String => {
                // 文字列型はi8のポインタ
                let i8_type = self.context.i8_type();
                let ptr_type = i8_type.ptr_type(inkwell::AddressSpace::Generic);
                ptr_type.into()
            },
            TypeKind::Array(elem_type) => {
                // 配列型は要素型のポインタ
                let elem_llvm_type = self.convert_type(elem_type)?;
                match elem_llvm_type {
                    BasicTypeEnum::ArrayType(array_type) => array_type.ptr_type(inkwell::AddressSpace::Generic).into(),
                    BasicTypeEnum::FloatType(float_type) => float_type.ptr_type(inkwell::AddressSpace::Generic).into(),
                    BasicTypeEnum::IntType(int_type) => int_type.ptr_type(inkwell::AddressSpace::Generic).into(),
                    BasicTypeEnum::PointerType(ptr_type) => ptr_type.ptr_type(inkwell::AddressSpace::Generic).into(),
                    BasicTypeEnum::StructType(struct_type) => struct_type.ptr_type(inkwell::AddressSpace::Generic).into(),
                    BasicTypeEnum::VectorType(vector_type) => vector_type.ptr_type(inkwell::AddressSpace::Generic).into(),
                    _ => {
                        return Err(EidosError::CodeGen(format!(
                            "未対応の要素型: {:?}", elem_type
                        )));
                    }
                }
            },
            TypeKind::Tuple(elem_types) => {
                // タプル型は構造体
                let mut llvm_elem_types = Vec::new();
                for elem_type in elem_types {
                    let llvm_elem_type = self.convert_type(elem_type)?;
                    llvm_elem_types.push(llvm_elem_type);
                }
                let struct_type = self.context.struct_type(&llvm_elem_types.as_slice(), false);
                struct_type.into()
            },
            TypeKind::Function { params, return_type } => {
                // 関数型は関数ポインタ
                let mut llvm_param_types = Vec::new();
                for param_type in params {
                    let llvm_param_type = self.convert_type(param_type)?;
                    llvm_param_types.push(llvm_param_type);
                }
                
                let llvm_return_type = self.convert_type(return_type)?;
                let fn_type = match llvm_return_type {
                    BasicTypeEnum::ArrayType(array_type) => array_type.fn_type(&llvm_param_types, false),
                    BasicTypeEnum::FloatType(float_type) => float_type.fn_type(&llvm_param_types, false),
                    BasicTypeEnum::IntType(int_type) => int_type.fn_type(&llvm_param_types, false),
                    BasicTypeEnum::PointerType(ptr_type) => ptr_type.fn_type(&llvm_param_types, false),
                    BasicTypeEnum::StructType(struct_type) => struct_type.fn_type(&llvm_param_types, false),
                    BasicTypeEnum::VectorType(vector_type) => vector_type.fn_type(&llvm_param_types, false),
                    _ => {
                        return Err(EidosError::CodeGen(format!(
                            "未対応の戻り値型: {:?}", return_type
                        )));
                    }
                };
                fn_type.ptr_type(inkwell::AddressSpace::Generic).into()
            },
            TypeKind::Struct { name, fields, .. } => {
                // 構造体型
                let struct_type = self.context.opaque_struct_type(name);
                
                // 型のキャッシュに仮登録（循環参照対策）
                self.type_cache.insert(ty.clone(), struct_type.into());
                
                // フィールドの型を取得
                let mut field_types = Vec::new();
                for field in fields {
                    let field_type = self.convert_type(&field.field_type)?;
                    field_types.push(field_type);
                }
                
                // 構造体の本体を設定
                struct_type.set_body(&field_types, false);
                struct_type.into()
            },
            TypeKind::Enum { name, variants, .. } => {
                // 列挙型は判別子とペイロードを持つ構造体として表現
                // タグ用のi32と、各バリアントのユニオンを含む
                let enum_type = self.context.opaque_struct_type(name);
                
                // タグフィールド（i32）
                let tag_type = self.context.i32_type();
                
                // ペイロードフィールド（最大サイズのバリアントに合わせる）
                // ここでは簡易的に空構造体を使用
                let payload_type = self.context.struct_type(&[], false);
                
                // 構造体の本体を設定
                enum_type.set_body(&[tag_type.into(), payload_type.into()], false);
                enum_type.into()
            },
            _ => {
                // その他の型は未対応
                return Err(EidosError::CodeGen(format!(
                    "未対応の型: {:?}", ty
                )));
            }
        };
        
        // キャッシュに登録
        self.type_cache.insert(ty.clone(), llvm_type);
        
        Ok(llvm_type)
    }
    
    /// リテラル値をLLVM値に変換
    fn build_literal(&self, builder: &Builder, literal: &Literal) -> Result<BasicValueEnum<'static>> {
        match literal {
            Literal::Int(value) => {
                let int_type = self.context.i64_type();
                let int_value = int_type.const_int(*value as u64, true);
                Ok(int_value.into())
            },
            Literal::Float(value) => {
                let float_type = self.context.f64_type();
                let float_value = float_type.const_float(*value);
                Ok(float_value.into())
            },
            Literal::Bool(value) => {
                let bool_type = self.context.bool_type();
                let bool_value = bool_type.const_int(*value as u64, false);
                Ok(bool_value.into())
            },
            Literal::Char(value) => {
                let char_type = self.context.i32_type();
                let char_value = char_type.const_int(*value as u64, false);
                Ok(char_value.into())
            },
            Literal::String(value) => {
                // 文字列をLLVM定数文字列に変換
                let string_value = self.context.const_string(value.as_bytes(), true);
                let global = self.context.create_global_string_ptr(&value, "str");
                Ok(global.as_pointer_value().into())
            },
            Literal::Unit => {
                let unit_type = self.context.struct_type(&[], false);
                let unit_value = unit_type.const_named_struct(&[]);
                Ok(unit_value.into())
            },
        }
    }
    
    /// ターゲットマシンを取得
    fn get_target_machine(&self, target: &CodegenTarget) -> Result<TargetMachine> {
        let triple = match target {
            CodegenTarget::Native => Target::get_host_target_triple(),
            CodegenTarget::Target(triple_str) => {
                inkwell::targets::TargetTriple::create(triple_str)
            },
        };
        
        let target = Target::from_triple(&triple)
            .map_err(|e| EidosError::CodeGen(format!("ターゲットの取得に失敗: {:?}", e)))?;
        
        // 最適化レベルを変換
        let opt_level = LLVMOptLevel::Default;
        
        // ターゲットマシンを作成
        let target_machine = target.create_target_machine(
            &triple,
            "generic", // CPU
            "", // features
            opt_level,
            RelocMode::Default,
            CodeModel::Default,
        )
        .ok_or_else(|| EidosError::CodeGen("ターゲットマシンの作成に失敗".to_string()))?;
        
        Ok(target_machine)
    }

    /// オペランドからLLVM値を生成
    fn build_operand(&self, builder: &Builder, operand: &Operand, value_map: &HashMap<String, BasicValueEnum<'static>>) -> Result<BasicValueEnum<'static>> {
        match operand {
            Operand::Literal(literal) => self.build_literal(builder, literal),
            Operand::Variable(name) => {
                value_map.get(name)
                    .cloned()
                    .ok_or_else(|| EidosError::CodeGen(format!("変数が見つかりません: {}", name)))
            },
            Operand::GlobalRef(name) => {
                self.global_map.get(name)
                    .map(|&ptr| ptr.into())
                    .ok_or_else(|| EidosError::CodeGen(format!("グローバル変数が見つかりません: {}", name)))
            },
            Operand::FunctionRef(name) => {
                self.function_map.get(name)
                    .map(|&func| func.as_global_value().as_pointer_value().into())
                    .ok_or_else(|| EidosError::CodeGen(format!("関数が見つかりません: {}", name)))
            },
            _ => Err(EidosError::CodeGen(format!("未対応のオペランド: {:?}", operand))),
        }
    }

    /// 命令を生成
    fn build_instruction(&self, builder: &Builder, instr: &Instruction, value_map: &mut HashMap<String, BasicValueEnum<'static>>) -> Result<Option<BasicValueEnum<'static>>> {
        match instr {
            Instruction::BinaryOp { op, lhs, rhs, result } => {
                let lhs_value = self.build_operand(builder, lhs, value_map)?;
                let rhs_value = self.build_operand(builder, rhs, value_map)?;
                
                let result_value = match op {
                    BinaryOp::Add => {
                        if lhs_value.is_int_value() {
                            let lhs_int = lhs_value.into_int_value();
                            let rhs_int = rhs_value.into_int_value();
                            builder.build_int_add(lhs_int, rhs_int, "add").unwrap().into()
                        } else if lhs_value.is_float_value() {
                            let lhs_float = lhs_value.into_float_value();
                            let rhs_float = rhs_value.into_float_value();
                            builder.build_float_add(lhs_float, rhs_float, "add").unwrap().into()
                        } else {
                            return Err(EidosError::CodeGen("加算は整数か浮動小数点数のみサポートされています".to_string()));
                        }
                    },
                    BinaryOp::Sub => {
                        if lhs_value.is_int_value() {
                            let lhs_int = lhs_value.into_int_value();
                            let rhs_int = rhs_value.into_int_value();
                            builder.build_int_sub(lhs_int, rhs_int, "sub").unwrap().into()
                        } else if lhs_value.is_float_value() {
                            let lhs_float = lhs_value.into_float_value();
                            let rhs_float = rhs_value.into_float_value();
                            builder.build_float_sub(lhs_float, rhs_float, "sub").unwrap().into()
                        } else {
                            return Err(EidosError::CodeGen("減算は整数か浮動小数点数のみサポートされています".to_string()));
                        }
                    },
                    BinaryOp::Mul => {
                        if lhs_value.is_int_value() {
                            let lhs_int = lhs_value.into_int_value();
                            let rhs_int = rhs_value.into_int_value();
                            builder.build_int_mul(lhs_int, rhs_int, "mul").unwrap().into()
                        } else if lhs_value.is_float_value() {
                            let lhs_float = lhs_value.into_float_value();
                            let rhs_float = rhs_value.into_float_value();
                            builder.build_float_mul(lhs_float, rhs_float, "mul").unwrap().into()
                        } else {
                            return Err(EidosError::CodeGen("乗算は整数か浮動小数点数のみサポートされています".to_string()));
                        }
                    },
                    BinaryOp::Div => {
                        if lhs_value.is_int_value() {
                            let lhs_int = lhs_value.into_int_value();
                            let rhs_int = rhs_value.into_int_value();
                            builder.build_int_signed_div(lhs_int, rhs_int, "div").unwrap().into()
                        } else if lhs_value.is_float_value() {
                            let lhs_float = lhs_value.into_float_value();
                            let rhs_float = rhs_value.into_float_value();
                            builder.build_float_div(lhs_float, rhs_float, "div").unwrap().into()
                        } else {
                            return Err(EidosError::CodeGen("除算は整数か浮動小数点数のみサポートされています".to_string()));
                        }
                    },
                    BinaryOp::Mod => {
                        if lhs_value.is_int_value() {
                            let lhs_int = lhs_value.into_int_value();
                            let rhs_int = rhs_value.into_int_value();
                            builder.build_int_signed_rem(lhs_int, rhs_int, "rem").unwrap().into()
                        } else {
                            return Err(EidosError::CodeGen("剰余は整数のみサポートされています".to_string()));
                        }
                    },
                    // その他の演算子...
                    _ => return Err(EidosError::CodeGen(format!("未対応の二項演算子: {:?}", op))),
                };
                
                // 結果を値マップに保存
                value_map.insert(result.clone(), result_value);
                Ok(Some(result_value))
            },
            Instruction::UnaryOp { op, operand, result } => {
                let operand_value = self.build_operand(builder, operand, value_map)?;
                
                let result_value = match op {
                    UnaryOp::Neg => {
                        if operand_value.is_int_value() {
                            let operand_int = operand_value.into_int_value();
                            builder.build_int_neg(operand_int, "neg").unwrap().into()
                        } else if operand_value.is_float_value() {
                            let operand_float = operand_value.into_float_value();
                            builder.build_float_neg(operand_float, "neg").unwrap().into()
                        } else {
                            return Err(EidosError::CodeGen("負数化は整数か浮動小数点数のみサポートされています".to_string()));
                        }
                    },
                    UnaryOp::Not => {
                        if operand_value.is_int_value() {
                            let operand_int = operand_value.into_int_value();
                            builder.build_not(operand_int, "not").unwrap().into()
                        } else {
                            return Err(EidosError::CodeGen("論理否定はブール値のみサポートされています".to_string()));
                        }
                    },
                    // その他の演算子...
                    _ => return Err(EidosError::CodeGen(format!("未対応の単項演算子: {:?}", op))),
                };
                
                // 結果を値マップに保存
                value_map.insert(result.clone(), result_value);
                Ok(Some(result_value))
            },
            Instruction::Call { function, args, result } => {
                debug!("LLVM: 関数呼び出しの実行 - 関数: {}, 結果: {:?}", function, result);
                
                // 関数を評価
                let function_value = match self.function_map.get(function) {
                    Some(f) => *f,
                    None => {
                        // 関数が見つからない場合、外部関数として宣言を試みる
                        self.declare_function(function, &[])?;
                        *self.function_map.get(function).ok_or_else(|| {
                            EidosError::CodeGen(format!("関数 {} の宣言後も取得できませんでした", function))
                        })?
                    }
                };
                
                // LLVMビルダーを取得
                let builder = self.context.create_builder();
                
                // 引数を評価
                let mut arg_values = Vec::new();
                for arg in args {
                    let arg_value = self.stack.get(arg).ok_or_else(|| {
                        EidosError::CodeGen(format!("引数 {} が見つかりません", arg))
                    })?;
                    arg_values.push(*arg_value);
                }
                
                // 引数の数をチェック
                let expected_param_count = function_value.count_params() as usize;
                if arg_values.len() != expected_param_count {
                    return Err(EidosError::CodeGen(format!(
                        "関数 {} の引数の数が一致しません。期待: {}, 実際: {}", 
                        function, expected_param_count, arg_values.len()
                    )));
                }
                
                // 引数を必要に応じて変換
                let mut final_args = Vec::new();
                for (i, (arg_value, param)) in arg_values.iter().zip(function_value.get_param_iter()).enumerate() {
                    let arg_type = arg_value.get_type();
                    let param_type = param.get_type();
                    
                    // 型が一致しない場合は変換
                    let converted_arg = if arg_type != param_type {
                        match (arg_type.is_int_type(), param_type.is_int_type()) {
                            (true, true) => {
                                // 整数型同士の変換
                                let arg_int = arg_value.into_int_value();
                                let arg_width = arg_int.get_type().get_bit_width();
                                let param_width = param_type.into_int_type().get_bit_width();
                                
                                if param_width > arg_width {
                                    // 拡張
                                    builder.build_int_s_extend(
                                        arg_int, 
                                        param_type.into_int_type(), 
                                        &format!("arg{}_ext", i)
                                    ).as_basic_value_enum()
                                } else if param_width < arg_width {
                                    // 切り詰め
                                    builder.build_int_truncate(
                                        arg_int,
                                        param_type.into_int_type(),
                                        &format!("arg{}_trunc", i)
                                    ).as_basic_value_enum()
                                } else {
                                    *arg_value
                                }
                            },
                            (false, false) if arg_type.is_float_type() && param_type.is_float_type() => {
                                // 浮動小数点型同士の変換
                                let arg_float = arg_value.into_float_value();
                                let arg_width = arg_float.get_type().get_bit_width();
                                let param_width = param_type.into_float_type().get_bit_width();
                                
                                if param_width > arg_width {
                                    // 拡張
                                    builder.build_float_ext(
                                        arg_float,
                                        param_type.into_float_type(),
                                        &format!("arg{}_ext", i)
                                    ).as_basic_value_enum()
                                } else if param_width < arg_width {
                                    // 切り詰め
                                    builder.build_float_trunc(
                                        arg_float,
                                        param_type.into_float_type(),
                                        &format!("arg{}_trunc", i)
                                    ).as_basic_value_enum()
                                } else {
                                    *arg_value
                                }
                            },
                            (_, _) if arg_type.is_pointer_type() && param_type.is_pointer_type() => {
                                // ポインタ型同士の変換（ビットキャスト）
                                builder.build_bitcast(
                                    *arg_value,
                                    param_type,
                                    &format!("arg{}_cast", i)
                                )
                            },
                            _ => {
                                return Err(EidosError::CodeGen(format!(
                                    "引数 {} の型 {:?} を {:?} に変換できません", 
                                    i, arg_type, param_type
                                )));
                            }
                        }
                    } else {
                        *arg_value
                    };
                    
                    final_args.push(converted_arg);
                }
                
                // 関数呼び出し
                let call_site = builder.build_call(
                    function_value,
                    &final_args,
                    result.as_deref().unwrap_or("")
                );
                
                // 呼び出し規約の設定
                if function.contains("_stdcall") {
                    call_site.set_call_convention(inkwell::values::CallConv::X86Stdcall);
                } else if function.contains("_fastcall") {
                    call_site.set_call_convention(inkwell::values::CallConv::X86Fastcall);
                }
                
                // テール呼び出しの設定
                if function.contains("_tail") || result.as_deref().unwrap_or("").contains("_tail") {
                    call_site.set_tail_call(true);
                }
                
                // 結果の処理
                if let Some(res) = result {
                    // 関数の戻り値の型を取得
                    let return_type = function_value.get_type().get_return_type();
                    
                    if let Some(ret_type) = return_type {
                        // 戻り値がある場合
                        if let Some(value) = call_site.try_as_basic_value().left() {
                            // 値をスタックに格納
                            self.stack.insert(res.clone(), value);
                            debug!("関数呼び出し結果をスタックに格納: {}", res);
                        } else {
                            // 戻り値がvoidの場合はUnit型を使用
                            let unit_value = self.context.struct_type(&[], false)
                                .const_named_struct(&[])
                                .as_basic_value_enum();
                            self.stack.insert(res.clone(), unit_value);
                            debug!("関数呼び出し結果（void）をUnitとしてスタックに格納: {}", res);
                        }
                    } else {
                        // 戻り値がvoidの場合
                        let unit_value = self.context.struct_type(&[], false)
                            .const_named_struct(&[])
                            .as_basic_value_enum();
                        self.stack.insert(res.clone(), unit_value);
                        debug!("void戻り値関数の呼び出し結果をUnitとしてスタックに格納: {}", res);
                    }
                }
                
                debug!("LLVM: 関数呼び出し完了 - {}", function);
                
                Ok(())
            },
            Instruction::Alloca { ty, result } => {
                // 型の変換
                let llvm_type = match self.convert_type(ty) {
                    Ok(ty) => ty,
                    Err(e) => return Err(e),
                };
                
                // アラインメント情報の取得
                let alignment = self.get_type_alignment(ty);
                
                // アロケーション命令を生成
                let alloca = builder.build_alloca(llvm_type, result).unwrap();
                
                // アラインメントを設定
                alloca.set_alignment(alignment).unwrap();
                
                // スタック位置の最適化（可能な限りレジスタ割り当てを促進）
                let entry_block = builder.get_insert_block().unwrap().get_parent().unwrap().get_first_block().unwrap();
                let current_block = builder.get_insert_block().unwrap();
                
                if entry_block != current_block {
                    // エントリブロックの先頭に移動
                    let current_position = builder.get_insert_point().unwrap();
                    builder.position_at_start(entry_block);
                    
                    // アロケーション命令を生成（エントリブロックの先頭に配置）
                    let optimized_alloca = builder.build_alloca(llvm_type, result).unwrap();
                    optimized_alloca.set_alignment(alignment).unwrap();
                    
                    // 元の位置に戻る
                    builder.position_at(current_block, current_position);
                    
                    value_map.insert(result.clone(), optimized_alloca.into());
                    Ok(Some(optimized_alloca.into()))
                } else {
                    // すでにエントリブロックにいる場合
                    value_map.insert(result.clone(), alloca.into());
                    Ok(Some(alloca.into()))
                }
            },
            Instruction::Load { address, result, ty } => {
                debug!("LLVM: load命令の実行 - 結果: {}, アドレス: {}", result, address);
                
                // アドレスを評価
                let ptr_value = self.stack.get(&address).ok_or_else(|| {
                    EidosError::CodeGen(format!("アドレス {}が見つかりません", address))
                })?;
                
                // ポインタ型かチェック
                let ptr_type = ptr_value.get_type();
                if !ptr_type.is_pointer_type() {
                    return Err(EidosError::CodeGen(format!(
                        "load命令のアドレス {}はポインタ型ではありません: {:?}", 
                        address, ptr_type
                    )));
                }
                
                // LLVMビルダーを取得
                let builder = self.get_builder()?;
                
                // 期待される型と一致するか確認
                let expected_type = self.convert_type(ty)?;
                let element_type = ptr_type.into_pointer_type().get_element_type();
                
                // 型のチェック（デバッグ用）
                debug!("Load命令: 期待される型: {:?}, 実際の要素型: {:?}", expected_type, element_type);
                
                // ロード元のアライメントを取得
                let alignment = self.get_type_alignment(ty);
                debug!("Load命令: アライメント設定: {}", alignment);
                
                // load命令を生成
                let load = builder.build_load(expected_type, *ptr_value, result);
                
                // アライメントを設定
                load.set_alignment(alignment).map_err(|e| {
                    EidosError::CodeGen(format!("アライメント設定エラー: {:?}", e))
                })?;
                
                // プロファイリング情報を設定（可能な場合）
                if cfg!(feature = "profiling") {
                    // プロファイリング情報のメタデータを作成
                    let metadata_string = format!("profile_data_for_{}", result);
                    let metadata = self.context.metadata_string(metadata_string.as_str());
                    let metadata_node = self.context.metadata_node(&[metadata.into()]);
                    
                    // メタデータをロード命令に関連付け
                    load.set_metadata(metadata_node, 0).map_err(|e| {
                        EidosError::CodeGen(format!("メタデータ設定エラー: {:?}", e))
                    })?;
                }
                
                // ボラティリティ（volatile）属性の設定
                if result.contains("atomic") || result.contains("volatile") {
                    load.set_volatile(true).map_err(|e| {
                        EidosError::CodeGen(format!("volatile属性設定エラー: {:?}", e))
                    })?;
                }
                
                // アトミック操作の設定
                if result.contains("atomic") {
                    load.set_atomic_ordering(AtomicOrdering::SequentiallyConsistent).map_err(|e| {
                        EidosError::CodeGen(format!("アトミック順序設定エラー: {:?}", e))
                    })?;
                }
                
                // 非NULL保証がある場合の最適化
                if result.contains("nonnull") {
                    // 非NULLメタデータを設定
                    let nonnull_kind_id = self.context.get_enum_attribute_kind_id("nonnull");
                    let nonnull_attr = self.context.create_enum_attribute(nonnull_kind_id, 0);
                    load.add_attribute(AttributeLoc::Return, nonnull_attr);
                }
                
                // 結果をスタックに格納
                self.stack.insert(result.clone(), load.into());
                debug!("LLVM: load命令完了 - 結果をスタックに追加: {}", result);
                
                Ok(())
            },
            Instruction::Store { ptr, value, ty } => {
                debug!("LLVM: store命令の実行 - ポインタ: {}, 値: {}", ptr, value);
                
                // ポインタを評価
                let ptr_value = self.stack.get(&ptr).ok_or_else(|| {
                    EidosError::CodeGen(format!("ポインタ {}が見つかりません", ptr))
                })?;
                
                // ポインタ型かチェック
                let ptr_type = ptr_value.get_type();
                if !ptr_type.is_pointer_type() {
                    return Err(EidosError::CodeGen(format!(
                        "store命令のポインタ {}はポインタ型ではありません: {:?}", 
                        ptr, ptr_type
                    )));
                }
                
                // 格納する値を評価
                let store_value = self.stack.get(&value).ok_or_else(|| {
                    EidosError::CodeGen(format!("値 {}が見つかりません", value))
                })?;
                
                // LLVMビルダーを取得
                let builder = self.get_builder()?;
                
                // ポインタ要素型と値の型の互換性をチェック
                let element_type = ptr_type.into_pointer_type().get_element_type();
                let value_type = store_value.get_type();
                
                debug!("Store命令: 要素型: {:?}, 値の型: {:?}", element_type, value_type);
                
                // 必要に応じて型変換を行う
                let final_value = if element_type.is_int_type() && value_type.is_int_type() {
                    let dest_int_type = element_type.into_int_type();
                    let src_int_type = value_type.into_int_type();
                    
                    if dest_int_type.get_bit_width() != src_int_type.get_bit_width() {
                        // ビット幅が異なる場合、適切な拡張または切り詰めを行う
                        if dest_int_type.get_bit_width() > src_int_type.get_bit_width() {
                            // 符号拡張
                            let extended = builder.build_int_s_extend(
                                store_value.into_int_value(), 
                                dest_int_type, 
                                &format!("{}_extended", value)
                            );
                            extended.as_basic_value_enum()
                        } else {
                            // 切り詰め
                            let truncated = builder.build_int_truncate(
                                store_value.into_int_value(), 
                                dest_int_type, 
                                &format!("{}_truncated", value)
                            );
                            truncated.as_basic_value_enum()
                        }
                    } else {
                        *store_value
                    }
                } else if element_type.is_float_type() && value_type.is_float_type() {
                    let dest_float_type = element_type.into_float_type();
                    let src_float_type = value_type.into_float_type();
                    
                    if dest_float_type != src_float_type {
                        // 浮動小数点型の変換
                        if dest_float_type.get_bit_width() > src_float_type.get_bit_width() {
                            // 拡張
                            let extended = builder.build_float_ext(
                                store_value.into_float_value(), 
                                dest_float_type, 
                                &format!("{}_extended", value)
                            );
                            extended.as_basic_value_enum()
                        } else {
                            // 切り詰め
                            let truncated = builder.build_float_trunc(
                                store_value.into_float_value(), 
                                dest_float_type, 
                                &format!("{}_truncated", value)
                            );
                            truncated.as_basic_value_enum()
                        }
                    } else {
                        *store_value
                    }
                } else if element_type.is_pointer_type() && value_type.is_pointer_type() {
                    // ポインタ型同士の場合、ビットキャストを行う
                    let bit_cast = builder.build_bitcast(
                        *store_value,
                        element_type,
                        &format!("{}_cast", value)
                    );
                    bit_cast
                } else {
                    // その他の型の場合は直接使用
                    *store_value
                };
                
                // アライメントを取得
                let alignment = self.get_type_alignment(ty);
                debug!("Store命令: アライメント設定: {}", alignment);
                
                // store命令を生成
                let store = builder.build_store(*ptr_value, final_value);
                
                // アライメントを設定
                store.set_alignment(alignment).map_err(|e| {
                    EidosError::CodeGen(format!("アライメント設定エラー: {:?}", e))
                })?;
                
                // ボラティリティ（volatile）属性の設定
                if ptr.contains("atomic") || ptr.contains("volatile") || value.contains("atomic") || value.contains("volatile") {
                    store.set_volatile(true).map_err(|e| {
                        EidosError::CodeGen(format!("volatile属性設定エラー: {:?}", e))
                    })?;
                }
                
                // アトミック操作の設定
                if ptr.contains("atomic") || value.contains("atomic") {
                    store.set_atomic_ordering(AtomicOrdering::SequentiallyConsistent).map_err(|e| {
                        EidosError::CodeGen(format!("アトミック順序設定エラー: {:?}", e))
                    })?;
                }
                
                debug!("LLVM: store命令完了");
                
                Ok(())
            },
            Instruction::GetElementPtr { ptr, indices, result } => {
                // ポインタをスタックにプッシュ
                let ptr_value = self.build_operand(builder, ptr, value_map)?;
                if !ptr_value.is_pointer_value() {
                    return Err(EidosError::CodeGen(format!("GEPの基底はポインタである必要があります。指定された値: {:?}", ptr)));
                }
                
                // ポインタ型を解析して要素型を取得
                let ptr_type = ptr_value.into_pointer_value().get_type();
                let pointee_type = ptr_type.get_element_type();
                
                debug!("GEP操作: ポインタ型 = {:?}, 要素型 = {:?}", ptr_type, pointee_type);
                
                // インデックスを評価
                let mut index_values = Vec::new();
                for (i, idx) in indices.iter().enumerate() {
                    let idx_value = self.build_operand(builder, idx, value_map)?;
                    if !idx_value.is_int_value() {
                        return Err(EidosError::CodeGen(format!("GEPのインデックス{}は整数である必要があります。指定された値: {:?}", i, idx)));
                    }
                    index_values.push(idx_value.into_int_value());
                }
                
                // 要素型が構造体で、インデックスが定数の場合の最適化
                if pointee_type.is_struct_type() && indices.len() >= 2 {
                    if let Operand::Literal(Literal::Int(field_idx)) = &indices[1] {
                        debug!("構造体フィールドアクセス: インデックス = {}", field_idx);
                    }
                }
                
                // 要素ポインタを計算
                let gep = unsafe {
                    // aligned accessにヒントを与える
                    let mut flags = inkwell::values::InstructionOpcode::GetElementPtr;
                    if let Some(alignment) = self.get_type_alignment(&pointee_type) {
                        if alignment > 1 {
                            debug!("GEP: アライメントヒント: {}", alignment);
                            // LLVMにアライメント情報を伝える
                            let gep = builder.build_gep(ptr_value.into_pointer_value(), &index_values, result).unwrap();
                            
                            // アライメント属性を設定
                            gep.set_alignment(alignment).unwrap();
                            
                            gep
                        } else {
                            // アライメントが1の場合は通常のGEP
                            builder.build_gep(ptr_value.into_pointer_value(), &index_values, result).unwrap()
                        }
                    } else {
                        // アライメント情報がない場合は通常のGEP
                        builder.build_gep(ptr_value.into_pointer_value(), &index_values, result).unwrap()
                    }
                };
                
                // 結果を値マップに格納
                value_map.insert(result.clone(), gep.into());
                Ok(Some(gep.into()))
            },
            Instruction::Branch { target } => {
                // 単純な分岐
                let target_block = value_map
                    .get(&format!("block_{:?}", target))
                    .and_then(|v| {
                        if v.is_basic_block() {
                            Some(v.into_basic_block())
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| EidosError::CodeGen(format!("分岐先ブロックが見つかりません: {:?}", target)))?;
                
                builder.build_unconditional_branch(target_block).unwrap();
                Ok(None)
            },
            Instruction::ConditionalBranch { condition, true_target, false_target } => {
                // 条件をスタックにプッシュ
                let cond_value = self.build_operand(builder, condition, value_map)?;
                if !cond_value.is_int_value() {
                    return Err(EidosError::CodeGen(format!("条件は整数（ブール）である必要があります。指定された値: {:?}", condition)));
                }
                
                // 条件値が1ビットのi1型であることを確認
                let cond_int = cond_value.into_int_value();
                if cond_int.get_type().get_bit_width() != 1 {
                    // 1ビットに変換する
                    debug!("条件値が1ビットでないため、比較操作を追加");
                    let zero = self.context.i32_type().const_int(0, false);
                    let cond_int = builder.build_int_compare(
                        inkwell::IntPredicate::NE,
                        cond_int,
                        zero,
                        "cond_i1"
                    ).unwrap();
                }
                
                // 真の分岐先と偽の分岐先を取得
                let true_block = value_map
                    .get(&format!("block_{:?}", true_target))
                    .and_then(|v| {
                        if v.is_basic_block() {
                            Some(v.into_basic_block())
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| EidosError::CodeGen(format!("真の分岐先ブロックが見つかりません: {:?}", true_target)))?;
                
                let false_block = value_map
                    .get(&format!("block_{:?}", false_target))
                    .and_then(|v| {
                        if v.is_basic_block() {
                            Some(v.into_basic_block())
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| EidosError::CodeGen(format!("偽の分岐先ブロックが見つかりません: {:?}", false_target)))?;
                
                // 最適化: 分岐予測ヒントを追加
                let branch_weights = match condition {
                    // リテラルの場合は確実な分岐を予測
                    Operand::Literal(Literal::Bool(true)) => Some((100, 0)),
                    Operand::Literal(Literal::Bool(false)) => Some((0, 100)),
                    // 標準的な比較操作の場合は確率的予測を行う
                    Operand::InstructionRef(_) => {
                        // 等値比較は通常false（一致しない）の確率が高い
                        if let Some(Instruction::BinaryOp { op, .. }) = func.instructions.get(&condition.as_instruction_id().unwrap()) {
                            match op.as_str() {
                                "eq" => Some((30, 70)),  // 等値比較は通常失敗する確率が高い
                                "ne" => Some((70, 30)),  // 不等比較は通常成功する確率が高い
                                _ => None
                            }
                        } else {
                            None
                        }
                    },
                    _ => None,
                };
                
                // 条件分岐を生成
                let br = if let Some((true_weight, false_weight)) = branch_weights {
                    debug!("分岐予測ヒント追加: true={}, false={}", true_weight, false_weight);
                    let metadata = self.context.metadata_node(&[
                        self.context.i32_type().const_int(true_weight as u64, false).into(),
                        self.context.i32_type().const_int(false_weight as u64, false).into()
                    ]);
                    
                    // 予測ヒント付き条件分岐
                    let br = builder.build_conditional_branch(
                        cond_int.into_int_value(),
                        true_block,
                        false_block
                    ).unwrap();
                    
                    // メタデータを設定
                    br.set_metadata(metadata, "branch_weights");
                    br
                } else {
                    // 通常の条件分岐
                    builder.build_conditional_branch(
                        cond_int.into_int_value(),
                        true_block,
                        false_block
                    ).unwrap()
                };
                
                Ok(None)
            },
            Instruction::Return { value } => {
                match value {
                    Some(val) => {
                        let return_value = self.build_operand(builder, val, value_map)?;
                        builder.build_return(Some(&return_value)).unwrap();
                    },
                    None => {
                        builder.build_return(None).unwrap();
                    }
                }
                Ok(None)
            },
            // その他の命令...
            _ => Err(EidosError::CodeGen(format!("未対応の命令: {:?}", instr))),
        }
    }

    /// 型のアライメントを取得
    fn get_type_alignment(&self, ty: &Type) -> u32 {
        match ty {
            Type::Int(bits) => {
                // 整数型のアライメント: ビット数/8（1バイト単位）
                // ただし、最小1バイト、最大8バイト（64ビット）
                let align = (bits / 8).max(1).min(8);
                align as u32
            },
            Type::Float(bits) => {
                // 浮動小数点型のアライメント: 32ビットなら4バイト、64ビットなら8バイト
                match bits {
                    32 => 4,
                    64 => 8,
                    _ => 4, // その他のサイズは4バイト
                }
            },
            Type::Array(_, size) => {
                // 配列型は要素数が多い場合は大きなアライメントを使用
                if *size > 8 {
                    16 // SIMD命令を使いやすくするため
                } else {
                    8
                }
            },
            Type::Pointer(_) => {
                // ポインタ型はターゲットのポインタサイズに合わせる
                self.context.data_layout().get_pointer_size() as u32
            },
            Type::Struct(_) => {
                // 構造体は通常16バイトアライメント（キャッシュライン考慮）
                16
            },
            _ => 8, // その他の型は8バイト
        }
    }

    /// Load命令のアライメントヒントを取得
    fn get_load_alignment(&self, ptr_ty: &Type) -> u32 {
        if let Type::Pointer(elem_ty) = ptr_ty {
            self.get_type_alignment(elem_ty)
        } else {
            8 // デフォルト
        }
    }

    /// Store命令のアライメントヒントを取得
    fn get_store_alignment(&self, ptr_ty: &Type, value_ty: &Type) -> u32 {
        if let Type::Pointer(elem_ty) = ptr_ty {
            self.get_type_alignment(elem_ty)
        } else {
            self.get_type_alignment(value_ty)
        }
    }

    /// メモリアクセスの最適化
    fn optimize_memory_access(&self, builder: &Builder, value: inkwell::values::PointerValue, alignment: u32) -> inkwell::values::PointerValue {
        // メモリアクセスが最適化可能なら、アライメントヒントを付加
        let ptr_type = value.get_type();
        builder.set_alignment(alignment, value);
        value
    }

    /// Instruction::Load命令の実装
    fn build_load(&mut self, builder: &Builder, ptr: &Operand, result: &str) -> Result<()> {
        // ポインタをスタックにプッシュし、評価
        self.push_operand(builder, ptr)?;
        
        // スタックから値をポップ
        let ptr_value = self.pop_value()?;
        
        // ポインタであることを確認
        if let Some(ptr) = ptr_value.as_pointer_value() {
            // ポインタ型を取得して要素型を決定
            let pointee_type = ptr.get_type().get_element_type();
            
            // アライメントヒントを取得
            let alignment = self.get_load_alignment(&self.get_operand_type(ptr)?);
            
            // アライメント最適化を適用
            let optimized_ptr = self.optimize_memory_access(builder, ptr, alignment);
            
            // 最適化されたポインタからロード
            let load = builder.build_load(pointee_type, optimized_ptr, result);
            
            // 結果を値マップに格納
            self.value_map.insert(result.to_string(), load.into());
            
            Ok(())
        } else {
            Err(EidosError::CodeGen(format!("ポインタ型が必要です（Load命令）: {:?}", ptr_value)))
        }
    }

    /// Instruction::Store命令の実装
    fn build_store(&mut self, builder: &Builder, ptr: &Operand, value: &Operand) -> Result<()> {
        // 値をスタックにプッシュして評価
        self.push_operand(builder, value)?;
        let value_to_store = self.pop_value()?;
        
        // ポインタをスタックにプッシュして評価
        self.push_operand(builder, ptr)?;
        let ptr_value = self.pop_value()?;
        
        // ポインタであることを確認
        if let Some(ptr) = ptr_value.as_pointer_value() {
            // アライメントヒントを取得
            let ptr_type = self.get_operand_type(ptr)?;
            let value_type = self.get_value_type(&value_to_store)?;
            let alignment = self.get_store_alignment(&ptr_type, &value_type);
            
            // アライメント最適化を適用
            let optimized_ptr = self.optimize_memory_access(builder, ptr, alignment);
            
            // 最適化されたポインタに書き込み
            builder.build_store(optimized_ptr, value_to_store);
            
            Ok(())
        } else {
            Err(EidosError::CodeGen(format!("ポインタ型が必要です（Store命令）: {:?}", ptr_value)))
        }
    }

    /// Instruction::GetElementPtr命令の実装
    fn build_getelemptr(&mut self, builder: &Builder, ptr: &Operand, indices: &[Operand], result: &str) -> Result<()> {
        debug!("GetElementPtr: ptr={:?}, indices={:?}", ptr, indices);
        
        // ポインタをスタックにプッシュし、評価
        self.push_operand(builder, ptr)?;
        
        // スタックから値をポップ
        let ptr_value = self.pop_value()?;
        
        // ポインタであることを確認
        if let Some(ptr) = ptr_value.as_pointer_value() {
            debug!("ポインタ型: {:?}", ptr.get_type());
            
            // ポインタの要素型を取得
            let elem_type = match ptr.get_type().get_element_type() {
                ty => {
                    debug!("要素型: {:?}", ty);
                    ty
                }
            };
            
            // インデックスを評価
            let mut index_values = Vec::new();
            for index in indices {
                self.push_operand(builder, index)?;
                let index_value = self.pop_value()?;
                
                // インデックスが整数型であることを確認
                if let Some(idx) = index_value.into_int_value() {
                    index_values.push(idx);
                } else {
                    return Err(EidosError::CodeGen(format!("インデックスは整数型が必要です: {:?}", index)));
                }
            }
            
            // インデックスが空の場合は空のインデックスリストを作成
            if index_values.is_empty() {
                index_values.push(self.context.i32_type().const_int(0, false));
            }
            
            // GetElementPtrを構築
            let gep = unsafe {
                builder.build_gep(elem_type, ptr, &index_values, result)
            };
            
            // アライメントヒントを計算して設定
            let ptr_type = self.get_operand_type(ptr)?;
            if let Type::Pointer(elem_ty) = ptr_type {
                let alignment = self.get_type_alignment(&elem_ty);
                builder.set_alignment(alignment, gep);
            }
            
            // 結果を値マップに格納
            self.value_map.insert(result.to_string(), gep.into());
            
            Ok(())
        } else {
            Err(EidosError::CodeGen(format!("ポインタ型が必要です（GetElementPtr命令）: {:?}", ptr_value)))
        }
    }

    fn declare_global(&mut self, name: &str, ty: &Type, initializer: Option<&Literal>) -> Result<()> {
        info!("グローバル変数を宣言: {}", name);
        
        // LLVMモジュールを作成（まだ作成していない場合）
        let llvm_module = self.context.create_module(&format!("global_module_{}", name));
        
        // 型を変換
        let llvm_type = self.convert_type(ty)?;
        
        // グローバル変数を作成
        let global = llvm_module.add_global(llvm_type, None, name);
        
        // リンケージを設定
        if initializer.is_some() {
            // 初期化子があれば、インライン化可能な外部リンケージ
            global.set_linkage(inkwell::module::Linkage::ExternalLinkage);
        } else {
            // 初期化子がなければ、単なる外部変数
            global.set_linkage(inkwell::module::Linkage::External);
        }
        
        // スレッドローカルストレージかチェック
        if name.starts_with("__thread_") || name.ends_with("_tls") {
            global.set_thread_local(true);
            debug!("グローバル変数 {} をスレッドローカルとして設定", name);
        }
        
        // セクション属性の設定
        if name.starts_with("__const_") {
            // 定数セクションに配置
            global.set_section(".rodata");
            global.set_constant(true);
        } else if name.starts_with("__bss_") {
            // 未初期化データセクションに配置
            global.set_section(".bss");
        } else if name.starts_with("__data_") {
            // 初期化済みデータセクションに配置
            global.set_section(".data");
        }
        
        // アライメント属性の設定
        let alignment = match ty {
            Type { kind: TypeKind::Int } => 8,
            Type { kind: TypeKind::Float } => 8,
            Type { kind: TypeKind::Array(ref elem_type, _) } => {
                // 配列のアライメントは要素型に基づく
                match elem_type.kind {
                    TypeKind::Int => 8,
                    TypeKind::Float => 8,
                    _ => 4,
                }
            },
            Type { kind: TypeKind::Struct { .. } } => 16, // 構造体は大きめのアライメント
            _ => 4, // デフォルトは4バイトアライメント
        };
        global.set_alignment(alignment);
        
        // 初期化子があれば設定
        if let Some(lit) = initializer {
            let init_val = match lit {
                Literal::Int(value) => {
                    let int_type = self.context.i64_type();
                    int_type.const_int(*value as u64, true).as_basic_value_enum()
                },
                Literal::Float(value) => {
                    let float_type = self.context.f64_type();
                    float_type.const_float(*value).as_basic_value_enum()
                },
                Literal::Bool(value) => {
                    let bool_type = self.context.bool_type();
                    bool_type.const_int(*value as u64, false).as_basic_value_enum()
                },
                Literal::Char(value) => {
                    let char_type = self.context.i32_type();
                    char_type.const_int(*value as u64, false).as_basic_value_enum()
                },
                Literal::String(value) => {
                    // 文字列リテラルはi8配列として定義
                    let i8_type = self.context.i8_type();
                    let array_type = i8_type.array_type(value.len() as u32 + 1); // +1 for null terminator
                    
                    // 各バイトを定数として作成
                    let mut values = Vec::new();
                    for byte in value.bytes() {
                        values.push(i8_type.const_int(byte as u64, false));
                    }
                    // NULL終端を追加
                    values.push(i8_type.const_int(0, false));
                    
                    // 定数配列を作成
                    array_type.const_array(&values).as_basic_value_enum()
                },
                Literal::Unit => {
                    let unit_type = self.context.struct_type(&[], false);
                    unit_type.const_named_struct(&[]).as_basic_value_enum()
                },
                _ => return Err(EidosError::CodeGen(format!("未対応の初期化子型: {:?}", lit))),
            };
            
            global.set_initializer(&init_val);
            
            // 定数の場合は属性を設定
            match lit {
                Literal::Int(_) | Literal::Float(_) | Literal::Bool(_) | Literal::Char(_) | Literal::String(_) => {
                    // イミュータブルな定数として設定
                    global.set_constant(true);
                    // 重複を排除するためにユニークなセクションに配置
                    global.set_section(".rodata.cst");
                    // 必要に応じてコンパイラが最適化できるようにする
                    global.set_unnamed_addr(true);
                },
                _ => {}
            }
        } else {
            // 初期化子がない場合はゼロで初期化
            match llvm_type {
                BasicTypeEnum::IntType(int_type) => {
                    global.set_initializer(&int_type.const_zero());
                },
                BasicTypeEnum::FloatType(float_type) => {
                    global.set_initializer(&float_type.const_zero());
                },
                BasicTypeEnum::PointerType(ptr_type) => {
                    global.set_initializer(&ptr_type.const_null());
                },
                BasicTypeEnum::StructType(struct_type) => {
                    // 構造体の各フィールドをゼロ初期化
                    let field_count = struct_type.count_fields();
                    let mut zero_fields = Vec::with_capacity(field_count as usize);
                    
                    for i in 0..field_count {
                        let field_type = struct_type.get_field_type_at_index(i)
                            .ok_or_else(|| EidosError::CodeGen(format!("構造体フィールド型の取得に失敗: {}.{}", name, i)))?;
                        
                        match field_type {
                            BasicTypeEnum::IntType(int_type) => {
                                // 整数フィールドをゼロ初期化
                                let int_zero = int_type.const_zero();
                                zero_fields.push(int_zero.as_basic_value_enum());
                            },
                            BasicTypeEnum::FloatType(float_type) => {
                                // 浮動小数点フィールドをゼロ初期化
                                let float_zero = float_type.const_zero();
                                zero_fields.push(float_zero.as_basic_value_enum());
                            },
                            BasicTypeEnum::PointerType(ptr_type) => {
                                // ポインタフィールドをNULLに初期化
                                let null_ptr = ptr_type.const_null();
                                zero_fields.push(null_ptr.as_basic_value_enum());
                            },
                            BasicTypeEnum::StructType(nested_struct_type) => {
                                // ネストした構造体をゼロ初期化
                                let struct_zero = nested_struct_type.const_zero();
                                zero_fields.push(struct_zero.as_basic_value_enum());
                            },
                            BasicTypeEnum::ArrayType(array_type) => {
                                // 配列をゼロ初期化
                                let array_zero = array_type.const_zero();
                                zero_fields.push(array_zero.as_basic_value_enum());
                            },
                            _ => return Err(EidosError::CodeGen(format!("未対応のフィールド型: {:?}", field_type))),
                        }
                    }
                    
                    global.set_initializer(&struct_type.const_named_struct(&zero_fields));
                },
                BasicTypeEnum::ArrayType(array_type) => {
                    // 配列をゼロ初期化
                    global.set_initializer(&array_type.const_zero());
                },
                _ => return Err(EidosError::CodeGen(format!("未対応のグローバル変数型: {:?}", llvm_type))),
            }
        }
        
        // グローバル変数マップに追加
        self.global_map.insert(name.to_string(), global);
        
        Ok(())
    }
}

impl Backend for LLVMBackend {
    fn name(&self) -> &str {
        "llvm"
    }
    
    fn compile(&self, module: &Module, options: &CodegenOptions) -> Result<Vec<u8>> {
        // LLVM モジュールを作成
        let llvm_module = self.context.create_module(&module.name);
        
        // 関数を生成
        for (name, func) in &module.functions {
            // 関数の型を取得
            let param_types: Vec<Type> = func.parameters.iter().map(|p| p.ty.clone()).collect();
            let return_type = func.return_type.clone();
            
            // LLVM 関数型を作成
            let mut llvm_param_types = Vec::new();
            for param_type in &param_types {
                let llvm_param_type = self.convert_type(param_type)?;
                llvm_param_types.push(llvm_param_type);
            }
            
            let llvm_return_type = self.convert_type(&return_type)?;
            let function_type = match llvm_return_type {
                BasicTypeEnum::IntType(int_type) => int_type.fn_type(&llvm_param_types, false),
                BasicTypeEnum::FloatType(float_type) => float_type.fn_type(&llvm_param_types, false),
                BasicTypeEnum::PointerType(ptr_type) => ptr_type.fn_type(&llvm_param_types, false),
                BasicTypeEnum::StructType(struct_type) => struct_type.fn_type(&llvm_param_types, false),
                _ => return Err(EidosError::CodeGen(format!("未対応の戻り値型: {:?}", return_type))),
            };
            
            // 関数を作成
            let function = llvm_module.add_function(&func.name, function_type, None);
            
            // 関数本体を生成
            let builder = self.context.create_builder();
            
            // 基本ブロックを作成
            let mut llvm_blocks = HashMap::new();
            for (block_id, _) in &func.blocks {
                let block = self.context.append_basic_block(function, &format!("block_{:?}", block_id));
                llvm_blocks.insert(*block_id, block);
            }
            
            // 値のマッピング
            let mut value_map = HashMap::new();
            
            // パラメータを設定
            for (i, param) in function.get_param_iter().enumerate() {
                let param_name = &func.parameters[i].name;
                param.set_name(param_name);
                value_map.insert(param_name.clone(), param.into());
            }
            
            // 各ブロックの命令を生成
            for (block_id, block) in &func.blocks {
                let llvm_block = llvm_blocks[block_id];
                builder.position_at_end(llvm_block);
                
                for instr in &block.instructions {
                    match self.build_instruction(&builder, instr, &mut value_map) {
                        Ok(_) => {},
                        Err(e) => {
                            error!("命令の生成に失敗: {:?} - {:?}", instr, e);
                            return Err(e);
                        }
                    }
                }
            }
        }
        
        // LLVMモジュールを検証
        llvm_module.verify().map_err(|err| {
            EidosError::CodeGen(format!("LLVMモジュールの検証に失敗: {:?}", err))
        })?;
        
        // ターゲットマシンを取得
        let target_machine = self.get_target_machine(&options.target)?;
        
        // 出力フォーマットを決定
        let file_type = match options.format {
            OutputFormat::Object => FileType::Object,
            OutputFormat::Assembly => FileType::Assembly,
            OutputFormat::LLVMIR => FileType::Assembly, // LLVMIRは別処理
        };
        
        // LLVM IRを直接出力する場合
        if options.format == OutputFormat::LLVMIR {
            let mut ir = String::new();
            llvm_module.print_to_string().write_to_string(&mut ir)
                .map_err(|e| EidosError::CodeGen(format!("LLVMモジュールの文字列化に失敗: {:?}", e)))?;
            return Ok(ir.into_bytes());
        }
        
        // 機械語コードを生成
        let code = target_machine.write_to_memory_buffer(&llvm_module, file_type)
            .map_err(|e| EidosError::CodeGen(format!("コード生成に失敗: {:?}", e)))?;
        
        Ok(code.as_slice().to_vec())
    }
    
    fn declare_function(&mut self, name: &str, params: &[Type], return_type: &Type) -> Result<()> {
        info!("外部関数を宣言: {}", name);
        
        // LLVMモジュールを作成（まだ作成していない場合）
        let llvm_module = self.context.create_module(&format!("module_{}", name));
        
        // パラメータ型を変換
        let mut llvm_param_types = Vec::new();
        for param_type in params {
            let llvm_param_type = self.convert_type(param_type)?;
            llvm_param_types.push(llvm_param_type);
        }
        
        // 戻り値の型を変換
        let llvm_return_type = self.convert_type(return_type)?;
        
        // 関数型を作成
        let function_type = match llvm_return_type {
            BasicTypeEnum::IntType(int_type) => int_type.fn_type(&llvm_param_types, false),
            BasicTypeEnum::FloatType(float_type) => float_type.fn_type(&llvm_param_types, false),
            BasicTypeEnum::PointerType(ptr_type) => ptr_type.fn_type(&llvm_param_types, false),
            BasicTypeEnum::StructType(struct_type) => struct_type.fn_type(&llvm_param_types, false),
            _ => return Err(EidosError::CodeGen(format!("未対応の戻り値型: {:?}", return_type))),
        };
        
        // 外部関数を宣言
        let linkage = inkwell::module::Linkage::External;
        let function = llvm_module.add_function(name, function_type, Some(linkage));
        
        // 関数属性を設定
        function.set_call_conventions(inkwell::values::CallConv::C.into());
        
        // パラメータ名を設定（デバッグ情報用）
        for (i, param) in function.get_param_iter().enumerate() {
            param.set_name(&format!("arg{}", i));
        }
        
        // プラットフォーム固有の属性を設定
        if cfg!(target_os = "windows") {
            if name.ends_with("_stdcall") || name.starts_with("__stdcall_") {
                function.set_call_conventions(inkwell::values::CallConv::X86Stdcall.into());
            }
        }
        
        // ABI情報の設定
        match return_type {
            Type::Struct(_) => {
                // 構造体の戻り値は通常、最初の引数としてポインタを受け取る形に変換される
                function.add_attribute(
                    inkwell::attributes::AttributeLoc::Return, 
                    self.context.create_enum_attribute(
                        inkwell::attributes::Attribute::get_named_enum_kind_id("sret"), 
                        0
                    )
                );
            },
            _ => {}
        }
        
        // 関数マップに追加
        self.function_map.insert(name.to_string(), function);
        
        Ok(())
    }
    
    fn declare_global(&mut self, name: &str, ty: &Type, initializer: Option<&Literal>) -> Result<()> {
        info!("グローバル変数を宣言: {}", name);
        
        // LLVMモジュールを作成（まだ作成していない場合）
        let llvm_module = self.context.create_module(&format!("global_module_{}", name));
        
        // 型を変換
        let llvm_type = self.convert_type(ty)?;
        
        // グローバル変数を作成
        let global = llvm_module.add_global(llvm_type, None, name);
        
        // リンケージを設定
        if initializer.is_some() {
            // 初期化子があれば、インライン化可能な外部リンケージ
            global.set_linkage(inkwell::module::Linkage::ExternalLinkage);
        } else {
            // 初期化子がなければ、単なる外部変数
            global.set_linkage(inkwell::module::Linkage::External);
        }
        
        // スレッドローカルストレージかチェック
        if name.starts_with("__thread_") || name.ends_with("_tls") {
            global.set_thread_local(true);
            debug!("グローバル変数 {} をスレッドローカルとして設定", name);
        }
        
        // セクション属性の設定
        if name.starts_with("__const_") {
            // 定数セクションに配置
            global.set_section(".rodata");
            global.set_constant(true);
        } else if name.starts_with("__bss_") {
            // 未初期化データセクションに配置
            global.set_section(".bss");
        } else if name.starts_with("__data_") {
            // 初期化済みデータセクションに配置
            global.set_section(".data");
        }
        
        // アライメント属性の設定
        let alignment = match ty {
            Type { kind: TypeKind::Int } => 8,
            Type { kind: TypeKind::Float } => 8,
            Type { kind: TypeKind::Array(ref elem_type, _) } => {
                // 配列のアライメントは要素型に基づく
                match elem_type.kind {
                    TypeKind::Int => 8,
                    TypeKind::Float => 8,
                    _ => 4,
                }
            },
            Type { kind: TypeKind::Struct { .. } } => 16, // 構造体は大きめのアライメント
            _ => 4, // デフォルトは4バイトアライメント
        };
        global.set_alignment(alignment);
        
        // 初期化子があれば設定
        if let Some(lit) = initializer {
            let init_val = match lit {
                Literal::Int(value) => {
                    let int_type = self.context.i64_type();
                    int_type.const_int(*value as u64, true).as_basic_value_enum()
                },
                Literal::Float(value) => {
                    let float_type = self.context.f64_type();
                    float_type.const_float(*value).as_basic_value_enum()
                },
                Literal::Bool(value) => {
                    let bool_type = self.context.bool_type();
                    bool_type.const_int(*value as u64, false).as_basic_value_enum()
                },
                Literal::Char(value) => {
                    let char_type = self.context.i32_type();
                    char_type.const_int(*value as u64, false).as_basic_value_enum()
                },
                Literal::String(value) => {
                    // 文字列リテラルはi8配列として定義
                    let i8_type = self.context.i8_type();
                    let array_type = i8_type.array_type(value.len() as u32 + 1); // +1 for null terminator
                    
                    // 各バイトを定数として作成
                    let mut values = Vec::new();
                    for byte in value.bytes() {
                        values.push(i8_type.const_int(byte as u64, false));
                    }
                    // NULL終端を追加
                    values.push(i8_type.const_int(0, false));
                    
                    // 定数配列を作成
                    array_type.const_array(&values).as_basic_value_enum()
                },
                Literal::Unit => {
                    let unit_type = self.context.struct_type(&[], false);
                    unit_type.const_named_struct(&[]).as_basic_value_enum()
                },
                _ => return Err(EidosError::CodeGen(format!("未対応の初期化子型: {:?}", lit))),
            };
            
            global.set_initializer(&init_val);
            
            // 定数の場合は属性を設定
            match lit {
                Literal::Int(_) | Literal::Float(_) | Literal::Bool(_) | Literal::Char(_) | Literal::String(_) => {
                    // イミュータブルな定数として設定
                    global.set_constant(true);
                    // 重複を排除するためにユニークなセクションに配置
                    global.set_section(".rodata.cst");
                    // 必要に応じてコンパイラが最適化できるようにする
                    global.set_unnamed_addr(true);
                },
                _ => {}
            }
        } else {
            // 初期化子がない場合はゼロで初期化
            match llvm_type {
                BasicTypeEnum::IntType(int_type) => {
                    global.set_initializer(&int_type.const_zero());
                },
                BasicTypeEnum::FloatType(float_type) => {
                    global.set_initializer(&float_type.const_zero());
                },
                BasicTypeEnum::PointerType(ptr_type) => {
                    global.set_initializer(&ptr_type.const_null());
                },
                BasicTypeEnum::StructType(struct_type) => {
                    // 構造体の各フィールドをゼロ初期化
                    let field_count = struct_type.count_fields();
                    let mut zero_fields = Vec::with_capacity(field_count as usize);
                    
                    for i in 0..field_count {
                        let field_type = struct_type.get_field_type_at_index(i)
                            .ok_or_else(|| EidosError::CodeGen(format!("構造体フィールド型の取得に失敗: {}.{}", name, i)))?;
                        
                        match field_type {
                            BasicTypeEnum::IntType(int_type) => {
                                // 整数フィールドをゼロ初期化
                                let int_zero = int_type.const_zero();
                                zero_fields.push(int_zero.as_basic_value_enum());
                            },
                            BasicTypeEnum::FloatType(float_type) => {
                                // 浮動小数点フィールドをゼロ初期化
                                let float_zero = float_type.const_zero();
                                zero_fields.push(float_zero.as_basic_value_enum());
                            },
                            BasicTypeEnum::PointerType(ptr_type) => {
                                // ポインタフィールドをNULLに初期化
                                let null_ptr = ptr_type.const_null();
                                zero_fields.push(null_ptr.as_basic_value_enum());
                            },
                            BasicTypeEnum::StructType(nested_struct_type) => {
                                // ネストした構造体をゼロ初期化
                                let struct_zero = nested_struct_type.const_zero();
                                zero_fields.push(struct_zero.as_basic_value_enum());
                            },
                            BasicTypeEnum::ArrayType(array_type) => {
                                // 配列をゼロ初期化
                                let array_zero = array_type.const_zero();
                                zero_fields.push(array_zero.as_basic_value_enum());
                            },
                            _ => return Err(EidosError::CodeGen(format!("未対応のフィールド型: {:?}", field_type))),
                        }
                    }
                    
                    global.set_initializer(&struct_type.const_named_struct(&zero_fields));
                },
                BasicTypeEnum::ArrayType(array_type) => {
                    // 配列をゼロ初期化
                    global.set_initializer(&array_type.const_zero());
                },
                _ => return Err(EidosError::CodeGen(format!("未対応のグローバル変数型: {:?}", llvm_type))),
            }
        }
        
        // グローバル変数マップに追加
        self.global_map.insert(name.to_string(), global);
        
        Ok(())
    }
} 