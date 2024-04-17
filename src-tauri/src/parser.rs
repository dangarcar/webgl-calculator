use std::iter::Peekable;

use crate::error::{self, AppError};
use tex_parser::ast::Token;

use self::{arithmetic::get_terms, ast::{BinaryOperation, Node, UnaryOperation}};

mod arithmetic;
mod ast;

//This is used because the '^' is not a Punctuation symbol in the tex_parser library and I can't change it, so I use '!' which isn't used anywhere else in my program
const EXP_SYMBOL: char = '?';

const BINARY_MACROS: [&str; 1] = ["frac"];
const UNARY_MACROS: [&str; 15] = ["sin", "cos", "tan", "sec", "csc", "cosec", "cotan", "mod", "floor", "abs", "ceil", "log", "ln", "sqrt", "sum"];
const CONSTANT_MACROS: [&str; 1] = ["pi"];
const VARIABLE_MACROS: [&str; 4] = ["theta", "rho", "phi", "lambda"];



//This functions only gets the type of node and its name, it does not construct the tree
/*fn macro_type(name: &str) -> error::Result<Node> {
    if CONSTANT_MACROS.contains(&name) {        
        Ok(Node::Constant { value:  match name {
            "pi" => std::f64::consts::PI,
            _ => std::f64::NAN,
        }})
    } else if VARIABLE_MACROS.contains(&name) {
        Ok(Node::Variable { 
            name: name.to_owned() 
        })
    } else if UNARY_MACROS.contains(&name) {
        Ok(Node::Unary { 
            name: name.to_owned(), 
            child: None 
        })
    } else if BINARY_MACROS.contains(&name) {
        Ok(Node::Binary { 
            name: name.to_owned(), 
            lhs: None, rhs: None 
        })
    } else {
        Err(AppError::ParseError(format!("'{:?}' isn't a know command", name).to_owned()))
    }
}*/

fn build_tree(tokens: &[Token]) -> error::Result<Node> {
    //Get terms of equation
    let terms = get_terms(tokens)?;
    if terms.len() == 0 {
        return Err(AppError::MathError("Can't operate on nothing".to_owned()))
    }

    //Get trees for each term
    let term_trees: Vec<error::Result<Node>> = terms.into_iter().rev()
        .map(|t| {
            let term_tokens = &tokens[t.range];
            match build_term(term_tokens.iter()) {
                Ok(term_tree) => {
                    if t.subtract { Ok(Node::Unary {
                        op_type: UnaryOperation::Minus,
                        child: Some(Box::new(term_tree))
                    })} else {
                        Ok(term_tree)
                    }
                },
                Err(e) => Err(e)
            }
        })
        .map(|e| { println!("{:?} ", e); e })
        .collect();

    if term_trees.len() == 1 {
        term_trees.into_iter().nth(0).unwrap()
    } else {
        let mut last = term_trees[0].clone();
        
        for t in term_trees.into_iter().skip(1) {
            last = Ok(Node::Binary { 
                op_type: BinaryOperation::Add, 
                lhs: Some(Box::new(last?)), 
                rhs: Some(Box::new(t?))
            });
        }
        
        last
    }
}

//TODO: Not even remotely done
fn build_term<'a, I: Iterator<Item = &'a Token>>(tokens: I) -> error::Result<Node> {
    let mut s = String::new();
    for t in tokens {
        s += &format!("{t:?}, "); 
    }
    Ok(Node::Variable { name: s })
}

/*fn build_tree(tokens: &[Token]) -> error::Result<Node> {
    let token = tokens.first()
    
    return match token {
        Token::Macro(mac) => {
            let macro_type = macro_type(&mac.name.content)?;
            match macro_type {
                Node::Variable{..} | Node::Constant {..} => Ok(macro_type),
                Node::Unary { name,..} => {
                    if let Some(Token::Group(group)) = tokens.iter().nth(1) {
                        Ok(Node::Unary {
                            name: name,
                            child: Some(Box::new(build_tree(&group.tokens)?)),
                        })
                    } else {
                        Err(AppError::MathError("A unary operator must have a child".to_owned()))
                    }
                },
                Node::Binary { name,..} => {
                    let lhs_group = tokens.iter().nth(1)
                            .ok_or(AppError::MathError("A binary operator must have a left-hand side".to_owned()))?
                            .group().ok_or(AppError::MathError("A binary operator must own a left-hand side".to_owned()))?;
                    let rhs_group = tokens.iter().nth(2)
                            .ok_or(AppError::MathError("A binary operator must have a right-hand side".to_owned()))?
                            .group().ok_or(AppError::MathError("A binary operator must own a right-hand side".to_owned()))?;
                    
                    Ok(Node::Binary { 
                        name: name, 
                        lhs: Some(Box::new(build_tree(&lhs_group.tokens)?)), 
                        rhs: Some(Box::new(build_tree(&rhs_group.tokens)?)),
                    })
                },
                Node::Unknown {..} => unreachable!("There aren't macros for x or y"),
            }
        },
        Token::Punctuation(punc) => {
            todo!()
        },
        Token::CharTokens(chars) => {
            todo!()
        },
        Token::Group(group) => {
            todo!()
        },
        t => Err(AppError::ParseError(format!("Unknown type of token {t:?}"))),
    };
}*/

pub fn parse_latex(eq: &str) -> error::Result<()> {
    let tokens = tokenize_string(eq)?;

    let tree = build_tree(&tokens)?;
    tree.print_tree();

    /*for i in &tokens {
        println!("{i:?}");
    }*/

    Ok(())
}

fn tokenize_string(eq: &str) -> error::Result<Vec<Token>>{
    let eq = sanitize_string(eq.to_string())?;

    let latex_doc = tex_parser::parse(&eq)
            .map_err(|e| AppError::ParseError(e.to_string()))?;

    let latex_doc: Vec<Token> = latex_doc.content.into_iter()
        .filter(|e| e.whitespace().is_none()) //Remove whitespaces
        .filter(|e| match e.macro_() { Some(m) => m.name.content != "cdot", None => true }) //Remove multiply symbols
        .collect();

    return Ok(latex_doc);
}

fn sanitize_string(mut eq: String) -> error::Result<String> {
    eq = eq.replace('^', &EXP_SYMBOL.to_string());
    
    //Replace \operatorname{name} with \name for simplicity
    while let Some(i) = eq.find("operatorname{") {
        if let Some(j) = eq.find("}") {
            let name= &eq[(i + "operatorname{".len())..j];
            let owned = eq[..i].to_owned();
            eq = owned + name + &eq[j+1..];
        } else {
            return Err(AppError::ParseError("Missing '}'".to_string()));
        }
    }
    
    //For simplicity in the parser library, as {} are recognized as groups
    eq = eq.replace("\\left(", "{");
    eq = eq.replace("\\right)", "}");
    
    return Ok(eq);
}