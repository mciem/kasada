use super::visitors::opcodes::GetType;
use super::visitors::values::GetValues;

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum ValueError {
    MathError(String),
    NotFound(String),
    InvalidType(String),
    IndexOutOfBounds(String),
}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValueError::NotFound(msg) => write!(f, "Value not found: {}", msg),
            ValueError::InvalidType(msg) => write!(f, "Invalid type: {}", msg),
            ValueError::IndexOutOfBounds(msg) => write!(f, "Index out of bounds: {}", msg),
            ValueError::MathError(msg) => write!(f, "Math error: {}", msg),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value<T = i64> {
    Literal(T),
    List(Vec<T>),
    String(String),
    None,
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Literal(n)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl Value {
    pub fn is_literal(&self) -> bool {
        matches!(self, Value::Literal(_))
    }

    pub fn is_list(&self) -> bool {
        matches!(self, Value::List(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    pub fn as_literal(&self) -> Result<i64, ValueError> {
        match self {
            Value::Literal(n) => Ok(*n),
            _ => Err(ValueError::InvalidType(
                "Expected literal value".to_string(),
            )),
        }
    }

    pub fn as_list(&self) -> Result<&Vec<i64>, ValueError> {
        match self {
            Value::List(n) => Ok(n),
            _ => Err(ValueError::InvalidType("Expected list value".to_string())),
        }
    }

    pub fn as_string(&self) -> Result<&String, ValueError> {
        match self {
            Value::String(n) => Ok(n),
            _ => Err(ValueError::InvalidType("Expected string value".to_string())),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Memory {
    pub local: Vec<Result<Value, ValueError>>,
    pub global: Vec<Result<Value, ValueError>>,
}

#[derive(Default)]
pub struct ValueHandler {
    pub chars: Vec<char>,
    pub get: GetValues,
    pub memory: Memory,
}

impl ValueHandler {
    pub fn new(chars: Vec<char>, get: GetValues) -> Self {
        Self {
            chars,
            get,
            memory: Memory {
                local: vec![
                    Err(ValueError::NotFound(
                        "Uninitialized local value".to_string()
                    ));
                    100
                ],
                global: vec![
                    Err(ValueError::NotFound(
                        "Uninitialized global value".to_string()
                    ));
                    10000
                ],
            },
        }
    }

    pub fn set_value(&mut self, x: Value, index: usize, global: bool) -> Result<(), ValueError> {
        let memory = if global {
            &mut self.memory.global
        } else {
            &mut self.memory.local
        };

        if index >= memory.len() {
            return Err(ValueError::IndexOutOfBounds(format!(
                "Index {} is out of bounds",
                index
            )));
        }

        memory[index] = Ok(x);
        Ok(())
    }

    pub fn get_value(
        &self,
        t: GetType,
        values: Vec<i64>,
        global: bool,
    ) -> Result<Value, ValueError> {
        let x = values[0];

        if global {
            let value = self.get_value(t, values.clone(), false)?;

            let index = value.as_literal()? as usize;
            if index >= self.memory.global.len() {
                return Err(ValueError::IndexOutOfBounds(format!(
                    "Global index {} is out of bounds",
                    index
                )));
            }
            return self.memory.global[index].clone();
        }

        match t {
            GetType::L => {
                let index = x >> 5;
                if index as usize >= self.memory.local.len() {
                    return Err(ValueError::IndexOutOfBounds(format!(
                        "Local index {} is out of bounds",
                        index
                    )));
                }
                self.memory.local[index as usize].clone()
            }

            GetType::E => {
                if x == self.get.strings as i64 {
                    let start = values[2] as usize;
                    let end = (values[1] + values[2]) as usize;

                    if end > self.chars.len() {
                        return Err(ValueError::IndexOutOfBounds(
                            "String index out of bounds".to_string(),
                        ));
                    }

                    let string: String = self.chars[start..end].iter().collect();
                    Ok(Value::String(string))
                } else if x & 1 == 1 {
                    Ok(Value::Literal(x >> 1))
                } else {
                    let index = x >> 5;
                    if index as usize >= self.memory.local.len() {
                        return Err(ValueError::IndexOutOfBounds(format!(
                            "Local index {} is out of bounds",
                            index
                        )));
                    }
                    self.memory.local[index as usize].clone()
                }
            }

            GetType::I => {
                let index = x >> 5;
                Ok(Value::Literal(index))
            }

            _ => Err(ValueError::InvalidType("Unknown GetType".to_string())),
        }
    }

    pub fn delete_value(&mut self, index: usize, global: bool) -> Result<(), ValueError> {
        let memory = if global {
            &mut self.memory.global
        } else {
            &mut self.memory.local
        };

        if index >= memory.len() {
            return Err(ValueError::IndexOutOfBounds(format!(
                "Index {} is out of bounds",
                index
            )));
        }

        memory[index] = Err(ValueError::NotFound("Value was deleted".to_string()));
        Ok(())
    }
}
