use super::utils::{Value, ValueType};

#[derive(Debug, Clone, PartialEq)]
pub enum MemoryType {
    LIST,
    LITERAL,
    UNKNOWN,
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
                    value: vec![i64::MIN],
                    value_type: MemoryType::UNKNOWN,
                };
                200
            ],

            global: vec![
                MemoryValue {
                    value: vec![i64::MIN],
                    value_type: MemoryType::UNKNOWN,
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

    pub fn set_list(&mut self, list: Vec<i64>, index: usize, global: bool) {
        let memory = if global {
            &mut self.global
        } else {
            &mut self.memory
        };

        memory[index] = MemoryValue {
            value: list.clone(),
            value_type: MemoryType::LIST,
        };
    }

    pub fn get_value(&self, x: Value, global: bool) -> Value {
        let memory = if global { &self.global } else { &self.memory };

        match x.value_type {
            ValueType::INDEX => {
                let value = memory[x.value as usize].value[0].clone();
                if value == i64::MIN {
                    Value {
                        value: 0,
                        value_type: ValueType::UNKNOWN,
                    }
                } else {
                    Value {
                        value: value.clone(),
                        value_type: ValueType::LITERAL,
                    }
                }
            }
            ValueType::LITERAL => x,
            _ => x,
        }
    }

    pub fn set_value(&mut self, x: i64, index: usize, global: bool) {
        let memory = if global {
            &mut self.global
        } else {
            &mut self.memory
        };

        memory[index] = MemoryValue {
            value: vec![x],
            value_type: MemoryType::LITERAL,
        };
    }
}
