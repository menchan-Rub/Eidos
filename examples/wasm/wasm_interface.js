// Eidos WebAssemblyインターフェース

// メモリとログ関数の設定
let wasmMemory;
const logBuffer = [];

// 環境関数 - コンソールへのログ出力
function console_log(ptr, len) {
  // メモリからテキストを取得
  const bytes = new Uint8Array(wasmMemory.buffer, ptr, len);
  const text = new TextDecoder('utf-8').decode(bytes);
  
  // コンソールとUIの両方にログを表示
  console.log(text);
  logBuffer.push(text);
  updateLogDisplay();
}

// UIのログ表示を更新
function updateLogDisplay() {
  const logElement = document.getElementById('log-output');
  if (logElement) {
    logElement.textContent = logBuffer.join('\n');
    // 自動スクロール
    logElement.scrollTop = logElement.scrollHeight;
  }
}

// 計算履歴をクリア
function clearLog() {
  logBuffer.length = 0;
  updateLogDisplay();
}

// 計算処理の実行
async function runCalculation(a, b, operation) {
  if (!window.wasmModule) {
    alert('WebAssemblyモジュールが読み込まれていません');
    return;
  }
  
  let result;
  
  try {
    switch (operation) {
      case 'add':
        result = window.wasmModule.exports.add(a, b);
        break;
      case 'subtract':
        result = window.wasmModule.exports.subtract(a, b);
        break;
      case 'multiply':
        result = window.wasmModule.exports.multiply(a, b);
        break;
      case 'divide':
        result = window.wasmModule.exports.divide(a, b);
        break;
      default:
        throw new Error(`未定義の演算: ${operation}`);
    }
    
    document.getElementById('result').textContent = result;
  } catch (error) {
    console.error('計算エラー:', error);
    document.getElementById('result').textContent = 'エラー';
  }
}

// WebAssemblyモジュールの読み込み
async function loadWasmModule(wasmUrl) {
  try {
    // WebAssemblyメモリの初期化（1ページ = 64KB）
    wasmMemory = new WebAssembly.Memory({ initial: 10, maximum: 100 });
    
    // インポートオブジェクトの設定
    const importObject = {
      env: {
        console_log,
        memory: wasmMemory
      }
    };
    
    // WebAssemblyモジュールの読み込みと初期化
    const response = await fetch(wasmUrl);
    const wasmBytes = await response.arrayBuffer();
    const wasmModule = await WebAssembly.instantiate(wasmBytes, importObject);
    
    // グローバル参照として保存
    window.wasmModule = wasmModule.instance;
    
    // メイン関数を実行
    const result = window.wasmModule.exports.main();
    console.log('WebAssemblyモジュール初期化完了、メイン関数の戻り値:', result);
    
    // UIを有効化
    document.getElementById('calculator-ui').classList.remove('disabled');
    document.getElementById('loading').style.display = 'none';
    
    return window.wasmModule;
  } catch (error) {
    console.error('WebAssemblyモジュールの読み込みエラー:', error);
    document.getElementById('loading').textContent = 'エラー: ' + error.message;
    throw error;
  }
}

// 計算機UIのイベント処理
function setupCalculatorUI() {
  document.getElementById('calculate').addEventListener('click', () => {
    const a = parseInt(document.getElementById('num1').value) || 0;
    const b = parseInt(document.getElementById('num2').value) || 0;
    const operation = document.getElementById('operation').value;
    
    runCalculation(a, b, operation);
  });
  
  document.getElementById('clear-log').addEventListener('click', clearLog);
}

// ページ読み込み時の初期化
window.addEventListener('DOMContentLoaded', () => {
  setupCalculatorUI();
  
  // WebAssemblyモジュールの読み込み
  loadWasmModule('simple_calculator.wasm')
    .catch(error => console.error('初期化エラー:', error));
}); 