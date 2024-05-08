// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod error;
mod parser;
mod compiler;

use std::{collections::HashMap, sync::Mutex};
use log::{debug, info, warn};
use parser::Node;
use tauri::State;
use std::str;
use serde::{Deserialize, Serialize};

use crate::{compiler::{ast_unknowns, compile_to_string}, error::AppError, parser::{parse_latex, simplify_tree}};

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    code: String,
    num: Option<f64>
}

#[derive(Debug, Default)]
pub struct GlobalState {
    variables: Mutex<HashMap<String, f64>>,
    functions: Mutex<HashMap<String, Box<Node>>>,
}

#[tauri::command]
fn process(eq: &str, state: State<GlobalState>) -> error::Result<Response> {
    info!("{eq}");

    let funcs = state.functions.lock()
        .map_err(|_| AppError::IoError("Couldn't access to the function map".to_owned()))?;
    let vars = state.variables.lock()
        .map_err(|_| AppError::IoError("Couldn't access to the variable map".to_owned()))?;

    let mut root = parse_latex(eq, &funcs).or_else(|e| { 
        warn!("{e:?}"); Err(e) 
    })?;
    root.print_tree();

    match process_ast(&mut root, &vars)? {
        Response { code, num: Some(n) } =>  {
            info!("Expression {eq} evaluates to {n}");
            Ok(Response { code, num: Some(n) })
        }
        Response { code, num: None } => {
            info!("Expression {eq} has been compiled to {code}");
            Ok(Response { code, num: None })
        }
    }
}

#[tauri::command]
fn add_variable(name: &str, content: &str, state: State<GlobalState>) -> error::Result<f64> {
    let funcs = state.functions.lock()
        .map_err(|_| AppError::IoError("Couldn't access to the function map".to_owned()))?;
    let mut vars = state.variables.lock()
        .map_err(|_| AppError::IoError("Couldn't access to the variable map".to_owned()))?;
    vars.remove(name);

    let mut root = parse_latex(content, &funcs).or_else(|e| { 
        warn!("{e:?}"); Err(e) 
    })?;
    
    let val = simplify_tree(&mut root, &vars).ok_or_else(|| {
        warn!("The variable {name} couldn't be evaluated to a value: {content}");
        AppError::MathError(format!("The variable must evaluate to a certain value"))
    })?;
    
    vars.insert(name.to_owned(), val);

    Ok(val)
}

#[tauri::command]
fn add_function(name: &str, content: &str, state: State<GlobalState>) -> error::Result<Response> {
    let vars = state.variables.lock()
        .map_err(|_| AppError::IoError("Couldn't access to the variable map".to_owned()))?;

    let fn_name =  name.chars().nth(0)
        .ok_or_else(|| AppError::ParseError("This function doesn't have name".to_owned()))?;
    let unknown = name.chars().nth(1)
        .ok_or_else(|| AppError::ParseError("This function doesn't have any variables".to_owned()))?;
    info!("{}({}) = {content}", fn_name, unknown);

    let mut funcs = state.functions.lock()
        .map_err(|_| AppError::IoError("Couldn't access to the function map".to_owned()))?;
    funcs.remove(&fn_name.to_string());

    let mut root = parse_latex(content, &funcs).or_else(|e| { 
        warn!("{e:?}"); Err(e) 
    })?;
    let (x, y) = ast_unknowns(&root)?;
    if !((x && unknown == 'x') || (y && unknown == 'y')) {
        return Err(AppError::ParseError(format!("The function {fn_name} does not match its unknowns")));
    }

    let response = match process_ast(&mut root, &vars)? {
        Response { code, num: Some(n) } =>  {
            info!("Expression {content} evaluates to {n}");
            Ok(Response { code, num: Some(n) })
        }
        Response { code, num: None } => {
            info!("Expression {content} has been compiled to {code}");
            Ok(Response { code, num: None })
        }
    };

    funcs.insert(fn_name.to_string(), Box::new(root));

    response
}

fn process_ast(root: &mut Node, variable_map: &HashMap<String, f64>) -> error::Result<Response> {
    let numeric_value = simplify_tree(root, variable_map);
    root.print_tree();

    if numeric_value.is_some() {
        Ok( Response { 
            code: String::new(), 
            num: numeric_value,
        } )   
    } else {
        let code = compile_to_string(&root, variable_map)?;

        Ok( Response {
            code,
            num: None
        } )
    }
}

fn main() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    debug!("Logger has been set up correctly");

    tauri::Builder::default()
        .manage(GlobalState::default() )
        .invoke_handler(tauri::generate_handler![process, add_variable, add_function])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
