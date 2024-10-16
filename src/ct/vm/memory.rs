use super::utils::{Value, ValueType};

#[derive(Debug, Clone, PartialEq)]
pub enum MemoryType {
    LIST,
    LITERAL,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryValue {
    pub value: Vec<i64>,
    pub value_type: MemoryType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Memory {
    pub memory: Vec<MemoryValue>,
    pub global: Vec<MemoryValue>,
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            memory: vec![
                MemoryValue {
                    value: vec![0],
                    value_type: MemoryType::LITERAL,
                };
                200
            ],

            global: vec![
                MemoryValue {
                    value: vec![0],
                    value_type: MemoryType::LITERAL,
                };
                10000
            ],
        }
    }

    pub fn contains(&self, x: i64) -> bool {
        for i in 0..self.memory.len() {
            if self.memory[i].value[0] == x {
                return true;
            }
        }

        false
    }

    pub fn set_list(&mut self, list: Vec<i64>, index: usize, l_index: usize) {
        self.memory[index] = MemoryValue {
            value: list.clone(),
            value_type: MemoryType::LIST,
        };

        self.memory[l_index] = MemoryValue {
            value: vec![list.len() as i64],
            value_type: MemoryType::LITERAL,
        };
    }

    pub fn get_value(&self, x: Value) -> Value {
        match x.value_type {
            ValueType::INDEX => Value {
                value: self.memory[x.value as usize].value[0].clone(),
                value_type: ValueType::LITERAL,
            },
            ValueType::LITERAL => x,
            _ => x,
        }
    }

    pub fn set_value(&mut self, x: i64, index: usize) {
        self.memory[index] = MemoryValue {
            value: vec![x],
            value_type: MemoryType::LITERAL,
        };
    }

    pub fn get_global_value(&self, x: Value) -> Value {
        match x.value_type {
            ValueType::INDEX => Value {
                value: self.global[x.value as usize].value[0].clone(),
                value_type: ValueType::LITERAL,
            },
            ValueType::LITERAL => x,
            _ => x,
        }
    }

    pub fn set_global_value(&mut self, x: i64, index: usize) {
        self.global[index] = MemoryValue {
            value: vec![x],
            value_type: MemoryType::LITERAL,
        };
    }
}
