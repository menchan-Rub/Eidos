# Eidosのメタプログラミング機能

## はじめに

Eidosはメタプログラミングを第一級の機能として提供する「言語を作る言語」です。この文書では、Eidosのメタプログラミング機能について詳しく解説します。

## メタプログラミングとは

メタプログラミングとは、プログラムがプログラム自体を生成、分析、変更する能力のことです。通常のプログラミングではデータを処理しますが、メタプログラミングではプログラムコード自体を扱います。

Eidosでは以下のメタプログラミングパラダイムをサポートしています：

1. **構文拡張** - 新しい言語構文を定義し、既存の言語を拡張する
2. **セマンティクス定義** - 構文に意味を与える
3. **マクロシステム** - コンパイル時にコード変換を行う
4. **リフレクション** - 実行時にプログラムの構造を調査・変更する

## 構文拡張（syntax）

`syntax`ブロックを使用して新しい構文ルールを定義できます。

```eidos
syntax sql {
    rule select_stmt = "SELECT" columns "FROM" table_name ("WHERE" condition)?;
    rule columns = "*" | column_list;
    rule column_list = identifier ("," identifier)*;
    rule table_name = identifier;
    rule condition = expr;
    rule expr = term (operator term)*;
    rule term = identifier | string | number;
    rule operator = "=" | "!=" | "<" | ">" | "<=" | ">=";
}
```

これにより、Eidosコード内でSQL風の構文を使用できるようになります：

```eidos
fn get_user_data(min_age: Int): Result<Vec<User>, DbError> {
    let result = SELECT name, email, age FROM users WHERE age >= min_age;
    // ...
}
```

## セマンティクス定義（semantics）

構文に意味を与えるには、`semantics`ブロックを使用します。

```eidos
semantics sql::select_stmt(select_token, columns, from_token, table, where_clause) => {
    let query = SqlQuery::new(table.text);
    
    match columns {
        ColumnSpec::All => query.select_all(),
        ColumnSpec::List(cols) => {
            for col in cols {
                query.add_column(col.text);
            }
        }
    }
    
    if let Some((_, condition)) = where_clause {
        query.set_condition(condition);
    }
    
    query.execute()
}
```

## 型システム拡張（typesystem）

DSLに型検査ルールを追加できます：

```eidos
typesystem sql::select_stmt(_, columns, _, table_name, where_clause) -> QueryResult<T> {
    // テーブルの型情報を取得
    let table_type = get_table_type(table_name);
    
    // 列の型を検証
    validate_columns(columns, table_type);
    
    // WHERE句の型を検証
    if let Some((_, condition)) = where_clause {
        validate_condition(condition, table_type);
    }
    
    // 結果の型を構築して返す
    build_result_type(columns, table_type)
}
```

## マクロシステム

Eidosにはコンパイル時のコード変換を行うためのマクロシステムがあります：

```eidos
// 関数呼び出しをログ出力するマクロ
macro logged_call(func_name, args...) => {
    let start_time = std::time::now();
    let result = $func_name($args...);
    let end_time = std::time::now();
    println!("関数 {} の実行時間: {}ms", stringify!($func_name), (end_time - start_time).milliseconds());
    result
}

// 使用例
fn process_data(items: Vec<Item>): Result<Summary, Error> {
    // ...
}

fn main(): Int {
    let data = get_items();
    
    // マクロを使用
    let result = logged_call!(process_data, data);
    
    // 展開後のコード：
    // let start_time = std::time::now();
    // let result = process_data(data);
    // let end_time = std::time::now();
    // println!("関数 process_data の実行時間: {}ms", (end_time - start_time).milliseconds());
    
    return 0;
}
```

## リフレクション

Eidosは実行時にプログラムの構造にアクセスするためのリフレクションAPIを提供します：

```eidos
fn print_struct_info<T>(): Unit {
    // 型情報にアクセス
    let type_info = reflect::get_type_info<T>();
    
    println!("構造体名: {}", type_info.name);
    println!("フィールド数: {}", type_info.fields.len());
    
    for field in type_info.fields {
        println!("  - {}: {}", field.name, field.type_name);
    }
}

// リフレクションを使用してJSON化
fn to_json<T>(value: T): String {
    let type_info = reflect::get_type_info<T>();
    let mut json = "{".to_string();
    
    for (i, field) in type_info.fields.iter().enumerate() {
        if i > 0 {
            json.push_str(", ");
        }
        
        let field_value = reflect::get_field_value(value, field.name);
        json.push_str(&format!("\"{}\": {}", field.name, field_value.to_json()));
    }
    
    json.push_str("}");
    json
}
```

## コンパイル時計算

Eidosではコンパイル時に計算を行うことができます：

```eidos
// コンパイル時定数
const PI: Float = 3.14159265358979323846;

// コンパイル時関数
consteval fn factorial(n: Int): Int {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

// 使用例
fn main(): Int {
    // コンパイル時に計算される
    const fact5: Int = factorial(5);
    
    println!("5! = {}", fact5); // "5! = 120"が出力される
    return 0;
}
```

## コンパイラプラグイン

Eidosはコンパイラプラグイン機能を提供し、コンパイルプロセスの各段階をカスタマイズできます：

```eidos
#[compiler_plugin]
pub struct CodeOptimizer {
    // ...
}

impl CompilerPass for CodeOptimizer {
    fn process_ast(&mut self, ast: &mut AST): Result<(), CompileError> {
        // ASTの最適化を実行
        optimize_expressions(ast);
        optimize_control_flow(ast);
        
        Ok(())
    }
}

// プラグインの登録
#[register_plugin]
fn register_plugins(registry: &mut PluginRegistry) {
    registry.register::<CodeOptimizer>(Phase::AfterTypeCheck);
}
```

## 言語内DSL創造パターン

Eidosで効果的なDSLを実装するためのパターン：

### 1. 段階的設計パターン

まず基本的な構文を定義し、その上に高レベルの抽象化を構築します：

```eidos
// 基本的なクエリDSL
syntax query {
    rule select = /* ... */;
    rule filter = /* ... */;
}

// 高レベルな抽象化
syntax orm {
    rule find_by = "find" entity_name "by" conditions;
    // ...
}

semantics orm::find_by(_, entity, _, conditions) => {
    // orm::find_by を query::select + query::filter に変換
    // ...
}
```

### 2. コンテキストアウェアパターン

DSLが使用されるコンテキストに応じて挙動を変えます：

```eidos
// 型に応じて挙動が変わるクエリDSL
with_context<User> {
    let user = find by id = 1;
    // User型に対して適切なクエリが生成される
}

with_context<Product> {
    let product = find by sku = "ABC123";
    // Product型に対して適切なクエリが生成される
}
```

## 高度なメタプログラミング例

### 状態機械DSL

```eidos
syntax statemachine {
    rule machine = "machine" identifier "{" state+ "}";
    rule state = "state" identifier "{" transition* "}";
    rule transition = "on" event_name "=>" target_state ("if" condition)? ";";
    rule event_name = identifier;
    rule target_state = identifier;
    rule condition = expr;
}

// 使用例
machine TrafficLight {
    state Red {
        on timeout => Green;
    }
    
    state Yellow {
        on timeout => Red;
    }
    
    state Green {
        on timeout => Yellow;
    }
}
```

### ネットワークプロトコルDSL

```eidos
syntax protocol {
    rule message = "message" identifier "{" field* "}";
    rule field = field_type identifier "=" number ";" comment?;
    rule field_type = "required" | "optional" | "repeated";
    rule comment = "//" text;
}

// 使用例
message Person {
    required string name = 1;
    required int32 id = 2;
    optional string email = 3;
}
```

## まとめ

Eidosのメタプログラミング機能を使用することで、特定のドメインに特化した言語を作成し、コードの可読性、保守性、表現力を向上させることができます。DSLを適切に設計することで、プログラマは問題領域の概念を直接扱うことができ、生産性が大幅に向上します。

Eidosの「言語を作る言語」というコンセプトは、プログラミング言語の未来への道を開きます。既存の言語を拡張したり、新しい言語を作成したりする能力により、さまざまな問題領域に対して最適な表現方法を提供することができます。 