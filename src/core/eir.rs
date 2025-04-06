use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;

use crate::core::types::{Type, TypeId};
use crate::core::symbol::SymbolId;

/// EIR (Eidos Intermediate Representation) モジュール
#[derive(Debug, Clone)]
pub struct Module {
    /// モジュールの名前
    pub name: String,
    /// モジュール内の関数
    pub functions: HashMap<FunctionId, Function>,
    /// グローバル変数
    pub globals: HashMap<String, Global>,
    /// 外部関数の宣言
    pub external_functions: HashMap<String, ExternalFunction>,
    /// 型情報
    pub types: HashMap<TypeId, Type>,
    /// エントリーポイント関数のID（存在する場合）
    pub entry_point: Option<FunctionId>,
}

impl Module {
    /// 新しいモジュールを作成
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            functions: HashMap::new(),
            globals: HashMap::new(),
            external_functions: HashMap::new(),
            types: HashMap::new(),
            entry_point: None,
        }
    }
    
    /// 関数を追加
    pub fn add_function(&mut self, function: Function) -> FunctionId {
        let id = function.id;
        self.functions.insert(id, function);
        id
    }
    
    /// グローバル変数を追加
    pub fn add_global(&mut self, name: &str, global: Global) {
        self.globals.insert(name.to_string(), global);
    }
    
    /// 外部関数を宣言
    pub fn declare_external_function(&mut self, name: &str, external: ExternalFunction) {
        self.external_functions.insert(name.to_string(), external);
    }
    
    /// 型を追加
    pub fn add_type(&mut self, ty: Type) -> TypeId {
        let id = ty.id;
        self.types.insert(id, ty);
        id
    }
    
    /// エントリーポイントを設定
    pub fn set_entry_point(&mut self, function_id: FunctionId) {
        self.entry_point = Some(function_id);
    }
    
    /// 関数IDから関数を取得
    pub fn get_function(&self, id: FunctionId) -> Option<&Function> {
        self.functions.get(&id)
    }
    
    /// 関数IDから関数を可変で取得
    pub fn get_function_mut(&mut self, id: FunctionId) -> Option<&mut Function> {
        self.functions.get_mut(&id)
    }
    
    /// 名前から関数を取得
    pub fn get_function_by_name(&self, name: &str) -> Option<&Function> {
        self.functions.values().find(|f| f.name == name)
    }
    
    /// グローバル変数を取得
    pub fn get_global(&self, name: &str) -> Option<&Global> {
        self.globals.get(name)
    }
    
    /// 型IDから型を取得
    pub fn get_type(&self, id: TypeId) -> Option<&Type> {
        self.types.get(&id)
    }
}

/// 関数ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId(pub u32);

impl fmt::Display for FunctionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "func_{}", self.0)
    }
}

/// 基本ブロックID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub u32);

impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "block_{}", self.0)
    }
}

/// 命令ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InstructionId(pub u32);

impl fmt::Display for InstructionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "instr_{}", self.0)
    }
}

/// レジスタID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegisterId(pub u32);

impl fmt::Display for RegisterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.0)
    }
}

/// EIR 関数
#[derive(Debug, Clone)]
pub struct Function {
    /// 関数ID
    pub id: FunctionId,
    /// 関数名
    pub name: String,
    /// 関数の型
    pub function_type: TypeId,
    /// パラメータ名と型
    pub parameters: Vec<(String, TypeId)>,
    /// 戻り値の型
    pub return_type: TypeId,
    /// 基本ブロック
    pub blocks: HashMap<BlockId, BasicBlock>,
    /// エントリーブロック
    pub entry_block: BlockId,
    /// 次に割り当てるブロックID
    pub next_block_id: u32,
    /// 次に割り当てるレジスタID
    pub next_register_id: u32,
    /// 次に割り当てる命令ID
    pub next_instruction_id: u32,
    /// レジスタの型情報
    pub register_types: HashMap<RegisterId, TypeId>,
    /// 関数の属性
    pub attributes: FunctionAttributes,
}

impl Function {
    /// 新しい関数を作成
    pub fn new(id: FunctionId, name: &str, function_type: TypeId, return_type: TypeId) -> Self {
        let entry_block_id = BlockId(0);
        let mut blocks = HashMap::new();
        blocks.insert(entry_block_id, BasicBlock::new(entry_block_id));
        
        Self {
            id,
            name: name.to_string(),
            function_type,
            parameters: Vec::new(),
            return_type,
            blocks,
            entry_block: entry_block_id,
            next_block_id: 1,
            next_register_id: 0,
            next_instruction_id: 0,
            register_types: HashMap::new(),
            attributes: FunctionAttributes::default(),
        }
    }
    
    /// 新しい基本ブロックを作成
    pub fn create_block(&mut self) -> BlockId {
        let id = BlockId(self.next_block_id);
        self.next_block_id += 1;
        self.blocks.insert(id, BasicBlock::new(id));
        id
    }
    
    /// 新しいレジスタを作成
    pub fn create_register(&mut self, type_id: TypeId) -> RegisterId {
        let id = RegisterId(self.next_register_id);
        self.next_register_id += 1;
        self.register_types.insert(id, type_id);
        id
    }
    
    /// 新しい命令IDを取得
    pub fn next_instruction_id(&mut self) -> InstructionId {
        let id = InstructionId(self.next_instruction_id);
        self.next_instruction_id += 1;
        id
    }
    
    /// ブロックに命令を追加
    pub fn add_instruction(&mut self, block_id: BlockId, instruction: Instruction) -> InstructionId {
        let instr_id = self.next_instruction_id();
        if let Some(block) = self.blocks.get_mut(&block_id) {
            block.instructions.push((instr_id, instruction));
        }
        instr_id
    }
    
    /// 命令を置き換え
    pub fn replace_instruction(&mut self, block_id: BlockId, instr_id: InstructionId, new_instruction: Instruction) -> bool {
        if let Some(block) = self.blocks.get_mut(&block_id) {
            for (id, instr) in &mut block.instructions {
                if *id == instr_id {
                    *instr = new_instruction;
                    return true;
                }
            }
        }
        false
    }
    
    /// ブロックを取得
    pub fn get_block(&self, id: BlockId) -> Option<&BasicBlock> {
        self.blocks.get(&id)
    }
    
    /// ブロックを可変で取得
    pub fn get_block_mut(&mut self, id: BlockId) -> Option<&mut BasicBlock> {
        self.blocks.get_mut(&id)
    }
    
    /// レジスタの型を取得
    pub fn get_register_type(&self, reg: RegisterId) -> Option<TypeId> {
        self.register_types.get(&reg).copied()
    }
    
    /// パラメータを追加
    pub fn add_parameter(&mut self, name: &str, type_id: TypeId) -> RegisterId {
        let reg_id = self.create_register(type_id);
        self.parameters.push((name.to_string(), type_id));
        reg_id
    }
    
    /// 関数の使用グラフを計算
    pub fn compute_use_graph(&self) -> FunctionUseGraph {
        let mut graph = FunctionUseGraph::new();
        
        // 各ブロックの命令をスキャンして使用情報を収集
        for (block_id, block) in &self.blocks {
            for (instr_id, instr) in &block.instructions {
                // 命令が定義するレジスタ
                if let Some(reg) = instr.defined_register() {
                    graph.register_defs.insert(reg, (*block_id, *instr_id));
                }
                
                // 命令が使用するレジスタ
                for reg in instr.used_registers() {
                    graph.register_uses.entry(reg).or_default().push((*block_id, *instr_id));
                }
            }
        }
        
        graph
    }
}

/// 基本ブロック
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// ブロックID
    pub id: BlockId,
    /// 命令列（ID, 命令）
    pub instructions: Vec<(InstructionId, Instruction)>,
    /// 終了命令（ブロックの最後）
    pub terminator: Option<Terminator>,
    /// ブロックのパラメータ（PhiノードやSSA形式の実装に使用）
    pub parameters: Vec<(RegisterId, TypeId)>,
    /// 前のブロック（制御フローグラフの構築に使用）
    pub predecessors: Vec<BlockId>,
}

impl BasicBlock {
    /// 新しい基本ブロックを作成
    pub fn new(id: BlockId) -> Self {
        Self {
            id,
            instructions: Vec::new(),
            terminator: None,
            parameters: Vec::new(),
            predecessors: Vec::new(),
        }
    }
    
    /// 終了命令を設定
    pub fn set_terminator(&mut self, terminator: Terminator) {
        self.terminator = Some(terminator);
    }
    
    /// ブロックパラメータを追加
    pub fn add_parameter(&mut self, register: RegisterId, type_id: TypeId) {
        self.parameters.push((register, type_id));
    }
    
    /// 前のブロックを追加
    pub fn add_predecessor(&mut self, block_id: BlockId) {
        if !self.predecessors.contains(&block_id) {
            self.predecessors.push(block_id);
        }
    }
}

/// ブロック終了命令
#[derive(Debug, Clone)]
pub enum Terminator {
    /// 無条件分岐
    Branch {
        target: BlockId,
        /// ターゲットブロックのパラメータに渡す引数
        args: Vec<Operand>,
    },
    /// 条件分岐
    BranchCond {
        condition: Operand,
        true_target: BlockId,
        true_args: Vec<Operand>,
        false_target: BlockId,
        false_args: Vec<Operand>,
    },
    /// 関数から戻る
    Return {
        value: Option<Operand>,
    },
    /// 無条件ジャンプテーブル（switch文など）
    Switch {
        value: Operand,
        default_target: BlockId,
        default_args: Vec<Operand>,
        cases: Vec<(Literal, BlockId, Vec<Operand>)>,
    },
    /// 関数間接呼び出し
    IndirectCall {
        function_ptr: Operand,
        arguments: Vec<Operand>,
        return_block: BlockId,
        return_args: Vec<Operand>,
    },
    /// アンリーチャブル（到達不能）
    Unreachable,
}

/// EIR 命令
#[derive(Debug, Clone)]
pub enum Instruction {
    /// 二項演算
    BinaryOp {
        op: BinaryOp,
        lhs: Operand,
        rhs: Operand,
        result: RegisterId,
    },
    /// 単項演算
    UnaryOp {
        op: UnaryOp,
        operand: Operand,
        result: RegisterId,
    },
    /// メモリロード
    Load {
        address: Operand,
        result: RegisterId,
    },
    /// メモリストア
    Store {
        address: Operand,
        value: Operand,
    },
    /// 関数呼び出し
    Call {
        function: String,
        arguments: Vec<Operand>,
        result: Option<RegisterId>,
    },
    /// 関数から戻る
    Return {
        value: Option<Operand>,
    },
    /// 無条件分岐
    Branch {
        target: BlockId,
    },
    /// 条件分岐
    BranchCond {
        condition: Operand,
        true_target: BlockId,
        false_target: BlockId,
    },
    /// スタックアロケーション
    Alloca {
        size: usize,
        result: RegisterId,
    },
    /// 構造体メンバーアクセス
    GetElementPtr {
        base: Operand,
        indices: Vec<Operand>,
        result: RegisterId,
    },
    /// キャスト
    Cast {
        value: Operand,
        target_type: TypeId,
        result: RegisterId,
    },
    /// PHIノード
    Phi {
        incoming: Vec<(Operand, BlockId)>,
        result: RegisterId,
    },
    /// 値を選択（select命令）
    Select {
        condition: Operand,
        true_value: Operand,
        false_value: Operand,
        result: RegisterId,
    },
    /// アトミック操作
    Atomic {
        op: AtomicOp,
        address: Operand,
        value: Option<Operand>,
        result: Option<RegisterId>,
    },
    /// 外部関数呼び出し
    ExternalCall {
        function: String,
        arguments: Vec<Operand>,
        result: Option<RegisterId>,
    },
    /// インライン・アセンブリ
    InlineAsm {
        asm: String,
        constraints: String,
        args: Vec<Operand>,
        result: Option<RegisterId>,
    },
    /// デバッグ情報
    DebugInfo {
        info: String,
    },
}

impl Instruction {
    /// この命令が定義するレジスタを取得
    pub fn defined_register(&self) -> Option<RegisterId> {
        match self {
            Self::BinaryOp { result, .. } => Some(*result),
            Self::UnaryOp { result, .. } => Some(*result),
            Self::Load { result, .. } => Some(*result),
            Self::Call { result, .. } => *result,
            Self::Alloca { result, .. } => Some(*result),
            Self::GetElementPtr { result, .. } => Some(*result),
            Self::Cast { result, .. } => Some(*result),
            Self::Phi { result, .. } => Some(*result),
            Self::Select { result, .. } => Some(*result),
            Self::Atomic { result, .. } => *result,
            Self::ExternalCall { result, .. } => *result,
            Self::InlineAsm { result, .. } => *result,
            _ => None,
        }
    }
    
    /// この命令が使用するレジスタを取得
    pub fn used_registers(&self) -> Vec<RegisterId> {
        let mut registers = Vec::new();
        
        // オペランドからレジスタを抽出する補助関数
        let extract_registers = |op: &Operand, regs: &mut Vec<RegisterId>| {
            if let Operand::Register(reg) = op {
                regs.push(*reg);
            }
        };
        
        match self {
            Self::BinaryOp { lhs, rhs, .. } => {
                extract_registers(lhs, &mut registers);
                extract_registers(rhs, &mut registers);
            },
            Self::UnaryOp { operand, .. } => {
                extract_registers(operand, &mut registers);
            },
            Self::Load { address, .. } => {
                extract_registers(address, &mut registers);
            },
            Self::Store { address, value, .. } => {
                extract_registers(address, &mut registers);
                extract_registers(value, &mut registers);
            },
            Self::Call { arguments, .. } => {
                for arg in arguments {
                    extract_registers(arg, &mut registers);
                }
            },
            Self::Return { value } => {
                if let Some(val) = value {
                    extract_registers(val, &mut registers);
                }
            },
            Self::BranchCond { condition, .. } => {
                extract_registers(condition, &mut registers);
            },
            Self::GetElementPtr { base, indices, .. } => {
                extract_registers(base, &mut registers);
                for idx in indices {
                    extract_registers(idx, &mut registers);
                }
            },
            Self::Cast { value, .. } => {
                extract_registers(value, &mut registers);
            },
            Self::Phi { incoming, .. } => {
                for (val, _) in incoming {
                    extract_registers(val, &mut registers);
                }
            },
            Self::Select { condition, true_value, false_value, .. } => {
                extract_registers(condition, &mut registers);
                extract_registers(true_value, &mut registers);
                extract_registers(false_value, &mut registers);
            },
            Self::Atomic { address, value, .. } => {
                extract_registers(address, &mut registers);
                if let Some(val) = value {
                    extract_registers(val, &mut registers);
                }
            },
            Self::ExternalCall { arguments, .. } => {
                for arg in arguments {
                    extract_registers(arg, &mut registers);
                }
            },
            Self::InlineAsm { args, .. } => {
                for arg in args {
                    extract_registers(arg, &mut registers);
                }
            },
            _ => {}
        }
        
        registers
    }
}

/// 命令オペランド
#[derive(Debug, Clone)]
pub enum Operand {
    /// レジスタ
    Register(RegisterId),
    /// リテラル値
    Literal(Literal),
    /// グローバル変数
    Global(String),
    /// 関数参照
    Function(FunctionId),
    /// 外部関数参照
    ExternalFunction(String),
    /// シンボル参照
    Symbol(SymbolId),
    /// ブロック参照
    Block(BlockId),
}

/// リテラル値
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// 整数値
    Int(i64),
    /// 浮動小数点数
    Float(f64),
    /// 真偽値
    Bool(bool),
    /// 文字
    Char(u32),
    /// 文字列
    String(String),
    /// unit値（空）
    Unit,
}

/// 二項演算子
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    /// 加算
    Add,
    /// 減算
    Sub,
    /// 乗算
    Mul,
    /// 除算
    Div,
    /// 剰余
    Rem,
    /// ビット論理積
    BitAnd,
    /// ビット論理和
    BitOr,
    /// ビット排他的論理和
    BitXor,
    /// 左シフト
    Shl,
    /// 右シフト
    Shr,
    /// 等価比較
    Eq,
    /// 非等価比較
    Ne,
    /// 小なり比較
    Lt,
    /// 以下比較
    Le,
    /// 大なり比較
    Gt,
    /// 以上比較
    Ge,
    /// 論理積
    And,
    /// 論理和
    Or,
}

/// 単項演算子
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    /// 符号反転
    Neg,
    /// 論理否定
    Not,
    /// ビット否定
    BitNot,
    /// 型キャスト
    Cast,
}

/// アトミック操作の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomicOp {
    /// 読み込み
    Load,
    /// 書き込み
    Store,
    /// Compare-and-Swap
    CAS,
    /// アトミック加算
    Add,
    /// アトミック減算
    Sub,
    /// アトミック論理積
    And,
    /// アトミック論理和
    Or,
    /// アトミック排他的論理和
    Xor,
}

/// グローバル変数
#[derive(Debug, Clone)]
pub struct Global {
    /// 名前
    pub name: String,
    /// 型
    pub ty: TypeId,
    /// 初期化子（存在する場合）
    pub initializer: Option<Literal>,
    /// リンケージの種類
    pub linkage: Linkage,
    /// アラインメント（必要な場合）
    pub alignment: Option<usize>,
    /// グローバル変数の属性
    pub attributes: GlobalAttributes,
}

/// リンケージの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Linkage {
    /// デフォルト
    Default,
    /// 外部リンケージ
    External,
    /// 内部リンケージ
    Internal,
    /// 弱いリンケージ
    Weak,
    /// プライベート
    Private,
}

/// グローバル変数の属性
#[derive(Debug, Clone, Default)]
pub struct GlobalAttributes {
    /// 定数かどうか
    pub is_constant: bool,
    /// スレッドローカルかどうか
    pub is_thread_local: bool,
    /// アドレスを取得可能かどうか
    pub is_addressable: bool,
    /// 外部から可視かどうか
    pub is_externally_visible: bool,
}

/// 関数属性
#[derive(Debug, Clone, Default)]
pub struct FunctionAttributes {
    /// インライン化の指示
    pub inline: InlineDirective,
    /// ノーリターン関数かどうか
    pub noreturn: bool,
    /// 純粋関数かどうか
    pub pure: bool,
    /// 副作用なしかどうか
    pub no_side_effects: bool,
    /// この関数の属性タグ
    pub tags: HashSet<String>,
}

/// インライン化の指示
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InlineDirective {
    /// デフォルト
    Default,
    /// インライン化を強制
    Always,
    /// インライン化を禁止
    Never,
    /// インライン化のヒント
    Hint,
}

impl Default for InlineDirective {
    fn default() -> Self {
        Self::Default
    }
}

/// 外部関数定義
#[derive(Debug, Clone)]
pub struct ExternalFunction {
    /// 関数名
    pub name: String,
    /// 関数の型
    pub function_type: TypeId,
    /// パラメータの型
    pub parameter_types: Vec<TypeId>,
    /// 戻り値の型
    pub return_type: TypeId,
    /// 呼び出し規約
    pub calling_convention: CallingConvention,
    /// 可変長引数かどうか
    pub is_variadic: bool,
}

/// 呼び出し規約
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallingConvention {
    /// デフォルト
    Default,
    /// C呼び出し規約
    C,
    /// Fastcall
    FastCall,
    /// System V ABI
    SystemV,
    /// Microsoft x64呼び出し規約
    Win64,
}

impl Default for CallingConvention {
    fn default() -> Self {
        Self::Default
    }
}

/// 関数の使用グラフ
#[derive(Debug, Clone, Default)]
pub struct FunctionUseGraph {
    /// レジスタの定義場所（レジスタ -> (ブロックID, 命令ID)）
    pub register_defs: HashMap<RegisterId, (BlockId, InstructionId)>,
    /// レジスタの使用場所（レジスタ -> [(ブロックID, 命令ID)]）
    pub register_uses: HashMap<RegisterId, Vec<(BlockId, InstructionId)>>,
}

impl FunctionUseGraph {
    /// 新しい使用グラフを作成
    pub fn new() -> Self {
        Self {
            register_defs: HashMap::new(),
            register_uses: HashMap::new(),
        }
    }
    
    /// レジスタの使用数を取得
    pub fn get_use_count(&self, reg: RegisterId) -> usize {
        self.register_uses.get(&reg).map_or(0, |uses| uses.len())
    }
    
    /// レジスタの定義場所を取得
    pub fn get_def_location(&self, reg: RegisterId) -> Option<(BlockId, InstructionId)> {
        self.register_defs.get(&reg).copied()
    }
    
    /// レジスタの使用場所を取得
    pub fn get_use_locations(&self, reg: RegisterId) -> Vec<(BlockId, InstructionId)> {
        self.register_uses.get(&reg).cloned().unwrap_or_default()
    }
}