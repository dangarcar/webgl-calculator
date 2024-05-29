use super::bytecode::Instruction;

const MAX_STACK_SIZE: usize = 1024;
const MAX_EXPR: usize = 1024;

/// This has been done so I can debug the code that runs GPU-side more easily 
#[derive(Debug)]
struct Interpreter {
    stack: [f64; MAX_STACK_SIZE],
    /// One past the last element in the stack
    stack_top: usize,
    program_counter: usize,
    current_expr: usize,

    program: Vec<Instruction>,
}

impl Interpreter {
    pub fn new(program: Vec<Instruction>) -> Self {
        Self {
            stack: [0.0; MAX_STACK_SIZE],
            program,
            stack_top: 0,
            program_counter: 0,
            current_expr: 0,
        }
    }

    pub fn run(&mut self, x: f64, y: f64) -> [f64; MAX_EXPR] {
        let mut output = [0.0; MAX_EXPR];

        while self.program_counter < self.program.len() {
            match &self.program[self.program_counter] {
                Instruction::Push(val) => self.push(*val),
                Instruction::PushX => self.push(x),
                Instruction::PushY => self.push(y),
                Instruction::Cpy => self.push(self.stack[self.stack_top-1]),

                Instruction::Ret => { self.current_expr += 1; },
                Instruction::Store => {
                    output[self.current_expr] = self.stack[self.stack_top-1];
                    self.pop();
                }
                Instruction::Pop => {
                    self.pop();
                }

                Instruction::Add => self.binary_op(|a, b| a + b),
                Instruction::Mul => self.binary_op(|a, b| a * b),
                Instruction::Div => self.binary_op(|a, b| a / b),
                Instruction::Pow => self.binary_op(|a, b| f64::powf(a, b)),
                
                Instruction::UnaryOperation(op) => self.unary_op(op.func().unwrap()),
            }
    
            self.program_counter += 1;
        }

        output
    }

    fn push(&mut self, val: f64) {
        self.stack[self.stack_top] = val;
        self.stack_top += 1;
    }

    fn pop(&mut self) -> f64 {
        let val = self.stack[self.stack_top-1];
        self.stack_top -= 1;
        val
    }

    fn binary_op(&mut self, op: fn(f64, f64) -> f64) {
        let b = self.pop();
        let a = self.pop();
        self.push(op(a, b));
    }

    fn unary_op(&mut self, op: fn(f64) -> f64) {
        let a = self.pop();
        self.push(op(a));
    }
}

#[cfg(test)]
mod test {
    use crate::{compiler::bytecode::{compile_to_bytecode, Instruction}, error, parser::{parse_latex, UnaryOperation}};

    use super::Interpreter;

    #[test]
    fn basic_operations() {
        // 2 + 4*5 = 22
        let program = vec![
            Instruction::Push(2.0),
            Instruction::Push(4.0),
            Instruction::Push(5.0),
            Instruction::Mul,
            Instruction::Add,
            Instruction::Store,
            Instruction::Ret,
        ];

        let mut int = Interpreter::new(program);

        let res = int.run(0.0, 0.0)[0];
        println!("{}", int.stack_top);
        assert!(res == 22.0);
    }

    #[test]
    fn unary_operations() {
        // 5 - 8/2 = 1
        let program = vec![
            Instruction::Push(5.0),
            Instruction::Push(8.0),
            Instruction::Push(2.0),
            Instruction::Div,
            Instruction::UnaryOperation(UnaryOperation::Minus),
            Instruction::Add,
            Instruction::Store,
            Instruction::Ret,
        ];

        let mut int = Interpreter::new(program);

        let res = int.run(0.0, 0.0)[0];
        println!("{}", int.stack_top);
        assert!(res == 1.0);
    }

    #[test]
    fn multiple_operations() {
        // 5 - 8/2 = 1 && 2 + 4*5 = 22
        let program = vec![
            Instruction::Push(5.0),
            Instruction::Push(8.0),
            Instruction::Push(2.0),
            Instruction::Div,
            Instruction::UnaryOperation(UnaryOperation::Minus),
            Instruction::Add,
            Instruction::Store,
            Instruction::Ret,

            Instruction::Push(2.0),
            Instruction::Push(4.0),
            Instruction::Push(5.0),
            Instruction::Mul,
            Instruction::Add,
            Instruction::Store,
            Instruction::Ret,
        ];

        let mut int = Interpreter::new(program);

        let res = int.run(0.0, 0.0);
        println!("{}", int.stack_top);
        assert_eq!(res[0], 1.0);
        assert_eq!(res[1], 22.0);
    }

    #[test]
    fn parser() -> error::Result<()> {
        //In this text there's no need to simplify
        let tree = parse_latex("x^2", &Default::default())?;
        
        let mut program = compile_to_bytecode(&tree, &Default::default(), 0)?;
        program.push(Instruction::Ret);

        for i in 1..100 {
            let a = Interpreter::new(program.clone()).run(i as f64, 0.0)[0];
            assert_eq!(a, (i*i) as f64);
        }

        Ok(())
    }
}