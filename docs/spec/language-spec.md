# Eidos 言語仕様書

## 1. はじめに

Eidosは「言語を作る言語」として設計された、高度なメタプログラミング言語です。
この文書はEidos言語の正式な仕様を定義します。

## 2. 字句構造

### 2.1 ファイル構造

Eidosプログラムは`.eid`拡張子を持つUTF-8エンコードのテキストファイルです。

### 2.2 コメント

```eidos
// 1行コメント

/* 
  複数行コメント
  ネストも可能
*/
```

### 2.3 識別子

識別子は英字またはアンダースコア(`_`)で始まり、その後に英数字またはアンダースコアが続きます。

```eidos
valid_identifier
_also_valid
Invalid123 // 大文字で始まる識別子は型名として使われます
```

### 2.4 キーワード

以下は予約語であり、識別子として使用できません：

```
as break const continue else enum extern false fn for 
if impl import let match mod move mut pub return 
self static struct syntax true type use where while
```

### 2.5 リテラル

#### 整数リテラル

```eidos
42        // 10進数
0xFF      // 16進数
0o77      // 8進数
0b1010    // 2進数
```

#### 浮動小数点リテラル

```eidos
3.14159
1.0e10
```

#### 文字列リテラル

```eidos
"Hello, world!"
"複数行文字列も
サポートされています"
r"Raw文字列（エスケープされない）"
```

#### ブール値リテラル

```eidos
true
false
```

## 3. 基本的な型

### 3.1 プリミティブ型

- `Int`: 整数型
- `Float`: 浮動小数点型
- `Bool`: 真偽値型
- `String`: 文字列型
- `Char`: 文字型
- `Unit`: 空の型（`()`で表される）

### 3.2 複合型

- `Array<T>`: 配列型
- `Tuple<T1, T2, ...>`: タプル型
- `Option<T>`: オプション型
- `Result<T, E>`: 結果型
- `Function<Args..., Ret>`: 関数型

## 4. 変数宣言

```eidos
let x = 42;                // 型推論
let y: Int = 42;           // 明示的な型
let mut z = 10;            // ミュータブル変数
```

## 5. 関数

### 5.1 関数宣言

```eidos
fn add(a: Int, b: Int): Int {
    return a + b;
}
```

### 5.2 ラムダ式

```eidos
let multiply = |a, b| a * b;
let typed_add = |a: Int, b: Int| -> Int { a + b };
```

## 6. 制御構造

### 6.1 条件分岐

```eidos
if condition {
    // then branch
} else if other_condition {
    // else if branch
} else {
    // else branch
}
```

### 6.2 ループ

```eidos
// whileループ
while condition {
    // body
}

// forループ
for item in collection {
    // body
}
```

### 6.3 パターンマッチング

```eidos
match value {
    Pattern1 => expression1,
    Pattern2 => {
        // 複数の文を持つブロック
        expression2
    },
    _ => default_expression, // ワイルドカードパターン
}
```

## 7. 構造体と列挙型

### 7.1 構造体

```eidos
struct Point {
    x: Int,
    y: Int,
}

// インスタンス化
let p = Point { x: 10, y: 20 };
```

### 7.2 列挙型

```eidos
enum Result<T, E> {
    Ok(T),
    Err(E),
}

// インスタンス化
let success = Result::Ok(42);
let failure = Result::Err("エラーが発生しました");
```

## 8. DSL拡張機能

### 8.1 構文拡張

```eidos
syntax arithmetic {
    rule expr = term ("+" term | "-" term)*;
    rule term = factor ("*" factor | "/" factor)*;
    rule factor = number | "(" expr ")";
}
```

### 8.2 セマンティクス定義

```eidos
semantics arithmetic::expr(left, op, right) => {
    match op {
        "+" => left + right,
        "-" => left - right,
        _ => panic!("Unsupported operator"),
    }
}
```

## 9. モジュールシステム

### 9.1 モジュール宣言

```eidos
mod graphics {
    pub fn draw_line(x1: Int, y1: Int, x2: Int, y2: Int) {
        // 実装
    }
}
```

### 9.2 インポート

```eidos
use graphics::draw_line;
use std::collections::HashMap;
```

## 10. エラー処理

### 10.1 Result型

```eidos
fn divide(a: Int, b: Int): Result<Int, String> {
    if b == 0 {
        return Err("Division by zero");
    }
    return Ok(a / b);
}
```

### 10.2 例外処理

```eidos
try {
    // エラーが発生する可能性のあるコード
} catch e: Error {
    // エラー処理
} finally {
    // クリーンアップコード
}
```

## 11. ジェネリクス

```eidos
fn identity<T>(value: T): T {
    return value;
}

struct Container<T> {
    value: T,
}
```

## 12. トレイト（インターフェース）

```eidos
trait Drawable {
    fn draw(self): Unit;
    fn bounding_box(self): Rectangle;
}

impl Drawable for Circle {
    fn draw(self): Unit {
        // 円を描画
    }
    
    fn bounding_box(self): Rectangle {
        // バウンディングボックスを計算
    }
}
```

## 付録A: 文法仕様

<文法の詳細なEBNF表記が入ります>

## 付録B: 標準ライブラリ

<標準ライブラリの詳細な説明が入ります> 