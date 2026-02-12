mod cli;
mod domain;
mod infra;
mod usecase;

use std::path::PathBuf;

use clap::Parser;
use cli::args::{Cli, Commands};
use cli::handler;
use domain::error::DomainError;
use infra::logger;
use infra::sqlite::{Database, SqliteMessageRepository, SqliteThreadRepository};
use usecase::cleanup::CleanupUseCase;
use usecase::hook::HookUseCase;
use usecase::message::MessageUseCase;
use usecase::thread::ThreadUseCase;

fn main() {
    let cli = Cli::parse();

    let result = run(cli);

    match result {
        Ok(()) => std::process::exit(0),
        Err(e) => {
            let (exit_code, user_msg) = classify_error(&e);
            logger::log_error(&format!("{:#}", e));
            eprintln!("エラー: {}", user_msg);
            std::process::exit(exit_code);
        }
    }
}

fn db_path() -> PathBuf {
    let data_dir = dirs_fallback();
    data_dir.join("aiboard.db")
}

fn dirs_fallback() -> PathBuf {
    if let Some(dir) = std::env::var_os("AIBOARD_DATA_DIR") {
        return PathBuf::from(dir);
    }
    if let Some(data) = std::env::var_os("LOCALAPPDATA") {
        return PathBuf::from(data).join("aiboard");
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".local").join("share").join("aiboard");
    }
    PathBuf::from(".aiboard")
}

fn run(cli: Cli) -> anyhow::Result<()> {
    let path = db_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let db = Database::open(&path)?;
    let conn = db.connection();

    let msg = || SqliteMessageRepository::new(conn);
    let thr = || SqliteThreadRepository::new(conn);

    let thread_uc = ThreadUseCase::new(thr(), msg());
    let message_uc = MessageUseCase::new(msg());
    let hook_uc = HookUseCase::new(msg());
    let cleanup_uc = CleanupUseCase::new(thr(), msg());
    let thread_uc2 = ThreadUseCase::new(thr(), msg());

    match cli.command {
        Commands::Message { action } => {
            handler::handle_message(action, &message_uc, &thread_uc2)?;
        }
        Commands::Thread { action } => {
            handler::handle_thread(action, &thread_uc)?;
        }
        Commands::Hook { action } => {
            handler::handle_hook(action, &hook_uc)?;
        }
        Commands::Cleanup { action } => {
            handler::handle_cleanup(action, &cleanup_uc)?;
        }
        Commands::Setup { action } => {
            handler::handle_setup(action)?;
        }
    }

    Ok(())
}

fn classify_error(e: &anyhow::Error) -> (i32, String) {
    if let Some(domain_err) = e.downcast_ref::<DomainError>() {
        (domain_err.exit_code(), domain_err.to_string())
    } else {
        (1, e.to_string())
    }
}
