use super::memory::Memory;
use super::parser::find_list;
use super::utils::get_value;
use super::visitors::opcodes::{GetType, Opcode, Opcodes};
use super::visitors::values::Values;

fn process_binary_operation<F>(
    op_name: &str,
    memory: &mut Memory,
    drained_bytes: &[i64],
    i: usize,
    opcodes: &[Opcode],
    operation: F,
) where
    F: Fn(i64, i64) -> i64,
{
    let opcode = opcodes
        .iter()
        .find(|opcode| opcode.index == drained_bytes[i] as usize);

    let opcode = match opcode {
        Some(opcode) => opcode,
        None => panic!(
            "Opcode not found: {:?}, opcode: {:?}",
            drained_bytes[i], op_name
        ),
    };

    let mut x = get_value(opcode.left, drained_bytes[i + 1]);
    let mut y = get_value(opcode.right, drained_bytes[i + 2]);
    let index = get_value(GetType::L, drained_bytes[i + 3]);

    x = memory.get_value(x);
    y = memory.get_value(y);

    let result = operation(x.value, y.value);

    memory.set_value(result, index.value as usize);

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
    list.drain(0..2);

    let mut memory = Memory::new();

    let mut key = Vec::new();

    let decoded_chars = decoded.chars().collect::<Vec<char>>();

    let mut start_looking = false;

    let mut last_string = String::new();

    for i in 0..drained_bytes.len() {
        let byte = drained_bytes[i];
        match byte {
            b if opcodes
                .get_property
                .iter()
                .any(|opcode| opcode.index == b as usize) =>
            {
                let opcode = opcodes
                    .get_property
                    .iter()
                    .find(|opcode| opcode.index == b as usize)
                    .unwrap();

                if drained_bytes[i + 2] == values.get_list[0] as i64 {
                    let z = get_value(opcode.left, drained_bytes[i + 1]);
                    let x = drained_bytes[i + 3] as usize;
                    let y = drained_bytes[i + 4] as usize;

                    let mut string = String::new();
                    if x + y <= decoded_chars.len() {
                        string = decoded_chars[y..(x + y)].iter().collect();
                        if string == "slice" {
                            list.pop();
                        }

                        if string == "length" && last_string == "slice" {
                            start_looking = true;

                            let index = get_value(GetType::L, drained_bytes[i + 5]);
                            memory.set_list(list.clone(), z.value as usize, index.value as usize);
                        }

                        if string == "fromCharCode" && start_looking {
                            list.pop();

                            let index = get_value(GetType::L, drained_bytes[i + 5]);
                            memory.set_list(list.clone(), z.value as usize, index.value as usize);
                        }

                        last_string = string.clone();
                    }

                    if start_looking && string != "" {
                        println!(
                            "GET_PROPERTY: {:?} {:?}[{:?}], save: {:?}",
                            i,
                            z,
                            string,
                            get_value(GetType::L, drained_bytes[i + 5])
                        );
                    }
                } else if start_looking {
                    let x = get_value(opcode.left, drained_bytes[i + 1]);
                    let index = get_value(GetType::L, drained_bytes[i + 2]);

                    if memory.get_value(x.clone()).value == (list.len() - 1) as i64 {
                        memory.set_value(list[list.len() - 1], index.value as usize);
                    }
                }
            }

            b if opcodes
                .store_global
                .iter()
                .any(|opcode| opcode.index == b as usize)
                && start_looking =>
            {
                let opcode = opcodes
                    .store_global
                    .iter()
                    .find(|opcode| opcode.index == b as usize)
                    .unwrap();

                let x = get_value(opcode.left, drained_bytes[i + 1]);
                let y = get_value(opcode.right, drained_bytes[i + 3]);

                let value = memory.get_value(y.clone());
                memory.set_global_value(value.value, x.value as usize);
            }

            b if opcodes
                .get_global
                .iter()
                .any(|opcode| opcode.index == b as usize)
                && start_looking =>
            {
                let opcode = opcodes
                    .get_global
                    .iter()
                    .find(|opcode| opcode.index == b as usize)
                    .unwrap();

                let x = get_value(opcode.right, drained_bytes[i + 1]);
                let y = get_value(GetType::L, drained_bytes[i + 2]);

                let value = memory.get_global_value(x.clone());
                memory.set_value(value.value, y.value as usize);
            }

            b if opcodes
                .r_shift
                .iter()
                .any(|opcode| opcode.index == b as usize)
                && start_looking =>
            {
                process_binary_operation(
                    "R_SHIFT",
                    &mut memory,
                    &drained_bytes,
                    i,
                    &opcodes.r_shift,
                    |x, y| x >> y,
                );
            }

            b if opcodes
                .l_shift
                .iter()
                .any(|opcode| opcode.index == b as usize)
                && start_looking =>
            {
                process_binary_operation(
                    "L_SHIFT",
                    &mut memory,
                    &drained_bytes,
                    i,
                    &opcodes.l_shift,
                    |x, y| x << y,
                );
            }

            b if opcodes.add.iter().any(|opcode| opcode.index == b as usize) && start_looking => {
                process_binary_operation(
                    "ADD",
                    &mut memory,
                    &drained_bytes,
                    i,
                    &opcodes.add,
                    |x, y| x + y,
                );
            }

            b if opcodes
                .subtract
                .iter()
                .any(|opcode| opcode.index == b as usize)
                && start_looking =>
            {
                process_binary_operation(
                    "SUB",
                    &mut memory,
                    &drained_bytes,
                    i,
                    &opcodes.subtract,
                    |x, y| x - y,
                );
            }

            b if opcodes.xor.iter().any(|opcode| opcode.index == b as usize) && start_looking => {
                process_binary_operation(
                    "XOR",
                    &mut memory,
                    &drained_bytes,
                    i,
                    &opcodes.xor,
                    |x, y| x ^ y,
                );
            }

            b if opcodes.and.iter().any(|opcode| opcode.index == b as usize) && start_looking => {
                process_binary_operation(
                    "AND",
                    &mut memory,
                    &drained_bytes,
                    i,
                    &opcodes.and,
                    |x, y| x & y,
                );
            }

            b if opcodes.or.iter().any(|opcode| opcode.index == b as usize) && start_looking => {
                process_binary_operation(
                    "OR",
                    &mut memory,
                    &drained_bytes,
                    i,
                    &opcodes.or,
                    |x, y| x | y,
                );
            }

            b if opcodes
                .divide
                .iter()
                .any(|opcode| opcode.index == b as usize)
                && start_looking =>
            {
                process_binary_operation(
                    "DIV",
                    &mut memory,
                    &drained_bytes,
                    i,
                    &opcodes.divide,
                    |x, y| x / y,
                );
            }

            b if opcodes
                .modulus
                .iter()
                .any(|opcode| opcode.index == b as usize)
                && start_looking =>
            {
                process_binary_operation(
                    "MOD",
                    &mut memory,
                    &drained_bytes,
                    i,
                    &opcodes.modulus,
                    |x, y| x % y,
                );
            }

            b if opcodes.not.iter().any(|opcode| opcode.index == b as usize) && start_looking => {
                let opcode = opcodes
                    .not
                    .iter()
                    .find(|opcode| opcode.index == b as usize)
                    .unwrap();

                let x = get_value(opcode.left, drained_bytes[i + 1]);
                let index = get_value(GetType::L, drained_bytes[i + 2]);

                let value = !x.value;
                memory.set_value(value, index.value as usize);

                println!("NOT: {:?} {:?} NOT {:?}; save: {:?}", i, x, value, index);
            }

            _ => {}
        }
    }

    key
}
