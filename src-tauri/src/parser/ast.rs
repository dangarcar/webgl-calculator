use std::fmt::Display;

use tex_parser::ast::Token;

#[derive(Debug, Clone)]
pub enum Node {
    Constant {
        value: f64
    },
    Variable {
        name: String
    },
    Unary {
        op_type: UnaryOperation,
        
        child: Option<Box<Node>>
    },
    Binary {
        op_type: BinaryOperation,
        
        lhs: Option<Box<Node>>,
        rhs: Option<Box<Node>>,
    },
    Unknown {
        name: String
    },
    Debug {
        content: Vec<Token>
    }
}

pub struct Ast {
    root: Box<Node>,

    evaluable: bool,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, PartialOrd, Ord, Eq)]
pub enum BinaryOperation {
    Add
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, PartialOrd, Ord, Eq)]
pub enum UnaryOperation {
    Minus
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::Binary { op_type, .. } => write!(f, "Binary {{ {:?} }}", op_type),
            Node::Unary { op_type, .. } => write!(f, "Unary {{ {:?} }}", op_type),
            _ => write!(f, "{:?}", self)
        }
    }
}

impl Node {
    pub fn print_tree(&self) {
        print_tree("", self, true);
    } 
}

fn print_tree(prefix: &str, root: &Node, last: bool) {
    print!("{prefix}{}", if last { "└──" } else { "├──" });
    println!("{root}");
    
    let new_prefix = prefix.to_owned() + if last { "    " } else { "|   " };
    match root {
        Node::Unary { child, .. } => {
            if let Some(c) = child {
                print_tree(&new_prefix, &c, true);
            }
        },
        Node::Binary { lhs, rhs, .. } => {
            if let Some(l) = lhs {
                print_tree(&new_prefix, &l, false);
            }
            if let Some(r) = rhs {
                print_tree(&new_prefix, &r, true);
            }
        },
        _ => (),
    }
}