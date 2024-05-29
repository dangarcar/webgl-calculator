use std::{collections::HashMap, f64::EPSILON};

use log::info;

use crate::{error::{self, AppError}, parser::{BinaryOperation, NAryOperation, Node, UnaryOperation}};

use self::bytecode::{compile_to_bytecode, print_instructions};

pub mod bytecode;
mod tests;
struct CompileState <'a> {
    variable_map: &'a HashMap<String, f64>,
    denominators: Vec<String>,
    expr_idx: usize,
}

pub fn compile_to_string(root: &Node, variable_map: &HashMap<String, f64>, expr_idx: usize) -> error::Result<String> {
    let bytecode = compile_to_bytecode(root, variable_map, expr_idx)?;
    print_instructions(&bytecode);

    let unknowns = ast_unknowns(root)?;
    if unknowns == (false, false) {
        return Err(AppError::MathError(format!("This equation doesn't have any unknowns")));
    }

    let mut compile_state = CompileState {
        variable_map,
        denominators: Vec::new(),
        expr_idx
    };

    let code = if let Node::Binary{ op_type: BinaryOperation::Equal, lhs: Some(lhs), rhs: Some(rhs) } = root {
        info!("The equation has an equal sign");

        let compiled_lhs = compile(lhs, &mut compile_state)?;
        let compiled_rhs = compile(rhs, &mut compile_state)?;

        format!("fsub({compiled_lhs}, {compiled_rhs})")
    } else {
        let code = compile(root, &mut compile_state)?;
        
        if unknowns.0 == false {
            format!("fsub({code}, x)")
        } else {
            format!("fsub({code}, y)")
        }
    };

    handle_denominators(code, &compile_state.denominators, expr_idx)
}

fn compile(root: &Node, compile_state: &mut CompileState) -> error::Result<String> {
    match root {
        Node::Constant { value } => Ok(format!("float({value})")),
        Node::Variable { name } => {
            let v = compile_state.variable_map.get(name).ok_or(AppError::IoError(format!("There are no variable called {name}")))?;
            Ok(format!("float({v})"))
        }
        Node::Unknown { name } => {
            Ok(name.clone()) 
        }
        Node::Unary { op_type, child } => {
            let child = child.as_ref().ok_or(AppError::MathError(format!("There is nothing to operate on in {op_type:?}")))?;
            let compiled_child = compile(&child, compile_state)?;

            match op_type {
                UnaryOperation::Minus => Ok(format!("fminus({compiled_child})")),
                UnaryOperation::Sin => Ok(format!("fsin({compiled_child})")), 
                UnaryOperation::Cos => Ok(format!("fcos({compiled_child})")), 
                UnaryOperation::Floor => Ok(format!("ffloor({compiled_child})")), 
                UnaryOperation::Abs => Ok(format!("fabs({compiled_child})")), 
                UnaryOperation::Ceil => Ok(format!("fceil({compiled_child})")), 
                UnaryOperation::Log => Ok(format!("flog({compiled_child})")), 
                UnaryOperation::Ln => Ok(format!("fln({compiled_child})")), 
                
                UnaryOperation::Tan => compile_div(format!("fsin({compiled_child})"), format!("fcos({compiled_child})"), compile_state), 
                UnaryOperation::Sqrt => Ok(format!("fexp(fmul(fln({compiled_child}), 0.5))")), 
                
                UnaryOperation::Fact => Err(AppError::MathError("Factorial isn't implemented yet!".to_owned())),
            }
        },
        Node::Binary { op_type, lhs, rhs } => {
            let lhs = lhs.as_ref().ok_or(AppError::MathError(format!("There is nothing in the left to operate on in {op_type:?}")))?;
            let rhs = rhs.as_ref().ok_or(AppError::MathError(format!("There is nothing in the right to operate on in {op_type:?}")))?;
            let compiled_lhs = compile(&lhs, compile_state)?;
            let compiled_rhs = compile(rhs, compile_state)?;
            
            match op_type {
                BinaryOperation::Division => compile_div(compiled_lhs, compiled_rhs, compile_state),
                BinaryOperation::Power => {
                    if let Node::Constant { value } = **rhs {
                        if f64::abs((value as i32) as f64 - value) < EPSILON {
                            return compile_pow_integer(&compiled_lhs, value as i32, compile_state);
                        }
                    }

                    Ok(format!("fexp(fmul(fln({compiled_lhs}), {compiled_rhs}))"))
                }
                BinaryOperation::Equal => Err(AppError::MathError("Equal is not an operation in this context".to_owned())),
            }
        }
        Node::NAry { op_type, children } => {
            let op = match op_type {
                NAryOperation::Add => "fadd",
                NAryOperation::Multiply => "fmul",
            };

            if children.len() < 2 { 
                Err(AppError::MathError(format!("A {op_type:?} cannot be of less than two terms"))) 
            } else {
                let mut code = format!("{op}({}, {})", compile(&children[0], compile_state)?, compile(&children[1], compile_state)?);

                for t in children.iter().skip(2) {
                    code = format!("{op}({code}, {})", compile(t, compile_state)?);
                }

                Ok(code)
            }
        }
    }
}

fn compile_div(num: String, den: String, compile_state: &mut CompileState) -> error::Result<String> {
    compile_state.denominators.push(den);
    Ok(format!("fdiv( {num}, var_{}_{} )", compile_state.expr_idx, compile_state.denominators.len()-1))
}

fn compile_pow_integer(code: &str, times: i32, compile_state: &mut CompileState) -> error::Result<String> {
    if times == 0 {
        Ok(format!("1.0"))
    } else if times < 0 {
        Ok(compile_div(
            "1.0".to_owned(), 
            compile_pow_integer(code, i32::abs(times), compile_state)?, 
            compile_state,
        )?)
    } else if times == 1 {
        Ok(format!("{code}"))
    } else if times == 2 {
        Ok(format!("fmul({code}, {code})"))
    } else {
        Ok(format!("fmul({code}, {})", compile_pow_integer(code, times-1, compile_state)?))
    }
}

pub fn ast_unknowns(root: &Node) -> error::Result<(bool, bool)> {
    match root {
        Node::Unknown { name } => {
            if name == "x" { Ok((true, false)) }
            else if name == "y" { Ok((false, true)) }
            else { Ok((false, false)) }
        }
        Node::Unary { child,..} => {
            if let Some(child) = child {
                ast_unknowns(child)
            } else {
                Err(AppError::EmptyError)
            }
        }
        Node::Binary {lhs, rhs,..} => {
            let mut unk = (false, false);
            if let Some(lhs) = lhs {
                let a = ast_unknowns(lhs)?;
                unk = (unk.0 | a.0, unk.1 | a.1);
            }
            if let Some(rhs) = rhs {
                let a = ast_unknowns(rhs)?;
                unk = (unk.0 | a.0, unk.1 | a.1);
            }

            Ok(unk)
        }
        Node::NAry { children, .. } => {
            let mut unk = (false, false);
            for node in children {
                let a = ast_unknowns(node)?;
                unk = (unk.0 | a.0, unk.1 | a.1);
            }

            Ok(unk)
        }
        _ => Ok((false, false))
    }
}

fn handle_denominators(code: String, denominators: &Vec<String>, expr_idx: usize) -> error::Result<String> {
    if denominators.len() > 32 {
        return Err(AppError::IoError(format!("A function can't have more than 32 denominators")));
    }

    let dens = denominators.iter().enumerate().fold(String::new(), |s, (i, e)| {
        s + &format!("
            float var_{expr_idx}_{i} = {e};
            ret.y <<= 1; 
            ret.y |= int(fneg(var_{expr_idx}_{i}));")
    });

    Ok(format!("{dens}
            ret.x = int(fneg({code}));"))
}