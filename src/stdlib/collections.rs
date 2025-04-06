use crate::core::{Result, EidosError};
use crate::core::types::{Type, TypeId, TypeKind, Field};
use crate::stdlib::{StdlibRegistry, StdlibFunction, StdlibModule, StdlibFunctionType};
use std::sync::{Mutex, atomic::{AtomicUsize, Ordering}};
use std::collections::HashMap;
use lazy_static::lazy_static;

/// コレクションモジュールの初期化
pub fn initialize(registry: &mut StdlibRegistry) -> Result<()> {
    // 基本型の登録
    let int_type = Type::int();
    let bool_type = Type::bool();
    let string_type = Type::string();
    let unit_type = Type::unit();
    
    // Vector（動的配列）型の定義
    let vector_elem_type = Type::generic("T");
    let vector_type = Type::new(
        TypeKind::Struct {
            name: "Vector".to_string(),
            fields: vec![
                Field {
                    name: "length".to_string(),
                    field_type: int_type.clone(),
                    is_public: false,
                },
                Field {
                    name: "capacity".to_string(),
                    field_type: int_type.clone(),
                    is_public: false,
                },
                Field {
                    name: "data".to_string(),
                    field_type: Type::array(vector_elem_type.clone()),
                    is_public: false,
                },
            ],
            methods: vec![],
            is_extern: false,
        },
    );
    registry.register_type("collections::Vector", vector_type.clone());
    
    // HashMap（ハッシュマップ）型の定義
    let hashmap_key_type = Type::generic("K");
    let hashmap_value_type = Type::generic("V");
    let hashmap_type = Type::new(
        TypeKind::Struct {
            name: "HashMap".to_string(),
            fields: vec![
                Field {
                    name: "size".to_string(),
                    field_type: int_type.clone(),
                    is_public: false,
                },
                Field {
                    name: "capacity".to_string(),
                    field_type: int_type.clone(),
                    is_public: false,
                },
                // 実際のハッシュマップの実装は内部的に行う
            ],
            methods: vec![],
            is_extern: false,
        },
    );
    registry.register_type("collections::HashMap", hashmap_type.clone());
    
    // HashSet（ハッシュセット）型の定義
    let hashset_elem_type = Type::generic("T");
    let hashset_type = Type::new(
        TypeKind::Struct {
            name: "HashSet".to_string(),
            fields: vec![
                Field {
                    name: "size".to_string(),
                    field_type: int_type.clone(),
                    is_public: false,
                },
                Field {
                    name: "capacity".to_string(),
                    field_type: int_type.clone(),
                    is_public: false,
                },
                // 実際のハッシュセットの実装は内部的に行う
            ],
            methods: vec![],
            is_extern: false,
        },
    );
    registry.register_type("collections::HashSet", hashset_type.clone());
    
    // LinkedList（連結リスト）型の定義
    let linkedlist_elem_type = Type::generic("T");
    let linkedlist_type = Type::new(
        TypeKind::Struct {
            name: "LinkedList".to_string(),
            fields: vec![
                Field {
                    name: "length".to_string(),
                    field_type: int_type.clone(),
                    is_public: false,
                },
                // ヘッドとテールのポインタは内部実装で管理
            ],
            methods: vec![],
            is_extern: false,
        },
    );
    registry.register_type("collections::LinkedList", linkedlist_type.clone());
    
    // Queue（キュー）型の定義
    let queue_elem_type = Type::generic("T");
    let queue_type = Type::new(
        TypeKind::Struct {
            name: "Queue".to_string(),
            fields: vec![
                Field {
                    name: "length".to_string(),
                    field_type: int_type.clone(),
                    is_public: false,
                },
                // 内部実装はLinkedListを使用
            ],
            methods: vec![],
            is_extern: false,
        },
    );
    registry.register_type("collections::Queue", queue_type.clone());
    
    // Stack（スタック）型の定義
    let stack_elem_type = Type::generic("T");
    let stack_type = Type::new(
        TypeKind::Struct {
            name: "Stack".to_string(),
            fields: vec![
                Field {
                    name: "length".to_string(),
                    field_type: int_type.clone(),
                    is_public: false,
                },
                // 内部実装はVectorを使用
            ],
            methods: vec![],
            is_extern: false,
        },
    );
    registry.register_type("collections::Stack", stack_type.clone());
    
    // PriorityQueue（優先度キュー）型の定義
    let priorityqueue_elem_type = Type::generic("T");
    let priorityqueue_type = Type::new(
        TypeKind::Struct {
            name: "PriorityQueue".to_string(),
            fields: vec![
                Field {
                    name: "length".to_string(),
                    field_type: int_type.clone(),
                    is_public: false,
                },
                // 内部実装はヒープを使用
            ],
            methods: vec![],
            is_extern: false,
        },
    );
    registry.register_type("collections::PriorityQueue", priorityqueue_type.clone());
    
    // Vector関数の登録
    
    // Vector::new - 新しいVectorを作成
    registry.register_function(StdlibFunction::new(
        "Vector::new",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![],
        vector_type.id,
        "新しい空のVector（動的配列）を作成します。",
    ));
    
    // Vector::with_capacity - 初期容量を指定してVectorを作成
    registry.register_function(StdlibFunction::new(
        "Vector::with_capacity",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("capacity".to_string(), int_type.id)],
        vector_type.id,
        "指定した初期容量でVector（動的配列）を作成します。",
    ));
    
    // Vector::push - Vectorに要素を追加
    registry.register_function(StdlibFunction::new(
        "Vector::push",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![
            ("vector".to_string(), vector_type.id),
            ("element".to_string(), vector_elem_type.id),
        ],
        unit_type.id,
        "Vectorの末尾に要素を追加します。",
    ));
    
    // Vector::pop - Vectorの末尾から要素を取り出す
    registry.register_function(StdlibFunction::new(
        "Vector::pop",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![("vector".to_string(), vector_type.id)],
        vector_elem_type.id,
        "Vectorの末尾から要素を取り出します。Vectorは空にできません。",
    ));
    
    // Vector::get - インデックスで要素を取得
    registry.register_function(StdlibFunction::new(
        "Vector::get",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![
            ("vector".to_string(), vector_type.id),
            ("index".to_string(), int_type.id),
        ],
        vector_elem_type.id,
        "指定したインデックスの要素を取得します。インデックスは0から始まります。",
    ));
    
    // Vector::set - インデックスで要素を設定
    registry.register_function(StdlibFunction::new(
        "Vector::set",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![
            ("vector".to_string(), vector_type.id),
            ("index".to_string(), int_type.id),
            ("element".to_string(), vector_elem_type.id),
        ],
        unit_type.id,
        "指定したインデックスに要素を設定します。インデックスは0から始まります。",
    ));
    
    // Vector::length - Vectorの長さを取得
    registry.register_function(StdlibFunction::new(
        "Vector::length",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("vector".to_string(), vector_type.id)],
        int_type.id,
        "Vectorの要素数を返します。",
    ));
    
    // Vector::capacity - Vectorの容量を取得
    registry.register_function(StdlibFunction::new(
        "Vector::capacity",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("vector".to_string(), vector_type.id)],
        int_type.id,
        "Vectorの現在の容量を返します。",
    ));
    
    // Vector::clear - Vectorをクリア
    registry.register_function(StdlibFunction::new(
        "Vector::clear",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![("vector".to_string(), vector_type.id)],
        unit_type.id,
        "Vectorのすべての要素を削除します。容量は変更されません。",
    ));
    
    // Vector::is_empty - Vectorが空かどうかを確認
    registry.register_function(StdlibFunction::new(
        "Vector::is_empty",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("vector".to_string(), vector_type.id)],
        bool_type.id,
        "Vectorが空かどうかを返します。",
    ));
    
    // HashMap関数の登録
    
    // HashMap::new - 新しいHashMapを作成
    registry.register_function(StdlibFunction::new(
        "HashMap::new",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![],
        hashmap_type.id,
        "新しい空のHashMap（ハッシュマップ）を作成します。",
    ));
    
    // HashMap::insert - キーと値のペアを追加
    registry.register_function(StdlibFunction::new(
        "HashMap::insert",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![
            ("map".to_string(), hashmap_type.id),
            ("key".to_string(), hashmap_key_type.id),
            ("value".to_string(), hashmap_value_type.id),
        ],
        unit_type.id,
        "HashMapにキーと値のペアを追加します。キーが既に存在する場合は値が上書きされます。",
    ));
    
    // HashMap::get - キーに対応する値を取得
    registry.register_function(StdlibFunction::new(
        "HashMap::get",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![
            ("map".to_string(), hashmap_type.id),
            ("key".to_string(), hashmap_key_type.id),
        ],
        hashmap_value_type.id,
        "キーに対応する値を取得します。キーが存在しない場合はエラーになります。",
    ));
    
    // HashMap::contains_key - キーが存在するかどうかを確認
    registry.register_function(StdlibFunction::new(
        "HashMap::contains_key",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![
            ("map".to_string(), hashmap_type.id),
            ("key".to_string(), hashmap_key_type.id),
        ],
        bool_type.id,
        "指定したキーがHashMapに存在するかどうかを返します。",
    ));
    
    // HashMap::remove - キーと対応する値を削除
    registry.register_function(StdlibFunction::new(
        "HashMap::remove",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![
            ("map".to_string(), hashmap_type.id),
            ("key".to_string(), hashmap_key_type.id),
        ],
        unit_type.id,
        "指定したキーと対応する値をHashMapから削除します。",
    ));
    
    // HashMap::size - HashMapのサイズを取得
    registry.register_function(StdlibFunction::new(
        "HashMap::size",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("map".to_string(), hashmap_type.id)],
        int_type.id,
        "HashMapのキーと値のペアの数を返します。",
    ));
    
    // HashMap::clear - HashMapをクリア
    registry.register_function(StdlibFunction::new(
        "HashMap::clear",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![("map".to_string(), hashmap_type.id)],
        unit_type.id,
        "HashMapのすべてのキーと値のペアを削除します。",
    ));
    
    // HashMap::is_empty - HashMapが空かどうかを確認
    registry.register_function(StdlibFunction::new(
        "HashMap::is_empty",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("map".to_string(), hashmap_type.id)],
        bool_type.id,
        "HashMapが空かどうかを返します。",
    ));
    
    // HashSet関数の登録
    
    // HashSet::new - 新しいHashSetを作成
    registry.register_function(StdlibFunction::new(
        "HashSet::new",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![],
        hashset_type.id,
        "新しい空のHashSet（ハッシュセット）を作成します。",
    ));
    
    // HashSet::add - 要素を追加
    registry.register_function(StdlibFunction::new(
        "HashSet::add",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![
            ("set".to_string(), hashset_type.id),
            ("element".to_string(), hashset_elem_type.id),
        ],
        bool_type.id,
        "HashSetに要素を追加します。要素が既に存在する場合はfalseを返します。そうでない場合はtrueを返します。",
    ));
    
    // HashSet::contains - 要素が含まれているかどうかを確認
    registry.register_function(StdlibFunction::new(
        "HashSet::contains",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![
            ("set".to_string(), hashset_type.id),
            ("element".to_string(), hashset_elem_type.id),
        ],
        bool_type.id,
        "HashSetに要素が含まれているかどうかを返します。",
    ));
    
    // HashSet::remove - 要素を削除
    registry.register_function(StdlibFunction::new(
        "HashSet::remove",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![
            ("set".to_string(), hashset_type.id),
            ("element".to_string(), hashset_elem_type.id),
        ],
        bool_type.id,
        "HashSetから要素を削除します。要素が存在した場合はtrueを返します。そうでない場合はfalseを返します。",
    ));
    
    // HashSet::size - HashSetのサイズを取得
    registry.register_function(StdlibFunction::new(
        "HashSet::size",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("set".to_string(), hashset_type.id)],
        int_type.id,
        "HashSetの要素数を返します。",
    ));
    
    // HashSet::clear - HashSetをクリア
    registry.register_function(StdlibFunction::new(
        "HashSet::clear",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![("set".to_string(), hashset_type.id)],
        unit_type.id,
        "HashSetのすべての要素を削除します。",
    ));
    
    // HashSet::is_empty - HashSetが空かどうかを確認
    registry.register_function(StdlibFunction::new(
        "HashSet::is_empty",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("set".to_string(), hashset_type.id)],
        bool_type.id,
        "HashSetが空かどうかを返します。",
    ));
    
    // LinkedList関数の登録
    
    // LinkedList::new - 新しいLinkedListを作成
    registry.register_function(StdlibFunction::new(
        "LinkedList::new",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![],
        linkedlist_type.id,
        "新しい空のLinkedList（連結リスト）を作成します。",
    ));
    
    // LinkedList::push_front - リストの先頭に要素を追加
    registry.register_function(StdlibFunction::new(
        "LinkedList::push_front",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![
            ("list".to_string(), linkedlist_type.id),
            ("element".to_string(), linkedlist_elem_type.id),
        ],
        unit_type.id,
        "LinkedListの先頭に要素を追加します。",
    ));
    
    // LinkedList::push_back - リストの末尾に要素を追加
    registry.register_function(StdlibFunction::new(
        "LinkedList::push_back",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![
            ("list".to_string(), linkedlist_type.id),
            ("element".to_string(), linkedlist_elem_type.id),
        ],
        unit_type.id,
        "LinkedListの末尾に要素を追加します。",
    ));
    
    // LinkedList::pop_front - リストの先頭から要素を取り出す
    registry.register_function(StdlibFunction::new(
        "LinkedList::pop_front",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![("list".to_string(), linkedlist_type.id)],
        linkedlist_elem_type.id,
        "LinkedListの先頭から要素を取り出します。LinkedListは空にできません。",
    ));
    
    // LinkedList::pop_back - リストの末尾から要素を取り出す
    registry.register_function(StdlibFunction::new(
        "LinkedList::pop_back",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![("list".to_string(), linkedlist_type.id)],
        linkedlist_elem_type.id,
        "LinkedListの末尾から要素を取り出します。LinkedListは空にできません。",
    ));
    
    // LinkedList::length - リストの長さを取得
    registry.register_function(StdlibFunction::new(
        "LinkedList::length",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("list".to_string(), linkedlist_type.id)],
        int_type.id,
        "LinkedListの要素数を返します。",
    ));
    
    // LinkedList::clear - リストをクリア
    registry.register_function(StdlibFunction::new(
        "LinkedList::clear",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![("list".to_string(), linkedlist_type.id)],
        unit_type.id,
        "LinkedListのすべての要素を削除します。",
    ));
    
    // LinkedList::is_empty - リストが空かどうかを確認
    registry.register_function(StdlibFunction::new(
        "LinkedList::is_empty",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("list".to_string(), linkedlist_type.id)],
        bool_type.id,
        "LinkedListが空かどうかを返します。",
    ));
    
    // Queue関数の登録
    
    // Queue::new - 新しいQueueを作成
    registry.register_function(StdlibFunction::new(
        "Queue::new",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![],
        queue_type.id,
        "新しい空のQueue（キュー）を作成します。",
    ));
    
    // Queue::enqueue - キューに要素を追加
    registry.register_function(StdlibFunction::new(
        "Queue::enqueue",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![
            ("queue".to_string(), queue_type.id),
            ("element".to_string(), queue_elem_type.id),
        ],
        unit_type.id,
        "Queueに要素を追加します（エンキュー）。",
    ));
    
    // Queue::dequeue - キューから要素を取り出す
    registry.register_function(StdlibFunction::new(
        "Queue::dequeue",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![("queue".to_string(), queue_type.id)],
        queue_elem_type.id,
        "Queueから要素を取り出します（デキュー）。Queueは空にできません。",
    ));
    
    // Queue::peek - キューの先頭要素を取得（削除せず）
    registry.register_function(StdlibFunction::new(
        "Queue::peek",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("queue".to_string(), queue_type.id)],
        queue_elem_type.id,
        "Queueの先頭要素を返します（削除せずに）。Queueは空にできません。",
    ));
    
    // Queue::length - キューの長さを取得
    registry.register_function(StdlibFunction::new(
        "Queue::length",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("queue".to_string(), queue_type.id)],
        int_type.id,
        "Queueの要素数を返します。",
    ));
    
    // Queue::clear - キューをクリア
    registry.register_function(StdlibFunction::new(
        "Queue::clear",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![("queue".to_string(), queue_type.id)],
        unit_type.id,
        "Queueのすべての要素を削除します。",
    ));
    
    // Queue::is_empty - キューが空かどうかを確認
    registry.register_function(StdlibFunction::new(
        "Queue::is_empty",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("queue".to_string(), queue_type.id)],
        bool_type.id,
        "Queueが空かどうかを返します。",
    ));
    
    // Stack関数の登録
    
    // Stack::new - 新しいStackを作成
    registry.register_function(StdlibFunction::new(
        "Stack::new",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![],
        stack_type.id,
        "新しい空のStack（スタック）を作成します。",
    ));
    
    // Stack::push - スタックに要素を追加
    registry.register_function(StdlibFunction::new(
        "Stack::push",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![
            ("stack".to_string(), stack_type.id),
            ("element".to_string(), stack_elem_type.id),
        ],
        unit_type.id,
        "Stackに要素を追加します（プッシュ）。",
    ));
    
    // Stack::pop - スタックから要素を取り出す
    registry.register_function(StdlibFunction::new(
        "Stack::pop",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![("stack".to_string(), stack_type.id)],
        stack_elem_type.id,
        "Stackから要素を取り出します（ポップ）。Stackは空にできません。",
    ));
    
    // Stack::peek - スタックの先頭要素を取得（削除せず）
    registry.register_function(StdlibFunction::new(
        "Stack::peek",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("stack".to_string(), stack_type.id)],
        stack_elem_type.id,
        "Stackの先頭要素を返します（削除せずに）。Stackは空にできません。",
    ));
    
    // Stack::length - スタックの長さを取得
    registry.register_function(StdlibFunction::new(
        "Stack::length",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("stack".to_string(), stack_type.id)],
        int_type.id,
        "Stackの要素数を返します。",
    ));
    
    // Stack::clear - スタックをクリア
    registry.register_function(StdlibFunction::new(
        "Stack::clear",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![("stack".to_string(), stack_type.id)],
        unit_type.id,
        "Stackのすべての要素を削除します。",
    ));
    
    // Stack::is_empty - スタックが空かどうかを確認
    registry.register_function(StdlibFunction::new(
        "Stack::is_empty",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("stack".to_string(), stack_type.id)],
        bool_type.id,
        "Stackが空かどうかを返します。",
    ));
    
    // PriorityQueue関数の登録
    
    // PriorityQueue::new - 新しいPriorityQueueを作成
    registry.register_function(StdlibFunction::new(
        "PriorityQueue::new",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![],
        priorityqueue_type.id,
        "新しい空のPriorityQueue（優先度キュー）を作成します。",
    ));
    
    // PriorityQueue::push - 優先度キューに要素を追加
    registry.register_function(StdlibFunction::new(
        "PriorityQueue::push",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![
            ("pq".to_string(), priorityqueue_type.id),
            ("element".to_string(), priorityqueue_elem_type.id),
            ("priority".to_string(), int_type.id),
        ],
        unit_type.id,
        "PriorityQueueに要素を追加します。priorityが高い（数値が大きい）ほど優先度が高くなります。",
    ));
    
    // PriorityQueue::pop - 優先度キューから最高優先度の要素を取り出す
    registry.register_function(StdlibFunction::new(
        "PriorityQueue::pop",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![("pq".to_string(), priorityqueue_type.id)],
        priorityqueue_elem_type.id,
        "PriorityQueueから最高優先度の要素を取り出します。PriorityQueueは空にできません。",
    ));
    
    // PriorityQueue::peek - 優先度キューの最高優先度の要素を取得（削除せず）
    registry.register_function(StdlibFunction::new(
        "PriorityQueue::peek",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("pq".to_string(), priorityqueue_type.id)],
        priorityqueue_elem_type.id,
        "PriorityQueueの最高優先度の要素を返します（削除せずに）。PriorityQueueは空にできません。",
    ));
    
    // PriorityQueue::length - 優先度キューの長さを取得
    registry.register_function(StdlibFunction::new(
        "PriorityQueue::length",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("pq".to_string(), priorityqueue_type.id)],
        int_type.id,
        "PriorityQueueの要素数を返します。",
    ));
    
    // PriorityQueue::clear - 優先度キューをクリア
    registry.register_function(StdlibFunction::new(
        "PriorityQueue::clear",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![("pq".to_string(), priorityqueue_type.id)],
        unit_type.id,
        "PriorityQueueのすべての要素を削除します。",
    ));
    
    // PriorityQueue::is_empty - 優先度キューが空かどうかを確認
    registry.register_function(StdlibFunction::new(
        "PriorityQueue::is_empty",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("pq".to_string(), priorityqueue_type.id)],
        bool_type.id,
        "PriorityQueueが空かどうかを返します。",
    ));
    
    Ok(())
}

// コレクションのインスタンスを管理するためのグローバル状態
lazy_static! {
    static ref VECTOR_INSTANCES: Mutex<HashMap<String, Vec<Value>>> = Mutex::new(HashMap::new());
    static ref HASHMAP_INSTANCES: Mutex<HashMap<String, HashMap<String, Value>>> = Mutex::new(HashMap::new());
    static ref INSTANCE_COUNTER: AtomicUsize = AtomicUsize::new(0);
}

/// コレクション値を表現する列挙型
#[derive(Clone, Debug)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Vector(String),   // ベクターへの参照ID
    HashMap(String),  // ハッシュマップへの参照ID
    Null,
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        // 文字列値から適切な型に変換
        if s == "null" || s == "nil" {
            return Value::Null;
        }
        
        if let Ok(i) = s.parse::<i64>() {
            return Value::Int(i);
        }
        
        if let Ok(f) = s.parse::<f64>() {
            return Value::Float(f);
        }
        
        if s == "true" {
            return Value::Bool(true);
        }
        
        if s == "false" {
            return Value::Bool(false);
        }
        
        // 特殊な参照形式を検出
        if s.starts_with("vector:") {
            return Value::Vector(s.to_string());
        }
        
        if s.starts_with("hashmap:") {
            return Value::HashMap(s.to_string());
        }
        
        // それ以外は文字列
        Value::String(s.to_string())
    }
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::Int(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::String(s) => s.clone(),
            Value::Vector(id) => id.clone(),
            Value::HashMap(id) => id.clone(),
            Value::Null => "null".to_string(),
        }
    }
}

/// 新しいインスタンスIDを生成
fn generate_instance_id(prefix: &str) -> String {
    let id = INSTANCE_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("{}:{}", prefix, id)
}

/// コレクション関数の実行
pub fn execute_function(function_name: &str, args: &[String]) -> Result<String> {
    // 引数をValue型に変換
    let values: Vec<Value> = args.iter().map(|s| Value::from(s.as_str())).collect();
    
    // 各コレクション関数の実装
    match function_name {
        // Vector関数
        "Vector::new" => {
            // 新しい空のVectorを作成
            let instance_id = generate_instance_id("vector");
            
            // グローバル状態に登録
            VECTOR_INSTANCES.lock().unwrap().insert(instance_id.clone(), Vec::new());
            
            Ok(instance_id)
        },
        "Vector::with_capacity" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("Vector::with_capacityには容量パラメータが必要です".to_string()));
            }
            
            // 容量を取得
            let capacity = match &values[0] {
                Value::Int(i) => *i as usize,
                _ => return Err(EidosError::Runtime("容量は整数である必要があります".to_string())),
            };
            
            // 指定された容量でベクターを初期化
            let instance_id = generate_instance_id("vector");
            VECTOR_INSTANCES.lock().unwrap().insert(instance_id.clone(), Vec::with_capacity(capacity));
            
            Ok(instance_id)
        },
        "Vector::push" => {
            if values.len() != 2 {
                return Err(EidosError::Runtime("Vector::pushにはベクターと要素が必要です".to_string()));
            }
            
            // ベクター参照を取得
            let vector_ref = match &values[0] {
                Value::Vector(id) => id.clone(),
                _ => return Err(EidosError::Runtime("第1引数はベクター参照である必要があります".to_string())),
            };
            
            // 要素を追加
            let element = values[1].clone();
            
            // 存在確認とpush操作
            let mut instances = VECTOR_INSTANCES.lock().unwrap();
            if let Some(vector) = instances.get_mut(&vector_ref) {
                vector.push(element);
                Ok(vector_ref)
            } else {
                Err(EidosError::Runtime(format!("ベクター参照 '{}' は無効です", vector_ref)))
            }
        },
        "Vector::pop" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("Vector::popにはベクターが必要です".to_string()));
            }
            
            // ベクター参照を取得
            let vector_ref = match &values[0] {
                Value::Vector(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数はベクター参照である必要があります".to_string())),
            };
            
            // 要素を取り出す
            let mut instances = VECTOR_INSTANCES.lock().unwrap();
            if let Some(vector) = instances.get_mut(&vector_ref) {
                if let Some(value) = vector.pop() {
                    Ok(value.to_string())
                } else {
                    Err(EidosError::Runtime("ベクターが空です".to_string()))
                }
            } else {
                Err(EidosError::Runtime(format!("ベクター参照 '{}' は無効です", vector_ref)))
            }
        },
        "Vector::get" => {
            if values.len() != 2 {
                return Err(EidosError::Runtime("Vector::getにはベクターとインデックスが必要です".to_string()));
            }
            
            // ベクター参照を取得
            let vector_ref = match &values[0] {
                Value::Vector(id) => id.clone(),
                _ => return Err(EidosError::Runtime("第1引数はベクター参照である必要があります".to_string())),
            };
            
            // インデックスを取得
            let index = match &values[1] {
                Value::Int(i) => *i as usize,
                _ => return Err(EidosError::Runtime("インデックスは整数である必要があります".to_string())),
            };
            
            // 要素を取得
            let instances = VECTOR_INSTANCES.lock().unwrap();
            if let Some(vector) = instances.get(&vector_ref) {
                if index < vector.len() {
                    Ok(vector[index].to_string())
                } else {
                    Err(EidosError::Runtime(format!("インデックス {} は範囲外です (len: {})", index, vector.len())))
                }
            } else {
                Err(EidosError::Runtime(format!("ベクター参照 '{}' は無効です", vector_ref)))
            }
        },
        "Vector::len" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("Vector::lenにはベクターが必要です".to_string()));
            }
            
            // ベクター参照を取得
            let vector_ref = match &values[0] {
                Value::Vector(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数はベクター参照である必要があります".to_string())),
            };
            
            // 長さを取得
            let instances = VECTOR_INSTANCES.lock().unwrap();
            if let Some(vector) = instances.get(&vector_ref) {
                Ok(vector.len().to_string())
            } else {
                Err(EidosError::Runtime(format!("ベクター参照 '{}' は無効です", vector_ref)))
            }
        },
        "Vector::is_empty" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("Vector::is_emptyにはベクターが必要です".to_string()));
            }
            
            // ベクター参照を取得
            let vector_ref = match &values[0] {
                Value::Vector(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数はベクター参照である必要があります".to_string())),
            };
            
            // 空かどうかを確認
            let instances = VECTOR_INSTANCES.lock().unwrap();
            if let Some(vector) = instances.get(&vector_ref) {
                Ok(vector.is_empty().to_string())
            } else {
                Err(EidosError::Runtime(format!("ベクター参照 '{}' は無効です", vector_ref)))
            }
        },
        "Vector::clear" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("Vector::clearにはベクターが必要です".to_string()));
            }
            
            // ベクター参照を取得
            let vector_ref = match &values[0] {
                Value::Vector(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数はベクター参照である必要があります".to_string())),
            };
            
            // ベクターをクリア
            let mut instances = VECTOR_INSTANCES.lock().unwrap();
            if let Some(vector) = instances.get_mut(&vector_ref) {
                vector.clear();
                Ok(vector_ref)
            } else {
                Err(EidosError::Runtime(format!("ベクター参照 '{}' は無効です", vector_ref)))
            }
        },
        
        // HashMap関数
        "HashMap::new" => {
            // 新しい空のHashMapを作成
            let instance_id = generate_instance_id("hashmap");
            
            // グローバル状態に登録
            HASHMAP_INSTANCES.lock().unwrap().insert(instance_id.clone(), HashMap::new());
            
            Ok(instance_id)
        },
        "HashMap::insert" => {
            if values.len() != 3 {
                return Err(EidosError::Runtime("HashMap::insertにはマップ、キー、値が必要です".to_string()));
            }
            
            // ハッシュマップ参照を取得
            let map_ref = match &values[0] {
                Value::HashMap(id) => id.clone(),
                _ => return Err(EidosError::Runtime("第1引数はハッシュマップ参照である必要があります".to_string())),
            };
            
            // キーと値を取得
            let key = values[1].to_string();
            let value = values[2].clone();
            
            // 要素を挿入
            let mut instances = HASHMAP_INSTANCES.lock().unwrap();
            if let Some(map) = instances.get_mut(&map_ref) {
                map.insert(key, value);
                Ok(map_ref)
            } else {
                Err(EidosError::Runtime(format!("ハッシュマップ参照 '{}' は無効です", map_ref)))
            }
        },
        "HashMap::get" => {
            if values.len() != 2 {
                return Err(EidosError::Runtime("HashMap::getにはマップとキーが必要です".to_string()));
            }
            
            // ハッシュマップ参照を取得
            let map_ref = match &values[0] {
                Value::HashMap(id) => id.clone(),
                _ => return Err(EidosError::Runtime("第1引数はハッシュマップ参照である必要があります".to_string())),
            };
            
            // キーを取得
            let key = values[1].to_string();
            
            // 値を取得
            let instances = HASHMAP_INSTANCES.lock().unwrap();
            if let Some(map) = instances.get(&map_ref) {
                if let Some(value) = map.get(&key) {
                    Ok(value.to_string())
                } else {
                    Err(EidosError::Runtime(format!("キー '{}' が見つかりません", key)))
                }
            } else {
                Err(EidosError::Runtime(format!("ハッシュマップ参照 '{}' は無効です", map_ref)))
            }
        },
        "HashMap::contains_key" => {
            if values.len() != 2 {
                return Err(EidosError::Runtime("HashMap::contains_keyにはマップとキーが必要です".to_string()));
            }
            
            // ハッシュマップ参照を取得
            let map_ref = match &values[0] {
                Value::HashMap(id) => id.clone(),
                _ => return Err(EidosError::Runtime("第1引数はハッシュマップ参照である必要があります".to_string())),
            };
            
            // キーを取得
            let key = values[1].to_string();
            
            // キーが存在するか確認
            let instances = HASHMAP_INSTANCES.lock().unwrap();
            if let Some(map) = instances.get(&map_ref) {
                Ok(map.contains_key(&key).to_string())
            } else {
                Err(EidosError::Runtime(format!("ハッシュマップ参照 '{}' は無効です", map_ref)))
            }
        },
        "HashMap::remove" => {
            if values.len() != 2 {
                return Err(EidosError::Runtime("HashMap::removeにはマップとキーが必要です".to_string()));
            }
            
            // ハッシュマップ参照を取得
            let map_ref = match &values[0] {
                Value::HashMap(id) => id.clone(),
                _ => return Err(EidosError::Runtime("第1引数はハッシュマップ参照である必要があります".to_string())),
            };
            
            // キーを取得
            let key = values[1].to_string();
            
            // キーを削除
            let mut instances = HASHMAP_INSTANCES.lock().unwrap();
            if let Some(map) = instances.get_mut(&map_ref) {
                if let Some(value) = map.remove(&key) {
                    Ok(value.to_string())
                } else {
                    Err(EidosError::Runtime(format!("キー '{}' が見つかりません", key)))
                }
            } else {
                Err(EidosError::Runtime(format!("ハッシュマップ参照 '{}' は無効です", map_ref)))
            }
        },
        "HashMap::len" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("HashMap::lenにはハッシュマップが必要です".to_string()));
            }
            
            // ハッシュマップ参照を取得
            let map_ref = match &values[0] {
                Value::HashMap(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数はハッシュマップ参照である必要があります".to_string())),
            };
            
            // サイズを取得
            let instances = HASHMAP_INSTANCES.lock().unwrap();
            if let Some(map) = instances.get(&map_ref) {
                Ok(map.len().to_string())
            } else {
                Err(EidosError::Runtime(format!("ハッシュマップ参照 '{}' は無効です", map_ref)))
            }
        },
        "HashMap::clear" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("HashMap::clearにはハッシュマップが必要です".to_string()));
            }
            
            // ハッシュマップ参照を取得
            let map_ref = match &values[0] {
                Value::HashMap(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数はハッシュマップ参照である必要があります".to_string())),
            };
            
            // ハッシュマップをクリア
            let mut instances = HASHMAP_INSTANCES.lock().unwrap();
            if let Some(map) = instances.get_mut(&map_ref) {
                map.clear();
                Ok(map_ref)
            } else {
                Err(EidosError::Runtime(format!("ハッシュマップ参照 '{}' は無効です", map_ref)))
            }
        },
        "HashMap::is_empty" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("HashMap::is_emptyにはハッシュマップが必要です".to_string()));
            }
            
            // ハッシュマップ参照を取得
            let map_ref = match &values[0] {
                Value::HashMap(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数はハッシュマップ参照である必要があります".to_string())),
            };
            
            // 空かどうかを確認
            let instances = HASHMAP_INSTANCES.lock().unwrap();
            if let Some(map) = instances.get(&map_ref) {
                Ok(map.is_empty().to_string())
            } else {
                Err(EidosError::Runtime(format!("ハッシュマップ参照 '{}' は無効です", map_ref)))
            }
        },
        
        // その他のコレクション関数
        _ => Err(EidosError::Runtime(format!("未知のコレクション関数: {}", function_name))),
    }
} 