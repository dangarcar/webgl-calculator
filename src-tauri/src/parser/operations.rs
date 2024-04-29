use crate::error;

use super::EXP_SYMBOL_STR;

#[derive(Debug, Clone, Copy)]
pub enum NAryOperation {
    Add, Multiply
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOperation {
    Division, Power
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOperation {
    Minus, Sin, Cos, Tan, Mod, Floor, Abs, Ceil, Log, Ln, Sqrt, Fact
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Constants {
    Pi, E
}

#[derive(Debug, Clone)]
pub enum OpType {
    Binary(BinaryOperation),
    Unary(UnaryOperation),
    NAry(NAryOperation),
    Constant(Constants),
}

pub fn get_op_type(name: &str) -> error::Result<OpType> {
    match name {
        "+" =>              Ok(OpType::NAry( NAryOperation::Add )),
        "-" =>              Ok(OpType::Unary( UnaryOperation::Minus )),
        EXP_SYMBOL_STR =>   Ok(OpType::Binary( BinaryOperation::Power )),
        "!" =>              Ok(OpType::Unary( UnaryOperation::Fact )),

        "frac" =>           Ok(OpType::Binary( BinaryOperation::Division )),
        "pi" =>             Ok(OpType::Constant( Constants::Pi )),
        "sin" =>            Ok(OpType::Unary( UnaryOperation::Sin )), 
        "cos" =>            Ok(OpType::Unary( UnaryOperation::Cos )),
        "tan" =>            Ok(OpType::Unary( UnaryOperation::Tan )),
        "mod" =>            Ok(OpType::Unary( UnaryOperation::Mod )), 
        "floor" =>          Ok(OpType::Unary( UnaryOperation::Floor )), 
        "abs" =>            Ok(OpType::Unary( UnaryOperation::Abs )), 
        "ceil" =>           Ok(OpType::Unary( UnaryOperation::Ceil )),
        "log" =>            Ok(OpType::Unary( UnaryOperation::Log )),
        "ln" =>             Ok(OpType::Unary( UnaryOperation::Ln )), 
        "sqrt" =>           Ok(OpType::Unary( UnaryOperation::Sqrt )),
        "theta"|"rho"|"phi"|"lambda" => Err(error::AppError::ParseError("The greek letters aren't implemented yet".to_owned())), //TODO: implement variables
        _ => Err(error::AppError::ParseError(format!("{name} is not a known operation")))
    }
}

impl Constants {
    pub fn value(&self) -> f64 {
        match self {
            Constants::Pi => std::f64::consts::PI,
            Constants::E => std::f64::consts::E,
        }
    }
}
