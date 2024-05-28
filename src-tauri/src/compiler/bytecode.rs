use std::{collections::HashMap, f64::EPSILON, vec};

use log::info;

use crate::{error::{self, AppError}, parser::{BinaryOperation, NAryOperation, Node, UnaryOperation}};

use super::{ast_unknowns, CompileState};

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    StExpr(usize), Push(f64), PushX, PushY, Cpy, Pop, Ret, Store, Add, Mul, Div, Pow, UnaryOperation(UnaryOperation)
}

pub fn compile_to_bytecode(root: &Node, variable_map: &HashMap<String, f64>, expr_idx: usize) -> error::Result<Vec<Instruction>> {
    let unknowns = ast_unknowns(root)?;
    if unknowns == (false, false) {
        return Err(AppError::MathError(format!("This equation doesn't have any unknowns")));
    }

    let mut compile_state = CompileState {
        variable_map,
        denominators: Vec::new(),
        expr_idx
    };

    let mut code = vec![Instruction::StExpr(expr_idx)];

    if let Node::Binary{ op_type: BinaryOperation::Equal, lhs: Some(lhs), rhs: Some(rhs) } = root {
        info!("The equation has an equal sign");

        let compiled_lhs = compile_bytecode(lhs, &mut compile_state)?;
        let compiled_rhs = compile_bytecode(rhs, &mut compile_state)?;

        code.extend(compiled_lhs);
        code.extend(compiled_rhs);
    } else {
        code.extend(compile_bytecode(root, &mut compile_state)?);
        
        if unknowns.0 == false {
            code.push(Instruction::PushX);
        } else {
            code.push(Instruction::PushY);
        }
    };

    code.push(Instruction::UnaryOperation(UnaryOperation::Minus));
    code.push(Instruction::Add);
    code.push(Instruction::Store);

    Ok(code)
}

fn compile_bytecode(root: &Node, compile_state: &mut CompileState) -> error::Result<Vec<Instruction>> {
    match root {
        Node::Constant { value } => Ok(vec![Instruction::Push(*value)]),
        Node::Variable { name } => {
            let v = compile_state.variable_map.get(name).ok_or(AppError::IoError(format!("There are no variable called {name}")))?;
            Ok(vec![Instruction::Push(*v)])
        }
        Node::Unknown { name } => {
            match name.as_str() {
                "x" => Ok(vec![Instruction::PushX]),
                "y" => Ok(vec![Instruction::PushY]),
                _ => Err(AppError::MathError(format!("There aren't any unknowns called: {name}")))
            }
        }
        Node::Unary { op_type, child } => {
            let child = child.as_ref().ok_or(AppError::MathError(format!("There is nothing to operate on in {op_type:?}")))?;
            let mut compiled_child = compile_bytecode(&child, compile_state)?;

            match op_type {                
                UnaryOperation::Fact => Err(AppError::MathError("Factorial isn't implemented yet!".to_owned())),
                
                op => {
                    compiled_child.push(Instruction::UnaryOperation(op.clone()));
                    Ok(compiled_child)
                }
            }
        },
        Node::Binary { op_type, lhs, rhs } => {
            let lhs = lhs.as_ref().ok_or(AppError::MathError(format!("There is nothing in the left to operate on in {op_type:?}")))?;
            let rhs = rhs.as_ref().ok_or(AppError::MathError(format!("There is nothing in the right to operate on in {op_type:?}")))?;
            let compiled_lhs = compile_bytecode(&lhs, compile_state)?;
            let compiled_rhs = compile_bytecode(rhs, compile_state)?;
            
            match op_type {
                BinaryOperation::Division => {
                    let mut compiled = compiled_lhs;
                    compiled.extend(compiled_rhs);
                    compiled.push(Instruction::Div);
                    Ok(compiled)
                },
                BinaryOperation::Power => {
                    let mut compiled = compiled_lhs;
                    
                    if let Node::Constant { value } = **rhs {
                        if f64::abs((value as i32) as f64 - value) < EPSILON {
                            compiled.extend(compile_pow_integer(value as i32, compile_state)?);
                            return Ok(compiled);
                        }
                    }

                    compiled.extend(compiled_rhs);
                    compiled.push(Instruction::Pow);
                    Ok(compiled)
                }
                BinaryOperation::Equal => Err(AppError::MathError("Equal is not an operation in this context".to_owned())),
            }
        }
        Node::NAry { op_type, children } => {
            let op = match op_type {
                NAryOperation::Add => Instruction::Add,
                NAryOperation::Multiply => Instruction::Mul,
            };

            if children.len() < 2 { 
                Err(AppError::MathError(format!("A {op_type:?} cannot be of less than two terms"))) 
            } else {
                let mut compiled: Vec<_> = children.iter().flat_map(|e| compile_bytecode(e, compile_state).unwrap()).collect();

                for _ in 0..children.len()-1 {
                    compiled.push(op.clone());
                }

                Ok(compiled)
            }
        }
    }
}

fn compile_pow_integer(times: i32, compile_state: &mut CompileState) -> error::Result<Vec<Instruction>> {
    if times == 0 {
        Ok(vec![Instruction::Pop, Instruction::Push(1.0)])
    } else if times == 1 {
        Ok(vec![])
    } else if times < 0 {
        let mut compiled = vec![Instruction::Push(1.0)];
        compiled.extend(compile_pow_integer(-times, compile_state)?);

        Ok(compiled)
    } else {
        let mut compiled = Vec::new();
        for _ in 0..times-1 {
            compiled.push(Instruction::Cpy);
        }
        for _ in 0..times-1 {
            compiled.push(Instruction::Mul);
        }

        Ok(compiled)
    }
}

pub fn print_instructions(instructions: &Vec<Instruction>) {
    for i in instructions {
        match i {
            Instruction::Store => println!("store"),
            Instruction::StExpr(i) => println!("st_expr {i}"), 
            Instruction::Push(x) => println!("push {x}"),
            Instruction::PushX => println!("push_x"),
            Instruction::PushY => println!("push_y"),
            Instruction::Cpy => println!("cpy"),
            Instruction::Pop => println!("pop"),
            Instruction::Ret => println!("ret"),
            Instruction::Add => println!("add"),
            Instruction::Mul => println!("mul"),
            Instruction::Div => println!("div"),
            Instruction::Pow => println!("pow"),
            Instruction::UnaryOperation(u) => println!("{u:?}"),
        }
    }
}

impl Instruction {
    pub fn to_number_pair(&self) -> error::Result<(u8, f64)> {
        match &self {
            //Basic operations
            Instruction::StExpr(i) =>   Ok((0, *i as f64)),
            Instruction::Push(val) =>     Ok((1, *val)),
            Instruction::PushX =>               Ok((2, 0.0)),
            Instruction::PushY =>               Ok((3, 0.0)),
            Instruction::Cpy =>                 Ok((4, 0.0)),
            Instruction::Pop =>                 Ok((5, 0.0)),
            Instruction::Store =>               Ok((6, 0.0)),
            Instruction::Ret =>                 Ok((7, 0.0)),
            
            //Binary operations
            Instruction::Add =>                 Ok((32 | 0, 0.0)),
            Instruction::Mul =>                 Ok((32 | 1, 0.0)),
            Instruction::Div =>                 Ok((32 | 2, 0.0)),
            Instruction::Pow =>                 Ok((32 | 3, 0.0)),
            
            //Unary operations
            Instruction::UnaryOperation(op) => {
                let op_code = match op {
                    UnaryOperation::Minus =>    0,
                    UnaryOperation::Sin =>      1,
                    UnaryOperation::Cos =>      2,
                    UnaryOperation::Floor =>    3,
                    UnaryOperation::Abs =>      4,
                    UnaryOperation::Ceil =>     5,
                    UnaryOperation::Log =>      6,
                    UnaryOperation::Ln =>       7,
                    UnaryOperation::Sqrt =>     8,
                    UnaryOperation::Tan =>      9,
                    UnaryOperation::Fact => Err(AppError::MathError(format!("Factorial isn't implemented in bytecode")))?,
                };

                Ok((64 | op_code, 0.0))
            }
        }
    }
}