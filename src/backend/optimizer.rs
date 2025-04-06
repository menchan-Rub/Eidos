use std::collections::{HashMap, HashSet};

use log::{debug, info};

use crate::core::Result;
use crate::core::eir::{Module, Function, FunctionId, BlockId, InstructionId, Instruction, Operand};

/// 最適化パス
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OptimizationPass {
    /// 定数畳み込み
    ConstantFolding,
    /// 不要コード削除
    DeadCodeElimination,
    /// 共通部分式削除
    CommonSubexpressionElimination,
    /// 関数インライン化
    FunctionInlining,
    /// ループの不変コード移動
    LoopInvariantCodeMotion,
    /// メモリToレジスタ
    MemoryToRegister,
    /// 命令の組み合わせ
    InstructionCombining,
    /// 制御フロー最適化
    ControlFlowOptimization,
    /// ループアンロール
    LoopUnrolling,
    /// SIMD最適化
    SIMDOptimization,
}

impl OptimizationPass {
    /// 全ての最適化パスのリストを取得
    pub fn all() -> Vec<Self> {
        vec![
            Self::ConstantFolding,
            Self::DeadCodeElimination,
            Self::CommonSubexpressionElimination,
            Self::FunctionInlining,
            Self::LoopInvariantCodeMotion,
            Self::MemoryToRegister,
            Self::InstructionCombining,
            Self::ControlFlowOptimization,
            Self::LoopUnrolling,
            Self::SIMDOptimization,
        ]
    }
}

/// 最適化レベル
#[derive(Debug, Clone, Copy)]
pub enum OptimizationLevel {
    /// 最適化なし
    None,
    /// サイズ最適化
    Size,
    /// 速度最適化（レベル1）
    Speed1,
    /// 速度最適化（レベル2）
    Speed2,
    /// 速度最適化（レベル3）
    Speed3,
}

impl From<u8> for OptimizationLevel {
    fn from(level: u8) -> Self {
        match level {
            0 => OptimizationLevel::None,
            1 => OptimizationLevel::Speed1,
            2 => OptimizationLevel::Speed2,
            3 => OptimizationLevel::Speed3,
            _ => OptimizationLevel::Speed2, // デフォルト
        }
    }
}

/// 最適化オプション
pub struct OptimizationOptions {
    /// 最適化レベル
    pub level: OptimizationLevel,
    /// サイズ最適化フラグ
    pub optimize_size: bool,
    /// インライン化しきい値
    pub inline_threshold: usize,
    /// ループアンロール係数
    pub unroll_factor: usize,
    /// SIMD最適化を有効にするか
    pub enable_simd: bool,
    /// プロファイル情報ベースの最適化
    pub profile_guided: bool,
    /// 無効化する最適化パス
    pub disabled_passes: HashSet<OptimizationPass>,
}

impl Default for OptimizationOptions {
    fn default() -> Self {
        Self {
            level: OptimizationLevel::Speed2,
            optimize_size: false,
            inline_threshold: 100,
            unroll_factor: 4,
            enable_simd: true,
            profile_guided: false,
            disabled_passes: HashSet::new(),
        }
    }
}

/// 最適化器
pub struct Optimizer {
    /// 最適化オプション
    options: OptimizationOptions,
    /// 関数実行数の統計
    fn_execution_counts: HashMap<FunctionId, usize>,
}

impl Optimizer {
    /// 新しい最適化器を作成
    pub fn new(options: OptimizationOptions) -> Self {
        Self {
            options,
            fn_execution_counts: HashMap::new(),
        }
    }
    
    /// デフォルトオプションで最適化器を作成
    pub fn default() -> Self {
        Self::new(OptimizationOptions::default())
    }
    
    /// 最適化レベルを指定して最適化器を作成
    pub fn with_level(level: u8) -> Self {
        let mut options = OptimizationOptions::default();
        options.level = OptimizationLevel::from(level);
        Self::new(options)
    }
    
    /// モジュールを最適化
    pub fn optimize_module(&mut self, module: &mut Module) -> Result<()> {
        info!("モジュール '{}' の最適化を開始", module.name);
        
        // 最適化レベルに応じた最適化パスを実行
        match self.options.level {
            OptimizationLevel::None => {
                // 最適化なし
                debug!("最適化スキップ: 最適化レベル = None");
            },
            OptimizationLevel::Size => {
                // サイズ最適化
                debug!("サイズ最適化を実行");
                self.run_size_optimization_passes(module)?;
            },
            OptimizationLevel::Speed1 => {
                // 速度最適化（レベル1）
                debug!("速度最適化（レベル1）を実行");
                self.run_speed1_optimization_passes(module)?;
            },
            OptimizationLevel::Speed2 => {
                // 速度最適化（レベル2）
                debug!("速度最適化（レベル2）を実行");
                self.run_speed2_optimization_passes(module)?;
            },
            OptimizationLevel::Speed3 => {
                // 速度最適化（レベル3）
                debug!("速度最適化（レベル3）を実行");
                self.run_speed3_optimization_passes(module)?;
            },
        }
        
        info!("モジュール '{}' の最適化が完了", module.name);
        Ok(())
    }
    
    /// サイズ最適化パスを実行
    fn run_size_optimization_passes(&mut self, module: &mut Module) -> Result<()> {
        // 定数畳み込み
        self.run_constant_folding(module)?;
        
        // 不要コード削除
        self.run_dead_code_elimination(module)?;
        
        // 制御フロー最適化
        self.run_control_flow_optimization(module)?;
        
        Ok(())
    }
    
    /// 速度最適化パス（レベル1）を実行
    fn run_speed1_optimization_passes(&mut self, module: &mut Module) -> Result<()> {
        debug!("Speed1最適化パスを実行");
        
        // 基本的な最適化パス
        if !self.options.disabled_passes.contains(&OptimizationPass::ConstantFolding) {
            self.run_constant_folding(module)?;
        }
        
        if !self.options.disabled_passes.contains(&OptimizationPass::DeadCodeElimination) {
            self.run_dead_code_elimination(module)?;
        }
        
        if !self.options.disabled_passes.contains(&OptimizationPass::CommonSubexpressionElimination) {
            self.run_common_subexpression_elimination(module)?;
        }
        
        // 必須の制御フロー最適化
        if !self.options.disabled_passes.contains(&OptimizationPass::ControlFlowOptimization) {
            self.run_control_flow_optimization(module)?;
        }
        
        // 基本的なメモリ最適化
        if !self.options.disabled_passes.contains(&OptimizationPass::MemoryToRegister) {
            self.run_memory_to_register(module)?;
        }
        
        Ok(())
    }
    
    /// 速度最適化パス（レベル2）を実行
    fn run_speed2_optimization_passes(&mut self, module: &mut Module) -> Result<()> {
        debug!("Speed2最適化パスを実行");
        
        // Speed1の最適化パスを含む
        self.run_speed1_optimization_passes(module)?;
        
        // さらに追加のパス
        if !self.options.disabled_passes.contains(&OptimizationPass::InstructionCombining) {
            self.run_instruction_combining(module)?;
        }
        
        if !self.options.disabled_passes.contains(&OptimizationPass::FunctionInlining) {
            // 通常の関数インライン化
            self.run_function_inlining(module, false)?;
        }
        
        // 再度、定数畳み込みと不要コード削除を実行
        if !self.options.disabled_passes.contains(&OptimizationPass::ConstantFolding) {
            self.run_constant_folding(module)?;
        }
        
        if !self.options.disabled_passes.contains(&OptimizationPass::DeadCodeElimination) {
            self.run_dead_code_elimination(module)?;
        }
        
        // 基本的なループ最適化
        if !self.options.disabled_passes.contains(&OptimizationPass::LoopInvariantCodeMotion) {
            self.run_loop_invariant_code_motion(module)?;
        }
        
        // 命令スケジューリング（基本的なもの）
        self.run_instruction_scheduling(module)?;
        
        Ok(())
    }
    
    /// 速度最適化パス（レベル3）を実行
    fn run_speed3_optimization_passes(&mut self, module: &mut Module) -> Result<()> {
        debug!("Speed3最適化パスを実行");
        
        // Speed2の最適化パスを含む
        self.run_speed2_optimization_passes(module)?;
        
        // さらに積極的な最適化
        if !self.options.disabled_passes.contains(&OptimizationPass::FunctionInlining) {
            // 積極的な関数インライン化
            self.run_function_inlining(module, true)?;
        }
        
        if !self.options.disabled_passes.contains(&OptimizationPass::LoopInvariantCodeMotion) {
            // 積極的なループの不変コード移動
            self.run_loop_invariant_code_motion(module)?;
        }
        
        // ループアンロール最適化
        if !self.options.disabled_passes.contains(&OptimizationPass::LoopUnrolling) {
            self.run_loop_unrolling(module)?;
        }
        
        // SIMD最適化
        if !self.options.disabled_passes.contains(&OptimizationPass::SIMDOptimization) {
            self.run_simd_optimization(module)?;
        }
        
        // レジスタ割り当て最適化
        self.run_register_allocation(module)?;
        
        // 命令スケジューリング
        self.run_instruction_scheduling(module)?;
        
        Ok(())
    }
    
    // 以下、個別の最適化パスの実装
    
    /// 定数畳み込み
    fn run_constant_folding(&mut self, module: &mut Module) -> Result<()> {
        debug!("定数畳み込み最適化を実行");
        
        for (func_id, func) in module.functions.iter_mut() {
            debug!("関数 '{}' の定数畳み込みを実行", func.name);
            
            // 各基本ブロックを処理
            for (block_id, block) in func.blocks.iter_mut() {
                let mut i = 0;
                while i < block.instructions.len() {
                    let instr_id = block.instructions[i];
                    if let Some(instr) = func.instructions.get(&instr_id) {
                        // 定数オペランドを持つ命令を検出
                        if let Some(folded_result) = self.fold_constants(instr) {
                            // 定数畳み込み可能な命令を定数に置き換え
                            let new_instr = Instruction::Constant {
                                result_type: instr.get_result_type(),
                                value: folded_result,
                            };
                            
                            // 元の命令を新しい命令に置き換え
                            func.instructions.insert(instr_id, new_instr);
                            
                            debug!("命令 {:?} を定数に畳み込み", instr_id);
                        }
                    }
                    i += 1;
                }
            }
        }
        
        Ok(())
    }
    
    /// 命令を定数畳み込む
    fn fold_constants(&self, instr: &Instruction) -> Option<Operand> {
        match instr {
            // 二項演算の定数畳み込み
            Instruction::BinaryOp { op, left, right, .. } => {
                if let (Operand::ConstantInt(l), Operand::ConstantInt(r)) = (left, right) {
                    match op.as_str() {
                        "add" => Some(Operand::ConstantInt(l + r)),
                        "sub" => Some(Operand::ConstantInt(l - r)),
                        "mul" => Some(Operand::ConstantInt(l * r)),
                        "div" => if *r != 0 { Some(Operand::ConstantInt(l / r)) } else { None },
                        "rem" => if *r != 0 { Some(Operand::ConstantInt(l % r)) } else { None },
                        "and" => Some(Operand::ConstantInt(l & r)),
                        "or" => Some(Operand::ConstantInt(l | r)),
                        "xor" => Some(Operand::ConstantInt(l ^ r)),
                        "shl" => Some(Operand::ConstantInt(l << r)),
                        "shr" => Some(Operand::ConstantInt(l >> r)),
                        "eq" => Some(Operand::ConstantBool(l == r)),
                        "ne" => Some(Operand::ConstantBool(l != r)),
                        "lt" => Some(Operand::ConstantBool(l < r)),
                        "le" => Some(Operand::ConstantBool(l <= r)),
                        "gt" => Some(Operand::ConstantBool(l > r)),
                        "ge" => Some(Operand::ConstantBool(l >= r)),
                        _ => None,
                    }
                } else if let (Operand::ConstantFloat(l), Operand::ConstantFloat(r)) = (left, right) {
                    match op.as_str() {
                        "add" => Some(Operand::ConstantFloat(l + r)),
                        "sub" => Some(Operand::ConstantFloat(l - r)),
                        "mul" => Some(Operand::ConstantFloat(l * r)),
                        "div" => if *r != 0.0 { Some(Operand::ConstantFloat(l / r)) } else { None },
                        "eq" => Some(Operand::ConstantBool(l == r)),
                        "ne" => Some(Operand::ConstantBool(l != r)),
                        "lt" => Some(Operand::ConstantBool(l < r)),
                        "le" => Some(Operand::ConstantBool(l <= r)),
                        "gt" => Some(Operand::ConstantBool(l > r)),
                        "ge" => Some(Operand::ConstantBool(l >= r)),
                        _ => None,
                    }
                } else {
                    None
                }
            },
            // 単項演算の定数畳み込み
            Instruction::UnaryOp { op, operand, .. } => {
                match (op.as_str(), operand) {
                    ("neg", Operand::ConstantInt(v)) => Some(Operand::ConstantInt(-v)),
                    ("neg", Operand::ConstantFloat(v)) => Some(Operand::ConstantFloat(-v)),
                    ("not", Operand::ConstantInt(v)) => Some(Operand::ConstantInt(!v)),
                    ("not", Operand::ConstantBool(v)) => Some(Operand::ConstantBool(!v)),
                    _ => None,
                }
            },
            // その他の命令は畳み込み不可
            _ => None,
        }
    }
    
    /// 不要コード削除
    fn run_dead_code_elimination(&mut self, module: &mut Module) -> Result<()> {
        debug!("不要コード削除最適化を実行");
        
        for (func_id, func) in module.functions.iter_mut() {
            debug!("関数 '{}' の不要コード削除を実行", func.name);
            
            // 使用されている命令のセット
            let mut used_instructions = HashSet::new();
            
            // 副作用のある命令と関数の終了命令をマーク
            for (instr_id, instr) in func.instructions.iter() {
                match instr {
                    Instruction::Return { .. } |
                    Instruction::Call { .. } |
                    Instruction::Store { .. } |
                    Instruction::Branch { .. } |
                    Instruction::ConditionalBranch { .. } => {
                        used_instructions.insert(*instr_id);
                    },
                    _ => {}
                }
            }
            
            // 使用されている命令から逆向きに依存関係をたどる
            let mut worklist: Vec<InstructionId> = used_instructions.iter().cloned().collect();
            while let Some(instr_id) = worklist.pop() {
                if let Some(instr) = func.instructions.get(&instr_id) {
                    // 命令の依存関係をたどる
                    for operand in instr.get_operands() {
                        if let Operand::InstructionRef(dep_id) = operand {
                            if !used_instructions.contains(&dep_id) {
                                used_instructions.insert(dep_id);
                                worklist.push(dep_id);
                            }
                        }
                    }
                }
            }
            
            // 未使用の命令を削除
            let all_instructions: HashSet<InstructionId> = func.instructions.keys().cloned().collect();
            let dead_instructions: Vec<InstructionId> = all_instructions.difference(&used_instructions).cloned().collect();
            
            // 各ブロックから未使用命令を削除
            for (block_id, block) in func.blocks.iter_mut() {
                // 未使用命令をブロックから削除
                block.instructions.retain(|instr_id| !dead_instructions.contains(instr_id));
            }
            
            // 命令テーブルから未使用命令を削除
            for instr_id in dead_instructions {
                debug!("未使用命令 {:?} を削除", instr_id);
                func.instructions.remove(&instr_id);
            }
            
            // 空のブロックをマージ
            self.merge_empty_blocks(func);
        }
        
        Ok(())
    }
    
    /// 空のブロックをマージ
    fn merge_empty_blocks(&self, func: &mut Function) {
        // 無条件分岐のみを含むブロックのマッピングを作成
        let mut branch_targets: HashMap<BlockId, BlockId> = HashMap::new();
        
        for (block_id, block) in &func.blocks {
            if block.instructions.len() == 1 {
                let instr_id = block.instructions[0];
                if let Some(Instruction::Branch { target }) = func.instructions.get(&instr_id) {
                    branch_targets.insert(*block_id, *target);
                }
            }
        }
        
        // 無条件分岐のみのブロックをスキップする参照を更新
        for (_, func_block) in func.blocks.iter_mut() {
            for instr_id in &func_block.instructions {
                if let Some(instr) = func.instructions.get_mut(instr_id) {
                    match instr {
                        Instruction::Branch { target } => {
                            while let Some(new_target) = branch_targets.get(target) {
                                *target = *new_target;
                            }
                        },
                        Instruction::ConditionalBranch { true_target, false_target, .. } => {
                            while let Some(new_target) = branch_targets.get(true_target) {
                                *true_target = *new_target;
                            }
                            while let Some(new_target) = branch_targets.get(false_target) {
                                *false_target = *new_target;
                            }
                        },
                        _ => {}
                    }
                }
            }
        }
    }
    
    /// 共通部分式削除
    fn run_common_subexpression_elimination(&mut self, module: &mut Module) -> Result<()> {
        debug!("共通部分式削除最適化を実行");
        
        for (func_id, func) in module.functions.iter_mut() {
            debug!("関数 '{}' の共通部分式削除を実行", func.name);
            
            // 式のハッシュマップ
            let mut expr_map: HashMap<String, InstructionId> = HashMap::new();
            
            // 各ブロックで命令を走査
            for (block_id, block) in func.blocks.iter() {
                // 現在のブロック内で有効な式のマップ
                let mut block_expr_map = expr_map.clone();
                
                for i in 0..block.instructions.len() {
                    let instr_id = block.instructions[i];
                    
                    if let Some(instr) = func.instructions.get(&instr_id) {
                        // 命令のハッシュを計算
                        let instr_hash = self.compute_instruction_hash(instr);
                        
                        if let Some(hash) = instr_hash {
                            // 同じ式がすでに計算されているか確認
                            if let Some(&existing_id) = block_expr_map.get(&hash) {
                                // 既存の命令を参照する命令に置き換え
                                debug!("共通部分式 {:?} を既存の命令 {:?} で置き換え", instr_id, existing_id);
                                
                                // すべての参照を更新
                                self.replace_instruction_refs(func, instr_id, existing_id);
                            } else {
                                // 新しい式をマップに追加
                                block_expr_map.insert(hash, instr_id);
                            }
                        }
                    }
                }
                
                // グローバルマップを更新
                expr_map = block_expr_map;
            }
        }
        
        Ok(())
    }
    
    /// 命令のハッシュを計算
    fn compute_instruction_hash(&self, instr: &Instruction) -> Option<String> {
        match instr {
            Instruction::BinaryOp { op, left, right, .. } => {
                Some(format!("binop:{}:{}:{}", op, self.operand_to_string(left), self.operand_to_string(right)))
            },
            Instruction::UnaryOp { op, operand, .. } => {
                Some(format!("unaryop:{}:{}", op, self.operand_to_string(operand)))
            },
            Instruction::Load { address, .. } => {
                Some(format!("load:{}", self.operand_to_string(address)))
            },
            Instruction::GetElementPtr { base, indices, .. } => {
                let indices_str: Vec<String> = indices.iter().map(|idx| self.operand_to_string(idx)).collect();
                Some(format!("gep:{}:{}", self.operand_to_string(base), indices_str.join(":")))
            },
            // メモリ書き込みや分岐などの副作用のある命令は除外
            Instruction::Store { .. } |
            Instruction::Call { .. } |
            Instruction::Branch { .. } |
            Instruction::ConditionalBranch { .. } |
            Instruction::Return { .. } => None,
            // その他の命令
            _ => Some(format!("{:?}", instr)),
        }
    }
    
    /// オペランドを文字列に変換
    fn operand_to_string(&self, op: &Operand) -> String {
        match op {
            Operand::InstructionRef(id) => format!("instr:{:?}", id),
            Operand::ConstantInt(val) => format!("int:{}", val),
            Operand::ConstantFloat(val) => format!("float:{}", val),
            Operand::ConstantBool(val) => format!("bool:{}", val),
            Operand::ConstantString(val) => format!("str:{}", val),
            Operand::GlobalRef(name) => format!("global:{}", name),
            Operand::FunctionRef(id) => format!("func:{:?}", id),
            Operand::Undef => "undef".to_string(),
            Operand::None => "none".to_string(),
        }
    }
    
    /// 命令参照を置き換え
    fn replace_instruction_refs(&self, func: &mut Function, old_id: InstructionId, new_id: InstructionId) {
        for (_, instr) in func.instructions.iter_mut() {
            instr.replace_operand_refs(old_id, new_id);
        }
    }
    
    /// 関数インライン化
    fn run_function_inlining(&mut self, module: &mut Module, aggressive: bool) -> Result<()> {
        debug!("関数インライン化最適化を実行 (aggressive: {})", aggressive);
        
        // インライン化する関数のリスト
        let mut inline_candidates: Vec<FunctionId> = Vec::new();
        
        // インライン化候補の関数を識別
        for (func_id, func) in &module.functions {
            // 小さい関数または頻繁に呼び出される関数をインライン化候補に
            let size = func.blocks.len();
            let call_count = self.fn_execution_counts.get(func_id).cloned().unwrap_or(0);
            
            // インライン化の閾値
            let threshold = if aggressive {
                // 積極的なインライン化
                self.options.inline_threshold * 2
            } else {
                self.options.inline_threshold
            };
            
            // サイズ*呼び出し回数がしきい値未満ならインライン化候補に
            if size * call_count < threshold {
                inline_candidates.push(*func_id);
            }
        }
        
        // 各関数内の関数呼び出しをインライン化
        for (caller_id, caller) in module.functions.iter_mut() {
            let mut inline_sites: Vec<(BlockId, usize, InstructionId, FunctionId)> = Vec::new();
            
            // インライン化する呼び出しサイトを特定
            for (block_id, block) in &caller.blocks {
                for (i, instr_id) in block.instructions.iter().enumerate() {
                    if let Some(Instruction::Call { function, .. }) = caller.instructions.get(instr_id) {
                        if let Operand::FunctionRef(callee_id) = function {
                            if inline_candidates.contains(callee_id) {
                                inline_sites.push((*block_id, i, *instr_id, *callee_id));
                            }
                        }
                    }
                }
            }
            
            // インライン化を実行
            for (block_id, instr_index, call_instr_id, callee_id) in inline_sites {
                // インライン化処理
                debug!("関数 {:?} 内の呼び出し {:?} を関数 {:?} でインライン化", caller_id, call_instr_id, callee_id);
                
                if let Some(callee) = module.functions.get(&callee_id).cloned() {
                    self.inline_function(module, caller, &block_id, call_instr_id, &callee)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// 関数呼び出しをインライン化
    fn inline_function(&self, module: &mut Module, caller: &mut Function, block_id: &BlockId, call_instr_id: InstructionId, callee: &Function) -> Result<()> {
        // 呼び出し命令を取得
        let call_instr = match caller.instructions.get(&call_instr_id) {
            Some(Instruction::Call { function, args, result }) => {
                (function.clone(), args.clone(), result.clone())
            },
            _ => return Err(EidosError::Optimization("呼び出し命令が見つかりません".to_string())),
        };
        
        // 呼び出し命令の引数と結果を取得
        let (_, args, result) = call_instr;
        
        // 呼び出し元ブロックを分割
        // 1. 呼び出し命令の前の部分（プレフィックス）
        // 2. インライン化された関数本体
        // 3. 呼び出し命令の後の部分（サフィックス）
        
        let caller_block = match caller.blocks.get(block_id) {
            Some(block) => block.clone(),
            None => return Err(EidosError::Optimization("呼び出し元ブロックが見つかりません".to_string())),
        };
        
        // 呼び出し命令のインデックスを取得
        let call_index = caller_block.instructions.iter().position(|&id| id == call_instr_id)
            .ok_or_else(|| EidosError::Optimization("呼び出し命令が見つかりません".to_string()))?;
        
        // プレフィックス部分とサフィックス部分に分割
        let prefix_instructions = caller_block.instructions[0..call_index].to_vec();
        let suffix_instructions = caller_block.instructions[call_index+1..].to_vec();
        
        // 新しいブロックIDを生成
        let inline_entry_block_id = BlockId::new();
        let inline_exit_block_id = BlockId::new();
        
        // 呼び出し元ブロックを更新（プレフィックス + インライン化された関数へのジャンプ）
        let mut new_caller_block = caller_block.clone();
        new_caller_block.instructions = prefix_instructions.clone();
        
        // プレフィックスブロックからインライン化された関数の入口ブロックへのジャンプを追加
        let jump_to_inline_id = InstructionId::new();
        let jump_to_inline = Instruction::Branch {
            target: inline_entry_block_id,
        };
        new_caller_block.instructions.push(jump_to_inline_id);
        caller.instructions.insert(jump_to_inline_id, jump_to_inline);
        
        // 呼び出し元ブロックを更新
        caller.blocks.insert(*block_id, new_caller_block);
        
        // 呼び出される関数のクローンを作成し、呼び出し元関数に追加
        let mut rewrite_map = HashMap::new();
        
        // 被呼び出し関数のパラメータを引数で置き換えるマッピングを作成
        let mut param_map = HashMap::new();
        for (i, param) in callee.parameters.iter().enumerate() {
            if i < args.len() {
                param_map.insert(param.name.clone(), args[i].clone());
            }
        }
        
        // 被呼び出し関数の各ブロックをクローン
        let original_entry = callee.entry_block.unwrap();
        let mut entry_block = None;
        
        for (orig_block_id, orig_block) in &callee.blocks {
            // 新しいブロックIDを生成
            let new_block_id = if *orig_block_id == original_entry {
                inline_entry_block_id
            } else {
                BlockId::new()
            };
            
            if *orig_block_id == original_entry {
                entry_block = Some(new_block_id);
            }
            
            // ブロックIDのマッピングを記録
            rewrite_map.insert(*orig_block_id, new_block_id);
            
            // 新しいブロックを作成
            let mut new_block = orig_block.clone();
            new_block.instructions = Vec::new();
            
            // 命令をクローン
            for &instr_id in &orig_block.instructions {
                if let Some(instr) = callee.instructions.get(&instr_id) {
                    let new_instr_id = InstructionId::new();
                    
                    // 命令を複製し、必要に応じて修正
                    let new_instr = match instr {
                        Instruction::Return { value } => {
                            // リターン命令を、結果をセットしてインライン化された関数の終了ブロックへのジャンプに変換
                            if let Some(val) = value {
                                // 戻り値がある場合、それを結果変数に代入
                                let store_result_id = InstructionId::new();
                                let store_result = Instruction::Store {
                                    ptr: Operand::Variable(result.clone()),
                                    value: val.clone(),
                                };
                                new_block.instructions.push(store_result_id);
                                caller.instructions.insert(store_result_id, store_result);
                            }
                            
                            // インライン化された関数の終了ブロックへジャンプ
                            Instruction::Branch {
                                target: inline_exit_block_id,
                            }
                        },
                        Instruction::Branch { target } => {
                            // ブロックターゲットを書き換え
                            Instruction::Branch {
                                target: *rewrite_map.get(target).unwrap_or(target),
                            }
                        },
                        Instruction::ConditionalBranch { condition, true_target, false_target } => {
                            // ブロックターゲットを書き換え
                            Instruction::ConditionalBranch {
                                condition: condition.clone(),
                                true_target: *rewrite_map.get(true_target).unwrap_or(true_target),
                                false_target: *rewrite_map.get(false_target).unwrap_or(false_target),
                            }
                        },
                        _ => instr.clone(),
                    };
                    
                    new_block.instructions.push(new_instr_id);
                    caller.instructions.insert(new_instr_id, new_instr);
                }
            }
            
            // 新しいブロックを呼び出し元関数に追加
            caller.blocks.insert(new_block_id, new_block);
        }
        
        // インライン化された関数の終了ブロックを作成
        let exit_block = Function::Block {
            instructions: suffix_instructions,
        };
        caller.blocks.insert(inline_exit_block_id, exit_block);
        
        // 呼び出し命令を削除
        caller.instructions.remove(&call_instr_id);
        
        Ok(())
    }
    
    /// ループの不変コード移動
    fn run_loop_invariant_code_motion(&mut self, module: &mut Module) -> Result<()> {
        debug!("ループ不変コード移動最適化を実行");
        
        for (func_id, func) in module.functions.iter_mut() {
            debug!("関数 '{}' のループ不変コード移動を実行", func.name);
            
            // ループを検出
            let loops = self.detect_loops(func);
            
            for loop_info in loops {
                // ループの不変命令を特定
                let invariant_instructions = self.identify_loop_invariants(func, &loop_info);
                
                // ループの不変命令をプリヘッダに移動
                self.move_invariants_to_preheader(func, &loop_info, &invariant_instructions);
            }
        }
        
        Ok(())
    }
    
    /// ループ構造を検出
    fn detect_loops(&self, func: &Function) -> Vec<LoopInfo> {
        let mut loops = Vec::new();
        
        // ドミネータツリーを構築
        let dominators = self.compute_dominators(func);
        
        // バックエッジを検出してループを見つける
        for (block_id, block) in &func.blocks {
            // ブロックの最後の命令を取得
            if let Some(&last_instr_id) = block.instructions.last() {
                if let Some(instr) = func.instructions.get(&last_instr_id) {
                    match instr {
                        Instruction::Branch { target } => {
                            // 無条件分岐: targetがcurrent_blockをドミネートする場合はループ
                            if dominators.get(block_id).map_or(false, |doms| doms.contains(target)) {
                                // ループ発見
                                let header = *target;
                                loops.push(self.construct_loop_info(func, header, *block_id, &dominators));
                            }
                        },
                        Instruction::ConditionalBranch { true_target, false_target, .. } => {
                            // 条件分岐: 各ターゲットがcurrent_blockをドミネートする場合はループ
                            if dominators.get(block_id).map_or(false, |doms| doms.contains(true_target)) {
                                // true_targetへのバックエッジを持つループ
                                let header = *true_target;
                                loops.push(self.construct_loop_info(func, header, *block_id, &dominators));
                            }
                            if dominators.get(block_id).map_or(false, |doms| doms.contains(false_target)) {
                                // false_targetへのバックエッジを持つループ
                                let header = *false_target;
                                loops.push(self.construct_loop_info(func, header, *block_id, &dominators));
                            }
                        },
                        _ => {}
                    }
                }
            }
        }
        
        loops
    }
    
    /// ドミネータを計算
    fn compute_dominators(&self, func: &Function) -> HashMap<BlockId, HashSet<BlockId>> {
        let mut dominators: HashMap<BlockId, HashSet<BlockId>> = HashMap::new();
        
        // すべてのブロックIDを取得
        let all_blocks: HashSet<BlockId> = func.blocks.keys().cloned().collect();
        
        // エントリブロックのドミネータはエントリブロック自身のみ
        if let Some(entry) = func.entry_block {
            let mut entry_doms = HashSet::new();
            entry_doms.insert(entry);
            dominators.insert(entry, entry_doms);
            
            // 他のすべてのブロックのドミネータは最初すべてのブロック
            for block_id in all_blocks.iter() {
                if *block_id != entry {
                    dominators.insert(*block_id, all_blocks.clone());
                }
            }
            
            let mut changed = true;
            while changed {
                changed = false;
                
                // 各ブロックのドミネータを更新
                for block_id in all_blocks.iter() {
                    if *block_id == entry {
                        continue; // エントリブロックはスキップ
                    }
                    
                    // 現在のブロックの前任者（predecessors）を取得
                    let predecessors = self.get_predecessors(func, *block_id);
                    if predecessors.is_empty() {
                        continue; // 到達不能なブロックはスキップ
                    }
                    
                    // 新しいドミネータセットを計算
                    let mut new_doms = all_blocks.clone();
                    
                    for pred in predecessors {
                        if let Some(pred_doms) = dominators.get(&pred) {
                            // 前任者のドミネータと交差
                            new_doms = new_doms.intersection(pred_doms).cloned().collect();
                        }
                    }
                    
                    // ブロック自身を追加
                    new_doms.insert(*block_id);
                    
                    // ドミネータが変更されたかチェック
                    if let Some(old_doms) = dominators.get(block_id) {
                        if old_doms != &new_doms {
                            dominators.insert(*block_id, new_doms);
                            changed = true;
                        }
                    }
                }
            }
        }
        
        dominators
    }
    
    /// ブロックの前任者（predecessors）を取得
    fn get_predecessors(&self, func: &Function, block_id: BlockId) -> Vec<BlockId> {
        let mut predecessors = Vec::new();
        
        for (pred_id, pred_block) in &func.blocks {
            // 最後の命令を取得
            if let Some(&last_instr_id) = pred_block.instructions.last() {
                if let Some(instr) = func.instructions.get(&last_instr_id) {
                    match instr {
                        Instruction::Branch { target } => {
                            if *target == block_id {
                                predecessors.push(*pred_id);
                            }
                        },
                        Instruction::ConditionalBranch { true_target, false_target, .. } => {
                            if *true_target == block_id || *false_target == block_id {
                                predecessors.push(*pred_id);
                            }
                        },
                        _ => {}
                    }
                }
            }
        }
        
        predecessors
    }
    
    /// ループ情報を構築
    fn construct_loop_info(&self, func: &Function, header: BlockId, backedge: BlockId, 
                          dominators: &HashMap<BlockId, HashSet<BlockId>>) -> LoopInfo {
        let mut body = HashSet::new();
        body.insert(header);
        
        // バックエッジブロックから遡って、ループ本体に含まれるブロックを特定
        let mut worklist = vec![backedge];
        while let Some(block_id) = worklist.pop() {
            if body.insert(block_id) {
                // 前任者をワークリストに追加
                for pred in self.get_predecessors(func, block_id) {
                    if pred != header && !body.contains(&pred) {
                        worklist.push(pred);
                    }
                }
            }
        }
        
        // プリヘッダブロックを特定
        let mut preheader = None;
        let header_preds = self.get_predecessors(func, header);
        for pred in header_preds {
            if !body.contains(&pred) {
                // ループ外からのエッジを持つブロックをプリヘッダとする
                preheader = Some(pred);
                break;
            }
        }
        
        // 出口ブロックを特定
        let mut exits = HashSet::new();
        for &block_id in &body {
            if let Some(block) = func.blocks.get(&block_id) {
                // ブロックの最後の命令を取得
                if let Some(&last_instr_id) = block.instructions.last() {
                    if let Some(instr) = func.instructions.get(&last_instr_id) {
                        match instr {
                            Instruction::Branch { target } => {
                                if !body.contains(target) {
                                    exits.insert(*target);
                                }
                            },
                            Instruction::ConditionalBranch { true_target, false_target, .. } => {
                                if !body.contains(true_target) {
                                    exits.insert(*true_target);
                                }
                                if !body.contains(false_target) {
                                    exits.insert(*false_target);
                                }
                            },
                            _ => {}
                        }
                    }
                }
            }
        }
        
        LoopInfo {
            header,
            body,
            preheader,
            exits,
        }
    }
    
    /// ループの不変命令を識別
    fn identify_loop_invariants(&self, func: &Function, loop_info: &LoopInfo) -> HashSet<InstructionId> {
        let mut invariants = HashSet::new();
        let mut changed = true;
        
        // 各ブロック内の全ての命令ID
        let mut all_instrs = HashSet::new();
        for &block_id in &loop_info.body {
            if let Some(block) = func.blocks.get(&block_id) {
                for &instr_id in &block.instructions {
                    all_instrs.insert(instr_id);
                }
            }
        }
        
        // 不変命令を繰り返し見つけるまで探索
        while changed {
            changed = false;
            
            for &block_id in &loop_info.body {
                if let Some(block) = func.blocks.get(&block_id) {
                    for &instr_id in &block.instructions {
                        // すでに不変と判定された命令はスキップ
                        if invariants.contains(&instr_id) {
                            continue;
                        }
                        
                        if let Some(instr) = func.instructions.get(&instr_id) {
                            // 命令の操作対象がすべてループ不変か確認
                            let deps = self.get_instruction_dependencies(instr);
                            let all_deps_invariant = deps.iter().all(|dep| {
                                match dep {
                                    Operand::InstructionRef(dep_id) => {
                                        // 依存命令がループ外か、既に不変と判定されているか
                                        !all_instrs.contains(dep_id) || invariants.contains(dep_id)
                                    },
                                    // 定数や外部参照は常に不変
                                    _ => true,
                                }
                            });
                            
                            // 副作用のない命令でかつすべての依存が不変の場合
                            if all_deps_invariant && !self.has_side_effects(instr) {
                                invariants.insert(instr_id);
                                changed = true;
                            }
                        }
                    }
                }
            }
        }
        
        invariants
    }
    
    /// 命令の依存関係を取得
    fn get_instruction_dependencies(&self, instr: &Instruction) -> Vec<Operand> {
        match instr {
            Instruction::BinaryOp { left, right, .. } => vec![left.clone(), right.clone()],
            Instruction::UnaryOp { operand, .. } => vec![operand.clone()],
            Instruction::Call { args, .. } => args.clone(),
            Instruction::Load { address, .. } => vec![address.clone()],
            Instruction::Store { address, value, .. } => vec![address.clone(), value.clone()],
            Instruction::GetElementPtr { base, indices, .. } => {
                let mut deps = vec![base.clone()];
                deps.extend(indices.clone());
                deps
            },
            Instruction::ConditionalBranch { condition, .. } => vec![condition.clone()],
            Instruction::Return { value } => {
                if let Some(val) = value {
                    vec![val.clone()]
                } else {
                    vec![]
                }
            },
            _ => vec![],
        }
    }
    
    /// 命令が副作用を持つかどうか
    fn has_side_effects(&self, instr: &Instruction) -> bool {
        match instr {
            Instruction::Store { .. } |
            Instruction::Call { .. } |
            Instruction::Branch { .. } |
            Instruction::ConditionalBranch { .. } |
            Instruction::Return { .. } => true,
            _ => false,
        }
    }
    
    /// 不変命令をループプリヘッダに移動
    fn move_invariants_to_preheader(&self, func: &mut Function, loop_info: &LoopInfo, invariants: &HashSet<InstructionId>) {
        // プリヘッダがない場合は作成
        let preheader_id = match loop_info.preheader {
            Some(id) => id,
            None => {
                // プリヘッダが存在しない場合は作成する
                debug!("ループヘッダー {:?} 用のプリヘッダーを作成", loop_info.header);
                return; // この実装では、プリヘッダが存在しない場合は処理をスキップ
            }
        };
        
        // プリヘッダブロックを取得
        let preheader = match func.blocks.get_mut(&preheader_id) {
            Some(block) => block,
            None => return, // プリヘッダが存在しない場合は処理をスキップ
        };
        
        // 移動する命令の依存関係グラフを構築
        let mut dependency_graph: HashMap<InstructionId, HashSet<InstructionId>> = HashMap::new();
        for &inv_id in invariants {
            if let Some(instr) = func.instructions.get(&inv_id) {
                let deps = self.get_instruction_dependencies(instr);
                let mut instr_deps = HashSet::new();
                
                for dep in deps {
                    if let Operand::InstructionRef(dep_id) = dep {
                        if invariants.contains(&dep_id) {
                            // この不変命令が別の不変命令に依存している
                            instr_deps.insert(dep_id);
                        }
                    }
                }
                
                dependency_graph.insert(inv_id, instr_deps);
            }
        }
        
        // 依存関係に基づいてソート（トポロジカルソート）
        let mut sorted_invariants = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();
        
        // 深さ優先探索でトポロジカルソートを実行
        fn visit(
            node: InstructionId,
            dependency_graph: &HashMap<InstructionId, HashSet<InstructionId>>,
            visited: &mut HashSet<InstructionId>,
            temp_visited: &mut HashSet<InstructionId>,
            sorted: &mut Vec<InstructionId>,
        ) -> bool {
            if temp_visited.contains(&node) {
                // 循環依存を検出
                return false;
            }
            
            if visited.contains(&node) {
                return true;
            }
            
            temp_visited.insert(node);
            
            if let Some(deps) = dependency_graph.get(&node) {
                for &dep in deps {
                    if !visit(dep, dependency_graph, visited, temp_visited, sorted) {
                        return false;
                    }
                }
            }
            
            temp_visited.remove(&node);
            visited.insert(node);
            sorted.push(node);
            
            true
        }
        
        // 全ての不変命令をトポロジカルソート
        for &inv_id in invariants {
            if !visited.contains(&inv_id) {
                if !visit(inv_id, &dependency_graph, &mut visited, &mut temp_visited, &mut sorted_invariants) {
                    debug!("循環依存を検出したため、不変命令 {:?} の移動をスキップ", inv_id);
                    continue;
                }
            }
        }
        
        // 不変命令をプリヘッダに移動
        let mut moved_instructions = HashSet::new();
        
        // プリヘッダの終端命令を一時的に保存
        let terminal_instr = if !preheader.instructions.is_empty() {
            preheader.instructions.pop()
        } else {
            None
        };
        
        // ソートした不変命令をプリヘッダに追加
        for &inv_id in &sorted_invariants {
            debug!("不変命令 {:?} をプリヘッダ {:?} に移動", inv_id, preheader_id);
            preheader.instructions.push(inv_id);
            moved_instructions.insert(inv_id);
        }
        
        // 終端命令を戻す
        if let Some(term) = terminal_instr {
            preheader.instructions.push(term);
        }
        
        // ループボディからの不変命令を削除
        for &block_id in &loop_info.body {
            if let Some(block) = func.blocks.get_mut(&block_id) {
                block.instructions.retain(|&instr_id| !moved_instructions.contains(&instr_id));
            }
        }
        
        debug!("ループ {:?} のために {} 個の不変命令をプリヘッダに移動", loop_info.header, moved_instructions.len());
    }
    
    /// メモリToレジスタ変換
    fn run_memory_to_register(&mut self, module: &mut Module) -> Result<()> {
        debug!("メモリToレジスタ最適化を実行");
        
        for (func_id, func) in module.functions.iter_mut() {
            debug!("関数 '{}' のメモリToレジスタ変換を実行", func.name);
            
            // アロケーション命令を特定
            let allocations = self.identify_allocations(func);
            
            // 各アロケーションをレジスタに昇格できるか分析
            for alloc_id in allocations {
                self.promote_allocation_to_register(func, alloc_id);
            }
        }
        
        Ok(())
    }
    
    /// アロケーション命令を特定
    fn identify_allocations(&self, func: &Function) -> Vec<InstructionId> {
        let mut allocs = Vec::new();
        
        // すべての基本ブロックを走査
        for (block_id, block) in &func.blocks {
            for &instr_id in &block.instructions {
                if let Some(Instruction::Alloca { .. }) = func.instructions.get(&instr_id) {
                    allocs.push(instr_id);
                }
            }
        }
        
        allocs
    }
    
    /// アロケーションをレジスタに昇格
    fn promote_allocation_to_register(&self, func: &mut Function, alloc_id: InstructionId) {
        // アロケーション命令を取得
        let alloc_instr = match func.instructions.get(&alloc_id) {
            Some(Instruction::Alloca { ty, result }) => (ty.clone(), result.clone()),
            _ => return,
        };
        let (alloc_type, alloc_var) = alloc_instr;
        
        // アロケーションへのすべてのLOAD/STORE命令を特定
        let mut loads = Vec::new();
        let mut stores = Vec::new();
        
        for (block_id, block) in &func.blocks {
            for &instr_id in &block.instructions {
                if let Some(instr) = func.instructions.get(&instr_id) {
                    match instr {
                        Instruction::Load { ptr, result } => {
                            if let Operand::Variable(var_name) = ptr {
                                if var_name == &alloc_var {
                                    loads.push((instr_id, result.clone()));
                                }
                            }
                        },
                        Instruction::Store { ptr, value } => {
                            if let Operand::Variable(var_name) = ptr {
                                if var_name == &alloc_var {
                                    stores.push((instr_id, value.clone()));
                                }
                            }
                        },
                        _ => {}
                    }
                }
            }
        }
        
        // アロケーションが昇格可能かチェック（アドレスを取得している場合は昇格不可）
        let can_promote = !self.address_taken(func, &alloc_var);
        
        if can_promote {
            // SSA形式の変数を生成
            let mut current_value = Operand::Undef;
            let mut value_at_block = HashMap::new();
            
            // 制御フローグラフを走査して変数の状態を追跡
            let cfg = self.build_cfg(func);
            let mut visited = HashSet::new();
            let mut worklist = Vec::new();
            
            if let Some(entry) = func.entry_block {
                worklist.push(entry);
                value_at_block.insert(entry, current_value.clone());
            }
            
            while let Some(block_id) = worklist.pop() {
                if !visited.insert(block_id) {
                    continue; // すでに訪問済み
                }
                
                // ブロック内の各命令を処理
                if let Some(block) = func.blocks.get(&block_id) {
                    for &instr_id in &block.instructions {
                        if let Some(instr) = func.instructions.get(&instr_id) {
                            match instr {
                                Instruction::Store { ptr, value } => {
                                    if let Operand::Variable(var_name) = ptr {
                                        if var_name == &alloc_var {
                                            // ストア命令を見つけた場合、現在の値を更新
                                            current_value = value.clone();
                                        }
                                    }
                                },
                                _ => {}
                            }
                        }
                    }
                }
                
                // 後続ブロックに現在の値を伝播
                if let Some(successors) = cfg.get(&block_id) {
                    for &succ in successors {
                        value_at_block.insert(succ, current_value.clone());
                        worklist.push(succ);
                    }
                }
            }
            
            // ロード命令を現在の値に置き換え
            for (load_id, load_result) in loads {
                // ロード命令を含むブロックを特定
                let mut containing_block_id = None;
                for (block_id, block) in &func.blocks {
                    if block.instructions.contains(&load_id) {
                        containing_block_id = Some(*block_id);
                        break;
                    }
                }
                
                if let Some(block_id) = containing_block_id {
                    if let Some(value) = value_at_block.get(&block_id) {
                        // ロード命令を値の直接代入に置き換え
                        let mov_instr = Instruction::BinaryOp {
                            op: String::from("mov"),
                            left: value.clone(),
                            right: Operand::Undef,
                            result: load_result,
                        };
                        func.instructions.insert(load_id, mov_instr);
                    }
                }
            }
            
            // ストア命令を削除（もう不要）
            for (store_id, _) in stores {
                // ストア命令を含むブロックからストア命令を削除
                for (block_id, block) in func.blocks.iter_mut() {
                    block.instructions.retain(|&id| id != store_id);
                }
                // 命令テーブルからも削除
                func.instructions.remove(&store_id);
            }
            
            // アロケーション命令を削除
            for (block_id, block) in func.blocks.iter_mut() {
                block.instructions.retain(|&id| id != alloc_id);
            }
            func.instructions.remove(&alloc_id);
        }
    }
    
    /// 変数のアドレスが取得されているかチェック
    fn address_taken(&self, func: &Function, var_name: &str) -> bool {
        for (_, instr) in &func.instructions {
            match instr {
                Instruction::GetElementPtr { base, .. } => {
                    if let Operand::Variable(name) = base {
                        if name == var_name {
                            return true;
                        }
                    }
                },
                _ => {}
            }
        }
        false
    }
    
    /// 制御フローグラフを構築
    fn build_cfg(&self, func: &Function) -> HashMap<BlockId, Vec<BlockId>> {
        let mut cfg = HashMap::new();
        
        for (block_id, block) in &func.blocks {
            let mut successors = Vec::new();
            
            // ブロックの最後の命令を取得
            if let Some(&last_instr_id) = block.instructions.last() {
                if let Some(instr) = func.instructions.get(&last_instr_id) {
                    match instr {
                        Instruction::Branch { target } => {
                            successors.push(*target);
                        },
                        Instruction::ConditionalBranch { true_target, false_target, .. } => {
                            successors.push(*true_target);
                            successors.push(*false_target);
                        },
                        _ => {}
                    }
                }
            }
            
            cfg.insert(*block_id, successors);
        }
        
        cfg
    }
    
    /// 命令の組み合わせ
    fn run_instruction_combining(&mut self, module: &mut Module) -> Result<()> {
        debug!("命令組み合わせ最適化を実行");
        
        for (func_id, func) in module.functions.iter_mut() {
            debug!("関数 '{}' の命令組み合わせを実行", func.name);
            
            let mut changed = true;
            while changed {
                changed = false;
                
                // すべての基本ブロックを走査
                for (block_id, block) in func.blocks.iter_mut() {
                    let mut i = 0;
                    while i < block.instructions.len() {
                        let instr_id = block.instructions[i];
                        
                        // 命令の組み合わせパターンをチェック
                        if self.try_combine_instructions(func, block_id, i) {
                            changed = true;
                            // 命令が組み合わされたので同じインデックスを再チェック
                        } else {
                            i += 1;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// 命令の組み合わせを試行
    fn try_combine_instructions(&self, func: &mut Function, block_id: &BlockId, index: usize) -> bool {
        let block = match func.blocks.get(block_id) {
            Some(b) => b,
            None => return false,
        };
        
        // ブロックの命令数チェック
        if index >= block.instructions.len() || index + 1 >= block.instructions.len() {
            return false;
        }
        
        // 現在の命令と次の命令を取得
        let curr_instr_id = block.instructions[index];
        let next_instr_id = block.instructions[index + 1];
        
        let curr_instr = match func.instructions.get(&curr_instr_id) {
            Some(i) => i,
            None => return false,
        };
        
        let next_instr = match func.instructions.get(&next_instr_id) {
            Some(i) => i,
            None => return false,
        };
        
        // 命令を組み合わせられるパターンをチェック
        match (curr_instr, next_instr) {
            // パターン1: 加算+乗算 → 変形（分配則）
            (
                Instruction::BinaryOp { op: op1, left: left1, right: right1, result: result1 },
                Instruction::BinaryOp { op: op2, left: left2, right: right2, result: result2 }
            ) if op1 == "add" && op2 == "mul" && result1 == left2 => {
                // 例: t1 = a + b; t2 = t1 * c; → t2 = a * c + b * c;
                if let Some(block_mut) = func.blocks.get_mut(block_id) {
                    // 新しい命令を生成
                    let mul1_id = InstructionId::new();
                    let mul2_id = InstructionId::new();
                    let add_id = InstructionId::new();
                    
                    let temp1 = format!("{}_temp1", result2);
                    let temp2 = format!("{}_temp2", result2);
                    
                    let mul1 = Instruction::BinaryOp {
                        op: "mul".to_string(),
                        left: left1.clone(),
                        right: right2.clone(),
                        result: temp1.clone(),
                    };
                    
                    let mul2 = Instruction::BinaryOp {
                        op: "mul".to_string(),
                        left: right1.clone(),
                        right: right2.clone(),
                        result: temp2.clone(),
                    };
                    
                    let add = Instruction::BinaryOp {
                        op: "add".to_string(),
                        left: Operand::Variable(temp1),
                        right: Operand::Variable(temp2),
                        result: result2.clone(),
                    };
                    
                    // 元の命令を置き換え
                    func.instructions.insert(mul1_id, mul1);
                    func.instructions.insert(mul2_id, mul2);
                    func.instructions.insert(add_id, add);
                    
                    // ブロックの命令を更新
                    block_mut.instructions.remove(index);  // 現在の命令を削除
                    block_mut.instructions.remove(index);  // 次の命令を削除（インデックスがずれるため同じindex）
                    block_mut.instructions.insert(index, mul1_id);
                    block_mut.instructions.insert(index + 1, mul2_id);
                    block_mut.instructions.insert(index + 2, add_id);
                    
                    return true;
                }
            },
            
            // パターン2: 連続する加算 → 単一の加算に統合
            (
                Instruction::BinaryOp { op: op1, left: left1, right: right1, result: result1 },
                Instruction::BinaryOp { op: op2, left: left2, right: right2, result: result2 }
            ) if op1 == "add" && op2 == "add" && result1 == left2 => {
                // 例: t1 = a + b; t2 = t1 + c; → t2 = a + b + c;
                if let Some(block_mut) = func.blocks.get_mut(block_id) {
                    // 新しい命令を生成（3項加算）
                    let new_add_id = InstructionId::new();
                    let temp = format!("{}_temp", result2);
                    
                    // まず最初の2項を加算
                    let first_add = Instruction::BinaryOp {
                        op: "add".to_string(),
                        left: left1.clone(),
                        right: right1.clone(),
                        result: temp.clone(),
                    };
                    
                    // その結果と3項目を加算
                    let second_add = Instruction::BinaryOp {
                        op: "add".to_string(),
                        left: Operand::Variable(temp),
                        right: right2.clone(),
                        result: result2.clone(),
                    };
                    
                    // 元の命令を置き換え
                    func.instructions.insert(new_add_id, first_add);
                    func.instructions.insert(next_instr_id, second_add);
                    
                    // ブロックの命令を更新
                    block_mut.instructions.remove(index);  // 現在の命令を削除
                    
                    return true;
                }
            },
            
            // パターン3: 比較命令の連鎖 → 短絡評価
            (
                Instruction::BinaryOp { op: op1, left: left1, right: right1, result: result1 },
                Instruction::BinaryOp { op: op2, left: left2, right: right2, result: result2 }
            ) if (op1 == "eq" || op1 == "ne" || op1 == "lt" || op1 == "le" || op1 == "gt" || op1 == "ge") &&
                 (op2 == "and" || op2 == "or") && 
                 (result1 == left2 || Operand::Variable(result1.clone()) == *left2) => {
                // 例: t1 = a < b; t2 = t1 && c; → 短絡評価を行う条件分岐に変換
                debug!("短絡評価パターンを検出: {} → {}", result1, result2);
                
                // 短絡評価のCFGを構築
                if self.update_cfg_for_shortcircuit(
                    func, 
                    block_id, 
                    index, 
                    result1, 
                    result2, 
                    op2, 
                    right2
                )? {
                    return true;
                }
            },
            
            // パターン4: 定数折りたたみを組み合わせ
            (
                Instruction::BinaryOp { op: op1, left: left1, right: right1, result: result1 },
                Instruction::BinaryOp { op: op2, left: left2, right: right2, result: result2 }
            ) if result1 == left2 => {
                // 両方の命令のオペランドが定数の場合、事前に計算
                if let (Operand::Literal(lit_left1), Operand::Literal(lit_right1)) = (left1, right1) {
                    if let Operand::Literal(lit_right2) = right2 {
                        // 最初の演算を評価
                        let intermediate_result = match (lit_left1, op1.as_str(), lit_right1) {
                            (Literal::Int(l), "add", Literal::Int(r)) => Literal::Int(l + r),
                            (Literal::Int(l), "sub", Literal::Int(r)) => Literal::Int(l - r),
                            (Literal::Int(l), "mul", Literal::Int(r)) => Literal::Int(l * r),
                            (Literal::Int(l), "div", Literal::Int(r)) => {
                                if *r == 0 {
                                    return false; // ゼロ除算は処理しない
                                }
                                Literal::Int(l / r)
                            },
                            (Literal::Float(l), "add", Literal::Float(r)) => Literal::Float(l + r),
                            (Literal::Float(l), "sub", Literal::Float(r)) => Literal::Float(l - r),
                            (Literal::Float(l), "mul", Literal::Float(r)) => Literal::Float(l * r),
                            (Literal::Float(l), "div", Literal::Float(r)) => {
                                if *r == 0.0 {
                                    return false; // ゼロ除算は処理しない
                                }
                                Literal::Float(l / r)
                            },
                            (Literal::Bool(l), "and", Literal::Bool(r)) => Literal::Bool(*l && *r),
                            (Literal::Bool(l), "or", Literal::Bool(r)) => Literal::Bool(*l || *r),
                            (Literal::Bool(l), "xor", Literal::Bool(r)) => Literal::Bool(*l != *r),
                            _ => return false, // サポートされていない演算
                        };
                        
                        // 次の演算を評価
                        let final_result = match (intermediate_result, op2.as_str(), lit_right2) {
                            (Literal::Int(l), "add", Literal::Int(r)) => Literal::Int(l + r),
                            (Literal::Int(l), "sub", Literal::Int(r)) => Literal::Int(l - r),
                            (Literal::Int(l), "mul", Literal::Int(r)) => Literal::Int(l * r),
                            (Literal::Int(l), "div", Literal::Int(r)) => {
                                if *r == 0 {
                                    return false; // ゼロ除算は処理しない
                                }
                                Literal::Int(l / r)
                            },
                            (Literal::Float(l), "add", Literal::Float(r)) => Literal::Float(l + r),
                            (Literal::Float(l), "sub", Literal::Float(r)) => Literal::Float(l - r),
                            (Literal::Float(l), "mul", Literal::Float(r)) => Literal::Float(l * r),
                            (Literal::Float(l), "div", Literal::Float(r)) => {
                                if *r == 0.0 {
                                    return false; // ゼロ除算は処理しない
                                }
                                Literal::Float(l / r)
                            },
                            (Literal::Bool(l), "and", Literal::Bool(r)) => Literal::Bool(l && *r),
                            (Literal::Bool(l), "or", Literal::Bool(r)) => Literal::Bool(l || *r),
                            (Literal::Bool(l), "xor", Literal::Bool(r)) => Literal::Bool(l != *r),
                            _ => return false, // サポートされていない演算
                        };
                        
                        // 2つの命令を1つの定数代入に置き換え
                        let new_instr = Instruction::BinaryOp {
                            op: "assign".to_string(),
                            left: Operand::Literal(final_result),
                            right: Operand::Literal(Literal::Unit),
                            result: result2.clone(),
                        };
                        
                        // 新しい命令を追加し、古い命令を削除
                        let new_instr_id = self.create_instruction_id();
                        func.instructions.insert(new_instr_id, new_instr);
                        func.instructions.remove(&first_id);
                        func.instructions.remove(&second_id);
                        
                        // ブロック内の命令リストを更新
                        if let Some(block) = func.blocks.get_mut(&block_id) {
                            let pos = block.instructions.iter().position(|&id| id == first_id).unwrap();
                            block.instructions[pos] = new_instr_id;
                            block.instructions.remove(pos + 1);
                        }
                        
                        debug!("定数折りたたみの最適化を適用: {} → {}", result1, result2);
                        return true;
                    }
                }
            },
            
            // その他のパターンも追加可能
            _ => {}
        }
        
        false
    }
    /// 制御フロー最適化
    fn run_control_flow_optimization(&mut self, module: &mut Module) -> Result<()> {
        debug!("制御フロー最適化を実行");
        
        for (func_id, func) in module.functions.iter_mut() {
            debug!("関数 '{}' の制御フロー最適化を実行", func.name);
            
            // 到達不能コードの削除
            self.remove_unreachable_code(func);
            
            // 分岐の単純化
            self.simplify_branches(func);
            
            // 基本ブロックのマージ
            self.merge_blocks(func);
        }
        
        Ok(())
    }
    
    /// 到達不能コードの削除
    fn remove_unreachable_code(&self, func: &mut Function) {
        // エントリポイントから到達可能なブロックを特定
        let reachable = self.find_reachable_blocks(func);
        
        // 到達不能なブロックを削除
        let all_blocks: HashSet<BlockId> = func.blocks.keys().cloned().collect();
        let unreachable: Vec<BlockId> = all_blocks.difference(&reachable).cloned().collect();
        
        for block_id in unreachable {
            debug!("到達不能ブロック {:?} を削除", block_id);
            func.blocks.remove(&block_id);
        }
    }
    
    /// 到達可能なブロックを特定
    fn find_reachable_blocks(&self, func: &Function) -> HashSet<BlockId> {
        let mut reachable = HashSet::new();
        let mut worklist = Vec::new();
        
        // エントリブロックをワークリストに追加
        if let Some(entry) = func.entry_block {
            worklist.push(entry);
            reachable.insert(entry);
        }
        
        // 到達可能なブロックを幅優先探索
        while let Some(block_id) = worklist.pop() {
            if let Some(block) = func.blocks.get(&block_id) {
                // ブロックの最後の命令を取得
                if let Some(&last_instr_id) = block.instructions.last() {
                    if let Some(instr) = func.instructions.get(&last_instr_id) {
                        // 分岐先をワークリストに追加
                        match instr {
                            Instruction::Branch { target } => {
                                if !reachable.contains(target) {
                                    reachable.insert(*target);
                                    worklist.push(*target);
                                }
                            },
                            Instruction::ConditionalBranch { true_target, false_target, .. } => {
                                if !reachable.contains(true_target) {
                                    reachable.insert(*true_target);
                                    worklist.push(*true_target);
                                }
                                if !reachable.contains(false_target) {
                                    reachable.insert(*false_target);
                                    worklist.push(*false_target);
                                }
                            },
                            _ => {}
                        }
                    }
                }
            }
        }
        
        reachable
    }
    
    /// 分岐の単純化
    fn simplify_branches(&self, func: &mut Function) {
        // 条件分岐の単純化
        for (block_id, block) in func.blocks.iter_mut() {
            if let Some(&last_instr_id) = block.instructions.last() {
                if let Some(Instruction::ConditionalBranch { condition, true_target, false_target }) = func.instructions.get(&last_instr_id).cloned() {
                    // 条件が定数の場合、無条件分岐に変換
                    if let Operand::ConstantBool(cond_val) = condition {
                        let target = if cond_val { true_target } else { false_target };
                        let new_instr = Instruction::Branch { target };
                        
                        if let Some(instr) = func.instructions.get_mut(&last_instr_id) {
                            *instr = new_instr;
                            debug!("条件分岐 {:?} を無条件分岐に単純化", last_instr_id);
                        }
                    }
                    // 両方の分岐先が同じ場合、無条件分岐に変換
                    else if true_target == false_target {
                        let new_instr = Instruction::Branch { target: true_target };
                        
                        if let Some(instr) = func.instructions.get_mut(&last_instr_id) {
                            *instr = new_instr;
                            debug!("条件分岐 {:?} を無条件分岐に単純化（同一ターゲット）", last_instr_id);
                        }
                    }
                }
            }
        }
    }
    
    /// 基本ブロックのマージ
    fn merge_blocks(&self, func: &mut Function) {
        // ブロックの後続を特定
        let mut successors: HashMap<BlockId, Vec<BlockId>> = HashMap::new();
        let mut predecessors: HashMap<BlockId, Vec<BlockId>> = HashMap::new();
        
        // 各ブロックの後続と前任者を特定
        for (block_id, block) in &func.blocks {
            // ブロックの最後の命令を取得
            if let Some(&last_instr_id) = block.instructions.last() {
                if let Some(instr) = func.instructions.get(&last_instr_id) {
                    match instr {
                        Instruction::Branch { target } => {
                            successors.entry(*block_id).or_default().push(*target);
                            predecessors.entry(*target).or_default().push(*block_id);
                        },
                        Instruction::ConditionalBranch { true_target, false_target, .. } => {
                            successors.entry(*block_id).or_default().push(*true_target);
                            successors.entry(*block_id).or_default().push(*false_target);
                            predecessors.entry(*true_target).or_default().push(*block_id);
                            predecessors.entry(*false_target).or_default().push(*block_id);
                        },
                        _ => {}
                    }
                }
            }
        }
        
        // マージ候補を特定
        let mut merge_candidates = Vec::new();
        
        for (block_id, block) in &func.blocks {
            // 後続が1つだけの場合
            if let Some(succs) = successors.get(block_id) {
                if succs.len() == 1 {
                    let succ_id = succs[0];
                    
                    // 後続ブロックの前任者が自分だけの場合、マージ候補
                    if let Some(preds) = predecessors.get(&succ_id) {
                        if preds.len() == 1 && preds[0] == *block_id {
                            merge_candidates.push((*block_id, succ_id));
                        }
                    }
                }
            }
        }
        
        // マージを実行
        let mut func_clone = func.clone(); // 元の関数のクローン
        
        for (from_id, to_id) in merge_candidates {
            let from_block = match func_clone.blocks.get(&from_id) {
                Some(b) => b.clone(),
                None => continue,
            };
            
            let to_block = match func_clone.blocks.get(&to_id) {
                Some(b) => b.clone(),
                None => continue,
            };
            
            // マージされたブロックの命令を作成
            let mut merged_instructions = Vec::new();
            
            // fromブロックの命令を追加（最後の分岐命令を除く）
            if !from_block.instructions.is_empty() {
                merged_instructions.extend(&from_block.instructions[0..from_block.instructions.len() - 1]);
            }
            
            // toブロックの命令を追加
            merged_instructions.extend(&to_block.instructions);
            
            // マージされたブロックを作成
            let merged_block = Function::Block {
                instructions: merged_instructions,
            };
            
            // fromブロックを更新
            func.blocks.insert(from_id, merged_block);
            
            // toブロックへの参照をfromブロックに更新
            for (_, block) in func.blocks.iter_mut() {
                let last_idx = block.instructions.len().saturating_sub(1);
                if let Some(&last_instr_id) = block.instructions.get(last_idx) {
                    if let Some(instr) = func.instructions.get_mut(&last_instr_id) {
                        match instr {
                            Instruction::Branch { target } if *target == to_id => {
                                *target = from_id;
                            },
                            Instruction::ConditionalBranch { true_target, false_target, .. } => {
                                if *true_target == to_id {
                                    *true_target = from_id;
                                }
                                if *false_target == to_id {
                                    *false_target = from_id;
                                }
                            },
                            _ => {}
                        }
                    }
                }
            }
            
            // toブロックを削除
            func.blocks.remove(&to_id);
        }
    }
    
    /// レジスタアロケーションのための依存関係グラフを構築
    fn build_dependency_graph(&self, func: &Function) -> HashMap<InstructionId, Vec<InstructionId>> {
        let mut dependency_graph = HashMap::new();
        
        // 各命令の依存関係を構築
        for (block_id, block) in &func.blocks {
            for (i, &instr_id) in block.instructions.iter().enumerate() {
                let mut dependencies = Vec::new();
                
                if let Some(instr) = func.instructions.get(&instr_id) {
                    // オペランドから依存する命令を特定
                    let operands = self.get_instruction_operands(instr);
                    
                    for operand in operands {
                        if let Operand::InstructionRef(dep_id) = operand {
                            dependencies.push(dep_id);
                        }
                    }
                    
                    // 同じブロック内での制御依存関係を追加
                    for j in 0..i {
                        let prev_instr_id = block.instructions[j];
                        if let Some(prev_instr) = func.instructions.get(&prev_instr_id) {
                            if self.has_side_effects(prev_instr) {
                                dependencies.push(prev_instr_id);
                            }
                        }
                    }
                }
                
                dependency_graph.insert(instr_id, dependencies);
            }
        }
        
        dependency_graph
    }
    
    /// 命令のオペランドを取得
    fn get_instruction_operands(&self, instr: &Instruction) -> Vec<Operand> {
        match instr {
            Instruction::BinaryOp { lhs, rhs, .. } => vec![lhs.clone(), rhs.clone()],
            Instruction::UnaryOp { operand, .. } => vec![operand.clone()],
            Instruction::Call { func, args, .. } => {
                let mut operands = vec![func.clone()];
                operands.extend(args.iter().cloned());
                operands
            },
            Instruction::Load { ptr, .. } => vec![ptr.clone()],
            Instruction::Store { ptr, value, .. } => vec![ptr.clone(), value.clone()],
            Instruction::GetElementPtr { ptr, indices, .. } => {
                let mut operands = vec![ptr.clone()];
                operands.extend(indices.iter().cloned());
                operands
            },
            Instruction::ConditionalBranch { condition, .. } => vec![condition.clone()],
            Instruction::Return { value } => {
                if let Some(val) = value {
                    vec![val.clone()]
                } else {
                    vec![]
                }
            },
            _ => vec![],
        }
    }
    
    /// レジスタ割り当て最適化
    fn run_register_allocation(&mut self, module: &mut Module) -> Result<()> {
        debug!("レジスタ割り当て最適化を実行");
        
        for (_, func) in module.functions.iter_mut() {
            // 変数の生存区間を分析
            let liveness = self.analyze_liveness(func);
            
            // 干渉グラフを構築
            let interference_graph = self.build_interference_graph(func, &liveness);
            
            // グラフ彩色によるレジスタ割り当て
            let register_allocation = self.color_graph(&interference_graph);
            
            // レジスタ割り当て結果に基づいて関数をリライト
            self.rewrite_function_with_register_allocation(func, &register_allocation)?;
        }
        
        Ok(())
    }
    
    /// 変数の生存区間分析
    fn analyze_liveness(&self, func: &Function) -> HashMap<String, Vec<(BlockId, usize)>> {
        let mut liveness = HashMap::new();
        
        // 各変数の定義点と使用点を記録
        let mut def_points = HashMap::new();
        let mut use_points = HashMap::new();
        
        // 関数の制御フローグラフを構築
        let cfg = self.build_cfg(func);
        
        // 各ブロックごとの定義変数と使用変数を収集
        for (block_id, block) in &func.blocks {
            for (instr_idx, &instr_id) in block.instructions.iter().enumerate() {
                if let Some(instr) = func.instructions.get(&instr_id) {
                    // 定義される変数
                    if let Some(var_name) = self.get_defined_variable(instr) {
                        def_points.entry(var_name)
                            .or_insert_with(Vec::new)
                            .push((*block_id, instr_idx));
                    }
                    
                    // 使用される変数
                    for var_name in self.get_used_variables(instr) {
                        use_points.entry(var_name)
                            .or_insert_with(Vec::new)
                            .push((*block_id, instr_idx));
                    }
                }
            }
        }
        
        // 生存区間を計算
        for (var_name, def_list) in &def_points {
            let uses = use_points.get(var_name).cloned().unwrap_or_default();
            
            // 各定義点から可能な使用点まで追跡
            for &(def_block, def_idx) in def_list {
                let mut live_ranges = Vec::new();
                
                // 同じブロック内の使用点を調査
                for &(use_block, use_idx) in &uses {
                    if def_block == use_block && use_idx > def_idx {
                        live_ranges.push((use_block, use_idx));
                    }
                }
                
                // 支配下にあるブロックの使用点を調査
                let dominated_blocks = self.find_dominated_blocks(func, def_block);
                for dominated_block in dominated_blocks {
                    for &(use_block, use_idx) in &uses {
                        if dominated_block == use_block {
                            live_ranges.push((use_block, use_idx));
                        }
                    }
                }
                
                // 生存区間を記録
                if !live_ranges.is_empty() {
                    liveness.entry(var_name.clone())
                        .or_insert_with(Vec::new)
                        .extend(live_ranges);
                }
            }
        }
        
        liveness
    }
    
    /// 定義される変数名を取得
    fn get_defined_variable(&self, instr: &Instruction) -> Option<String> {
        match instr {
            Instruction::BinaryOp { result, .. } => Some(result.clone()),
            Instruction::UnaryOp { result, .. } => Some(result.clone()),
            Instruction::Call { result, .. } => Some(result.clone()),
            Instruction::Load { result, .. } => Some(result.clone()),
            Instruction::Alloca { result, .. } => Some(result.clone()),
            Instruction::GetElementPtr { result, .. } => Some(result.clone()),
            _ => None,
        }
    }
    
    /// 使用される変数名を取得
    fn get_used_variables(&self, instr: &Instruction) -> Vec<String> {
        let mut used_vars = Vec::new();
        
        match instr {
            Instruction::BinaryOp { lhs, rhs, .. } => {
                if let Operand::Variable(name) = lhs {
                    used_vars.push(name.clone());
                }
                if let Operand::Variable(name) = rhs {
                    used_vars.push(name.clone());
                }
            },
            Instruction::UnaryOp { operand, .. } => {
                if let Operand::Variable(name) = operand {
                    used_vars.push(name.clone());
                }
            },
            Instruction::Call { args, .. } => {
                for arg in args {
                    if let Operand::Variable(name) = arg {
                        used_vars.push(name.clone());
                    }
                }
            },
            Instruction::Load { ptr, .. } => {
                if let Operand::Variable(name) = ptr {
                    used_vars.push(name.clone());
                }
            },
            Instruction::Store { ptr, value, .. } => {
                if let Operand::Variable(name) = ptr {
                    used_vars.push(name.clone());
                }
                if let Operand::Variable(name) = value {
                    used_vars.push(name.clone());
                }
            },
            Instruction::GetElementPtr { ptr, indices, .. } => {
                if let Operand::Variable(name) = ptr {
                    used_vars.push(name.clone());
                }
                for idx in indices {
                    if let Operand::Variable(name) = idx {
                        used_vars.push(name.clone());
                    }
                }
            },
            Instruction::ConditionalBranch { condition, .. } => {
                if let Operand::Variable(name) = condition {
                    used_vars.push(name.clone());
                }
            },
            Instruction::Return { value } => {
                if let Some(val) = value {
                    if let Operand::Variable(name) = val {
                        used_vars.push(name.clone());
                    }
                }
            },
            _ => {}
        }
        
        used_vars
    }
    
    /// あるブロックが支配するブロックを見つける
    fn find_dominated_blocks(&self, func: &Function, dominator: BlockId) -> Vec<BlockId> {
        let mut dominated = Vec::new();
        let cfg = self.build_cfg(func);
        
        // 単純な深さ優先探索
        let mut visited = HashSet::new();
        let mut stack = vec![dominator];
        
        while let Some(block_id) = stack.pop() {
            if !visited.insert(block_id) {
                continue;
            }
            
            if block_id != dominator {
                dominated.push(block_id);
            }
            
            if let Some(successors) = cfg.get(&block_id) {
                stack.extend(successors);
            }
        }
        
        dominated
    }
    
    /// 干渉グラフを構築
    fn build_interference_graph(&self, func: &Function, liveness: &HashMap<String, Vec<(BlockId, usize)>>) -> HashMap<String, HashSet<String>> {
        let mut interference_graph = HashMap::new();
        
        // 各変数ペアについて生存区間の重なりをチェック
        let var_names: Vec<String> = liveness.keys().cloned().collect();
        
        for i in 0..var_names.len() {
            let var1 = &var_names[i];
            let ranges1 = &liveness[var1];
            
            for j in i+1..var_names.len() {
                let var2 = &var_names[j];
                let ranges2 = &liveness[var2];
                
                // 生存区間が重なるかチェック
                let interferes = ranges1.iter().any(|&(block1, idx1)| {
                    ranges2.iter().any(|&(block2, idx2)| {
                        block1 == block2 && idx1 == idx2
                    })
                });
                
                if interferes {
                    // 干渉関係を記録（双方向）
                    interference_graph.entry(var1.clone())
                        .or_insert_with(HashSet::new)
                        .insert(var2.clone());
                    
                    interference_graph.entry(var2.clone())
                        .or_insert_with(HashSet::new)
                        .insert(var1.clone());
                }
            }
        }
        
        interference_graph
    }
    
    /// グラフ彩色アルゴリズムによるレジスタ割り当て
    fn color_graph(&self, interference_graph: &HashMap<String, HashSet<String>>) -> HashMap<String, usize> {
        let mut allocation = HashMap::new();
        
        // 変数の次数（干渉数）でソート
        let mut nodes: Vec<String> = interference_graph.keys().cloned().collect();
        nodes.sort_by_key(|var| {
            interference_graph.get(var).map_or(0, |neighbors| neighbors.len())
        });
        nodes.reverse();  // 次数の高い順
        
        // レジスタ（色）の最大数
        let max_colors = 32;  // 利用可能なレジスタ数
        
        // グラフ彩色
        for var in nodes {
            let neighbors = interference_graph.get(&var).cloned().unwrap_or_default();
            
            // 隣接ノードが使用している色を除外
            let mut used_colors = HashSet::new();
            for neighbor in &neighbors {
                if let Some(&color) = allocation.get(neighbor) {
                    used_colors.insert(color);
                }
            }
            
            // 利用可能な最小の色を選択
            let mut color = 0;
            while color < max_colors && used_colors.contains(&color) {
                color += 1;
            }
            
            // 色の上限に達した場合はスピルするが、ここでは簡略化のためスピルは考慮しない
            if color < max_colors {
                allocation.insert(var, color);
            }
        }
        
        allocation
    }
    
    /// レジスタ割り当てに基づいて関数をリライト
    fn rewrite_function_with_register_allocation(&self, func: &mut Function, allocation: &HashMap<String, usize>) -> Result<()> {
        debug!("レジスタ割り当て結果: {:?}", allocation);
        
        // 各変数の使用箇所を修正
        for (block_id, block) in func.blocks.iter_mut() {
            for &instr_id in &block.instructions.clone() {
                if let Some(instruction) = func.instructions.get_mut(&instr_id) {
                    // 命令内の変数をレジスタに置き換え
                    match instruction {
                        Instruction::BinaryOp { left, right, result, .. } => {
                            // オペランドの置き換え
                            self.replace_operand_with_register(left, allocation);
                            self.replace_operand_with_register(right, allocation);
                            
                            // 結果変数をレジスタに置き換え
                            if let Some(&reg) = allocation.get(result) {
                                *result = format!("reg{}", reg);
                            }
                        },
                        Instruction::UnaryOp { operand, result, .. } => {
                            self.replace_operand_with_register(operand, allocation);
                            if let Some(&reg) = allocation.get(result) {
                                *result = format!("reg{}", reg);
                            }
                        },
                        Instruction::Call { args, result, .. } => {
                            // 引数をレジスタに置き換え
                            for arg in args {
                                self.replace_operand_with_register(arg, allocation);
                            }
                            
                            // 結果をレジスタに置き換え
                            if let Some(res) = result {
                                if let Some(&reg) = allocation.get(res) {
                                    *res = format!("reg{}", reg);
                                }
                            }
                        },
                        Instruction::Load { address, result } => {
                            self.replace_operand_with_register(address, allocation);
                            if let Some(&reg) = allocation.get(result) {
                                *result = format!("reg{}", reg);
                            }
                        },
                        Instruction::Store { address, value } => {
                            self.replace_operand_with_register(address, allocation);
                            self.replace_operand_with_register(value, allocation);
                        },
                        Instruction::GetElementPtr { base, indices, result } => {
                            self.replace_operand_with_register(base, allocation);
                            for idx in indices {
                                self.replace_operand_with_register(idx, allocation);
                            }
                            if let Some(&reg) = allocation.get(result) {
                                *result = format!("reg{}", reg);
                            }
                        },
                        Instruction::Alloca { result, .. } => {
                            if let Some(&reg) = allocation.get(result) {
                                *result = format!("reg{}", reg);
                            }
                        },
                        Instruction::ConditionalBranch { condition, .. } => {
                            self.replace_operand_with_register(condition, allocation);
                        },
                        Instruction::Return { value } => {
                            if let Some(val) = value {
                                self.replace_operand_with_register(val, allocation);
                            }
                        },
                        Instruction::Phi { incoming_values, result } => {
                            // PHI命令の入力値を置き換え
                            for (value, _) in incoming_values {
                                self.replace_operand_with_register(value, allocation);
                            }
                            
                            // 結果をレジスタに置き換え
                            if let Some(&reg) = allocation.get(result) {
                                *result = format!("reg{}", reg);
                            }
                        },
                        // 他の命令タイプも同様に処理
                        _ => {}
                    }
                }
            }
        }
        
        // スタックアロケーションを削除（レジスタに昇格されたもの）
        let mut to_remove = Vec::new();
        for (instr_id, instr) in &func.instructions {
            if let Instruction::Alloca { result, .. } = instr {
                // このアロケーションがレジスタに置き換えられた場合
                if allocation.contains_key(result) {
                    to_remove.push(*instr_id);
                }
            }
        }
        
        // 不要なアロケーション命令を削除
        for instr_id in to_remove {
            func.instructions.remove(&instr_id);
            // ブロックからも削除
            for (_, block) in func.blocks.iter_mut() {
                block.instructions.retain(|&id| id != instr_id);
            }
        }
        
        debug!("レジスタ割り当てに基づく関数リライトが完了");
        Ok(())
    }
    
    /// オペランドをレジスタに置き換え
    fn replace_operand_with_register(&self, operand: &mut Operand, allocation: &HashMap<String, usize>) {
        match operand {
            Operand::Variable(name) => {
                if let Some(&reg) = allocation.get(name) {
                    *name = format!("reg{}", reg);
                }
            },
            Operand::GetElementPtr { base, indices } => {
                self.replace_operand_with_register(base, allocation);
                for idx in indices {
                    self.replace_operand_with_register(idx, allocation);
                }
            },
            Operand::BinaryOp { left, right, .. } => {
                self.replace_operand_with_register(left, allocation);
                self.replace_operand_with_register(right, allocation);
            },
            Operand::UnaryOp { operand, .. } => {
                self.replace_operand_with_register(operand, allocation);
            },
            _ => {}
        }
    }
    
    /// 命令スケジューリング最適化
    fn run_instruction_scheduling(&mut self, module: &mut Module) -> Result<()> {
        debug!("命令スケジューリング最適化を実行");
        
        for (_, func) in module.functions.iter_mut() {
            // 依存関係グラフを構築
            let dependency_graph = self.build_dependency_graph(func);
            
            // 各ブロックの命令をスケジューリング
            for (block_id, block) in func.blocks.iter_mut() {
                let scheduled_instructions = self.schedule_block_instructions(func, &dependency_graph, block);
                block.instructions = scheduled_instructions;
            }
        }
        
        Ok(())
    }
    
    /// ブロック内の命令をスケジューリング
    fn schedule_block_instructions(&self, 
                                  func: &Function, 
                                  dependency_graph: &HashMap<InstructionId, Vec<InstructionId>>, 
                                  block: &Function::Block) -> Vec<InstructionId> {
        // 現在のブロックの命令IDを取得
        let current_instructions = &block.instructions;
        
        // 依存関係グラフから準備ができた命令を優先するスケジューリング
        let mut scheduled = Vec::new();
        let mut remaining: HashSet<InstructionId> = current_instructions.iter().cloned().collect();
        
        // 命令間の依存関係を表す有向グラフ
        let mut reverse_deps = HashMap::new();
        let mut in_degree = HashMap::new();
        
        // 逆依存グラフを構築
        for &instr_id in current_instructions {
            let deps = dependency_graph.get(&instr_id).cloned().unwrap_or_default();
            
            // このブロック内の依存のみを考慮
            let block_deps: Vec<InstructionId> = deps.into_iter()
                .filter(|dep_id| remaining.contains(dep_id))
                .collect();
            
            in_degree.insert(instr_id, block_deps.len());
            
            for &dep_id in &block_deps {
                reverse_deps.entry(dep_id)
                    .or_insert_with(Vec::new)
                    .push(instr_id);
            }
        }
        
        // トポロジカルソートによるスケジューリング
        let mut ready_queue: Vec<InstructionId> = in_degree.iter()
            .filter(|&(_, &count)| count == 0)
            .map(|(&id, _)| id)
            .collect();
        
        while !ready_queue.is_empty() {
            // 最適な命令を選択（ここでは単純に先頭を選択）
            let instr_id = ready_queue.remove(0);
            
            // スケジュールに追加
            scheduled.push(instr_id);
            remaining.remove(&instr_id);
            
            // 依存関係を更新
            if let Some(dependents) = reverse_deps.get(&instr_id) {
                for &dep_id in dependents {
                    let count = in_degree.entry(dep_id).or_insert(0);
                    *count -= 1;
                    
                    if *count == 0 {
                        ready_queue.push(dep_id);
                    }
                }
            }
        }
        
        // 残りの命令も追加（循環依存がある場合）
        for &instr_id in current_instructions {
            if remaining.contains(&instr_id) {
                scheduled.push(instr_id);
            }
        }
        
        scheduled
    }
    
    /// ループアンロール最適化
    fn run_loop_unrolling(&mut self, module: &mut Module) -> Result<()> {
        debug!("ループアンロール最適化を実行");
        
        for (func_id, func) in module.functions.iter_mut() {
            // ループを検出
            let loops = self.detect_loops(func);
            
            // 各ループについてアンロールを検討
            for loop_info in loops {
                // アンロール可能か評価
                if self.can_unroll_loop(func, &loop_info) {
                    debug!("ループ（ヘッダー: {:?}）をアンロール", loop_info.header);
                    self.unroll_loop(func, &loop_info, self.options.unroll_factor)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// ループがアンロール可能かどうか判定
    fn can_unroll_loop(&self, func: &Function, loop_info: &LoopInfo) -> bool {
        // ループの複雑さを評価
        let loop_size = loop_info.body.len();
        
        // 小さなループのみアンロール対象とする
        if loop_size > 10 {
            return false;
        }
        
        // ループの反復回数が既知かチェック
        if let Some(trip_count) = self.analyze_loop_trip_count(func, loop_info) {
            // 反復回数が少ない場合やアンロール係数の倍数の場合は有益
            if trip_count <= self.options.unroll_factor || trip_count % self.options.unroll_factor == 0 {
                return true;
            }
            
            // 非常に多い反復回数の場合はアンロールしない
            if trip_count > 100 {
                return false;
            }
        }
        
        // 副作用のないループかチェック
        let has_safe_instructions = loop_info.body.iter().all(|block_id| {
            if let Some(block) = func.blocks.get(block_id) {
                block.instructions.iter().all(|instr_id| {
                    if let Some(instr) = func.instructions.get(instr_id) {
                        !self.has_complex_side_effects(instr)
                    } else {
                        false
                    }
                })
            } else {
                false
            }
        });
        
        has_safe_instructions
    }
    
    /// 命令が複雑な副作用を持つかチェック
    fn has_complex_side_effects(&self, instr: &Instruction) -> bool {
        match instr {
            Instruction::Call { .. } => true,  // 関数呼び出しは複雑な副作用を持つ可能性がある
            Instruction::Store { .. } => false, // 単純なストアは許容
            _ => self.has_side_effects(instr),
        }
    }
    
    /// ループの反復回数を分析
    fn analyze_loop_trip_count(&self, func: &Function, loop_info: &LoopInfo) -> Option<usize> {
        // ループ内のブロックを取得
        let header_block = func.blocks.get(&loop_info.header)?;
        
        // ループの終了条件を分析
        for instr_id in &header_block.instructions {
            if let Some(Instruction::ConditionalBranch { condition, true_target, false_target }) = func.instructions.get(instr_id) {
                // ループを出るパスを特定
                let exit_target = if loop_info.body.contains(true_target) {
                    false_target
                } else if loop_info.body.contains(false_target) {
                    true_target
                } else {
                    continue;
                };
                
                // 条件が定数比較かチェック
                if let Operand::InstructionRef(cond_id) = condition {
                    if let Some(Instruction::BinaryOp { op, lhs, rhs, .. }) = func.instructions.get(cond_id) {
                        match op {
                            BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                                // インダクション変数と上限値を特定
                                if let (Operand::Variable(var_name), Operand::Literal(Literal::Int(limit))) = (lhs, rhs) {
                                    // インダクション変数の増分を特定
                                    if let Some(increment) = self.find_induction_variable_increment(func, loop_info, var_name) {
                                        // 反復回数を計算
                                        return Some((*limit as usize) / increment);
                                    }
                                } else if let (Operand::Literal(Literal::Int(limit)), Operand::Variable(var_name)) = (lhs, rhs) {
                                    // インダクション変数の増分を特定
                                    if let Some(increment) = self.find_induction_variable_increment(func, loop_info, var_name) {
                                        // 反復回数を計算
                                        return Some((*limit as usize) / increment);
                                    }
                                }
                            },
                            _ => {}
                        }
                    }
                }
            }
        }
        
        None
    }
    
    /// インダクション変数の増分を特定
    fn find_induction_variable_increment(&self, func: &Function, loop_info: &LoopInfo, var_name: &str) -> Option<usize> {
        // ループ内でのインダクション変数の更新を探す
        for block_id in &loop_info.body {
            if let Some(block) = func.blocks.get(block_id) {
                for instr_id in &block.instructions {
                    if let Some(Instruction::BinaryOp { op, lhs, rhs, result }) = func.instructions.get(instr_id) {
                        if result == var_name && op == &BinaryOp::Add {
                            // 変数 = 変数 + 定数 のパターンを探す
                            if let (Operand::Variable(lhs_var), Operand::Literal(Literal::Int(increment))) = (lhs, rhs) {
                                if lhs_var == var_name {
                                    return Some(*increment as usize);
                                }
                            } else if let (Operand::Literal(Literal::Int(increment)), Operand::Variable(rhs_var)) = (lhs, rhs) {
                                if rhs_var == var_name {
                                    return Some(*increment as usize);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        None
    }
    
    /// ループをアンロール
    fn unroll_loop(&mut self, func: &mut Function, loop_info: &LoopInfo, factor: usize) -> Result<()> {
        if factor <= 1 {
            return Ok(());  // アンロール不要
        }
        
        // オリジナルループのボディブロックをコピー
        let body_blocks: Vec<BlockId> = loop_info.body.iter().cloned().collect();
        
        // ループヘッダーの特定と終了条件の取得
        let header_block = match func.blocks.get(&loop_info.header) {
            Some(block) => block.clone(),
            None => return Err(EidosError::Optimization("ループヘッダーが見つかりません".to_string())),
        };
        
        // 終了条件をチェックするブロックとそのインデックスを特定
        let (exit_check_instr_id, exit_condition) = self.find_loop_exit_condition(func, loop_info)?;
        
        // ループボディをfactor回複製
        for i in 1..factor {
            // 各ブロックを複製し、新しいIDでマッピング
            let mut block_mapping = HashMap::new();
            
            for &original_id in &body_blocks {
                if original_id == loop_info.header {
                    continue;  // ヘッダーは特別に処理
                }
                
                let new_block_id = BlockId::new();
                block_mapping.insert(original_id, new_block_id);
                
                // ブロックをコピー
                if let Some(original_block) = func.blocks.get(&original_id) {
                    let mut new_block = original_block.clone();
                    
                    // 命令をコピーし、IDを更新
                    let mut new_instructions = Vec::new();
                    let mut instr_mapping = HashMap::new();
                    
                    for &instr_id in &original_block.instructions {
                        let new_instr_id = InstructionId::new();
                        instr_mapping.insert(instr_id, new_instr_id);
                        
                        // 命令をコピー
                        if let Some(instr) = func.instructions.get(&instr_id) {
                            let new_instr = instr.clone();
                            func.instructions.insert(new_instr_id, new_instr);
                            new_instructions.push(new_instr_id);
                        }
                    }
                    
                    // ブロック内の命令を更新
                    new_block.instructions = new_instructions;
                    
                    // 新しいブロックを追加
                    func.blocks.insert(new_block_id, new_block);
                }
            }
            
            // ブロック間の接続を修正
            for (&original_id, &new_id) in &block_mapping {
                if let Some(block) = func.blocks.get_mut(&new_id) {
                    // 最後の命令がジャンプ系かチェック
                    if let Some(&last_instr_id) = block.instructions.last() {
                        if let Some(instr) = func.instructions.get_mut(&last_instr_id) {
                            match instr {
                                Instruction::Branch { target } => {
                                    // ジャンプ先を更新
                                    if let Some(&new_target) = block_mapping.get(target) {
                                        *target = new_target;
                                    } else if *target == loop_info.header {
                                        // 次の繰り返しへ
                                        if i < factor - 1 {
                                            let next_iteration_start = *block_mapping.values().next().unwrap();
                                            *target = next_iteration_start;
                                        }
                                    }
                                },
                                Instruction::ConditionalBranch { true_target, false_target, .. } => {
                                    // 条件分岐先を更新
                                    if let Some(&new_true) = block_mapping.get(true_target) {
                                        *true_target = new_true;
                                    } else if *true_target == loop_info.header {
                                        // 次の繰り返しへ
                                        if i < factor - 1 {
                                            let next_iteration_start = *block_mapping.values().next().unwrap();
                                            *true_target = next_iteration_start;
                                        }
                                    }
                                    
                                    if let Some(&new_false) = block_mapping.get(false_target) {
                                        *false_target = new_false;
                                    } else if *false_target == loop_info.header {
                                        // 次の繰り返しへ
                                        if i < factor - 1 {
                                            let next_iteration_start = *block_mapping.values().next().unwrap();
                                            *false_target = next_iteration_start;
                                        }
                                    }
                                },
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        
        // 最後に終了条件のチェックを調整
        // ループカウンタの増分を反映
        if let Some(instr) = func.instructions.get_mut(&exit_check_instr_id) {
            if let Instruction::BinaryOp { lhs, rhs, .. } = instr {
                if let Operand::Variable(var_name) = lhs {
                    // インダクション変数の増分を反映
                    if let Some(increment) = self.find_induction_variable_increment(func, loop_info, var_name) {
                        // 新しいカウンタ値を計算
                        if let Operand::Literal(Literal::Int(limit)) = rhs {
                            let new_limit = *limit + (increment as i64) * (factor as i64 - 1);
                            *rhs = Operand::Literal(Literal::Int(new_limit));
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// ループの終了条件を特定
    fn find_loop_exit_condition(&self, func: &Function, loop_info: &LoopInfo) -> Result<(InstructionId, Operand)> {
        // ループヘッダーブロックを取得
        let header_block = match func.blocks.get(&loop_info.header) {
            Some(block) => block,
            None => return Err(EidosError::Optimization("ループヘッダーが見つかりません".to_string())),
        };
        
        // 条件分岐命令を探す
        for &instr_id in &header_block.instructions {
            if let Some(Instruction::ConditionalBranch { condition, true_target, false_target }) = func.instructions.get(&instr_id) {
                // ループを出るパスを特定
                if !loop_info.body.contains(true_target) || !loop_info.body.contains(false_target) {
                    return Ok((instr_id, condition.clone()));
                }
            }
        }
        
        Err(EidosError::Optimization("ループの終了条件が見つかりません".to_string()))
    }
    
    /// SIMD最適化
    fn run_simd_optimization(&mut self, module: &mut Module) -> Result<()> {
        debug!("SIMD最適化を実行");
        
        if !self.options.enable_simd {
            debug!("SIMD最適化は無効化されています");
            return Ok(());
        }
        
        for (_func_id, func) in module.functions.iter_mut() {
            // ベクトル化可能なループを特定
            let loops = self.detect_loops(func);
            
            for loop_info in loops {
                // ベクトル化可能か評価
                if self.can_vectorize_loop(func, &loop_info) {
                    debug!("ループ（ヘッダー: {:?}）をベクトル化", loop_info.header);
                    self.vectorize_loop(func, &loop_info)?;
                }
            }
            
            // ベクトル命令のパターンを特定
            self.identify_and_convert_vector_patterns(func)?;
        }
        
        Ok(())
    }
    
    /// ループがベクトル化可能かどうか判定
    fn can_vectorize_loop(&self, func: &Function, loop_info: &LoopInfo) -> bool {
        // 依存関係がないかチェック
        let mut memory_accesses = HashMap::new();
        let mut has_cross_iteration_dependency = false;
        
        // ループ内の各ブロックで使用・定義されるメモリアクセスを収集
        for &block_id in &loop_info.body {
            if let Some(block) = func.blocks.get(&block_id) {
                for &instr_id in &block.instructions {
                    if let Some(instr) = func.instructions.get(&instr_id) {
                        match instr {
                            Instruction::Load { address, result } => {
                                memory_accesses.insert(result.clone(), (address.clone(), true)); // true = 読み取り
                            },
                            Instruction::Store { address, value } => {
                                // 書き込みパターンをチェック
                                if let Operand::Variable(var_name) = value {
                                    // この変数が同じループ内で読み取られているかチェック
                                    if memory_accesses.values().any(|(acc_addr, is_read)| 
                                        *is_read && matches!(acc_addr, Operand::Variable(name) if name == var_name)) {
                                        has_cross_iteration_dependency = true;
                                    }
                                }
                                
                                memory_accesses.insert(address.clone().to_string(), (value.clone(), false)); // false = 書き込み
                            },
                            _ => {}
                        }
                    }
                }
            }
        }
        
        // インダクション変数が1つだけあるか確認
        let induction_var = self.find_loop_induction_variable(func, loop_info);
        if induction_var.is_none() {
            return false;
        }
        
        // メモリアクセスパターンがインダクション変数に依存した連続アクセスかチェック
        let continuous_memory_access = self.has_continuous_memory_access(func, loop_info, induction_var.unwrap());
        
        // ベクトル化を妨げる依存関係がなく、連続メモリアクセスパターンがあればベクトル化可能
        !has_cross_iteration_dependency && 
        continuous_memory_access && 
        // ループが十分シンプルかつ規則的
        loop_info.is_simple_counting_loop(func)
    }
    
    /// 連続したメモリアクセスパターンを持つか検証
    fn has_continuous_memory_access(&self, func: &Function, loop_info: &LoopInfo, induction_var: &str) -> bool {
        // 連続したメモリアクセス（配列アクセス）を検出
        for &block_id in &loop_info.body {
            if let Some(block) = func.blocks.get(&block_id) {
                for &instr_id in &block.instructions {
                    if let Some(instr) = func.instructions.get(&instr_id) {
                        match instr {
                            Instruction::GetElementPtr { base, indices, .. } => {
                                // インデックスがインダクション変数を含むか確認
                                for idx in indices {
                                    if let Operand::Variable(var_name) = idx {
                                        if var_name == induction_var {
                                            return true;
                                        }
                                    }
                                }
                            },
                            _ => {}
                        }
                    }
                }
            }
        }
        false
    }
    
    /// ループのインダクション変数を特定
    fn find_loop_induction_variable(&self, func: &Function, loop_info: &LoopInfo) -> Option<&str> {
        // ループヘッダーの条件分岐を検索
        if let Some(header_block) = func.blocks.get(&loop_info.header) {
            for &instr_id in &header_block.instructions {
                if let Some(Instruction::ConditionalBranch { condition, .. }) = func.instructions.get(&instr_id) {
                    if let Operand::InstructionRef(cond_id) = condition {
                        if let Some(Instruction::BinaryOp { op, left, right, .. }) = func.instructions.get(cond_id) {
                            if *op == "lt" || *op == "le" || *op == "gt" || *op == "ge" {
                                // 比較演算の片方が変数の場合
                                if let Operand::Variable(var_name) = left {
                                    if loop_info.is_induction_variable(func, var_name) {
                                        return Some(var_name);
                                    }
                                } else if let Operand::Variable(var_name) = right {
                                    if loop_info.is_induction_variable(func, var_name) {
                                        return Some(var_name);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }
    
    /// ループをベクトル化
    fn vectorize_loop(&mut self, func: &mut Function, loop_info: &LoopInfo) -> Result<()> {
        let induction_var = match self.find_loop_induction_variable(func, loop_info) {
            Some(var) => var,
            None => return Err(EidosError::Optimization("ベクトル化可能なインダクション変数が見つかりません".to_string())),
        };
        
        // ベクトル化のステップ数を決定
        let vector_width = 4; // 4要素のSIMDを使用
        
        if let Some(preheader) = loop_info.preheader {
            if let Some(preheader_block) = func.blocks.get_mut(&preheader) {
                // プリヘッダーに新しい変数を追加（SIMD化用）
                let simd_setup_id = self.create_instruction_id();
                let simd_var = format!("{}_simd", induction_var);
                
                // インダクション変数の増分を取得
                let increment = match self.find_induction_variable_increment(func, loop_info, induction_var) {
                    Some(inc) => inc,
                    None => return Err(EidosError::Optimization("インダクション変数の増分が特定できません".to_string())),
                };
                
                // SIMD初期化命令を作成（[i, i+1, i+2, i+3]のようなベクトルを初期化）
                let init_instr = Instruction::SIMDInit {
                    base: Operand::Variable(induction_var.to_string()),
                    step: increment,
                    width: vector_width,
                    result: simd_var.clone(),
                };
                
                func.instructions.insert(simd_setup_id, init_instr);
                preheader_block.instructions.push(simd_setup_id);
            }
        }
        
        // ベクトル化可能な操作を特定してSIMD命令に変換
        let mut simd_blocks = HashMap::new();
        let mut vector_ops = Vec::new();
        
        // ループ本体のベクトル化
        for &block_id in &loop_info.body {
            if let Some(block) = func.blocks.get(&block_id) {
                // 新しいベクトル化ブロックを作成
                let simd_block_id = BlockId(block_id.0 + 1000); // 一意なIDを生成
                let mut simd_block = BasicBlock::new(simd_block_id);
                
                // 命令をベクトル化
                for &instr_id in &block.instructions {
                    if let Some(instr) = func.instructions.get(&instr_id) {
                        // ベクトル化可能な命令をチェック
                        match instr {
                            Instruction::BinaryOp { op, left, right, result } => {
                                // 算術演算をベクトル化
                                if *op == "add" || *op == "sub" || *op == "mul" || *op == "div" {
                                    // ベクトル版の変数名を生成
                                    let simd_result = format!("{}_simd", result);
                                    
                                    // ベクトル版の入力オペランドを生成
                                    let simd_left = self.simd_operand(left, induction_var);
                                    let simd_right = self.simd_operand(right, induction_var);
                                    
                                    // SIMD命令を生成
                                    let simd_op_id = self.create_instruction_id();
                                    let simd_op = Instruction::SIMDBinaryOp {
                                        op: op.clone(),
                                        left: simd_left,
                                        right: simd_right,
                                        result: simd_result,
                                    };
                                    
                                    func.instructions.insert(simd_op_id, simd_op);
                                    simd_block.instructions.push(simd_op_id);
                                    vector_ops.push((instr_id, simd_op_id));
                                }
                            },
                            Instruction::Load { address, result } => {
                                // ロード操作をベクトル化
                                if let Some(mem_pattern) = self.check_memory_access_pattern(address, induction_var) {
                                    let simd_result = format!("{}_simd", result);
                                    
                                    // ベクトルロード命令を生成
                                    let simd_load_id = self.create_instruction_id();
                                    let simd_load = Instruction::SIMDLoad {
                                        base_address: mem_pattern.base,
                                        stride: mem_pattern.stride,
                                        result: simd_result,
                                    };
                                    
                                    func.instructions.insert(simd_load_id, simd_load);
                                    simd_block.instructions.push(simd_load_id);
                                    vector_ops.push((instr_id, simd_load_id));
                                }
                            },
                            Instruction::Store { address, value } => {
                                // ストア操作をベクトル化
                                if let Some(mem_pattern) = self.check_memory_access_pattern(address, induction_var) {
                                    let simd_value = self.simd_operand(value, induction_var);
                                    
                                    // ベクトルストア命令を生成
                                    let simd_store_id = self.create_instruction_id();
                                    let simd_store = Instruction::SIMDStore {
                                        base_address: mem_pattern.base,
                                        stride: mem_pattern.stride,
                                        value: simd_value,
                                    };
                                    
                                    func.instructions.insert(simd_store_id, simd_store);
                                    simd_block.instructions.push(simd_store_id);
                                    vector_ops.push((instr_id, simd_store_id));
                                }
                            },
                            _ => {
                                // ベクトル化できない命令は通常通りコピー
                                simd_block.instructions.push(instr_id);
                            }
                        }
                    }
                }
                
                // インダクション変数の更新を修正（ベクトル幅分進める）
                let update_id = self.create_instruction_id();
                let update_instr = Instruction::BinaryOp {
                    op: "add".to_string(),
                    left: Operand::Variable(induction_var.to_string()),
                    right: Operand::Literal(Literal::Int(vector_width as i64 * increment as i64)),
                    result: induction_var.to_string(),
                };
                
                func.instructions.insert(update_id, update_instr);
                simd_block.instructions.push(update_id);
                
                // 新しいブロックを保存
                simd_blocks.insert(block_id, simd_block);
            }
        }
        
        // 元のループ構造を変更して、ベクトル化ブロックを組み込む
        if let Some(header_block) = func.blocks.get_mut(&loop_info.header) {
            // 条件分岐命令を修正（ループカウンタの増分を考慮）
            for i in 0..header_block.instructions.len() {
                if let Some(instr_id) = header_block.instructions.get(i) {
                    if let Some(Instruction::ConditionalBranch { condition, true_target, false_target }) = func.instructions.get_mut(instr_id) {
                        // 条件式を修正（ベクトル化のためのバウンドチェック追加）
                        let new_cond_id = self.create_instruction_id();
                        let original_cond = condition.clone();
                        
                        let vector_bound_check = Instruction::BinaryOp {
                            op: "lt".to_string(),
                            left: Operand::Variable(induction_var.to_string()),
                            right: Operand::InstructionRef(new_cond_id),
                            result: format!("{}_vector_check", induction_var),
                        };
                        
                        func.instructions.insert(new_cond_id, vector_bound_check);
                        
                        // 分岐先をベクトル化ブロックに修正
                        if let Some(simd_block) = simd_blocks.get(true_target) {
                            *true_target = simd_block.id;
                        }
                    }
                }
            }
        }
        
        // すべてのSIMDブロックを関数に追加
        for (_, block) in simd_blocks {
            func.blocks.insert(block.id, block);
        }
        
        // ベクトル化変換をログ
        debug!("ループを{}個のSIMD操作に変換", vector_ops.len());
        
        Ok(())
    }
    
    /// オペランドのベクトル版を生成
    fn simd_operand(&self, operand: &Operand, induction_var: &str) -> Operand {
        match operand {
            Operand::Variable(var) => {
                if var == induction_var {
                    // インダクション変数の場合はSIMD専用変数を使用
                    Operand::Variable(format!("{}_simd", var))
                } else {
                    // その他の変数はブロードキャスト
                    Operand::SIMDBroadcast(Box::new(operand.clone()))
                }
            },
            Operand::Literal(_) => {
                // リテラルはブロードキャスト
                Operand::SIMDBroadcast(Box::new(operand.clone()))
            },
            _ => operand.clone(),
        }
    }
    
    /// メモリアクセスパターンを解析
    struct MemoryPattern {
        base: Operand,
        stride: i32,
    }
    
    fn check_memory_access_pattern(&self, address: &Operand, induction_var: &str) -> Option<MemoryPattern> {
        match address {
            Operand::GetElementPtr { base, indices } => {
                // GEP命令の場合、インデックスにインダクション変数が含まれるか確認
                for idx in indices {
                    if let Operand::Variable(var) = idx {
                        if var == induction_var {
                            return Some(MemoryPattern {
                                base: base.clone(),
                                stride: 1,  // デフォルトのストライド
                            });
                        }
                    }
                }
                None
            },
            Operand::BinaryOp { op, left, right } => {
                // アドレス計算の場合（例: base + i * 4）
                if *op == "add" {
                    if let Operand::BinaryOp { op: mul_op, left: mul_left, right: mul_right } = &**right {
                        if *mul_op == "mul" {
                            // i * stride パターンを検出
                            if let Operand::Variable(var) = &**mul_left {
                                if var == induction_var {
                                    if let Operand::Literal(Literal::Int(stride)) = &**mul_right {
                                        return Some(MemoryPattern {
                                            base: left.clone(),
                                            stride: *stride as i32,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                None
            },
            _ => None,
        }
    }
    
    /// ベクトル命令パターンを特定して変換
    fn identify_and_convert_vector_patterns(&mut self, func: &mut Function) -> Result<()> {
        // 特定のパターンを探して、ベクトル命令に変換
        // 例: 連続した同一操作 (a[i] = b[i] + c[i] for i=0..n)
        
        // 基本ブロックごとにスキャン
        for (block_id, block) in func.blocks.iter_mut() {
            let mut i = 0;
            while i + 3 < block.instructions.len() {
                // 4つ以上の連続した同種命令をチェック
                let mut pattern_length = 1;
                let base_instr_id = block.instructions[i];
                
                if let Some(base_pattern) = func.instructions.get(&base_instr_id) {
                    // パターンの種類を判別
                    match base_pattern {
                        Instruction::BinaryOp { op: base_op, left: base_left, right: base_right, result: base_result } => {
                            // 同じ演算子の連続をチェック
                            let mut operands = vec![(base_left.clone(), base_right.clone(), base_result.clone())];
                            
                            for j in i+1..block.instructions.len() {
                                if let Some(Instruction::BinaryOp { op, left, right, result }) = func.instructions.get(&block.instructions[j]) {
                                    if op == base_op && self.can_vectorize_together(base_left, left) && self.can_vectorize_together(base_right, right) {
                                        pattern_length += 1;
                                        operands.push((left.clone(), right.clone(), result.clone()));
                                    } else {
                                        break;
                                    }
                                } else {
                                    break;
                                }
                            }
                            
                            // 4つ以上の同一演算があればベクトル命令に変換
                            if pattern_length >= 4 {
                                debug!("{}個の連続{}演算をベクトル命令に変換", pattern_length, base_op);
                                
                                // SIMDベクトルを作成
                                let simd_left_id = self.create_instruction_id();
                                let simd_right_id = self.create_instruction_id();
                                let simd_result_id = self.create_instruction_id();
                                
                                let left_values: Vec<Operand> = operands.iter().map(|(l, _, _)| l.clone()).collect();
                                let right_values: Vec<Operand> = operands.iter().map(|(_, r, _)| r.clone()).collect();
                                let results: Vec<String> = operands.iter().map(|(_, _, r)| r.clone()).collect();
                                
                                // ベクトル構築命令
                                let simd_left = Instruction::SIMDVector {
                                    elements: left_values,
                                    result: format!("simd_left_{}", simd_left_id.0),
                                };
                                
                                let simd_right = Instruction::SIMDVector {
                                    elements: right_values,
                                    result: format!("simd_right_{}", simd_right_id.0),
                                };
                                
                                // SIMD演算命令
                                let simd_op = Instruction::SIMDBinaryOp {
                                    op: base_op.clone(),
                                    left: Operand::Variable(format!("simd_left_{}", simd_left_id.0)),
                                    right: Operand::Variable(format!("simd_right_{}", simd_right_id.0)),
                                    result: format!("simd_result_{}", simd_result_id.0),
                                };
                                
                                // 結果の抽出命令
                                let extract_instrs = (0..pattern_length).map(|idx| {
                                    let extract_id = self.create_instruction_id();
                                    (extract_id, Instruction::SIMDExtract {
                                        vector: Operand::Variable(format!("simd_result_{}", simd_result_id.0)),
                                        index: idx,
                                        result: results[idx].clone(),
                                    })
                                }).collect::<Vec<_>>();
                                
                                // 元の命令を置き換え
                                func.instructions.insert(simd_left_id, simd_left);
                                func.instructions.insert(simd_right_id, simd_right);
                                func.instructions.insert(simd_result_id, simd_op);
                                
                                for (extract_id, extract_instr) in extract_instrs {
                                    func.instructions.insert(extract_id, extract_instr);
                                }
                                
                                // ブロック内の元の命令を置き換え
                                block.instructions[i] = simd_left_id;
                                block.instructions[i+1] = simd_right_id;
                                block.instructions[i+2] = simd_result_id;
                                
                                // 抽出命令を追加
                                for (idx, (extract_id, _)) in extract_instrs.iter().enumerate() {
                                    if idx < 3 {
                                        // 最初の3つは既存の命令スロットを使用
                                        block.instructions[i+3+idx] = *extract_id;
                                    } else if i+3+idx < block.instructions.len() {
                                        // 既存のスロットがあれば使用
                                        block.instructions[i+3+idx] = *extract_id;
                                    } else {
                                        // 新しく追加
                                        block.instructions.push(*extract_id);
                                    }
                                }
                                
                                // 命令数が一致しない場合は調整
                                if pattern_length > 4 {
                                    // 余分な命令を削除
                                    block.instructions.drain(i+3+extract_instrs.len()..i+pattern_length);
                                }
                                
                                i += extract_instrs.len() + 3;
                                continue;
                            }
                        },
                        _ => {}
                    }
                }
                
                // パターンに該当しなければ次の命令へ
                i += 1;
            }
        }
        
        Ok(())
    }
    
    /// 二つのオペランドがベクトル化で一緒に処理できるか判定
    fn can_vectorize_together(&self, a: &Operand, b: &Operand) -> bool {
        match (a, b) {
            (Operand::Variable(a_name), Operand::Variable(b_name)) => {
        }

        false
    }

    /// オペランドが定数かどうかを判定
    fn is_constant(&self, operand: &Operand) -> bool {
        match operand {
            Operand::Literal(_) => true,
            _ => false,
        }
    }

    /// ループのバックエッジを取得
    fn get_back_edges(&self, func: &Function) -> Vec<BlockId> {
        let mut back_edges = Vec::new();

        for &block_id in &self.body {
            if block_id == self.header {
                continue;
            }

            if let Some(block) = func.blocks.get(&block_id) {
                // ブロックの後継がヘッダーブロックを含むか確認
                if block.successors.contains(&self.header) {
                    back_edges.push(block_id);
                }
            }
        }

        back_edges
    }

    /// ループが単純なカウントループかどうかを判定
    pub fn is_simple_counting_loop(&self, func: &Function) -> bool {
        // 単純なカウントループの条件:
        // 1. 単一の退出条件
        // 2. 誘導変数が存在する
        // 3. 誘導変数が定数で初期化される
        // 4. 誘導変数が一定量ずつ増減される

        // 単一の退出ブロックを確認
        if self.exits.len() != 1 {
            return false;
        }

        // ループの誘導変数を見つける
        let mut induction_var = None;
        
        // ヘッダブロックで比較に使われている変数を探す
        if let Some(header_block) = func.blocks.get(&self.header) {
            for &instr_id in &header_block.instructions {
                if let Some(Instruction::ConditionalBranch { condition, .. }) = func.instructions.get(&instr_id) {
                    if let Operand::InstructionRef(cond_id) = condition {
                        if let Some(Instruction::BinaryOp { left, right, .. }) = func.instructions.get(cond_id) {
                            if let Operand::Variable(var_name) = left {
                                if self.is_induction_variable(func, var_name) {
                                    induction_var = Some(var_name.clone());
                                    break;
                                }
                            } else if let Operand::Variable(var_name) = right {
                                if self.is_induction_variable(func, var_name) {
                                    induction_var = Some(var_name.clone());
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        // 誘導変数が見つからなければfalse
        if induction_var.is_none() {
            return false;
        }

        // 誘導変数の初期化が定数かを確認
        let induction_var = induction_var.unwrap();
        let mut has_constant_init = false;

        // プリヘッダブロックで誘導変数が定数で初期化されるか確認
        if let Some(preheader) = self.preheader {
            if let Some(preheader_block) = func.blocks.get(&preheader) {
                for &instr_id in &preheader_block.instructions {
                    if let Some(Instruction::BinaryOp { op, right, result, .. }) = func.instructions.get(&instr_id) {
                        if op == "assign" && result == &induction_var {
                            if let Operand::Literal(Literal::Int(_)) = right {
                                has_constant_init = true;
                                break;
                            }
                        }
                    }
                }
            }
        }

        // 定数で初期化されていなければfalse
        if !has_constant_init {
            return false;
        }

        // 誘導変数が一定量ずつ更新されるか確認
        let mut has_constant_step = false;
        
        for &block_id in &self.body {
            if let Some(block) = func.blocks.get(&block_id) {
                for &instr_id in &block.instructions {
                    if let Some(Instruction::BinaryOp { op, left, right, result }) = func.instructions.get(&instr_id) {
                        if (op == "add" || op == "sub") && result == &induction_var {
                            if left == &Operand::Variable(induction_var.clone()) {
                                if let Operand::Literal(Literal::Int(_)) = right {
                                    has_constant_step = true;
                                    break;
                                }
                            } else if right == &Operand::Variable(induction_var.clone()) {
                                if let Operand::Literal(Literal::Int(_)) = left {
                                    has_constant_step = true;
                                    break;
                                }
                            }
                        }
                    }
                }
                
                if has_constant_step {
                    break;
                }
            }
        }

        has_constant_step
    }
}