// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::info;
use tauri::State;
use webgl_calculator::{error, GlobalState, Response};

#[tauri::command]
fn process(eq: &str, state: State<GlobalState>, expr_idx: usize) -> error::Result<Response> {
    webgl_calculator::process(eq, state, expr_idx)
}

#[tauri::command]
fn add_variable(name: &str, content: &str, state: State<GlobalState>) -> error::Result<f64> {
    webgl_calculator::add_variable(name, content, state)
}

#[tauri::command]
fn add_function(name: &str, content: &str, state: State<GlobalState>, expr_idx: usize) -> error::Result<Response> {
    webgl_calculator::add_function(name, content, state, expr_idx)
}

#[tauri::command]
fn delete_function(name: &str, state: State<GlobalState>) -> error::Result<()> {
    webgl_calculator::delete_function(name, state)
}

#[tauri::command]
fn delete_variable(name: &str, state: State<GlobalState>) -> error::Result<()> {
    webgl_calculator::delete_variable(name, state)
}

fn main() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    info!("Logger has been set up correctly");

    tauri::Builder::default()
        .manage(GlobalState::default() )
        .invoke_handler(tauri::generate_handler![process, add_variable, add_function, delete_function, delete_variable])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
