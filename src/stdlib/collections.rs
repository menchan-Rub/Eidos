use crate::core::{Result, EidosError};
use crate::core::types::{Type, TypeId, TypeKind, Field};
use crate::stdlib::{StdlibRegistry, StdlibFunction, StdlibModule, StdlibFunctionType};
use std::sync::{Mutex, atomic::{AtomicUsize, Ordering}};
use std::collections::{HashMap, BinaryHeap};
use std::cmp::{Ord, PartialOrd, Ordering as CmpOrdering};
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
                Field {
                    name: "buckets".to_string(),
                    field_type: Type::array(Type::tuple(vec![
                        hashmap_key_type.clone(),
                        hashmap_value_type.clone()
                    ])),
                    is_public: false,
                },
                Field {
                    name: "hash_function".to_string(),
                    field_type: Type::function(
                        vec![hashmap_key_type.clone()],
                        int_type.clone()
                    ),
                    is_public: false,
                },
                Field {
                    name: "load_factor".to_string(),
                    field_type: Type::float(),
                    is_public: false,
                }
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
                Field {
                    name: "map".to_string(),
                    field_type: Type::new(
                        TypeKind::Struct {
                            name: "HashMap".to_string(),
                            fields: vec![],
                            methods: vec![],
                            is_extern: true,
                        }
                    ),
                    is_public: false,
                }
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
            methods: vec![
                // 新しいLinkedListの作成
                ("new".to_string(), Type::function(vec![], linkedlist_type.clone())),
                // 先頭に要素を追加
                ("push_front".to_string(), Type::function(
                    vec![linkedlist_elem_type.clone()], 
                    unit_type.clone()
                )),
                // 末尾に要素を追加
                ("push_back".to_string(), Type::function(
                    vec![linkedlist_elem_type.clone()], 
                    unit_type.clone()
                )),
                // 先頭の要素を取り出す
                ("pop_front".to_string(), Type::function(
                    vec![], 
                    linkedlist_elem_type.clone()
                )),
                // 末尾の要素を取り出す
                ("pop_back".to_string(), Type::function(
                    vec![], 
                    linkedlist_elem_type.clone()
                )),
                // リストの長さを取得
                ("length".to_string(), Type::function(
                    vec![], 
                    int_type.clone()
                )),
                // リストをクリア
                ("clear".to_string(), Type::function(
                    vec![], 
                    unit_type.clone()
                )),
                // リストが空かどうか
                ("is_empty".to_string(), Type::function(
                    vec![], 
                    bool_type.clone()
                )),
            ],
            is_extern: false,
        },
    );
    registry.register_type("collections::LinkedList", linkedlist_type.clone());
    
    // Queue（キュー）型の定義 - 循環バッファによる効率的な実装
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
                Field {
                    name: "capacity".to_string(),
                    field_type: int_type.clone(),
                    is_public: false,
                },
                // 循環バッファとして実装するための内部フィールド
                Field {
                    name: "data".to_string(),
                    field_type: Type::array(queue_elem_type.clone()),
                    is_public: false,
                },
                Field {
                    name: "front".to_string(), // 先頭要素のインデックス
                    field_type: int_type.clone(),
                    is_public: false,
                },
                Field {
                    name: "rear".to_string(), // 次に挿入する位置のインデックス
                    field_type: int_type.clone(),
                    is_public: false,
                },
                Field {
                    name: "growth_factor".to_string(), // 拡張時の成長係数
                    field_type: float_type.clone(),
                    is_public: false,
                },
                Field {
                    name: "min_capacity".to_string(), // 最小容量
                    field_type: int_type.clone(),
                    is_public: false,
                }
            ],
            methods: vec![
                // 新しいQueueの作成
                ("new".to_string(), Type::function(vec![], queue_type.clone())),
                // 要素を追加（エンキュー）
                ("enqueue".to_string(), Type::function(
                    vec![queue_elem_type.clone()], 
                    unit_type.clone()
                )),
                // 要素を取り出す（デキュー）
                ("dequeue".to_string(), Type::function(
                    vec![], 
                    queue_elem_type.clone()
                )),
                // 先頭の要素を覗き見
                ("peek".to_string(), Type::function(
                    vec![], 
                    queue_elem_type.clone()
                )),
                // キューの長さを取得
                ("length".to_string(), Type::function(
                    vec![], 
                    int_type.clone()
                )),
                // キューをクリア
                ("clear".to_string(), Type::function(
                    vec![], 
                    unit_type.clone()
                )),
                // キューが空かどうか
                ("is_empty".to_string(), Type::function(
                    vec![], 
                    bool_type.clone()
                )),
            ],
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
                Field {
                    name: "capacity".to_string(),
                    field_type: int_type.clone(),
                    is_public: false,
                },
                // 実際のスタックの実装は内部的に行う
                Field {
                    name: "data".to_string(),
                    field_type: Type::array(stack_elem_type.clone()),
                    is_public: false,
                },
                Field {
                    name: "top".to_string(), 
                    field_type: int_type.clone(),
                    is_public: false,
                },
                Field {
                    name: "growth_factor".to_string(), // 拡張時の成長係数
                    field_type: float_type.clone(),
                    is_public: false,
                },
                Field {
                    name: "min_capacity".to_string(), // 最小容量
                    field_type: int_type.clone(),
                    is_public: false,
                }
            ],
            methods: vec![
                // 新しいStackの作成
                ("new".to_string(), Type::function(vec![], stack_type.clone())),
                // 初期容量を指定してStackを作成
                ("with_capacity".to_string(), Type::function(
                    vec![int_type.clone()], 
                    stack_type.clone()
                )),
                // 要素をプッシュ
                ("push".to_string(), Type::function(
                    vec![stack_elem_type.clone()], 
                    unit_type.clone()
                )),
                // 要素をポップ
                ("pop".to_string(), Type::function(
                    vec![], 
                    stack_elem_type.clone()
                )),
                // 先頭の要素を覗き見
                ("peek".to_string(), Type::function(
                    vec![], 
                    stack_elem_type.clone()
                )),
                // スタックの長さを取得
                ("length".to_string(), Type::function(
                    vec![], 
                    int_type.clone()
                )),
                // スタックの容量を取得
                ("capacity".to_string(), Type::function(
                    vec![], 
                    int_type.clone()
                )),
                // スタックをクリア
                ("clear".to_string(), Type::function(
                    vec![], 
                    unit_type.clone()
                )),
                // スタックが空かどうか
                ("is_empty".to_string(), Type::function(
                    vec![], 
                    bool_type.clone()
                )),
                // スタックの容量を予約
                ("reserve".to_string(), Type::function(
                    vec![int_type.clone()], 
                    unit_type.clone()
                )),
                // スタックの容量を現在の長さに合わせて最適化
                ("shrink_to_fit".to_string(), Type::function(
                    vec![], 
                    unit_type.clone()
                )),
                // スタックの内容を配列として取得
                ("to_array".to_string(), Type::function(
                    vec![], 
                    Type::array(stack_elem_type.clone())
                )),
                // 配列からスタックを作成
                ("from_array".to_string(), Type::function(
                    vec![Type::array(stack_elem_type.clone())], 
                    stack_type.clone()
                )),
            ],
            is_extern: false,
        },
    );
    registry.register_type("collections::Stack", stack_type.clone());
    
    // PriorityQueue（優先度キュー）型の定義
    let pqueue_elem_type = Type::generic("T");
    let pqueue_type = Type::new(
        TypeKind::Struct {
            name: "PriorityQueue".to_string(),
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
                // 実際の優先度キューの実装は内部的に行う
                Field {
                    name: "heap".to_string(),
                    field_type: Type::array(
                        Type::tuple(vec![
                            int_type.clone(),           // 優先度
                            pqueue_elem_type.clone()    // 要素
                        ])
                    ),
                    is_public: false,
                },
                Field {
                    name: "compare_fn".to_string(),
                    field_type: Type::function(
                        vec![int_type.clone(), int_type.clone()],
                        int_type.clone()
                    ),
                    is_public: false,
                },
                Field {
                    name: "growth_factor".to_string(), // 拡張時の成長係数
                    field_type: float_type.clone(),
                    is_public: false,
                },
                Field {
                    name: "min_capacity".to_string(), // 最小容量
                    field_type: int_type.clone(),
                    is_public: false,
                },
                Field {
                    name: "is_max_heap".to_string(), // 最大ヒープか最小ヒープか
                    field_type: bool_type.clone(),
                    is_public: false,
                }
            ],
            methods: vec![
                // 新しいPriorityQueueの作成
                ("new".to_string(), Type::function(vec![], pqueue_type.clone())),
                // 要素を優先度付きで追加
                ("push".to_string(), Type::function(
                    vec![pqueue_elem_type.clone(), int_type.clone()], 
                    unit_type.clone()
                )),
                // 最高優先度の要素を取り出す
                ("pop".to_string(), Type::function(
                    vec![], 
                    pqueue_elem_type.clone()
                )),
                // 最高優先度の要素を覗き見
                ("peek".to_string(), Type::function(
                    vec![], 
                    pqueue_elem_type.clone()
                )),
                // 優先度キューの長さを取得
                ("length".to_string(), Type::function(
                    vec![], 
                    int_type.clone()
                )),
                // 優先度キューをクリア
                ("clear".to_string(), Type::function(
                    vec![], 
                    unit_type.clone()
                )),
                // 優先度キューが空かどうか
                ("is_empty".to_string(), Type::function(
                    vec![], 
                    bool_type.clone()
                )),
            ],
            is_extern: false,
        },
    );
    registry.register_type("collections::PriorityQueue", pqueue_type.clone());
    
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
    registry.register_function(StdlibFunction::new(
        "LinkedList::push_front",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![
            ("list".to_string(), linkedlist_type.id),
            ("value".to_string(), linkedlist_elem_type.id),
        ],
        unit_type.id,
        "LinkedListの先頭に要素を追加します。",
    ));
    registry.register_function(StdlibFunction::new(
        "LinkedList::push_back",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![
            ("list".to_string(), linkedlist_type.id),
            ("value".to_string(), linkedlist_elem_type.id),
        ],
        unit_type.id,
        "LinkedListの末尾に要素を追加します。",
    ));
    registry.register_function(StdlibFunction::new(
        "LinkedList::pop_front",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![("list".to_string(), linkedlist_type.id)],
        linkedlist_elem_type.id,
        "LinkedListの先頭から要素を取り出します。LinkedListは空にできません。",
    ));
    registry.register_function(StdlibFunction::new(
        "LinkedList::pop_back",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![("list".to_string(), linkedlist_type.id)],
        linkedlist_elem_type.id,
        "LinkedListの末尾から要素を取り出します。LinkedListは空にできません。",
    ));
    registry.register_function(StdlibFunction::new(
        "LinkedList::length",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("list".to_string(), linkedlist_type.id)],
        int_type.id,
        "LinkedListの要素数を返します。",
    ));
    registry.register_function(StdlibFunction::new(
        "LinkedList::clear",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![("list".to_string(), linkedlist_type.id)],
        unit_type.id,
        "LinkedListのすべての要素を削除します。",
    ));
    registry.register_function(StdlibFunction::new(
        "LinkedList::is_empty",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("list".to_string(), linkedlist_type.id)],
        bool_type.id,
        "LinkedListが空かどうかを返します。",
    ));
    
    // Queue関数の登録
    registry.register_function(StdlibFunction::new(
        "Queue::new",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![],
        queue_type.id,
        "新しい空のQueue（キュー）を作成します。",
    ));
    registry.register_function(StdlibFunction::new(
        "Queue::enqueue",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![
            ("queue".to_string(), queue_type.id),
            ("value".to_string(), queue_elem_type.id),
        ],
        unit_type.id,
        "Queueに要素を追加します（エンキュー）。",
    ));
    registry.register_function(StdlibFunction::new(
        "Queue::dequeue",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![("queue".to_string(), queue_type.id)],
        queue_elem_type.id,
        "Queueから要素を取り出します（デキュー）。Queueは空にできません。",
    ));
    registry.register_function(StdlibFunction::new(
        "Queue::peek",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("queue".to_string(), queue_type.id)],
        queue_elem_type.id,
        "Queueの先頭要素を返します（削除せずに）。Queueは空にできません。",
    ));
    registry.register_function(StdlibFunction::new(
        "Queue::length",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("queue".to_string(), queue_type.id)],
        int_type.id,
        "Queueの要素数を返します。",
    ));
    registry.register_function(StdlibFunction::new(
        "Queue::clear",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![("queue".to_string(), queue_type.id)],
        unit_type.id,
        "Queueのすべての要素を削除します。",
    ));
    registry.register_function(StdlibFunction::new(
        "Queue::is_empty",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("queue".to_string(), queue_type.id)],
        bool_type.id,
        "Queueが空かどうかを返します。",
    ));
    
    // Stack関数の登録
    registry.register_function(StdlibFunction::new(
        "Stack::new",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![],
        stack_type.id,
        "新しい空のStack（スタック）を作成します。",
    ));
    registry.register_function(StdlibFunction::new(
        "Stack::push",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![
            ("stack".to_string(), stack_type.id),
            ("value".to_string(), stack_elem_type.id),
        ],
        unit_type.id,
        "Stackに要素を追加します（プッシュ）。",
    ));
    registry.register_function(StdlibFunction::new(
        "Stack::pop",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![("stack".to_string(), stack_type.id)],
        stack_elem_type.id,
        "Stackから要素を取り出します（ポップ）。Stackは空にできません。",
    ));
    registry.register_function(StdlibFunction::new(
        "Stack::peek",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("stack".to_string(), stack_type.id)],
        stack_elem_type.id,
        "Stackの先頭要素を返します（削除せずに）。Stackは空にできません。",
    ));
    registry.register_function(StdlibFunction::new(
        "Stack::length",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("stack".to_string(), stack_type.id)],
        int_type.id,
        "Stackの要素数を返します。",
    ));
    registry.register_function(StdlibFunction::new(
        "Stack::clear",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![("stack".to_string(), stack_type.id)],
        unit_type.id,
        "Stackのすべての要素を削除します。",
    ));
    registry.register_function(StdlibFunction::new(
        "Stack::is_empty",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("stack".to_string(), stack_type.id)],
        bool_type.id,
        "Stackが空かどうかを返します。",
    ));
    
    // PriorityQueue関数の登録
    registry.register_function(StdlibFunction::new(
        "PriorityQueue::new",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![],
        pqueue_type.id,
        "新しい空のPriorityQueue（優先度キュー）を作成します。",
    ));
    registry.register_function(StdlibFunction::new(
        "PriorityQueue::push",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![
            ("pq".to_string(), pqueue_type.id),
            ("element".to_string(), pqueue_elem_type.id),
            ("priority".to_string(), int_type.id),
        ],
        unit_type.id,
        "PriorityQueueに要素を優先度付きで追加します。優先度が高い（数値が大きい）要素が先に取り出されます。",
    ));
    registry.register_function(StdlibFunction::new(
        "PriorityQueue::pop",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![("pq".to_string(), pqueue_type.id)],
        pqueue_elem_type.id,
        "PriorityQueueから最高優先度の要素を取り出します。PriorityQueueは空にできません。",
    ));
    registry.register_function(StdlibFunction::new(
        "PriorityQueue::peek",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("pq".to_string(), pqueue_type.id)],
        pqueue_elem_type.id,
        "PriorityQueueの最高優先度の要素を返します（削除せずに）。PriorityQueueは空にできません。",
    ));
    registry.register_function(StdlibFunction::new(
        "PriorityQueue::length",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("pq".to_string(), pqueue_type.id)],
        int_type.id,
        "PriorityQueueの要素数を返します。",
    ));
    registry.register_function(StdlibFunction::new(
        "PriorityQueue::clear",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![("pq".to_string(), pqueue_type.id)],
        unit_type.id,
        "PriorityQueueのすべての要素を削除します。",
    ));
    registry.register_function(StdlibFunction::new(
        "PriorityQueue::is_empty",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("pq".to_string(), pqueue_type.id)],
        bool_type.id,
        "PriorityQueueが空かどうかを返します。",
    ));
    
    // コレクション変換関数
    registry.register_function(StdlibFunction::new(
        "Vector::from_list",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("list".to_string(), linkedlist_type.id)],
        vector_type.id,
        "連結リストをベクターに変換します。",
    ));
    
    registry.register_function(StdlibFunction::new(
        "LinkedList::from_vector",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("vector".to_string(), vector_type.id)],
        linkedlist_type.id,
        "ベクターを連結リストに変換します。",
    ));
    
    registry.register_function(StdlibFunction::new(
        "HashSet::from_vector",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("vector".to_string(), vector_type.id)],
        hashset_type.id,
        "ベクターをハッシュセットに変換します。",
    ));
    
    registry.register_function(StdlibFunction::new(
        "Vector::from_hashset",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("set".to_string(), hashset_type.id)],
        vector_type.id,
        "ハッシュセットをベクターに変換します。",
    ));
    
    registry.register_function(StdlibFunction::new(
        "Queue::from_vector",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("vector".to_string(), vector_type.id)],
        queue_type.id,
        "ベクターをキューに変換します。",
    ));
    
    registry.register_function(StdlibFunction::new(
        "Vector::from_queue",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("queue".to_string(), queue_type.id)],
        vector_type.id,
        "キューをベクターに変換します。",
    ));
    
    registry.register_function(StdlibFunction::new(
        "Stack::from_vector",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("vector".to_string(), vector_type.id)],
        stack_type.id,
        "ベクターをスタックに変換します。",
    ));
    
    registry.register_function(StdlibFunction::new(
        "Vector::from_stack",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("stack".to_string(), stack_type.id)],
        vector_type.id,
        "スタックをベクターに変換します。",
    ));
    
    // コレクション一般操作
    registry.register_function(StdlibFunction::new(
        "Collection::clone",
        StdlibModule::Collections,
        StdlibFunctionType::Pure,
        vec![("collection".to_string(), vector_type.id)],
        vector_type.id,
        "コレクションをクローンします。",
    ));
    
    registry.register_function(StdlibFunction::new(
        "Collection::drop",
        StdlibModule::Collections,
        StdlibFunctionType::Effectful,
        vec![("collection".to_string(), vector_type.id)],
        unit_type.id,
        "コレクションを削除します。",
    ));
    
    Ok(())
}

// コレクションのインスタンスを管理するためのグローバル状態
lazy_static! {
    static ref VECTOR_INSTANCES: Mutex<HashMap<String, Vec<Value>>> = Mutex::new(HashMap::new());
    static ref HASHMAP_INSTANCES: Mutex<HashMap<String, HashMap<String, Value>>> = Mutex::new(HashMap::new());
    static ref HASHSET_INSTANCES: Mutex<HashMap<String, std::collections::HashSet<String>>> = Mutex::new(HashMap::new());
    static ref LINKEDLIST_INSTANCES: Mutex<HashMap<String, std::collections::LinkedList<Value>>> = Mutex::new(HashMap::new());
    static ref QUEUE_INSTANCES: Mutex<HashMap<String, std::collections::VecDeque<Value>>> = Mutex::new(HashMap::new());
    static ref STACK_INSTANCES: Mutex<HashMap<String, Vec<Value>>> = Mutex::new(HashMap::new());
    static ref PRIORITYQUEUE_INSTANCES: Mutex<HashMap<String, BinaryHeap<PriorityItem>>> = Mutex::new(HashMap::new());
    static ref INSTANCE_COUNTER: AtomicUsize = AtomicUsize::new(0);
}

// コレクションリソースの管理
lazy_static! {
    static ref COLLECTION_REF_COUNTS: Mutex<HashMap<String, usize>> = Mutex::new(HashMap::new());
}

/// 優先度キューのアイテム
#[derive(Clone, Debug, Eq, PartialEq)]
struct PriorityItem {
    priority: i64,
    value: Value,
}

// 優先度キューのアイテムをOrdering実装
impl Ord for PriorityItem {
    fn cmp(&self, other: &Self) -> CmpOrdering {
        // 優先度が高い（数値が大きい）方が先に出てくるように
        // Reverse ordering to make it a max-heap
        other.priority.cmp(&self.priority)
    }
}

impl PartialOrd for PriorityItem {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
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
    HashSet(String),  // ハッシュセットへの参照ID
    LinkedList(String), // 連結リストへの参照ID
    Queue(String),    // キューへの参照ID
    Stack(String),    // スタックへの参照ID
    PriorityQueue(String), // 優先度キューへの参照ID
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
        
        if s.starts_with("hashset:") {
            return Value::HashSet(s.to_string());
        }
        
        if s.starts_with("linkedlist:") {
            return Value::LinkedList(s.to_string());
        }
        
        if s.starts_with("queue:") {
            return Value::Queue(s.to_string());
        }
        
        if s.starts_with("stack:") {
            return Value::Stack(s.to_string());
        }
        
        if s.starts_with("priorityqueue:") {
            return Value::PriorityQueue(s.to_string());
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
            Value::HashSet(id) => id.clone(),
            Value::LinkedList(id) => id.clone(),
            Value::Queue(id) => id.clone(),
            Value::Stack(id) => id.clone(),
            Value::PriorityQueue(id) => id.clone(),
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
        
        // HashSet関数
        "HashSet::new" => {
            // 新しい空のHashSetを作成
            let instance_id = generate_instance_id("hashset");
            
            // グローバル状態に登録
            HASHSET_INSTANCES.lock().unwrap().insert(instance_id.clone(), std::collections::HashSet::new());
            
            Ok(instance_id)
        },
        "HashSet::add" => {
            if values.len() != 2 {
                return Err(EidosError::Runtime("HashSet::addにはセットと要素が必要です".to_string()));
            }
            
            // ハッシュセット参照を取得
            let set_ref = match &values[0] {
                Value::HashSet(id) => id.clone(),
                _ => return Err(EidosError::Runtime("第1引数はハッシュセット参照である必要があります".to_string())),
            };
            
            // 要素を取得
            let element = values[1].to_string();
            
            // 要素を追加
            let mut instances = HASHSET_INSTANCES.lock().unwrap();
            if let Some(set) = instances.get_mut(&set_ref) {
                let result = set.insert(element);
                Ok(result.to_string())
            } else {
                Err(EidosError::Runtime(format!("ハッシュセット参照 '{}' は無効です", set_ref)))
            }
        },
        "HashSet::contains" => {
            if values.len() != 2 {
                return Err(EidosError::Runtime("HashSet::containsにはセットと要素が必要です".to_string()));
            }
            
            // ハッシュセット参照を取得
            let set_ref = match &values[0] {
                Value::HashSet(id) => id.clone(),
                _ => return Err(EidosError::Runtime("第1引数はハッシュセット参照である必要があります".to_string())),
            };
            
            // 要素を取得
            let element = values[1].to_string();
            
            // 要素が含まれているか確認
            let instances = HASHSET_INSTANCES.lock().unwrap();
            if let Some(set) = instances.get(&set_ref) {
                Ok(set.contains(&element).to_string())
            } else {
                Err(EidosError::Runtime(format!("ハッシュセット参照 '{}' は無効です", set_ref)))
            }
        },
        "HashSet::remove" => {
            if values.len() != 2 {
                return Err(EidosError::Runtime("HashSet::removeにはセットと要素が必要です".to_string()));
            }
            
            // ハッシュセット参照を取得
            let set_ref = match &values[0] {
                Value::HashSet(id) => id.clone(),
                _ => return Err(EidosError::Runtime("第1引数はハッシュセット参照である必要があります".to_string())),
            };
            
            // 要素を取得
            let element = values[1].to_string();
            
            // 要素を削除
            let mut instances = HASHSET_INSTANCES.lock().unwrap();
            if let Some(set) = instances.get_mut(&set_ref) {
                let result = set.remove(&element);
                Ok(result.to_string())
            } else {
                Err(EidosError::Runtime(format!("ハッシュセット参照 '{}' は無効です", set_ref)))
            }
        },
        "HashSet::size" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("HashSet::sizeにはハッシュセットが必要です".to_string()));
            }
            
            // ハッシュセット参照を取得
            let set_ref = match &values[0] {
                Value::HashSet(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数はハッシュセット参照である必要があります".to_string())),
            };
            
            // サイズを取得
            let instances = HASHSET_INSTANCES.lock().unwrap();
            if let Some(set) = instances.get(&set_ref) {
                Ok(set.len().to_string())
            } else {
                Err(EidosError::Runtime(format!("ハッシュセット参照 '{}' は無効です", set_ref)))
            }
        },
        "HashSet::clear" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("HashSet::clearにはハッシュセットが必要です".to_string()));
            }
            
            // ハッシュセット参照を取得
            let set_ref = match &values[0] {
                Value::HashSet(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数はハッシュセット参照である必要があります".to_string())),
            };
            
            // ハッシュセットをクリア
            let mut instances = HASHSET_INSTANCES.lock().unwrap();
            if let Some(set) = instances.get_mut(&set_ref) {
                set.clear();
                Ok(set_ref)
            } else {
                Err(EidosError::Runtime(format!("ハッシュセット参照 '{}' は無効です", set_ref)))
            }
        },
        "HashSet::is_empty" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("HashSet::is_emptyにはハッシュセットが必要です".to_string()));
            }
            
            // ハッシュセット参照を取得
            let set_ref = match &values[0] {
                Value::HashSet(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数はハッシュセット参照である必要があります".to_string())),
            };
            
            // 空かどうかを確認
            let instances = HASHSET_INSTANCES.lock().unwrap();
            if let Some(set) = instances.get(&set_ref) {
                Ok(set.is_empty().to_string())
            } else {
                Err(EidosError::Runtime(format!("ハッシュセット参照 '{}' は無効です", set_ref)))
            }
        },
        
        // LinkedList関数
        "LinkedList::new" => {
            // 新しい空のLinkedListを作成
            let instance_id = generate_instance_id("linkedlist");
            
            // グローバル状態に登録
            LINKEDLIST_INSTANCES.lock().unwrap().insert(instance_id.clone(), std::collections::LinkedList::new());
            
            Ok(instance_id)
        },
        "LinkedList::push_front" => {
            if values.len() != 2 {
                return Err(EidosError::Runtime("LinkedList::push_frontにはリストと要素が必要です".to_string()));
            }
            
            // リスト参照を取得
            let list_ref = match &values[0] {
                Value::LinkedList(id) => id.clone(),
                _ => return Err(EidosError::Runtime("第1引数は連結リスト参照である必要があります".to_string())),
            };
            
            // 要素を取得
            let element = values[1].clone();
            
            // 要素を先頭に追加
            let mut instances = LINKEDLIST_INSTANCES.lock().unwrap();
            if let Some(list) = instances.get_mut(&list_ref) {
                list.push_front(element);
                Ok(list_ref)
            } else {
                Err(EidosError::Runtime(format!("連結リスト参照 '{}' は無効です", list_ref)))
            }
        },
        "LinkedList::push_back" => {
            if values.len() != 2 {
                return Err(EidosError::Runtime("LinkedList::push_backにはリストと要素が必要です".to_string()));
            }
            
            // リスト参照を取得
            let list_ref = match &values[0] {
                Value::LinkedList(id) => id.clone(),
                _ => return Err(EidosError::Runtime("第1引数は連結リスト参照である必要があります".to_string())),
            };
            
            // 要素を取得
            let element = values[1].clone();
            
            // 要素を末尾に追加
            let mut instances = LINKEDLIST_INSTANCES.lock().unwrap();
            if let Some(list) = instances.get_mut(&list_ref) {
                list.push_back(element);
                Ok(list_ref)
            } else {
                Err(EidosError::Runtime(format!("連結リスト参照 '{}' は無効です", list_ref)))
            }
        },
        "LinkedList::pop_front" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("LinkedList::pop_frontにはリストが必要です".to_string()));
            }
            
            // リスト参照を取得
            let list_ref = match &values[0] {
                Value::LinkedList(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数は連結リスト参照である必要があります".to_string())),
            };
            
            // 先頭要素を取り出す
            let mut instances = LINKEDLIST_INSTANCES.lock().unwrap();
            if let Some(list) = instances.get_mut(&list_ref) {
                if let Some(value) = list.pop_front() {
                    Ok(value.to_string())
                } else {
                    Err(EidosError::Runtime("リストが空です".to_string()))
                }
            } else {
                Err(EidosError::Runtime(format!("連結リスト参照 '{}' は無効です", list_ref)))
            }
        },
        "LinkedList::pop_back" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("LinkedList::pop_backにはリストが必要です".to_string()));
            }
            
            // リスト参照を取得
            let list_ref = match &values[0] {
                Value::LinkedList(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数は連結リスト参照である必要があります".to_string())),
            };
            
            // 末尾要素を取り出す
            let mut instances = LINKEDLIST_INSTANCES.lock().unwrap();
            if let Some(list) = instances.get_mut(&list_ref) {
                if let Some(value) = list.pop_back() {
                    Ok(value.to_string())
                } else {
                    Err(EidosError::Runtime("リストが空です".to_string()))
                }
            } else {
                Err(EidosError::Runtime(format!("連結リスト参照 '{}' は無効です", list_ref)))
            }
        },
        "LinkedList::length" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("LinkedList::lengthにはリストが必要です".to_string()));
            }
            
            // リスト参照を取得
            let list_ref = match &values[0] {
                Value::LinkedList(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数は連結リスト参照である必要があります".to_string())),
            };
            
            // 長さを取得
            let instances = LINKEDLIST_INSTANCES.lock().unwrap();
            if let Some(list) = instances.get(&list_ref) {
                Ok(list.len().to_string())
            } else {
                Err(EidosError::Runtime(format!("連結リスト参照 '{}' は無効です", list_ref)))
            }
        },
        "LinkedList::clear" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("LinkedList::clearにはリストが必要です".to_string()));
            }
            
            // リスト参照を取得
            let list_ref = match &values[0] {
                Value::LinkedList(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数は連結リスト参照である必要があります".to_string())),
            };
            
            // リストをクリア
            let mut instances = LINKEDLIST_INSTANCES.lock().unwrap();
            if let Some(list) = instances.get_mut(&list_ref) {
                list.clear();
                Ok(list_ref)
            } else {
                Err(EidosError::Runtime(format!("連結リスト参照 '{}' は無効です", list_ref)))
            }
        },
        "LinkedList::is_empty" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("LinkedList::is_emptyにはリストが必要です".to_string()));
            }
            
            // リスト参照を取得
            let list_ref = match &values[0] {
                Value::LinkedList(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数は連結リスト参照である必要があります".to_string())),
            };
            
            // 空かどうかを確認
            let instances = LINKEDLIST_INSTANCES.lock().unwrap();
            if let Some(list) = instances.get(&list_ref) {
                Ok(list.is_empty().to_string())
            } else {
                Err(EidosError::Runtime(format!("連結リスト参照 '{}' は無効です", list_ref)))
            }
        },
        
        // Queue関数
        "Queue::new" => {
            // 新しいQueueを作成
            let instance_id = generate_instance_id("queue");
            
            // グローバル状態に登録
            QUEUE_INSTANCES.lock().unwrap().insert(instance_id.clone(), std::collections::VecDeque::new());
            
            Ok(instance_id)
        },
        "Queue::enqueue" => {
            if values.len() != 2 {
                return Err(EidosError::Runtime("Queue::enqueueにはキューと要素が必要です".to_string()));
            }
            
            // キュー参照を取得
            let queue_ref = match &values[0] {
                Value::Queue(id) => id.clone(),
                _ => return Err(EidosError::Runtime("第1引数はキュー参照である必要があります".to_string())),
            };
            
            // 要素を取得
            let element = values[1].clone();
            
            // 要素をエンキュー
            let mut instances = QUEUE_INSTANCES.lock().unwrap();
            if let Some(queue) = instances.get_mut(&queue_ref) {
                queue.push_back(element);
                Ok(queue_ref)
            } else {
                Err(EidosError::Runtime(format!("キュー参照 '{}' は無効です", queue_ref)))
            }
        },
        "Queue::dequeue" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("Queue::dequeueにはキューが必要です".to_string()));
            }
            
            // キュー参照を取得
            let queue_ref = match &values[0] {
                Value::Queue(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数はキュー参照である必要があります".to_string())),
            };
            
            // 要素をデキュー
            let mut instances = QUEUE_INSTANCES.lock().unwrap();
            if let Some(queue) = instances.get_mut(&queue_ref) {
                if let Some(value) = queue.pop_front() {
                    Ok(value.to_string())
                } else {
                    Err(EidosError::Runtime("キューが空です".to_string()))
                }
            } else {
                Err(EidosError::Runtime(format!("キュー参照 '{}' は無効です", queue_ref)))
            }
        },
        "Queue::peek" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("Queue::peekにはキューが必要です".to_string()));
            }
            
            // キュー参照を取得
            let queue_ref = match &values[0] {
                Value::Queue(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数はキュー参照である必要があります".to_string())),
            };
            
            // 先頭要素を覗き見
            let instances = QUEUE_INSTANCES.lock().unwrap();
            if let Some(queue) = instances.get(&queue_ref) {
                if let Some(value) = queue.front() {
                    Ok(value.to_string())
                } else {
                    Err(EidosError::Runtime("キューが空です".to_string()))
                }
            } else {
                Err(EidosError::Runtime(format!("キュー参照 '{}' は無効です", queue_ref)))
            }
        },
        "Queue::length" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("Queue::lengthにはキューが必要です".to_string()));
            }
            
            // キュー参照を取得
            let queue_ref = match &values[0] {
                Value::Queue(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数はキュー参照である必要があります".to_string())),
            };
            
            // 長さを取得
            let instances = QUEUE_INSTANCES.lock().unwrap();
            if let Some(queue) = instances.get(&queue_ref) {
                Ok(queue.len().to_string())
            } else {
                Err(EidosError::Runtime(format!("キュー参照 '{}' は無効です", queue_ref)))
            }
        },
        "Queue::clear" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("Queue::clearにはキューが必要です".to_string()));
            }
            
            // キュー参照を取得
            let queue_ref = match &values[0] {
                Value::Queue(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数はキュー参照である必要があります".to_string())),
            };
            
            // キューをクリア
            let mut instances = QUEUE_INSTANCES.lock().unwrap();
            if let Some(queue) = instances.get_mut(&queue_ref) {
                queue.clear();
                Ok(queue_ref)
            } else {
                Err(EidosError::Runtime(format!("キュー参照 '{}' は無効です", queue_ref)))
            }
        },
        "Queue::is_empty" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("Queue::is_emptyにはキューが必要です".to_string()));
            }
            
            // キュー参照を取得
            let queue_ref = match &values[0] {
                Value::Queue(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数はキュー参照である必要があります".to_string())),
            };
            
            // 空かどうかを確認
            let instances = QUEUE_INSTANCES.lock().unwrap();
            if let Some(queue) = instances.get(&queue_ref) {
                Ok(queue.is_empty().to_string())
            } else {
                Err(EidosError::Runtime(format!("キュー参照 '{}' は無効です", queue_ref)))
            }
        },
        
        // Stack関数
        "Stack::new" => {
            // 新しいStackを作成
            let instance_id = generate_instance_id("stack");
            
            // グローバル状態に登録
            STACK_INSTANCES.lock().unwrap().insert(instance_id.clone(), Vec::new());
            
            Ok(instance_id)
        },
        "Stack::push" => {
            if values.len() != 2 {
                return Err(EidosError::Runtime("Stack::pushにはスタックと要素が必要です".to_string()));
            }
            
            // スタック参照を取得
            let stack_ref = match &values[0] {
                Value::Stack(id) => id.clone(),
                _ => return Err(EidosError::Runtime("第1引数はスタック参照である必要があります".to_string())),
            };
            
            // 要素を取得
            let element = values[1].clone();
            
            // 要素をプッシュ
            let mut instances = STACK_INSTANCES.lock().unwrap();
            if let Some(stack) = instances.get_mut(&stack_ref) {
                stack.push(element);
                Ok(stack_ref)
            } else {
                Err(EidosError::Runtime(format!("スタック参照 '{}' は無効です", stack_ref)))
            }
        },
        "Stack::pop" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("Stack::popにはスタックが必要です".to_string()));
            }
            
            // スタック参照を取得
            let stack_ref = match &values[0] {
                Value::Stack(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数はスタック参照である必要があります".to_string())),
            };
            
            // 要素をポップ
            let mut instances = STACK_INSTANCES.lock().unwrap();
            if let Some(stack) = instances.get_mut(&stack_ref) {
                if let Some(value) = stack.pop() {
                    Ok(value.to_string())
                } else {
                    Err(EidosError::Runtime("スタックが空です".to_string()))
                }
            } else {
                Err(EidosError::Runtime(format!("スタック参照 '{}' は無効です", stack_ref)))
            }
        },
        "Stack::peek" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("Stack::peekにはスタックが必要です".to_string()));
            }
            
            // スタック参照を取得
            let stack_ref = match &values[0] {
                Value::Stack(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数はスタック参照である必要があります".to_string())),
            };
            
            // 先頭要素を覗き見
            let instances = STACK_INSTANCES.lock().unwrap();
            if let Some(stack) = instances.get(&stack_ref) {
                if let Some(value) = stack.last() {
                    Ok(value.to_string())
                } else {
                    Err(EidosError::Runtime("スタックが空です".to_string()))
                }
            } else {
                Err(EidosError::Runtime(format!("スタック参照 '{}' は無効です", stack_ref)))
            }
        },
        "Stack::length" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("Stack::lengthにはスタックが必要です".to_string()));
            }
            
            // スタック参照を取得
            let stack_ref = match &values[0] {
                Value::Stack(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数はスタック参照である必要があります".to_string())),
            };
            
            // 長さを取得
            let instances = STACK_INSTANCES.lock().unwrap();
            if let Some(stack) = instances.get(&stack_ref) {
                Ok(stack.len().to_string())
            } else {
                Err(EidosError::Runtime(format!("スタック参照 '{}' は無効です", stack_ref)))
            }
        },
        "Stack::clear" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("Stack::clearにはスタックが必要です".to_string()));
            }
            
            // スタック参照を取得
            let stack_ref = match &values[0] {
                Value::Stack(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数はスタック参照である必要があります".to_string())),
            };
            
            // スタックをクリア
            let mut instances = STACK_INSTANCES.lock().unwrap();
            if let Some(stack) = instances.get_mut(&stack_ref) {
                stack.clear();
                Ok(stack_ref)
            } else {
                Err(EidosError::Runtime(format!("スタック参照 '{}' は無効です", stack_ref)))
            }
        },
        "Stack::is_empty" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("Stack::is_emptyにはスタックが必要です".to_string()));
            }
            
            // スタック参照を取得
            let stack_ref = match &values[0] {
                Value::Stack(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数はスタック参照である必要があります".to_string())),
            };
            
            // 空かどうかを確認
            let instances = STACK_INSTANCES.lock().unwrap();
            if let Some(stack) = instances.get(&stack_ref) {
                Ok(stack.is_empty().to_string())
            } else {
                Err(EidosError::Runtime(format!("スタック参照 '{}' は無効です", stack_ref)))
            }
        },
        
        // PriorityQueue関数
        "PriorityQueue::new" => {
            // 新しいPriorityQueueを作成
            let instance_id = generate_instance_id("priorityqueue");
            
            // グローバル状態に登録
            PRIORITYQUEUE_INSTANCES.lock().unwrap().insert(instance_id.clone(), BinaryHeap::new());
            
            Ok(instance_id)
        },
        "PriorityQueue::push" => {
            if values.len() != 3 {
                return Err(EidosError::Runtime("PriorityQueue::pushには優先度キュー、要素、優先度が必要です".to_string()));
            }
            
            // 優先度キュー参照を取得
            let pq_ref = match &values[0] {
                Value::PriorityQueue(id) => id.clone(),
                _ => return Err(EidosError::Runtime("第1引数は優先度キュー参照である必要があります".to_string())),
            };
            
            // 要素と優先度を取得
            let element = values[1].clone();
            let priority = match &values[2] {
                Value::Int(i) => *i,
                _ => return Err(EidosError::Runtime("優先度は整数である必要があります".to_string())),
            };
            
            // 要素を追加
            let mut instances = PRIORITYQUEUE_INSTANCES.lock().unwrap();
            if let Some(pq) = instances.get_mut(&pq_ref) {
                pq.push(PriorityItem { priority, value: element });
                Ok(pq_ref)
            } else {
                Err(EidosError::Runtime(format!("優先度キュー参照 '{}' は無効です", pq_ref)))
            }
        },
        "PriorityQueue::pop" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("PriorityQueue::popには優先度キューが必要です".to_string()));
            }
            
            // 優先度キュー参照を取得
            let pq_ref = match &values[0] {
                Value::PriorityQueue(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数は優先度キュー参照である必要があります".to_string())),
            };
            
            // 要素を取り出す
            let mut instances = PRIORITYQUEUE_INSTANCES.lock().unwrap();
            if let Some(pq) = instances.get_mut(&pq_ref) {
                if let Some(item) = pq.pop() {
                    Ok(item.value.to_string())
                } else {
                    Err(EidosError::Runtime("優先度キューが空です".to_string()))
                }
            } else {
                Err(EidosError::Runtime(format!("優先度キュー参照 '{}' は無効です", pq_ref)))
            }
        },
        "PriorityQueue::peek" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("PriorityQueue::peekには優先度キューが必要です".to_string()));
            }
            
            // 優先度キュー参照を取得
            let pq_ref = match &values[0] {
                Value::PriorityQueue(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数は優先度キュー参照である必要があります".to_string())),
            };
            
            // 先頭要素を覗き見
            let instances = PRIORITYQUEUE_INSTANCES.lock().unwrap();
            if let Some(pq) = instances.get(&pq_ref) {
                if let Some(item) = pq.peek() {
                    Ok(item.value.to_string())
                } else {
                    Err(EidosError::Runtime("優先度キューが空です".to_string()))
                }
            } else {
                Err(EidosError::Runtime(format!("優先度キュー参照 '{}' は無効です", pq_ref)))
            }
        },
        "PriorityQueue::length" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("PriorityQueue::lengthには優先度キューが必要です".to_string()));
            }
            
            // 優先度キュー参照を取得
            let pq_ref = match &values[0] {
                Value::PriorityQueue(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数は優先度キュー参照である必要があります".to_string())),
            };
            
            // 長さを取得
            let instances = PRIORITYQUEUE_INSTANCES.lock().unwrap();
            if let Some(pq) = instances.get(&pq_ref) {
                Ok(pq.len().to_string())
            } else {
                Err(EidosError::Runtime(format!("優先度キュー参照 '{}' は無効です", pq_ref)))
            }
        },
        "PriorityQueue::clear" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("PriorityQueue::clearには優先度キューが必要です".to_string()));
            }
            
            // 優先度キュー参照を取得
            let pq_ref = match &values[0] {
                Value::PriorityQueue(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数は優先度キュー参照である必要があります".to_string())),
            };
            
            // 優先度キューをクリア
            let mut instances = PRIORITYQUEUE_INSTANCES.lock().unwrap();
            if let Some(pq) = instances.get_mut(&pq_ref) {
                pq.clear();
                Ok(pq_ref)
            } else {
                Err(EidosError::Runtime(format!("優先度キュー参照 '{}' は無効です", pq_ref)))
            }
        },
        "PriorityQueue::is_empty" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("PriorityQueue::is_emptyには優先度キューが必要です".to_string()));
            }
            
            // 優先度キュー参照を取得
            let pq_ref = match &values[0] {
                Value::PriorityQueue(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数は優先度キュー参照である必要があります".to_string())),
            };
            
            // 空かどうかを確認
            let instances = PRIORITYQUEUE_INSTANCES.lock().unwrap();
            if let Some(pq) = instances.get(&pq_ref) {
                Ok(pq.is_empty().to_string())
            } else {
                Err(EidosError::Runtime(format!("優先度キュー参照 '{}' は無効です", pq_ref)))
            }
        },
        
        // コレクション変換関数
        "Vector::from_list" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("Vector::from_listにはリストが必要です".to_string()));
            }
            
            // リスト参照を取得
            let list_ref = match &values[0] {
                Value::LinkedList(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数は連結リスト参照である必要があります".to_string())),
            };
            
            // 新しいベクターを作成
            let vector_id = generate_instance_id("vector");
            let mut vector = Vec::new();
            
            // リストの要素をベクターにコピー
            let instances = LINKEDLIST_INSTANCES.lock().unwrap();
            if let Some(list) = instances.get(&list_ref) {
                for item in list.iter() {
                    vector.push(item.clone());
                }
                
                // ベクターを登録
                VECTOR_INSTANCES.lock().unwrap().insert(vector_id.clone(), vector);
                Ok(vector_id)
            } else {
                Err(EidosError::Runtime(format!("連結リスト参照 '{}' は無効です", list_ref)))
            }
        },
        "LinkedList::from_vector" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("LinkedList::from_vectorにはベクターが必要です".to_string()));
            }
            
            // ベクター参照を取得
            let vector_ref = match &values[0] {
                Value::Vector(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数はベクター参照である必要があります".to_string())),
            };
            
            // 新しいリンクドリストを作成
            let list_id = generate_instance_id("linkedlist");
            let mut list = std::collections::LinkedList::new();
            
            // ベクターの要素をリストにコピー
            let instances = VECTOR_INSTANCES.lock().unwrap();
            if let Some(vector) = instances.get(&vector_ref) {
                for item in vector {
                    list.push_back(item.clone());
                }
                
                // リストを登録
                LINKEDLIST_INSTANCES.lock().unwrap().insert(list_id.clone(), list);
                Ok(list_id)
            } else {
                Err(EidosError::Runtime(format!("ベクター参照 '{}' は無効です", vector_ref)))
            }
        },
        "HashSet::from_vector" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("HashSet::from_vectorにはベクターが必要です".to_string()));
            }
            
            // ベクター参照を取得
            let vector_ref = match &values[0] {
                Value::Vector(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数はベクター参照である必要があります".to_string())),
            };
            
            // 新しいハッシュセットを作成
            let set_id = generate_instance_id("hashset");
            let mut set = std::collections::HashSet::new();
            
            // ベクターの要素をセットに追加
            let instances = VECTOR_INSTANCES.lock().unwrap();
            if let Some(vector) = instances.get(&vector_ref) {
                for item in vector {
                    // HashSetは文字列のみを扱うため、すべての値を文字列に変換
                    set.insert(item.to_string());
                }
                
                // セットを登録
                HASHSET_INSTANCES.lock().unwrap().insert(set_id.clone(), set);
                Ok(set_id)
            } else {
                Err(EidosError::Runtime(format!("ベクター参照 '{}' は無効です", vector_ref)))
            }
        },
        "Vector::from_hashset" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("Vector::from_hashsetにはハッシュセットが必要です".to_string()));
            }
            
            // ハッシュセット参照を取得
            let set_ref = match &values[0] {
                Value::HashSet(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数はハッシュセット参照である必要があります".to_string())),
            };
            
            // 新しいベクターを作成
            let vector_id = generate_instance_id("vector");
            let mut vector = Vec::new();
            
            // セットの要素をベクターに追加
            let instances = HASHSET_INSTANCES.lock().unwrap();
            if let Some(set) = instances.get(&set_ref) {
                for item in set {
                    // 文字列をValueに変換
                    vector.push(Value::String(item.clone()));
                }
                
                // ベクターを登録
                VECTOR_INSTANCES.lock().unwrap().insert(vector_id.clone(), vector);
                Ok(vector_id)
            } else {
                Err(EidosError::Runtime(format!("ハッシュセット参照 '{}' は無効です", set_ref)))
            }
        },
        "Queue::from_vector" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("Queue::from_vectorにはベクターが必要です".to_string()));
            }
            
            // ベクター参照を取得
            let vector_ref = match &values[0] {
                Value::Vector(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数はベクター参照である必要があります".to_string())),
            };
            
            // 新しいキューを作成
            let queue_id = generate_instance_id("queue");
            let mut queue = std::collections::VecDeque::new();
            
            // ベクターの要素をキューに追加
            let instances = VECTOR_INSTANCES.lock().unwrap();
            if let Some(vector) = instances.get(&vector_ref) {
                for item in vector {
                    queue.push_back(item.clone());
                }
                
                // キューを登録
                QUEUE_INSTANCES.lock().unwrap().insert(queue_id.clone(), queue);
                Ok(queue_id)
            } else {
                Err(EidosError::Runtime(format!("ベクター参照 '{}' は無効です", vector_ref)))
            }
        },
        "Vector::from_queue" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("Vector::from_queueにはキューが必要です".to_string()));
            }
            
            // キュー参照を取得
            let queue_ref = match &values[0] {
                Value::Queue(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数はキュー参照である必要があります".to_string())),
            };
            
            // 新しいベクターを作成
            let vector_id = generate_instance_id("vector");
            let mut vector = Vec::new();
            
            // キューの要素をベクターにコピー（順序を保持）
            let instances = QUEUE_INSTANCES.lock().unwrap();
            if let Some(queue) = instances.get(&queue_ref) {
                for item in queue.iter() {
                    vector.push(item.clone());
                }
                
                // ベクターを登録
                VECTOR_INSTANCES.lock().unwrap().insert(vector_id.clone(), vector);
                Ok(vector_id)
            } else {
                Err(EidosError::Runtime(format!("キュー参照 '{}' は無効です", queue_ref)))
            }
        },
        "Stack::from_vector" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("Stack::from_vectorにはベクターが必要です".to_string()));
            }
            
            // ベクター参照を取得
            let vector_ref = match &values[0] {
                Value::Vector(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数はベクター参照である必要があります".to_string())),
            };
            
            // 新しいスタックを作成
            let stack_id = generate_instance_id("stack");
            let mut stack = Vec::new();
            
            // ベクターの要素をスタックにコピー
            let instances = VECTOR_INSTANCES.lock().unwrap();
            if let Some(vector) = instances.get(&vector_ref) {
                for item in vector {
                    stack.push(item.clone());
                }
                
                // スタックを登録
                STACK_INSTANCES.lock().unwrap().insert(stack_id.clone(), stack);
                Ok(stack_id)
            } else {
                Err(EidosError::Runtime(format!("ベクター参照 '{}' は無効です", vector_ref)))
            }
        },
        "Vector::from_stack" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("Vector::from_stackにはスタックが必要です".to_string()));
            }
            
            // スタック参照を取得
            let stack_ref = match &values[0] {
                Value::Stack(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数はスタック参照である必要があります".to_string())),
            };
            
            // 新しいベクターを作成
            let vector_id = generate_instance_id("vector");
            let mut vector = Vec::new();
            
            // スタックの要素をベクターにコピー
            let instances = STACK_INSTANCES.lock().unwrap();
            if let Some(stack) = instances.get(&stack_ref) {
                for item in stack {
                    vector.push(item.clone());
                }
                
                // ベクターを登録
                VECTOR_INSTANCES.lock().unwrap().insert(vector_id.clone(), vector);
                Ok(vector_id)
            } else {
                Err(EidosError::Runtime(format!("スタック参照 '{}' は無効です", stack_ref)))
            }
        },
        
        // コレクション一般操作
        "Collection::clone" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("Collection::cloneにはコレクション参照が必要です".to_string()));
            }
            
            // コレクション参照を取得
            let collection_id = match &values[0] {
                Value::Vector(id) | Value::HashMap(id) | Value::HashSet(id) | 
                Value::LinkedList(id) | Value::Queue(id) | Value::Stack(id) |
                Value::PriorityQueue(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数はコレクション参照である必要があります".to_string())),
            };
            
            // コレクションをクローン
            clone_collection(&collection_id)
        },
        "Collection::drop" => {
            if values.len() != 1 {
                return Err(EidosError::Runtime("Collection::dropにはコレクション参照が必要です".to_string()));
            }
            
            // コレクション参照を取得
            let collection_id = match &values[0] {
                Value::Vector(id) | Value::HashMap(id) | Value::HashSet(id) | 
                Value::LinkedList(id) | Value::Queue(id) | Value::Stack(id) |
                Value::PriorityQueue(id) => id.clone(),
                _ => return Err(EidosError::Runtime("引数はコレクション参照である必要があります".to_string())),
            };
            
            // コレクションを削除
            CollectionManager::remove_instance(&collection_id)?;
            Ok("null".to_string())
        },
        
        // その他のコレクション関数
        _ => Err(EidosError::Runtime(format!("未知のコレクション関数: {}", function_name))),
    }
} 

// コレクションのメモリ管理
pub struct CollectionManager;

impl CollectionManager {
    /// インスタンスが有効かどうかを確認
    pub fn is_valid_instance(instance_id: &str) -> bool {
        if instance_id.starts_with("vector:") {
            VECTOR_INSTANCES.lock().unwrap().contains_key(instance_id)
        } else if instance_id.starts_with("hashmap:") {
            HASHMAP_INSTANCES.lock().unwrap().contains_key(instance_id)
        } else if instance_id.starts_with("hashset:") {
            HASHSET_INSTANCES.lock().unwrap().contains_key(instance_id)
        } else if instance_id.starts_with("linkedlist:") {
            LINKEDLIST_INSTANCES.lock().unwrap().contains_key(instance_id)
        } else if instance_id.starts_with("queue:") {
            QUEUE_INSTANCES.lock().unwrap().contains_key(instance_id)
        } else if instance_id.starts_with("stack:") {
            STACK_INSTANCES.lock().unwrap().contains_key(instance_id)
        } else if instance_id.starts_with("priorityqueue:") {
            PRIORITYQUEUE_INSTANCES.lock().unwrap().contains_key(instance_id)
        } else {
            false
        }
    }
    
    /// インスタンスを削除
    pub fn remove_instance(instance_id: &str) -> Result<()> {
        // 参照カウントを減らし、0になったら削除
        if decrement_reference_count(instance_id) {
            if instance_id.starts_with("vector:") {
                VECTOR_INSTANCES.lock().unwrap().remove(instance_id);
            } else if instance_id.starts_with("hashmap:") {
                HASHMAP_INSTANCES.lock().unwrap().remove(instance_id);
            } else if instance_id.starts_with("hashset:") {
                HASHSET_INSTANCES.lock().unwrap().remove(instance_id);
            } else if instance_id.starts_with("linkedlist:") {
                LINKEDLIST_INSTANCES.lock().unwrap().remove(instance_id);
            } else if instance_id.starts_with("queue:") {
                QUEUE_INSTANCES.lock().unwrap().remove(instance_id);
            } else if instance_id.starts_with("stack:") {
                STACK_INSTANCES.lock().unwrap().remove(instance_id);
            } else if instance_id.starts_with("priorityqueue:") {
                PRIORITYQUEUE_INSTANCES.lock().unwrap().remove(instance_id);
            } else {
                return Err(EidosError::Runtime(format!("無効なコレクション参照: {}", instance_id)));
            }
        }
        
        Ok(())
    }
    
    /// 全てのインスタンスをクリア（テスト用）
    #[cfg(test)]
    pub fn clear_all() {
        VECTOR_INSTANCES.lock().unwrap().clear();
        HASHMAP_INSTANCES.lock().unwrap().clear();
        HASHSET_INSTANCES.lock().unwrap().clear();
        LINKEDLIST_INSTANCES.lock().unwrap().clear();
        QUEUE_INSTANCES.lock().unwrap().clear();
        STACK_INSTANCES.lock().unwrap().clear();
        PRIORITYQUEUE_INSTANCES.lock().unwrap().clear();
        COLLECTION_REF_COUNTS.lock().unwrap().clear();
    }
    
    /// コレクションの内容をダンプ（デバッグ用）
    #[cfg(debug_assertions)]
    pub fn dump_collection(instance_id: &str) -> Result<String> {
        if instance_id.starts_with("vector:") {
            let instances = VECTOR_INSTANCES.lock().unwrap();
            if let Some(vector) = instances.get(instance_id) {
                let elements: Vec<String> = vector.iter().map(|v| v.to_string()).collect();
                Ok(format!("[{}]", elements.join(", ")))
            } else {
                Err(EidosError::Runtime(format!("無効なベクター参照: {}", instance_id)))
            }
        } else if instance_id.starts_with("hashmap:") {
            let instances = HASHMAP_INSTANCES.lock().unwrap();
            if let Some(map) = instances.get(instance_id) {
                let mut entries = Vec::new();
                for (k, v) in map {
                    entries.push(format!("{}: {}", k, v.to_string()));
                }
                Ok(format!("{{{}}}", entries.join(", ")))
            } else {
                Err(EidosError::Runtime(format!("無効なハッシュマップ参照: {}", instance_id)))
            }
        } else {
            Err(EidosError::Runtime(format!("ダンプ未対応のコレクション: {}", instance_id)))
        }
    }
    
    /// 参照カウントを取得（デバッグ用）
    #[cfg(debug_assertions)]
    pub fn get_reference_count(instance_id: &str) -> usize {
        let ref_counts = COLLECTION_REF_COUNTS.lock().unwrap();
        *ref_counts.get(instance_id).unwrap_or(&0)
    }
}

// Value型にメソッドを追加
impl Value {
    /// このValueがコレクション参照かどうかを確認
    pub fn is_collection_reference(&self) -> bool {
        match self {
            Value::Vector(_) | Value::HashMap(_) | Value::HashSet(_) | 
            Value::LinkedList(_) | Value::Queue(_) | Value::Stack(_) | 
            Value::PriorityQueue(_) => true,
            _ => false,
        }
    }
    
    /// コレクション参照からIDを取得
    pub fn get_collection_id(&self) -> Option<String> {
        match self {
            Value::Vector(id) | Value::HashMap(id) | Value::HashSet(id) | 
            Value::LinkedList(id) | Value::Queue(id) | Value::Stack(id) | 
            Value::PriorityQueue(id) => Some(id.clone()),
            _ => None,
        }
    }
}

/// コレクションの参照カウントを増やす
fn increment_reference_count(id: &str) {
    let mut ref_counts = COLLECTION_REF_COUNTS.lock().unwrap();
    *ref_counts.entry(id.to_string()).or_insert(0) += 1;
}

/// コレクションの参照カウントを減らす
/// 0になった場合はインスタンスを削除し、trueを返す
fn decrement_reference_count(id: &str) -> bool {
    let mut ref_counts = COLLECTION_REF_COUNTS.lock().unwrap();
    
    if let Some(count) = ref_counts.get_mut(id) {
        *count -= 1;
        if *count == 0 {
            ref_counts.remove(id);
            return true;
        }
    }
    
    false
}

/// コレクションIDを生成
fn generate_instance_id(prefix: &str) -> String {
    let id = format!("{}:{}", prefix, INSTANCE_COUNTER.fetch_add(1, Ordering::SeqCst));
    // 新しく生成したIDの参照カウントを1にする
    increment_reference_count(&id);
    id
}

/// コレクションインスタンスのクローンを作成
pub fn clone_collection(collection_id: &str) -> Result<String> {
    if !CollectionManager::is_valid_instance(collection_id) {
        return Err(EidosError::Runtime(format!("無効なコレクション参照: {}", collection_id)));
    }
    
    // 既存のインスタンスの参照カウントを増やす
    increment_reference_count(collection_id);
    
    Ok(collection_id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // テスト開始前にコレクションをクリア
    fn setup() {
        CollectionManager::clear_all();
    }
    
    #[test]
    fn test_vector_operations() {
        setup();
        
        // 新しいベクターを作成
        let vector_id = execute_function("Vector::new", &[]).unwrap();
        assert!(vector_id.starts_with("vector:"));
        
        // 要素を追加
        let _ = execute_function("Vector::push", &[vector_id.clone(), "42".to_string()]).unwrap();
        let _ = execute_function("Vector::push", &[vector_id.clone(), "hello".to_string()]).unwrap();
        let _ = execute_function("Vector::push", &[vector_id.clone(), "true".to_string()]).unwrap();
        
        // 長さを確認
        let length = execute_function("Vector::length", &[vector_id.clone()]).unwrap();
        assert_eq!(length, "3");
        
        // 要素を取得
        let item = execute_function("Vector::get", &[vector_id.clone(), "0".to_string()]).unwrap();
        assert_eq!(item, "42");
        let item = execute_function("Vector::get", &[vector_id.clone(), "1".to_string()]).unwrap();
        assert_eq!(item, "hello");
        
        // 要素をポップ
        let item = execute_function("Vector::pop", &[vector_id.clone()]).unwrap();
        assert_eq!(item, "true");
        
        // 長さを再確認
        let length = execute_function("Vector::length", &[vector_id.clone()]).unwrap();
        assert_eq!(length, "2");
        
        // クリア
        let _ = execute_function("Vector::clear", &[vector_id.clone()]).unwrap();
        let is_empty = execute_function("Vector::is_empty", &[vector_id.clone()]).unwrap();
        assert_eq!(is_empty, "true");
    }
    
    #[test]
    fn test_hashmap_operations() {
        setup();
        
        // 新しいハッシュマップを作成
        let map_id = execute_function("HashMap::new", &[]).unwrap();
        assert!(map_id.starts_with("hashmap:"));
        
        // キーと値を追加
        let _ = execute_function("HashMap::insert", &[map_id.clone(), "key1".to_string(), "value1".to_string()]).unwrap();
        let _ = execute_function("HashMap::insert", &[map_id.clone(), "key2".to_string(), "42".to_string()]).unwrap();
        
        // 要素を取得
        let value = execute_function("HashMap::get", &[map_id.clone(), "key1".to_string()]).unwrap();
        assert_eq!(value, "value1");
        
        // 存在確認
        let contains = execute_function("HashMap::contains_key", &[map_id.clone(), "key2".to_string()]).unwrap();
        assert_eq!(contains, "true");
        let contains = execute_function("HashMap::contains_key", &[map_id.clone(), "key3".to_string()]).unwrap();
        assert_eq!(contains, "false");
        
        // 要素を削除
        let _ = execute_function("HashMap::remove", &[map_id.clone(), "key1".to_string()]).unwrap();
        let contains = execute_function("HashMap::contains_key", &[map_id.clone(), "key1".to_string()]).unwrap();
        assert_eq!(contains, "false");
        
        // 長さを確認
        let length = execute_function("HashMap::length", &[map_id.clone()]).unwrap();
        assert_eq!(length, "1");
    }
    
    #[test]
    fn test_hashset_operations() {
        setup();
        
        // 新しいハッシュセットを作成
        let set_id = execute_function("HashSet::new", &[]).unwrap();
        assert!(set_id.starts_with("hashset:"));
        
        // 要素を追加
        let _ = execute_function("HashSet::add", &[set_id.clone(), "apple".to_string()]).unwrap();
        let _ = execute_function("HashSet::add", &[set_id.clone(), "banana".to_string()]).unwrap();
        let _ = execute_function("HashSet::add", &[set_id.clone(), "apple".to_string()]).unwrap(); // 重複
        
        // 要素数を確認（重複は追加されない）
        let size = execute_function("HashSet::size", &[set_id.clone()]).unwrap();
        assert_eq!(size, "2");
        
        // 存在確認
        let contains = execute_function("HashSet::contains", &[set_id.clone(), "apple".to_string()]).unwrap();
        assert_eq!(contains, "true");
        let contains = execute_function("HashSet::contains", &[set_id.clone(), "orange".to_string()]).unwrap();
        assert_eq!(contains, "false");
        
        // 要素を削除
        let _ = execute_function("HashSet::remove", &[set_id.clone(), "apple".to_string()]).unwrap();
        let contains = execute_function("HashSet::contains", &[set_id.clone(), "apple".to_string()]).unwrap();
        assert_eq!(contains, "false");
    }
    
    #[test]
    fn test_reference_counting() {
        setup();
        
        // 新しいベクターを作成
        let vector_id = execute_function("Vector::new", &[]).unwrap();
        
        // クローンを作成（参照カウント増加）
        let clone_id = execute_function("Collection::clone", &[vector_id.clone()]).unwrap();
        assert_eq!(vector_id, clone_id); // 同じIDを返す
        
        // 最初の参照を削除しても、まだインスタンスは存在する
        let _ = execute_function("Collection::drop", &[vector_id.clone()]).unwrap();
        
        // クローンを通じてまだアクセス可能
        let is_empty = execute_function("Vector::is_empty", &[clone_id.clone()]).unwrap();
        assert_eq!(is_empty, "true");
        
        // クローンも削除すると、インスタンスは完全に削除される
        let _ = execute_function("Collection::drop", &[clone_id.clone()]).unwrap();
        
        // これ以上アクセスできない
        let result = execute_function("Vector::is_empty", &[clone_id.clone()]);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_collection_conversions() {
        setup();
        
        // ベクターを作成して要素を追加
        let vector_id = execute_function("Vector::new", &[]).unwrap();
        let _ = execute_function("Vector::push", &[vector_id.clone(), "a".to_string()]).unwrap();
        let _ = execute_function("Vector::push", &[vector_id.clone(), "b".to_string()]).unwrap();
        let _ = execute_function("Vector::push", &[vector_id.clone(), "c".to_string()]).unwrap();
        
        // ベクターからリンクドリストに変換
        let list_id = execute_function("LinkedList::from_vector", &[vector_id.clone()]).unwrap();
        assert!(list_id.starts_with("linkedlist:"));
        
        // リンクドリストからベクターに変換
        let vector2_id = execute_function("Vector::from_list", &[list_id.clone()]).unwrap();
        
        // 変換後のベクターの長さを確認
        let length = execute_function("Vector::length", &[vector2_id.clone()]).unwrap();
        assert_eq!(length, "3");
        
        // ベクターからハッシュセットに変換
        let set_id = execute_function("HashSet::from_vector", &[vector_id.clone()]).unwrap();
        
        // セットの要素数を確認
        let size = execute_function("HashSet::size", &[set_id.clone()]).unwrap();
        assert_eq!(size, "3");
        
        // 重複要素を持つベクターからハッシュセットに変換
        let vector3_id = execute_function("Vector::new", &[]).unwrap();
        let _ = execute_function("Vector::push", &[vector3_id.clone(), "a".to_string()]).unwrap();
        let _ = execute_function("Vector::push", &[vector3_id.clone(), "a".to_string()]).unwrap(); // 重複
        
        let set2_id = execute_function("HashSet::from_vector", &[vector3_id.clone()]).unwrap();
        let size = execute_function("HashSet::size", &[set2_id.clone()]).unwrap();
        assert_eq!(size, "1"); // 重複は削除される
    }
}