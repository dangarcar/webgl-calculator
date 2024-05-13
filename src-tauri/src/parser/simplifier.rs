use std::{collections::HashMap, f64::consts::LN_10, ops::Deref};

use crate::error::{self, AppError};

use super::{ast::Node, BinaryOperation, NAryOperation, UnaryOperation};

pub fn simplify_tree(root: &mut Node, variable_map: &HashMap<String, f64>) -> Option<f64> {
    match root {
        Node::Unknown {..} => None,
        Node::Constant {value} => Some(*value),
        Node::Variable { name } => variable_map.get(name).copied(),
        Node::Unary { op_type, child } => {
            let child = child.as_mut().unwrap();
            if let Some(n) = simplify_tree(child, variable_map) {
                let f = op_type.func().unwrap();
                let val = f(n);
                *root = Node::Constant { value: val };
                Some(val)
            } else { None }
        },
        Node::Binary { op_type, lhs, rhs } => {
            let lhs = simplify_tree(lhs.as_mut().unwrap(), variable_map);
            let rhs = simplify_tree(rhs.as_mut().unwrap(), variable_map);
            if lhs.is_some() && rhs.is_some() {
                let f = op_type.func().unwrap();
                let val = f(lhs.unwrap(), rhs.unwrap());
                *root = Node::Constant { value: val };
                Some(val)
            } else { 
                None
            }
        },
        Node::NAry { op_type, children } => {
            let mut children: Vec<Box<Node>> = children.iter()
                .flat_map(|e| {
                    if let Node::NAry { op_type: child_op, children: greatchildren } = e.deref() {
                        if *child_op == *op_type {
                            return greatchildren.to_owned();
                        }
                    }
                    vec![e.to_owned()]
                })
                .collect();

            let cnst = children
                .iter_mut()
                .filter_map(|e| simplify_tree(e, variable_map))
                .reduce(|acc, e| (op_type.func().unwrap())(acc, e));
            
            let mut new_children: Vec<Box<Node>> = children
                .into_iter()
                .filter_map(|mut e| { 
                    match simplify_tree(&mut e, variable_map) {
                        Some(..) => None,
                        None => Some(e.to_owned()),
                    }
                })
                .collect();

            if let Some(x) = cnst { //Add constants to vector
                new_children.push(Box::new(Node::Constant { value: x }));
            }

            match op_type {
                NAryOperation::Add => {
                    new_children = new_children.into_iter().filter(|e| {
                        if let Node::Constant { value } = e.deref() {
                            return *value != 0.0;
                        }

                        true
                    }).collect()
                }
                NAryOperation::Multiply => {
                    if new_children.iter().any(|e| {
                        if let Node::Constant { value } = e.deref() {
                            return *value == 0.0;
                        }

                        false
                    }) {
                        new_children.clear();
                    }
                }
            }

            if new_children.is_empty() {
                *root = Node::Constant { value: 0.0 };
                Some(0.0)
            } else {
                if new_children.len() == 1 {
                    *root = *new_children.first().unwrap().to_owned()
                } else {
                    *root = Node::NAry { op_type: *op_type, children: new_children };
                }

                None
            }            
        },
    }
}

/// Substitute the unknowns of the tree by the variable
pub fn substitute_func(root: &mut Node, variable: &Node) -> error::Result<()> {
    match root {
        Node::Unknown { .. } => {
            *root = variable.clone();
        }
        Node::Unary { child, .. } => {
            if let Some(c) = child {
                substitute_func(c, variable)?;
            }
        },
        Node::Binary { lhs, rhs, .. } => {
            if let Some(l) = lhs {
                substitute_func(l, variable)?;
            }
            if let Some(r) = rhs {
                substitute_func(r, variable)?;
            }
        },
        Node::NAry { children,.. } => {
            for n in children.iter_mut() {
                substitute_func(n, variable)?;
            }
        }
        _ => (),
    }

    Ok(())
}

pub fn derive_function(root: &Node) -> error::Result<Box<Node>> {
    let answer = match root {
        Node::Unknown { .. } => Node::Constant { value: 1.0 },
        Node::Constant { .. } | Node::Variable { .. }=> Node::Constant { value: 0.0 },
        Node::Unary { op_type, child } => {
            if let Some(child) = child {
                match op_type {
                    UnaryOperation::Minus => *child.to_owned(),
                    UnaryOperation::Ln => { // f'(x)/f(x)
                        Node::divide(
                            *derive_function(&child)?, 
                            *child.to_owned()
                        )
                    }
                    UnaryOperation::Sin => { // cos(f(x)) * f'(x)
                        Node::multiply(
                            Node::op(UnaryOperation::Cos, *child.to_owned()),
                            *derive_function(&child)? 
                        )
                    }
                    UnaryOperation::Cos => { // -sin(x) * f'(x)
                        Node::op(UnaryOperation::Minus, 
                            Node::multiply(
                                Node::op(UnaryOperation::Sin, *child.to_owned()), 
                                *derive_function(&child)?
                            )
                        )
                    }
                    UnaryOperation::Tan => { // 1 / cos(x)^2
                        let cosfx = Node::op(UnaryOperation::Cos, *child.to_owned());
                        Node::divide(
                            Node::Constant { value: 1.0 }, 
                            Node::multiply(cosfx.to_owned(), cosfx.to_owned())
                        )
                    }
                    UnaryOperation::Sqrt => { // f'(x) / 2*sqrt(f(x))
                        Node::divide(
                            *derive_function(&child)?, 
                            Node::multiply(
                                Node::Constant { value: 2.0 }, 
                                Node::op(UnaryOperation::Sqrt, *child.to_owned())
                            )
                        )
                    }
                    UnaryOperation::Log => { // f'(x) / ln10*f(x)
                        Node::divide(
                            *derive_function(&child)?, 
                            Node::multiply(
                                Node::Constant { value: LN_10 }, 
                                Node::op(UnaryOperation::Sqrt, *child.to_owned())
                            )
                        )
                    }
                    
                    _ => Err(AppError::MathError(format!("Function {op_type:?} is not derivable in R")))?
                }
            } else {
                Err(AppError::ParseError(format!("There aren't any variable to call this function {op_type:?}")))?
            }
        }
        Node::Binary { op_type, lhs, rhs } => {
            let lhs = lhs.as_deref().ok_or(AppError::EmptyError)?;
            let rhs = rhs.as_deref().ok_or(AppError::EmptyError)?;

            match op_type {
                BinaryOperation::Power => { 
                    if let Node::Constant { value } = *rhs { // a*f(x)^(a-1)*f'(x)
                        Node::NAry { 
                            op_type: NAryOperation::Multiply, 
                            children: vec![
                                Box::new(Node::Constant { value }), //a
                                Box::new(Node::Binary { //f(x)^(a-1)
                                    op_type: BinaryOperation::Power, 
                                    lhs: Some(Box::new(lhs.to_owned())), //f(x)
                                    rhs: Some(Box::new(Node::Constant { value: value - 1.0 })) // a-1
                                }),
                                derive_function(lhs)? //f'(x)
                            ],
                        }
                    } else { //f(x)^g(x) * (g'(x)*ln(f(x)) + g(x)*f'(x)/f(x))
                        let chain = Node::add(
                            Node::multiply(
                                *derive_function(rhs)?, 
                                Node::op(UnaryOperation::Ln, lhs.to_owned()),
                            ),
                            Node::multiply(
                                rhs.to_owned(), 
                                Node::divide(*derive_function(lhs)?, lhs.to_owned())
                            )
                        );

                        Node::multiply(root.clone(), chain)
                    }
                }
                BinaryOperation::Division => { // (f'(x)*g(x) - f(x)*g'(x)) / g(x)^2
                    let numerator = Node::substract(
                        Node::multiply(*derive_function(lhs)?, rhs.to_owned()), 
                        Node::multiply(lhs.to_owned(), *derive_function(rhs)?)
                    );

                    Node::divide(
                        numerator,
                        Node::multiply(rhs.to_owned(), rhs.to_owned())
                    )
                }
                BinaryOperation::Equal => Err(AppError::MathError("You can't derive an equal sign".to_owned()))?,
            }
        }
        Node::NAry { op_type, children } => {
            match op_type {
                NAryOperation::Add => {
                    let derivatives: Result<Vec<Box<Node>>, AppError> = children.iter()
                        .map(|e| derive_function(&e))
                        .collect();
                    
                    Node::NAry { 
                        op_type: NAryOperation::Add, 
                        children: derivatives? 
                    }
                }
                NAryOperation::Multiply => {
                    let mut derivatives = Vec::new();
                    for i in 0..children.len() {
                        let mut product = Vec::new();
                        for j in 0..children.len() {
                            if i == j {
                                product.push(Box::new(*derive_function(&mut children[j].to_owned())?));
                            } else {
                                product.push(children[j].to_owned());
                            }
                        }

                        derivatives.push(Box::new( Node::NAry { op_type: NAryOperation::Multiply, children: product } ));
                    }

                    Node::NAry { 
                        op_type: NAryOperation::Add, 
                        children: derivatives
                    }
                }
            }
        }
    };

    Ok(Box::new(answer))
} 