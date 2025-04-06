# Eidos - 言語を作る言語

Eidosは「言語を作る言語」として設計された、高度なドメイン固有言語（DSL）構築システムです。

## 概要

Eidosは以下の特徴を持っています：

- **強力なDSL構築機能**: 独自の言語拡張を簡単に定義できます
- **型安全**: 強力な型システムによりコンパイル時にエラーを検出します
- **性能**: LLVMベースのバックエンドにより高速な実行を実現します
- **安全性**: 所有権に基づいたメモリ管理システム

## ディレクトリ構成

```
/src
  /frontend         # 字句解析・構文解析
  /core             # 型システム・AST・EIR
  /dsl              # DSL拡張機構
  /backend          # LLVM/WASM/IR出力
  /stdlib           # 標準ライブラリ
  /tools            # CLI, LSP, ビルドシステム
/tests              # 単体・統合テスト
/docs               # ユーザー＆開発ドキュメント
/examples           # 言語の使用例・DSL例
```

## インストール

```bash
git clone https://github.com/menchan-Rub/Eidos.git
cd eidos
cargo build --release
```

## 使い方

基本的な使い方：

```bash
# Eidosプログラムをコンパイル
eidos build path/to/program.eid

# インタラクティブモード
eidos repl
```

## ライセンス

MITライセンスの下で公開されています。 