use super::memory::Memory;
use super::parser::find_list;
use super::utils::{get_value, ValueType};
use super::visitors::opcodes::{GetType, Opcode, Opcodes};
use super::visitors::values::Values;
use rustc_hash::FxHashMap;

fn process_binary_operation<F>(
    op_name: &str,
    memory: &mut Memory,
    drained_bytes: &[i64],
    i: usize,
    opcodes: &FxHashMap<usize, Opcode>,
    operation: F,
) where
    F: Fn(i64, i64) -> i64,
{
    let opcode = opcodes.get(&(drained_bytes[i] as usize));

    let opcode = match opcode {
        Some(opcode) => opcode,
        _ => panic!(
            "Opcode not found: {:?}, opcode: {:?}",
            drained_bytes[i], op_name
        ),
    };

    let mut x = get_value(opcode.left, drained_bytes[i + 1], memory);
    let mut y = get_value(opcode.right, drained_bytes[i + 2], memory);
    let index = get_value(GetType::L, drained_bytes[i + 3], memory);

    x = memory.get_value(x, false)[0];
    y = memory.get_value(y, false)[0];

    let result = operation(x.value, y.value);

    memory.set_value(result, index.value as usize, false);

    println!(
        "{}: {:?} {:?} {} {:?} = {:?}; save: {:?}",
        op_name, i, x, op_name, y, result, index
    );
}

pub fn get_key(
    drained_bytes: Vec<i64>,
    decoded: String,
    opcodes: Opcodes,
    values: Values,
) -> Vec<u8> {
    let mut list = find_list(decoded.clone());
    list.drain(0..1);

    let decoded_chars = decoded.chars().collect::<Vec<char>>();
    let mut start_looking = false;
    let mut last_string = String::new();
    let mut memory = Memory::new();
    let mut skip = 0;
    let mut key = Vec::new();

    for i in 0..drained_bytes.len() {
        if skip > 0 {
            skip -= 1;
            continue;
        }

        let byte = drained_bytes[i];
        match byte {
            b if opcodes.get_property.contains_key(&(b as usize)) => {
                let opcode = opcodes.get_property.get(&(b as usize)).unwrap();

                if drained_bytes[i + 2] == values.get_list[0] as i64 {
                    let x = drained_bytes[i + 3] as usize;
                    let y = drained_bytes[i + 4] as usize;
                    let z = get_value(opcode.left, drained_bytes[i + 1], &mut memory);

                    if x + y <= decoded_chars.len() {
                        let string: String = decoded_chars[y..(x + y)].iter().collect();

                        if string == "slice" && start_looking {
                            list.drain(0..1);
                        }

                        if string == "slice" && last_string == "toString" {
                            start_looking = true;

                            memory.set_list(list.clone(), z.value as usize, false);
                        } else if string == "length" && start_looking {
                            let index = get_value(GetType::L, drained_bytes[i + 5], &mut memory);

                            memory.set_value(list.len() as i64, index.value as usize, false);
                        }

                        last_string = string.clone();
                    }

                    skip = 5;
                } else if start_looking {
                    let x = get_value(opcode.left, drained_bytes[i + 1], &mut memory);
                    let y = get_value(opcode.right, drained_bytes[i + 2], &mut memory);
                    let index = get_value(GetType::L, drained_bytes[i + 3], &mut memory);

                    if memory.get_value(y.clone(), false)[0].value == (list.len() - 1) as i64 {
                        memory.set_value(list[list.len() - 1], index.value as usize, false);
                        list.pop();
                    } else if memory.get_value(y.clone(), false)[0].value == 1 as i64 {
                        memory.set_value(list[1], index.value as usize, false);

                        list.drain(0..1);
                    }

                    skip = 3;
                } else {
                    let x = get_value(opcode.left, drained_bytes[i + 1], &mut memory);
                    let y = get_value(opcode.right, drained_bytes[i + 2], &mut memory);
                    let index = get_value(GetType::L, drained_bytes[i + 3], &mut memory);

                    if x.value_type == ValueType::INDEX
                        && y.value_type == ValueType::LITERAL
                        && y.value == 1
                    {
                        memory.set_value(list[0], index.value as usize, false);
                    }

                    skip = 3;
                }
            }

            b if opcodes.get.contains_key(&(b as usize)) && start_looking => {
                let opcode = opcodes.get.get(&(b as usize)).unwrap();

                let x = get_value(opcode.left, drained_bytes[i + 1], &mut memory);
                let index = get_value(GetType::L, drained_bytes[i + 2], &mut memory);

                let value = memory.get_value(x.clone(), false)[0];

                println!("getttt: {:?}, {:?}", index, value);

                memory.set_value(value.value, index.value as usize, false);

                skip = 2;
            }

            b if opcodes.store_global.contains_key(&(b as usize)) && start_looking => {
                let opcode = opcodes.store_global.get(&(b as usize)).unwrap();

                let mut z = i.clone();
                loop {
                    z += 1;

                    let x = get_value(opcode.left, drained_bytes[z], &mut memory);
                    if x.value_type == ValueType::UNKNOWN {
                        continue;
                    }

                    let y = get_value(opcode.right, drained_bytes[z + 1], &mut memory);
                    let values = memory.get_value(y.clone(), false);

                    if values.len() == 1 {
                        memory.set_value(values[0].value, x.value as usize, true);
                    } else {
                        memory.set_list(
                            values.iter().map(|&x| x.value).collect(),
                            x.value as usize,
                            true,
                        );
                    }

                    break;
                }

                skip = z - i;
            }

            b if opcodes.get_global.contains_key(&(b as usize)) && start_looking => {
                let opcode = opcodes.get_global.get(&(b as usize)).unwrap();

                let mut z = i.clone();
                loop {
                    z += 1;

                    let mut x = get_value(opcode.right, drained_bytes[z], &mut memory);
                    if x.value_type == ValueType::UNKNOWN {
                        continue;
                    }
                    x.value_type = ValueType::INDEX;

                    let index = get_value(opcode.right, drained_bytes[z + 1], &mut memory);
                    let values = memory.get_value(x.clone(), true);

                    if values.len() == 1 {
                        memory.set_value(values[0].value, index.value as usize, false);
                    } else {
                        memory.set_list(
                            values.iter().map(|&x| x.value).collect(),
                            index.value as usize,
                            false,
                        );
                    }

                    break;
                }

                skip = z - i;
            }

            b if opcodes.r_shift.contains_key(&(b as usize)) && start_looking => {
                process_binary_operation(
                    "R_SHIFT",
                    &mut memory,
                    &drained_bytes,
                    i,
                    &opcodes.r_shift,
                    |x, y| x >> y,
                );

                skip = 3;
            }

            b if opcodes.l_shift.contains_key(&(b as usize)) && start_looking => {
                process_binary_operation(
                    "L_SHIFT",
                    &mut memory,
                    &drained_bytes,
                    i,
                    &opcodes.l_shift,
                    |x, y| x << y,
                );

                skip = 3;
            }

            b if opcodes.add.contains_key(&(b as usize)) && start_looking => {
                process_binary_operation(
                    "ADD",
                    &mut memory,
                    &drained_bytes,
                    i,
                    &opcodes.add,
                    |x, y| x + y,
                );

                skip = 3;
            }

            b if opcodes.subtract.contains_key(&(b as usize)) && start_looking => {
                process_binary_operation(
                    "SUB",
                    &mut memory,
                    &drained_bytes,
                    i,
                    &opcodes.subtract,
                    |x, y| x - y,
                );

                skip = 3;
            }

            b if opcodes.xor.contains_key(&(b as usize)) && start_looking => {
                process_binary_operation(
                    "XOR",
                    &mut memory,
                    &drained_bytes,
                    i,
                    &opcodes.xor,
                    |x, y| x ^ y,
                );

                skip = 3;
            }

            b if opcodes.and.contains_key(&(b as usize)) && start_looking => {
                process_binary_operation(
                    "AND",
                    &mut memory,
                    &drained_bytes,
                    i,
                    &opcodes.and,
                    |x, y| x & y,
                );

                skip = 3;
            }

            b if opcodes.or.contains_key(&(b as usize)) && start_looking => {
                process_binary_operation(
                    "OR",
                    &mut memory,
                    &drained_bytes,
                    i,
                    &opcodes.or,
                    |x, y| x | y,
                );

                skip = 3;
            }

            b if opcodes.divide.contains_key(&(b as usize)) && start_looking => {
                process_binary_operation(
                    "DIV",
                    &mut memory,
                    &drained_bytes,
                    i,
                    &opcodes.divide,
                    |x, y| x / y,
                );

                skip = 3;
            }

            b if opcodes.modulus.contains_key(&(b as usize)) && start_looking => {
                process_binary_operation(
                    "MOD",
                    &mut memory,
                    &drained_bytes,
                    i,
                    &opcodes.modulus,
                    |x, y| x % y,
                );

                skip = 3;
            }

            b if opcodes.not.contains_key(&(b as usize)) && start_looking => {
                let opcode = opcodes.not.get(&(b as usize)).unwrap();

                let mut x = get_value(opcode.left, drained_bytes[i + 1], &mut memory);
                let index = get_value(GetType::L, drained_bytes[i + 2], &mut memory);

                x = memory.get_value(x, false)[0];

                let value = !x.value;
                memory.set_value(value, index.value as usize, false);

                println!("NOT: {:?} {:?} NOT {:?}; save: {:?}", i, x, value, index);

                skip = 2;
            }
            _ => {}
        }
    }

    key
}
