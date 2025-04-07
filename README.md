# Eidos - 言語を作る言語

<div align="center">
  <strong>世界最高水準の言語設計DSL</strong>
</div>

<div align="center">
  高度に最適化された汎用言語やDSLを簡潔に記述できる言語システム
</div>

<br />

<div align="center">
  <a href="https://github.com/eidos-lang/eidos/actions">
    <img src="https://github.com/eidos-lang/eidos/workflows/CI/badge.svg" alt="CI Status" />
  </a>
  <a href="https://github.com/eidos-lang/eidos/releases">
    <img src="https://img.shields.io/github/v/release/eidos-lang/eidos?include_prereleases" alt="Latest Release" />
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/github/license/eidos-lang/eidos" alt="License" />
  </a>
</div>

<br />

## 📖 概要

**Eidos**は、世界最高水準の言語設計DSL（Domain-Specific Language）であり、以下の目的で設計されています：

- 高度に最適化された汎用言語やOS構築用の特化DSLを簡潔に記述できる
- Rust等の安全性を持ちながら、記述量・抽象性・柔軟性を飛躍的に向上
- LLVM等のコード生成基盤に自然に統合可能
- 複数の目的別言語（派生DSL）をEidosを用いて簡単に作成・最適化可能

## 🚀 特徴

- **拡張可能な構文**: 言語の構文をプロジェクトごとに拡張可能
- **型安全**: 強力な型推論と型チェックを提供
- **所有権システム**: メモリ安全性を保証する所有権モデル
- **DSL作成支援**: 新しいドメイン特化言語を簡単に作成可能
- **マルチターゲット**: LLVM IR、WebAssembly、Cなど複数のバックエンド対応

## 📋 使い方

### インストール

```bash
# ソースからビルド
git clone https://github.com/eidos-lang/eidos.git
cd eidos
cargo build --release

# バイナリをインストール
cargo install --path .
```

### 基本的なコマンド

```bash
# ファイルをコンパイル
eid build hello.eid

# コンパイルして実行
eid run hello.eid

# 型チェックのみ実行
eid check hello.eid

# REPLを起動
eid repl
```

### シンプルなプログラム例

```eidos
// hello.eid
fn main(): Int {
    println("Hello, World!");
    return 0;
}
```

### DSL定義例

```eidos
// math_dsl.eid
syntax arithmetic {
  rule expr = term "+" term;
  rule term = number;
}

semantics expr => |a, b| => a + b;

fn main(): Int {
    let result = 2 + 3;
    println("結果: {}", result);
    return 0;
}
```

## 🛠️ 開発

### 必要なもの

- Rust 1.60以上
- LLVM 16.0
- Cargo

### テスト実行

```bash
# すべてのテストを実行
cargo test

# 特定のテストを実行
cargo test lexer

# 統合テストのみ実行
cargo test --test integration
```

### ドキュメントの生成

```bash
cargo doc --open
```

## 📚 ドキュメント

詳細なドキュメントは以下でご覧いただけます：

- [言語仕様書](docs/spec/language-spec.md)
- [DSL作成ガイド](docs/tutorials/dsl-guide.md)
- [型システム解説](docs/spec/type-system.md)
- [コマンドラインツール](docs/tools/cli.md)

## 🤝 貢献

貢献を歓迎します！以下の方法で貢献できます：

1. このリポジトリをフォーク
2. 機能ブランチを作成: `git checkout -b my-new-feature`
3. 変更をコミット: `git commit -am 'Add some feature'`
4. ブランチをプッシュ: `git push origin my-new-feature`
5. プルリクエストを提出

## 📜 ライセンス

このプロジェクトはMITライセンスの下で公開されています - 詳細は[LICENSE](LICENSE)ファイルをご覧ください。 