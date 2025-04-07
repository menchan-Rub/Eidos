# Eidos コマンドラインツール

Eidosは強力なコマンドラインインターフェイス（CLI）を提供しており、Eidosプログラムのコンパイル、実行、デバッグをサポートします。

## インストール

Eidosのコマンドラインツールをインストールするには：

```bash
# ソースからビルド
git clone https://github.com/eidos-lang/eidos.git
cd eidos
cargo build --release

# バイナリをインストール
cargo install --path .
```

インストールが完了すると、`eid`コマンドが使用できるようになります。

## 基本的なコマンド

### コンパイル: `eid build`

Eidosプログラムをコンパイルします：

```bash
eid build [オプション] <ファイル>
```

#### オプション:

- `-o, --output <ファイル>`: 出力ファイルを指定
- `--opt-level <0-3>`: 最適化レベルを設定（デフォルト: 2）
- `--debug`: デバッグ情報を含める
- `--target <ターゲット>`: コンパイルターゲットを指定（native, llvm, wasm, c）
- `--verbose`: 詳細な出力を表示

#### 例:

```bash
# 基本的なコンパイル
eid build src/main.eid

# 出力ファイルを指定
eid build src/main.eid -o bin/program

# 最適化レベルを設定
eid build src/main.eid --opt-level 3

# WebAssemblyにコンパイル
eid build src/main.eid --target wasm
```

### 実行: `eid run`

Eidosプログラムをコンパイルして実行します：

```bash
eid run [オプション] <ファイル> [引数...]
```

#### オプション:

- `--opt-level <0-3>`: 最適化レベルを設定（デフォルト: 2）
- `--debug`: デバッグ情報を含める
- `--verbose`: 詳細な出力を表示

#### 例:

```bash
# 基本的な実行
eid run src/main.eid

# 引数を渡す
eid run src/main.eid arg1 arg2

# デバッグ情報付きで実行
eid run --debug src/main.eid
```

### 型チェック: `eid check`

Eidosプログラムの型チェックのみを行います：

```bash
eid check <ファイル>
```

#### 例:

```bash
eid check src/main.eid
```

### REPL: `eid repl`

対話型コンソール（REPL）を起動します：

```bash
eid repl [オプション]
```

#### オプション:

- `--preload <ファイル...>`: REPLの起動時にロードするファイルを指定

#### 例:

```bash
# 基本的なREPL起動
eid repl

# ファイルをプリロード
eid repl --preload lib/utils.eid lib/math.eid
```

### ドキュメント生成: `eid doc`

ソースコードからドキュメントを生成します：

```bash
eid doc [オプション] <ファイル/ディレクトリ...>
```

#### オプション:

- `--output <ディレクトリ>`: 出力先ディレクトリを指定
- `--private`: 非公開アイテムも含める
- `--open`: 生成後にブラウザで開く

#### 例:

```bash
# プロジェクト全体のドキュメントを生成
eid doc src/

# 特定ファイルのドキュメントを生成
eid doc src/lib.eid
```

### フォーマット: `eid fmt`

Eidosコードを自動的にフォーマットします：

```bash
eid fmt [オプション] <ファイル/ディレクトリ...>
```

#### オプション:

- `--check`: フォーマットのみチェックし、変更は行わない

#### 例:

```bash
# ファイルをフォーマット
eid fmt src/main.eid

# ディレクトリ内のすべてのファイルをフォーマット
eid fmt src/
```

## 高度な機能

### プロファイリング: `eid profile`

プログラムのパフォーマンスを分析します：

```bash
eid profile [オプション] <ファイル> [引数...]
```

#### オプション:

- `--output <ファイル>`: プロファイリング結果の出力先
- `--format <format>`: 出力フォーマット（text, json, flamegraph）

#### 例:

```bash
# 基本的なプロファイリング
eid profile src/main.eid

# フレームグラフ形式で出力
eid profile --format flamegraph src/main.eid
```

### バイナリ解析: `eid analyze`

コンパイルされたバイナリを解析します：

```bash
eid analyze [オプション] <ファイル>
```

#### 例:

```bash
eid analyze bin/program
```

### 言語サーバー: `eid language-server`

言語サーバープロトコルのサーバーを起動します（IDEやエディタの統合用）：

```bash
eid language-server
```

## 環境変数

Eidosコマンドラインツールの動作に影響を与える環境変数：

- `EIDOS_PATH`: Eidosライブラリを検索するディレクトリ
- `EIDOS_STDLIB`: 標準ライブラリのディレクトリ
- `EIDOS_LOG_LEVEL`: ログレベル（debug, info, warn, error）
- `EIDOS_HOME`: Eidosの設定とキャッシュを保存するディレクトリ

## 設定ファイル

Eidosはプロジェクトルートの`.eidos.toml`ファイルから設定を読み込みます：

```toml
# .eidos.toml の例
[build]
optimization = 2
debug = false
target = "native"

[format]
indent_size = 4
max_width = 100

[dependencies]
std = "0.1.0"
math = { path = "../math" }
```

## トラブルシューティング

### 一般的な問題の解決法

- **依存関係の問題**: `eid clean`コマンドでキャッシュをクリアしてみてください
- **コンパイルエラー**: 詳細なエラー情報を表示するには`--verbose`フラグを使用
- **パフォーマンス問題**: `--opt-level 3`で最適化レベルを上げてみてください

### ログの収集

```bash
EIDOS_LOG_LEVEL=debug eid build src/main.eid 2> log.txt
```

## まとめ

Eidosコマンドラインツールは、Eidosプログラミング言語を使用する上で重要な役割を果たします。
基本的なコンパイルや実行だけでなく、開発プロセス全体をサポートする機能が揃っています。 