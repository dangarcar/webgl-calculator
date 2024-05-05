use std::{collections::HashMap, f64::EPSILON};

use log::info;

use crate::{error::{self, AppError}, parser::{BinaryOperation, NAryOperation, Node, UnaryOperation}};

pub fn compile_to_string(root: &Node, variable_map: &HashMap<String, f64>) -> error::Result<String> {
    let unknowns = ast_unknowns(root)?;
    if unknowns == (false, false) {
        return Err(AppError::MathError(format!("This equation doesn't have any unknowns")));
    }

    let code = if let Node::Binary{ op_type: BinaryOperation::Equal, lhs: Some(lhs), rhs: Some(rhs) } = root {
        info!("The equation has an equal sign");
        if unknowns.0 == false {
            return Err(AppError::MathError(format!("This equation doesn't have x")));
        } else if unknowns.1 == false {
            return Err(AppError::MathError(format!("This equation doesn't have y")));
        }

        let compiled_lhs = compile(lhs, variable_map)?;
        let compiled_rhs = compile(rhs, variable_map)?;

        format!("fsub({compiled_lhs}, {compiled_rhs})")
    } else {
        let code = compile(root, variable_map)?;
        
        if unknowns.0 == false {
            format!("fsub({code}, x)")
        } else {
            format!("fsub({code}, y)")
        }
    };

    Ok(format!("return fneg({code});"))
}

fn compile(root: &Node, variable_map: &HashMap<String, f64>) -> error::Result<String> {
    match root {
        Node::Constant { value } => Ok(format!("float({value})")),
        Node::Variable { name } => {
            let v = variable_map.get(name).ok_or(AppError::IoError(format!("There are no variable called {name}")))?;
            Ok(format!("float({v})"))
        }
        Node::Unknown { name } => {
            Ok(name.clone()) 
         }
        Node::Unary { op_type, child } => {
            let child = child.as_ref().ok_or(AppError::MathError(format!("There is nothing to operate on in {op_type:?}")))?;
            let compiled_child = compile(&child, variable_map)?;

            match op_type {
                UnaryOperation::Minus => Ok(format!("fminus({compiled_child})")),
                UnaryOperation::Sin => Ok(format!("fsin({compiled_child})")), 
                UnaryOperation::Cos => Ok(format!("fcos({compiled_child})")), 
                UnaryOperation::Tan => Ok(format!("fdiv( fsin({compiled_child}), fcos({compiled_child}))")), 
                UnaryOperation::Floor => Ok(format!("ffloor({compiled_child})")), 
                UnaryOperation::Abs => Ok(format!("fabs({compiled_child})")), 
                UnaryOperation::Ceil => Ok(format!("fceil({compiled_child})")), 
                UnaryOperation::Log => Ok(format!("flog({compiled_child})")), 
                UnaryOperation::Ln => Ok(format!("fln({compiled_child})")), 
                UnaryOperation::Sqrt => Ok(format!("fpow({compiled_child}, 0.5)")), 
                UnaryOperation::Fact => Err(AppError::MathError("Factorial isn't implemented yet!".to_owned())),
            }
        },
        Node::Binary { op_type, lhs, rhs } => {
            let lhs = lhs.as_ref().ok_or(AppError::MathError(format!("There is nothing in the left to operate on in {op_type:?}")))?;
            let rhs = rhs.as_ref().ok_or(AppError::MathError(format!("There is nothing in the right to operate on in {op_type:?}")))?;
            let compiled_lhs = compile(&lhs, variable_map)?;
            let compiled_rhs = compile(rhs, variable_map)?;
            
            match op_type {
                BinaryOperation::Division => Ok(format!("fdiv({compiled_lhs}, {compiled_rhs})")),
                BinaryOperation::Power => {
                    if let Node::Constant { value } = **rhs {
                        if value.fract() < EPSILON {
                            return Ok(compile_pow(&compiled_lhs, value as i32));
                        }
                    }

                    Ok(format!("fexp(fmul(fln({compiled_lhs}), {compiled_rhs}))"))
                }
                BinaryOperation::Mod => Ok(format!("fmod({compiled_lhs}, {compiled_rhs})")),
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
                let mut code = format!("{op}({}, {})", compile(&children[0], variable_map)?, compile(&children[1], variable_map)?);

                for t in children.iter().skip(2) {
                    code = format!("{op}({code}, {})", compile(t, variable_map)?);
                }

                Ok(code)
            }
        }
    }
}

fn compile_pow(code: &str, times: i32) -> String {
    if times == 0 {
        format!("1.0")
    } else if times < 0 {
        format!("fdiv(1.0, {})", compile_pow(code, -times))
    } else if times == 1 {
        format!("{code}")
    } else if times == 2 {
        format!("fmul({code}, {code})")
    } else {
        format!("fmul({code}, {})", compile_pow(code, times-1))
    }
}

fn ast_unknowns(root: &Node) -> error::Result<(bool, bool)> {
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