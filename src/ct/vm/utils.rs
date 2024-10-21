use super::memory::Memory;
use super::visitors::opcodes::GetType;

use lazy_static::lazy_static;

const CHARSET: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    INDEX,
    LITERAL,
    UNKNOWN,
}

lazy_static! {
    static ref CHARSET_INDEX: [usize; 256] = {
        let mut arr = [62; 256];
        for (i, &c) in CHARSET.as_bytes().iter().enumerate() {
            arr[c as usize] = i;
        }
        arr
    };
}

#[derive(Debug, Clone)]
pub struct Value {
    pub value: i64,
    pub value_type: ValueType,
}

pub fn get_vm_bytes(instructions: &str) -> Vec<i64> {
    let bytes = instructions.as_bytes();
    let mut result = Vec::with_capacity(instructions.len() / 2);

    let mut m = 0;
    while m < bytes.len() {
        let mut h = 0;
        let mut l = 1;

        loop {
            let x = CHARSET_INDEX[bytes[m] as usize] as i64;
            m += 1;

            h += l * (x % 50 as i64);
            if x < 50 as i64 {
                result.push(h as i64);
                break;
            }

            h += 50 as i64 * l;
            l *= 12 as i64;
        }
    }

    result
}

fn decode_string(encoded: &[i64], index: &mut usize) -> String {
    *index += 1;
    let length = encoded[*index] as usize;
    *index += 1;

    (0..length)
        .map(|_| {
            let k = encoded[*index];
            *index += 1;
            let char_code = (k as u32 & 0xFFFFFFC0) | ((k * 59) & 63) as u32;
            char::from_u32(char_code).unwrap_or('\0')
        })
        .collect()
}

pub fn decode_vm_bytes(mut vm_bytes: Vec<i64>) -> (String, Vec<i64>) {
    let len = vm_bytes.len() as i64;
    let offset = (vm_bytes[len as usize - 1] ^ (len + 4)) as usize;

    let f_len = vm_bytes[offset + 1] as usize + 2;
    let f: Vec<i64> = vm_bytes[offset..offset + f_len].to_vec();
    vm_bytes.drain(offset..offset + f_len);

    let decoded = decode_string(&f, &mut 0);
    (decoded, vm_bytes)
}

pub fn get_value(t: GetType, x: i64, memory: &mut Memory) -> Value {
    match t {
        GetType::L => Value {
            value: x >> 5,
            value_type: ValueType::INDEX,
        },
        GetType::E => {
            if x & 1 == 1 {
                Value {
                    value: x >> 1,
                    value_type: ValueType::LITERAL,
                }
            } else {
                Value {
                    value: x >> 5,
                    value_type: ValueType::INDEX,
                }
            }
        }
        GetType::I => {
            let index = Value {
                value: x >> 5,
                value_type: ValueType::INDEX,
            };

            Value {
                value: memory.get_value(index, false).value,
                value_type: ValueType::LITERAL,
            }
        }
        _ => Value {
            value: 0,
            value_type: ValueType::UNKNOWN,
        },
    }
}
