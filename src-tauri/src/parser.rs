use std::iter::Peekable;

use crate::error::{self, AppError};
use tex_parser::ast::Token;

use self::{arithmetic::get_terms, ast::Node, operations::{get_op_type, Constants, NAryOperation, OpType, UnaryOperation}};

mod arithmetic;
mod ast;
mod operations;

//This is used because the '^' is not a Punctuation symbol in the tex_parser library and I can't change it, so I use '!' which isn't used anywhere else in my program
const EXP_SYMBOL: char = '?';
const EXP_SYMBOL_STR: &str = "?";

pub fn parse_latex(eq: &str) -> error::Result<Node> {
    let tokens = tokenize_string(eq)?;

    let tree = build_tree(&tokens)?;
    println!();
    tree.print_tree();

    Ok(tree)
}

fn build_tree(tokens: &[Token]) -> error::Result<Node> {
    //Get terms of equation
    let terms = get_terms(tokens)?;
    if terms.len() == 0 {
        return Err(AppError::MathError("Can't operate on nothing".to_owned()))
    }

    //Get trees for each term
    let term_trees: error::Result<Vec<Node>> = terms.into_iter().rev()
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
        .collect();
    let term_trees = term_trees?;

    if term_trees.len() == 1 {
        Ok(term_trees.into_iter().nth(0).unwrap())
    } else {
        Ok(Node::NAry { 
            op_type: NAryOperation::Add, 
            children: term_trees.into_iter().map(|e| Box::new(e)).collect()
        })
    }
}

fn build_term<'a, I: Iterator<Item = &'a Token>>(tokens: I) -> error::Result<Node> {
    let mut factors = Vec::new();
    let mut tokens = tokens.peekable();
    loop {
        let result = build_factor(tokens);
        match result {
            Ok((factor, tks)) => {
                factors.push(Box::new(factor));
                tokens = tks;
            },
            Err(e) => { 
                if let AppError::EmptyError = e { break; } 
                else { return Err(e); } 
            }
        }
    }

    match factors.len() {
        0 => Err(AppError::ParseError("This term is empty".to_owned())),
        1 => Ok(*(factors.into_iter().next().unwrap())),
        _ => Ok(Node::NAry { op_type: NAryOperation::Multiply, children: factors })
    }
}

fn build_factor<'a, I: Iterator<Item = &'a Token>>(mut tokens: Peekable<I>) -> error::Result<(Node, Peekable<I>)> {
    let token = tokens.next()
        .ok_or(AppError::EmptyError)?;

    let node = match token {
        Token::Group(group) => build_tree(&group.tokens),
        Token::Number(n) => Ok(Node::Constant { value: n.parse().map_err(|_| AppError::ParseError(format!("Couldn't parse number {}",n.content)))? }),
        Token::Macro(mac) => {
            match get_op_type(&mac.name.content)? {
                OpType::Binary(op) => {
                    let (lhs, tks) = build_factor(tokens)?;
                    tokens = tks;
                    let (rhs, tks) = build_factor(tokens)?;
                    tokens = tks;

                    Ok(Node::Binary { 
                        op_type: op, 
                        lhs: Some(Box::new(lhs)), 
                        rhs: Some(Box::new(rhs)) 
                    })
                },
                OpType::Unary(op) => {
                    let (child, tks) = build_factor(tokens)?;
                    tokens = tks;
                    
                    Ok(Node::Unary { 
                        op_type: op, 
                        child: Some(Box::new(child)) 
                    })
                },
                OpType::Constant(cte) => Ok(Node::Constant { value: cte.value() }),
                _ => Err(AppError::ParseError(format!("This doesn't make sense inside a factor: {}", mac.name.content))),
            }
        },
        Token::CharTokens(tok) => {
            if tok.content == "e" {
                Ok(Node::Constant { value: Constants::E.value() })
            } else if tok.content == "x" || tok.content == "y" {
                Ok( Node::Unknown { name: tok.content.to_owned() } )
            } else {
                Ok( Node::Variable { name: tok.content.to_owned() } )
            }
        },
        t => Err(AppError::ParseError(format!("This shouldn't be in a factor: {t:?}"))),
    }?;

    let next_node = if tokens.peek().is_some() && tokens.peek().unwrap().punctuation().is_some() {
        let operation = tokens.next().unwrap().punctuation().unwrap();
        let op_type = get_op_type(&operation.ch.to_string())?;
        
        match op_type {
            OpType::Unary(unary) => Ok( Node::Unary { 
                op_type: unary, 
                child: Some(Box::new(node))
            }),
            OpType::Binary(bin) => {
                let (rhs, tks) = build_factor(tokens)?;
                tokens = tks;
                Ok( Node::Binary { 
                    op_type: bin, 
                    lhs: Some(Box::new(node)), 
                    rhs: Some(Box::new(rhs)) 
                })
            },
            _ => Err(AppError::ParseError(format!("{op_type:?} isn't expected here in a factor after a symbol")))
        }
    } else {
        Ok(node)
    };

    Ok((next_node?, tokens))
}

fn tokenize_string(eq: &str) -> error::Result<Vec<Token>>{
    let eq = sanitize_string(eq.to_string())?;

    let latex_doc = tex_parser::parse(&eq)
            .map_err(|e| AppError::ParseError(e.to_string()))?;

    Ok( filter_token_stream(latex_doc.content) )
}

fn filter_token_stream(tokens: Vec<Token>) -> Vec<Token> {
    let mut filtered: Vec<Token> = tokens.into_iter()
        .filter(|e| e.whitespace().is_none()) //Remove whitespaces
        .filter(|e| match e.macro_() { Some(m) => m.name.content != "cdot", None => true }) //Remove multiply symbols
        .collect();
    
    for v in filtered.iter_mut() {
        if let Some(g) = v.group_mut() {
            let t = g.tokens.clone();
            g.tokens = filter_token_stream(t);
        }
    }

    filtered
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