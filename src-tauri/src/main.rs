//! Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use log::info;
use tauri::State;
use webgl_calculator::{error::{self, AppError}, CompilerState, Response};

type TauriState = Mutex<CompilerState>;

#[tauri::command]
fn process(eq: &str, state: State<TauriState>, expr_idx: usize) -> error::Result<Response> {
    let state = state.lock().map_err(|_| AppError::IoError("Couldn't read the variables and functions table".to_owned()))?;
    webgl_calculator::process(eq, &state, expr_idx)
}

#[tauri::command]
fn add_variable(name: &str, content: &str, state: State<TauriState>) -> error::Result<f64> {
    let mut state = state.lock().map_err(|_| AppError::IoError("Couldn't read the variables and functions table".to_owned()))?;
    webgl_calculator::add_variable(name, content, &mut state)
}

#[tauri::command]
fn add_function(name: &str, content: &str, state: State<TauriState>, expr_idx: usize) -> error::Result<Response> {
    let mut state = state.lock().map_err(|_| AppError::IoError("Couldn't read the variables and functions table".to_owned()))?;
    webgl_calculator::add_function(name, content, &mut state, expr_idx)
}

#[tauri::command]
fn delete_function(name: &str, state: State<TauriState>) -> error::Result<()> {
    let mut state = state.lock().map_err(|_| AppError::IoError("Couldn't read the variables and functions table".to_owned()))?;
    webgl_calculator::delete_function(name, &mut state)
}

#[tauri::command]
fn delete_variable(name: &str, state: State<TauriState>) -> error::Result<()> {
    let mut state = state.lock().map_err(|_| AppError::IoError("Couldn't read the variables and functions table".to_owned()))?;
    webgl_calculator::delete_variable(name, &mut state)
}

fn main() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    info!("Logger has been set up correctly");

    tauri::Builder::default()
        .manage(TauriState::default() )
        .invoke_handler(tauri::generate_handler![process, add_variable, add_function, delete_function, delete_variable])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
