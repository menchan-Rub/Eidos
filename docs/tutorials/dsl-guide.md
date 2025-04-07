# Eidos DSL作成ガイド

## はじめに

Eidosの最も強力な機能の一つは、ドメイン特化言語（Domain-Specific Language、DSL）を簡単に定義し、使用できる点です。
このガイドでは、Eidosを使って独自のDSLを作成する方法を解説します。

## DSLとは何か

ドメイン特化言語（DSL）は、特定の問題領域のための専用言語です。特定のタスクを簡潔に表現できるよう設計されており、汎用プログラミング言語よりも限定された機能を持ちますが、その領域では高い表現力を発揮します。

例えば：
- SQLはデータベースクエリのためのDSL
- HTMLはウェブページ構造記述のためのDSL
- RegExpは正規表現のためのDSL

## EidosにおけるシンプルなDSL例

まず、Eidosで数式を扱うためのシンプルなDSLの例を見てみましょう：

```eidos
// math_dsl.eid

// 構文定義
syntax math {
    rule expr = term ("+" term | "-" term)*;
    rule term = factor ("*" factor | "/" factor)*;
    rule factor = number | "(" expr ")";
}

// セマンティクス定義
semantics math::expr(left, op, right) => {
    match op {
        "+" => left + right,
        "-" => left - right,
        _ => panic!("不明な演算子")
    }
}

semantics math::term(left, op, right) => {
    match op {
        "*" => left * right,
        "/" => left / right,
        _ => panic!("不明な演算子")
    }
}

// 使用例
fn main(): Int {
    // DSLを使用した数式
    let result = 3 + 4 * (2 - 1);
    println("結果: {}", result);
    return 0;
}
```

## DSL作成の基本ステップ

DSLの作成は以下の手順に従います：

1. 構文（Syntax）の定義
2. セマンティクス（意味）の定義
3. 型チェックルールの定義（オプション）
4. DSLの使用とテスト

### 1. 構文の定義

`syntax`ブロックを使用して文法ルールを定義します：

```eidos
syntax <dsl名> {
    rule <ルール名> = <パターン>;
    // 追加のルール...
}
```

構文定義では以下の表記が使えます：

- シーケンス：`A B` (Aの後にB)
- 選択：`A | B` (AまたはB)
- 0回以上の繰り返し：`A*`
- 1回以上の繰り返し：`A+`
- オプション：`A?` (Aがあるかもしれないし、ないかもしれない)
- グループ化：`(A B)`
- リテラル：`"keyword"`

### 2. セマンティクスの定義

`semantics`キーワードを使用して構文ルールの意味を定義します：

```eidos
semantics <dsl名>::<ルール名>(<パラメータ>) => {
    // 意味解釈の実装
}
```

または、より単純な形式：

```eidos
semantics <dsl名>::<ルール名> => |<パラメータ>| <式>;
```

### 3. 型チェックルールの定義

型チェックルールを定義することで、DSLの文法が型安全であることを保証できます：

```eidos
typesystem <dsl名>::<ルール名>(<パラメータ>) -> <型> {
    // 型チェックロジック
}
```

## 実践的なDSL例：JSON構築DSL

以下はJSON構造を簡単に構築するためのDSLの例です：

```eidos
// json_dsl.eid

// 構文定義
syntax json {
    rule object = "{" (pair ("," pair)*)? "}";
    rule pair = string ":" value;
    rule array = "[" (value ("," value)*)? "]";
    rule value = string | number | object | array | "true" | "false" | "null";
}

// セマンティクス定義
semantics json::object(pairs) => {
    let obj = JsonObject::new();
    for pair in pairs {
        obj.insert(pair.key, pair.value);
    }
    obj
}

semantics json::array(values) => {
    let arr = JsonArray::new();
    for value in values {
        arr.push(value);
    }
    arr
}

// 使用例
fn main(): Unit {
    let config = {
        "name": "設定ファイル",
        "version": 1.0,
        "enabled": true,
        "features": ["編集", "保存", "共有"],
        "settings": {
            "theme": "dark",
            "fontSize": 12
        }
    };
    
    println("{}", config.to_string());
}
```

## DSLの高度な機能

### カスタム演算子の定義

```eidos
syntax math {
    // カスタム演算子 **（べき乗）を定義
    rule power = factor "**" factor;
}

semantics math::power(base, exp) => {
    base.pow(exp)
}
```

### マクロとDSLの併用

```eidos
macro logged_json(body) => {
    let start_time = std::time::now();
    let result = { body };
    let end_time = std::time::now();
    println("JSON生成時間: {}ms", (end_time - start_time).milliseconds());
    result
}

fn main(): Unit {
    let config = logged_json({
        "name": "設定ファイル",
        "settings": { "theme": "dark" }
    });
}
```

### DSLスコープの制限

```eidos
fn process_data(): Unit {
    // mathスコープ内でのみ数式DSLが有効
    with_scope math {
        let a = 1 + 2 * 3;
    }
    
    // ここでは通常のEidos構文が使われる
    let b = 1.add(2.multiply(3));
}
```

## ベストプラクティス

1. **DSLを簡潔に保つ**: 一般的なプログラミング機能はEidosに任せ、DSLは特定のドメインに焦点を当てましょう。

2. **文法の曖昧さを避ける**: 文法が曖昧だと解析が難しくなり、バグの原因になります。

3. **エラーメッセージを充実させる**: セマンティクス定義内でエラーチェックをし、わかりやすいエラーメッセージを返しましょう。

4. **ドキュメントとテストを充実させる**: DSLのすべての機能のドキュメントとテストを作成しましょう。

5. **段階的に開発する**: 小さく始めて、機能を段階的に追加していくことで、複雑さを管理しやすくなります。

## まとめ

EidosのDSL機能を使用すると、特定のドメインの問題を表現するための専用言語を簡単に作成できます。
これにより、より読みやすく、メンテナンスしやすいコードを書くことができます。

Eidosのポリシーは「言語を作る言語」であり、DSL機能はその中心的な役割を果たします。
ドメイン特化言語を適切に使用することで、複雑なプログラミングタスクをより単純に、より表現力豊かに解決できるようになります。 