use std::error::Error;
use crate::interpret::{Interpreter, InterpreterError, Opcode};
use crate::vm::VM;
use crate::vm::VMValue;

pub struct VMEnv {
    vm: VM,
    interpreter: Interpreter
}

impl VMEnv {

    pub fn of(vm: VM, interpreter: Interpreter) -> Self {
        return VMEnv { vm, interpreter }
    }

    pub fn execute(&mut self, code: &Vec<u8>) -> Result<(), InterpreterError> {
        for instruction in code {
            match self.interpreter.decode(instruction.clone())? {
                Opcode::AConstNull => self.aconst_null(),
                Opcode::IConst(v) => self.iconst(v as i32),
                Opcode::IAdd => self.iadd(),
                Opcode::IAnd => {}
                Opcode::I2B => {}
                Opcode::I2C => {}
                Opcode::I2D => {}
                Opcode::I2F => {}
                Opcode::I2L => {}
                Opcode::I2S => {}
                Opcode::IMul => {}
            }
        }
        Ok(())
    }

    pub fn iconst(&mut self, val: i32) {
        self.vm.push(VMValue::Int(val));
    }

    pub fn aconst_null(&mut self) {
        self.vm.push(VMValue::Null);
    }

    pub fn iadd(&mut self) {
        let a = self.vm.pop().int();
        let b = self.vm.pop().int();
        self.vm.push(VMValue::Int(a + b));
    }

    pub fn print(&mut self) {
        let v = self.vm.pop();
        println!("Interop > print {}", v);
    }
}