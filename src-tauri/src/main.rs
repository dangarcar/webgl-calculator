// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod error;
mod parser;

use std::sync::Mutex;
use log::{debug, info, warn};
use tauri::State;
use std::str;
use serde::{Deserialize, Serialize};

use crate::{error::AppError, parser::{parse_latex, simplify_tree}};

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    code: String,
    num: Option<f64>
}

#[tauri::command]
fn process(eq: &str) -> error::Result<Response> {    
    let mut root = parse_latex(eq).or_else(|e| { 
        warn!("{e:?}"); Err(e) 
    })?;
    root.print_tree();

    let numeric_value = simplify_tree(&mut root);
    root.print_tree();

    if numeric_value.is_some() {
        info!("Expression {eq} evaluates to {}", numeric_value.unwrap());

        Ok( Response { 
            code: String::new(), 
            num: numeric_value 
        } )   
    } else {
        info!("Expression {eq} has been compiled to be shown");

        Ok( Response {
            code: String::from("return fneg(fsub(x*x, y));"),
            num: None
        } )
    }
}

#[tauri::command]
fn add_variable(name: char, content: &str) -> error::Result<()> {
    todo!();
    Ok(())
}

fn main() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    debug!("Logger has been set up correctly");

    tauri::Builder::default()
        /*.manage(Mem { 
            data: Mutex::new(Response { x0: Some(SIDE/2), y0: Some(SIDE/2) }) 
        })*/
        .invoke_handler(tauri::generate_handler![process, add_variable])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
