# Eidos: 言語仕様書（完全版）

## 概要
**Eidos**は、世界最高水準の言語設計DSL（Domain-Specific Language）であり、以下の目的で設計されています：

- 高度に最適化された汎用言語やOS構築用の特化DSLを簡潔に記述できる。
- Rust等の安全性を持ちながら、記述量・抽象性・柔軟性を飛躍的に向上。
- LLVM等のコード生成基盤に自然に統合可能。
- 複数の目的別言語（派生DSL）をEidosを用いて簡単に作成・最適化可能。

本仕様書はEidosの全機能、文法、型システム、実行モデル、拡張性、実装戦略に関して網羅的に記述します。これは“言語を作るための言語”として、最高峰の開発生産性と安全性を保証します。

---

## 技術要件（超詳細）

### 言語構造：
- **言語レベル**: メタ言語（他言語を定義・構築可能）
- **想定用途**:
  - Rust/C/C++代替の高速低レベル言語
  - DSL生成器（OS構築DSL、UI定義DSL、分散並列DSLなど）
  - アプリケーションスクリプト
- **トランスパイル対象**: LLVM IR, WASM, C, Rust, JS, Zig, Assembly（予定）

### コンパイラ構成：
- **フロントエンド**:
  - 字句解析：Unicode準拠、マルチバイト対応、トークン定義のDSL化可能
  - 構文解析：PEGベース＋DSLによる構文変更
  - AST構造：構文・意味論を保持する多層構成ノード（スコープ／型情報同居）

- **中間表現（EIR: Eidos Intermediate Representation）**：
  - SSAベース、再構成可能（CFG、Hir、Mir）
  - 全てのスコープ・型注釈・属性情報を保持
  - EIR上での最適化エンジン（定数伝播、ループアンローリング、型剰余計算）

- **バックエンド**:
  - LLVM IR生成（最適化フラグ付き）
  - WASM（GC対応、WASI対応）
  - Cコード自動生成（移植性重視）
  - Rust/JS構文トランスレータ（FaaS/サーバレス用）

### 型システム：
- 静的型付け／型推論の両立（Hindley-Milner ＋ DEP型推論）
- ユーザー定義型システム：DSLごとに推論アルゴリズム／型演算子定義可能
- 型ファンクタ（型に関する関数）：コンパイル時型合成
- 型注釈（アトリビュート）例： `@noalias`, `@bounded`, `@threadsafe`
- 所有権システム内蔵： move/copy/borrow/ref/check を明示的に制御
- ライフタイム、型パラメータ境界、インターフェース継承対応

### 実行モデル：
- 静的解析＋コンパイル：JIT／AOT切替可能
- コンパイルパス：解析→展開→型推論→EIR生成→最適化→ターゲット生成
- 静的検証機能：null安全、境界チェック、未定義動作の排除、レース解析
- 並列実行モデル：所有権とスレッド境界の型的分離により安全にスレッド分離

### メモリモデル：
- 所有権＋借用（Rust風）＋アノテーションで最適化可能
- Arena／Stack／GCモデルをDSL単位で切替可能
- コンパイル時レイアウト決定、型サイズ・配置可視化可能

### DSL定義機能：
- 拡張可能構文定義エンジン：BNF／PEG混在、左再帰対応、構文圧縮式あり
- セマンティクス定義：文法→ASTノード→意味論→EIR への一貫接続
- DSLプロファイル：独立した構文・型・標準関数セットの切替機能
- ドメイン制約：金融DSL、安全組込DSLなど制限付きDSLの記述可能

### 標準ライブラリ：
- 高精度数値／多倍長整数／複素数／ベクトル／線形代数／BLAS相当機能
- OS抽象：非同期IO、タスク制御、プロセス制御、ファイルシステム
- ネットワーク：TCP/UDP/WebSocket/WASM向けソケット抽象
- スレッド：コルーチン／軽量スレッド／パラレルスケジューラ搭載
- メタ構文支援：AST変形／コードテンプレート／マクロ展開器

### ビルド＆パッケージ：
- `eid build`, `eid run`, `eid test`, `eid lint`, `eid export` など多機能CLI
- DSLごとのビルド定義／依存解決／プロファイル切替／ワークスペース管理
- モジュール単位の型・構文キャッシュ／インクリメンタルビルド

### VSCode統合：
- LSPプロトコル完全準拠、DSLごとの構文色分け対応
- コード補完／引数ヒント／型定義ジャンプ／リファクタリング提案
- ASTビュー／EIRビュー／型推論経路可視化
- ステップデバッガ：AST / メモリ構造 / スコープ / スレッド追跡

---

## 言語文法仕様（詳細ドキュメント）

### 1. 基本文法：
```eidos
fn factorial(n: Int): Int {
  if n <= 1 {
    return 1;
  } else {
    return n * factorial(n - 1);
  }
}
```

### 2. DSL構文＆意味定義：
```eidos
syntax arithmetic {
  rule expr = term "+" term;
  rule term = number;
}

semantics expr => |a, b| => a + b;
```

### 3. 所有権・借用：
```eidos
let a = [1, 2, 3];
let b = move a; // a is now invalid
```

### 4. 型と関数の定義：
```eidos
type Result<T, E> = Ok(T) | Err(E);
fn safe_div(x: Int, y: Int): Result<Int, String> {
  if y == 0 {
    return Err("Divide by zero");
  }
  return Ok(x / y);
}
```

### 5. マクロと展開：
```eidos
macro trace(x) => print("[TRACE] ", x);
```

### 6. コンパイル時計算：
```eidos
const N = compute_fibonacci(20); // コンパイル時に評価
```

### 7. モジュールと名前空間：
```eidos
mod util::math;
use util::math::sum;
```

### 8. テスト仕様：
```eidos
test "fibonacci base case" {
  assert_eq(fibonacci(1), 1);
}
```

---

## 今後の拡張ロードマップ：
- 自己記述型DSL（EidosでEidosコンパイラを定義）
- 分散並列対応DSLのテンプレート（ExoLang, ClusterDef）
- 自動DSLジェネレータ（言語設計のGUI支援）
- 他言語とのASTレベル相互変換（Rust, TypeScript）
- EIR最適化のAI支援（強化学習による最適化順序選択）
- 言語進化ログトラッカー（構文・型・意味論のバージョン管理）

---

さらなるドキュメント：
- DSL記法一覧（全構文の表形式リファレンス）
- EIR命令セットリファレンス（各命令の意味・最適化対象）
- デバッグ構造ガイド（AST→EIR→LLVM IR のトレース付き）
- 実行時環境チューニング法（レイアウト最適化、並列プリミティブ）
- 「EidosでEidosを書く」チュートリアル（段階的に自己言語を構築）

