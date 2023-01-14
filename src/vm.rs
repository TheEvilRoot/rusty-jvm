use std::fmt::{Display, Formatter};

#[derive(Copy, Clone)]
pub enum VMValue {
    Int(i32),
    Long(i64),
    Byte(u8),
    Float(f32),
    Double(f64),
    Null
}

impl Display for VMValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VMValue::Int(v) => f.write_str(format!("Int({})", v).as_str()),
            VMValue::Long(v) => f.write_str(format!("Long({})", v).as_str()),
            VMValue::Byte(v) => f.write_str(format!("Byte({})", v).as_str()),
            VMValue::Float(v) => f.write_str(format!("Float({})", v).as_str()),
            VMValue::Double(v) => f.write_str(format!("Double({})", v).as_str()),
            VMValue::Null => f.write_str("Null")
        }
    }
}

impl VMValue {
    pub fn int(self) -> i32 {
        match self {
            Self::Int(v) => v,
            _ => panic!("Expected int, got {}", self)
        }
    }
}

pub struct VM {
    interop_stack: Vec<VMValue>,
    interop_stack_ptr: usize,
    interop_stack_size: usize
}

impl VM {
    pub fn new(initial_iterop_capacity: usize) -> Self {
        VM {
            interop_stack_ptr: 0,
            interop_stack_size: 0,
            interop_stack: Vec::with_capacity(initial_iterop_capacity)
        }
    }

    pub(crate) fn push(&mut self, val: VMValue) {
        if self.interop_stack_ptr == self.interop_stack_size {
            self.interop_stack_size += 1;
            self.interop_stack.push(val);
            self.interop_stack_ptr += 1;
        } else if self.interop_stack_ptr < self.interop_stack_size {
            self.interop_stack[self.interop_stack_ptr] = val;
            self.interop_stack_ptr += 1;
        } else {
            panic!("interop stack ptr violation: size:{} ptr:{}", self.interop_stack_size, self.interop_stack_ptr)
        }
    }
    pub fn pop(&mut self) -> VMValue {
        if self.interop_stack_ptr == 0 {
            panic!("interop stack ptr violation: pop at {} size:{}", self.interop_stack_ptr, self.interop_stack_size)
        }
        self.interop_stack_ptr -= 1;
        self.interop_stack[self.interop_stack_ptr].clone()
    }
}


