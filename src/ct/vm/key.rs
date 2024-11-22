use super::parser::find_list;
use super::value_handler::{Value, ValueError, ValueHandler};
use super::visitors::opcodes::{GetType, OpcodeType, OpcodesHandler};
use super::visitors::values::GetValues;

#[derive(Default)]
pub struct KeyBuilder {
    pub drained_bytes: Vec<i64>,
    pub handler: ValueHandler,
    pub opcodes: OpcodesHandler,
    pub list: Vec<i64>,
    pub get: GetValues,

    pub key: Vec<u8>,
    pub iv: Vec<u8>,
}

impl KeyBuilder {
    pub fn new(
        drained_bytes: Vec<i64>,
        decoded: String,
        get: GetValues,
        opcodes: OpcodesHandler,
    ) -> Self {
        let handler = ValueHandler::new(decoded.chars().collect(), get.clone());
        let list = find_list(decoded);

        Self {
            drained_bytes,
            handler,
            opcodes,
            list,
            get,

            key: Vec::new(),
            iv: Vec::new(),
        }
    }

    pub fn generate(&mut self) -> Result<Vec<u8>, ValueError> {
        let mut list_loaded = false;
        let mut list_string_loaded = false;

        let mut skip = 0;
        for i in 0..self.drained_bytes.len() {
            if skip > 0 {
                skip -= 1;
                continue;
            }

            if let Ok(opcode) = self.opcodes.get(self.drained_bytes[i] as usize) {
                if opcode.is_executable() && list_loaded {
                    let x = self
                        .handler
                        .get_value(opcode.left, vec![self.drained_bytes[i + 1]], false)?
                        .as_literal()?;

                    let y = self
                        .handler
                        .get_value(opcode.left, vec![self.drained_bytes[i + 2]], false)?
                        .as_literal()?;

                    let result = opcode.execute(x, y)?;
                    let index = self
                        .handler
                        .get_value(GetType::L, vec![self.drained_bytes[i + 3]], false)?
                        .as_literal()?;

                    self.handler
                        .set_value(Value::Literal(result), index as usize, false)?;
                }

                match opcode.op_type {
                    OpcodeType::GetProperty => {
                        if !list_loaded {
                            continue;
                        }

                        let x = self.handler.get_value(
                            opcode.left,
                            vec![self.drained_bytes[i + 1]],
                            false,
                        )?;

                        let y = self.handler.get_value(
                            opcode.left,
                            vec![
                                self.drained_bytes[i + 2],
                                self.drained_bytes[i + 3],
                                self.drained_bytes[i + 4],
                            ],
                            false,
                        )?;

                        if y.is_string() {
                            // string handling here
                            // ...

                            skip = 5;
                        } else {
                            let _index = self.handler.get_value(
                                GetType::I,
                                vec![self.drained_bytes[i + 3]],
                                false,
                            )?;

                            // non string handling
                            // ...

                            skip = 3;
                        }
                    }

                    /*
                    OpcodeType::Apply => {
                        have to do this ...
                    }
                    */
                    OpcodeType::Get => {
                        if !list_loaded {
                            continue;
                        }

                        let x = self.handler.get_value(
                            opcode.left,
                            vec![
                                self.drained_bytes[i + 1],
                                self.drained_bytes[i + 2],
                                self.drained_bytes[i + 3],
                            ],
                            false,
                        )?;

                        let index = if x.is_string() {
                            self.handler
                                .get_value(GetType::I, vec![self.drained_bytes[i + 4]], false)?
                                .as_literal()?
                        } else {
                            self.handler
                                .get_value(GetType::I, vec![self.drained_bytes[i + 2]], false)?
                                .as_literal()?
                        };

                        self.handler.set_value(x, index as usize, false)?;

                        skip = 2;
                    }

                    OpcodeType::StoreGlobal => {
                        if !list_string_loaded {
                            if let Ok(x) = self.handler.get_value(
                                opcode.right,
                                vec![
                                    self.drained_bytes[i + 2],
                                    self.drained_bytes[i + 3],
                                    self.drained_bytes[i + 4],
                                ],
                                false,
                            ) {
                                if x.is_string() {
                                    let str = x.as_string()?;
                                    if str.as_bytes().len() == 0 {
                                        continue;
                                    }

                                    if str.as_bytes()[0] == '[' as u8 {
                                        list_string_loaded = true;

                                        let index = self.handler.get_value(
                                            opcode.left,
                                            vec![self.drained_bytes[i + 1]],
                                            false,
                                        )?;
                                        println!("StoreGlobal : {:?} : {:?}", index, str);
                                        self.handler.set_value(
                                            x,
                                            index.as_literal()? as usize,
                                            true,
                                        )?;
                                    };
                                }
                            }

                            continue;
                        }

                        if !list_loaded {
                            continue;
                        }

                        let mut i_2 = i.clone();
                        loop {
                            i_2 += 1;

                            if let Ok(x) = self.handler.get_value(
                                opcode.left,
                                vec![self.drained_bytes[i_2]],
                                false,
                            ) {
                                let y = self.handler.get_value(
                                    opcode.right,
                                    vec![
                                        self.drained_bytes[i_2 + 1],
                                        self.drained_bytes[i_2 + 2],
                                        self.drained_bytes[i_2 + 3],
                                    ],
                                    false,
                                )?;

                                self.handler.set_value(y, x.as_literal()? as usize, true)?;

                                break;
                            }
                        }
                    }

                    OpcodeType::GetGlobal => {
                        if !list_string_loaded {
                            continue;
                        }

                        if !list_loaded {
                            if let Ok(x) = self.handler.get_value(
                                opcode.right,
                                vec![self.drained_bytes[i + 1]],
                                true,
                            ) {
                                println!(
                                    "GetGlobal : {:?} : {:?}",
                                    self.drained_bytes[i + 1] >> 1,
                                    x
                                );
                            }
                            continue;
                        }

                        let mut i_2 = i.clone();

                        loop {
                            i_2 += 1;

                            if let Ok(x) = self.handler.get_value(
                                opcode.right,
                                vec![self.drained_bytes[i_2]],
                                true,
                            ) {
                                let y = self.handler.get_value(
                                    GetType::I,
                                    vec![self.drained_bytes[i_2 + 1]],
                                    false,
                                )?;

                                self.handler.set_value(x, y.as_literal()? as usize, false)?;

                                break;
                            }
                        }
                    }

                    OpcodeType::Not => {
                        if !list_loaded {
                            continue;
                        }

                        let x = self.handler.get_value(
                            opcode.left,
                            vec![self.drained_bytes[i + 1]],
                            false,
                        )?;

                        let index = self
                            .handler
                            .get_value(GetType::I, vec![self.drained_bytes[i + 2]], false)?
                            .as_literal()?;

                        let value = !x.as_literal()?;
                        self.handler
                            .set_value(Value::Literal(value), index as usize, false)?;

                        skip = 2;
                    }

                    OpcodeType::Void => {
                        if !list_loaded {
                            continue;
                        }

                        let x = self.handler.get_value(
                            GetType::I,
                            vec![self.drained_bytes[i + 2]],
                            false,
                        )?;

                        self.handler.delete_value(x.as_literal()? as usize, true)?;

                        skip = 2;
                    }

                    _ => {}
                }
            }
        }

        Ok(self.key.clone())
    }
}
