use clap::{Parser, Subcommand};
use log::info;
use std::path::PathBuf;
use std::process;

mod frontend;
mod core;
mod dsl;
mod backend;
mod stdlib;
mod tools;

/// Eidos - 言語を作る言語
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// ログレベル
    #[clap(long, default_value = "info")]
    log_level: String,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Eidosプログラムをコンパイル
    Build {
        /// コンパイル対象のファイル
        #[clap(value_parser)]
        file: PathBuf,

        /// 最適化レベル（0-3）
        #[clap(short, long, default_value = "2")]
        opt_level: u8,

        /// 出力ファイル
        #[clap(short, long)]
        output: Option<PathBuf>,
    },
    /// インタラクティブモード（REPL）を起動
    Repl {
        /// 初期ライブラリをロード
        #[clap(short, long)]
        preload: Option<Vec<PathBuf>>,
    },
    /// 型チェックのみ実行
    Check {
        /// チェック対象のファイル
        #[clap(value_parser)]
        file: PathBuf,
    },
    /// Eidosプログラムを実行
    Run {
        /// 実行対象のファイル
        #[clap(value_parser)]
        file: PathBuf,
        
        /// コマンド引数
        #[clap(last = true)]
        args: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    
    // ロギングの初期化
    env_logger::Builder::from_env(env_logger::Env::default()
        .default_filter_or(&cli.log_level))
        .init();
    
    info!("Eidos コンパイラが起動しました");
    
    let result = match cli.command {
        Commands::Build { file, opt_level, output } => {
            info!("ビルドモード: ファイル={}, 最適化レベル={}", file.display(), opt_level);
            tools::compiler::compile_file(&file, opt_level, output)
        },
        Commands::Repl { preload } => {
            info!("REPLモード");
            tools::repl::start_repl(preload)
        },
        Commands::Check { file } => {
            info!("型チェックモード: ファイル={}", file.display());
            tools::compiler::typecheck_file(&file)
        },
        Commands::Run { file, args } => {
            info!("実行モード: ファイル={}", file.display());
            tools::runner::run_file(&file, args)
        },
    };
    
    match result {
        Ok(_) => {
            info!("処理が正常に完了しました");
            process::exit(0);
        },
        Err(e) => {
            eprintln!("エラー: {}", e);
            process::exit(1);
        }
    }
}
