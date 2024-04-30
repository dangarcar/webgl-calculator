use std::f64;

use crate::error;

use super::EXP_SYMBOL_STR;

#[derive(Debug, Clone, Copy)]
pub enum NAryOperation {
    Add, Multiply
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOperation {
    Division, Power, Mod,
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOperation {
    Minus, Sin, Cos, Tan, Floor, Abs, Ceil, Log, Ln, Sqrt, Fact,
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
        "!" =>              Ok(OpType::Unary( UnaryOperation::Fact )),
        EXP_SYMBOL_STR =>   Ok(OpType::Binary( BinaryOperation::Power )),

        "frac" =>           Ok(OpType::Binary( BinaryOperation::Division )),
        "mod" =>            Ok(OpType::Binary( BinaryOperation::Mod )), 
        "pi" =>             Ok(OpType::Constant( Constants::Pi )),
        
        "sin" =>            Ok(OpType::Unary( UnaryOperation::Sin )), 
        "cos" =>            Ok(OpType::Unary( UnaryOperation::Cos )),
        "tan" =>            Ok(OpType::Unary( UnaryOperation::Tan )),
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

impl UnaryOperation {
    pub fn factorial(n: f64) -> f64 {
        let n = n as i32;
        let mut ans = 0;
        for i in 2..=n {
            ans = ans * i;
        }
        ans as f64
    }

    pub fn minus(n: f64) -> f64 {
        -n
    }

    pub fn func(&self) -> error::Result<fn(f64) -> f64> {
        match self {
            Self::Sin => Ok(f64::sin),
            Self::Cos => Ok(f64::cos),
            Self::Tan => Ok(f64::tan),
            Self::Abs => Ok(f64::abs),
            Self::Ceil => Ok(f64::ceil),
            Self::Fact => Ok(Self::factorial),
            Self::Floor => Ok(f64::floor),
            Self::Ln => Ok(f64::ln),
            Self::Log => Ok(f64::log10),
            Self::Minus => Ok(Self::minus),
            Self::Sqrt => Ok(f64::sqrt),
            //_ => Err(error::AppError::MathError(format!("There's no operation called {self:?}"))),
        }
    }
}

impl BinaryOperation {
    pub fn fmod(a: f64, b: f64) -> f64 {
        a % b
    }

    pub fn div(a: f64, b: f64) -> f64 {
        a / b
    }
    
    pub fn func(&self) -> error::Result<fn(f64,f64) -> f64> {
        match self {
            Self::Division => Ok(Self::div),
            Self::Mod => Ok(Self::fmod),
            Self::Power => Ok(f64::powf),
        }
    }
}

impl NAryOperation {
    pub fn sum(a: f64, b: f64) -> f64 {
        a + b
    }
    
    pub fn mult(a: f64, b: f64) -> f64 {
        a * b
    }

    pub fn func(&self) -> error::Result<fn(f64,f64) -> f64> {
        match self {
            Self::Add => Ok(Self::sum),
            Self::Multiply => Ok(Self::mult),
        }
    }
}

#[cfg(test)]
mod test {
    use super::BinaryOperation;

    #[test]
    fn float_mod() {
        let m = BinaryOperation::fmod(2.0, 1.5);
        assert_eq!(m, 0.5);
    }
}