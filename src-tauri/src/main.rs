// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod error;
mod parser;

use std::{collections::HashMap, sync::Mutex};
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

#[derive(Debug, Default)]
pub struct GlobalState {
    variables: Mutex<HashMap<String, f64>>,
}

#[tauri::command]
fn process(eq: &str, state: State<GlobalState>) -> error::Result<Response> {
    let vars = state.variables.lock()
        .map_err(|_| AppError::IoError("Couldn't access to the variable map".to_owned()))?;

    let mut root = parse_latex(eq).or_else(|e| { 
        warn!("{e:?}"); Err(e) 
    })?;
    root.print_tree();

    let numeric_value = simplify_tree(&mut root, &vars);
    root.print_tree();

    if numeric_value.is_some() {
        info!("Expression {eq} evaluates to {}", numeric_value.unwrap());

        Ok( Response { 
            code: String::new(), 
            num: numeric_value,
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
fn add_variable(name: &str, content: &str, state: State<GlobalState>) -> error::Result<()> {
    let mut vars = state.variables.lock()
        .map_err(|_| AppError::IoError("Couldn't access to the variable map".to_owned()))?;
    vars.remove(name);

    let mut root = parse_latex(content).or_else(|e| { 
        warn!("{e:?}"); Err(e) 
    })?;
    
    let val = simplify_tree(&mut root, &vars).ok_or_else(|| {
        warn!("The variable {name} couldn't be evaluated to a value: {content}");
        AppError::MathError(format!("The variable must evaluate to a certain value"))
    })?;
    
    vars.insert(name.to_owned(), val);

    Ok(())
}

fn main() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    debug!("Logger has been set up correctly");

    tauri::Builder::default()
        .manage(GlobalState::default() )
        .invoke_handler(tauri::generate_handler![process, add_variable])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
