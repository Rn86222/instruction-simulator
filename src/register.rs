use crate::{fpu_emulator::FloatingPoint, types::*};

#[derive(Copy, Clone)]
pub struct IntRegister {
    value: Int,
}

impl IntRegister {
    pub fn new() -> Self {
        IntRegister { value: 0 }
    }
    pub fn set(&mut self, value: Int) {
        self.value = value;
    }
    pub fn get(&self) -> Int {
        self.value
    }
}

#[derive(Copy, Clone)]
pub struct FloatRegister {
    value: FloatingPoint,
}

impl FloatRegister {
    pub fn new() -> Self {
        FloatRegister {
            value: FloatingPoint::new(0),
        }
    }
    pub fn set(&mut self, value: FloatingPoint) {
        self.value = value;
    }
    pub fn get(&self) -> FloatingPoint {
        self.value
    }
}
