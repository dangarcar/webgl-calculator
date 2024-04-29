// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod error;
mod parser;

use std::sync::Mutex;
use tauri::State;
use std::str;
use serde::{Deserialize, Serialize};

use crate::{error::AppError, parser::parse_latex};

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    code: String,
    num: Option<f64>
}

#[tauri::command]
//TODO: make this function calculate something
fn process(eq: &str) -> error::Result<Response> {
    println!("{}", eq);
    
    parse_latex(eq)?;
    
    if eq.contains('z') { return Err(AppError::ParseError("A equation can't contain z".to_string())); }

    return Ok(Response {
        code: String::from("return fneg(fsub(x*x, y));"),
        num: if eq.contains('q') { Some(1.41) } else { None }
    });
}

#[tauri::command]
fn add_variable(name: char, content: &str) -> error::Result<()> {
    todo!();
    Ok(())
}

fn main() {
    tauri::Builder::default()
        /*.manage(Mem { 
            data: Mutex::new(Response { x0: Some(SIDE/2), y0: Some(SIDE/2) }) 
        })*/
        .invoke_handler(tauri::generate_handler![process, add_variable])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
