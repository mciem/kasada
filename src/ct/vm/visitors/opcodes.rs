use oxc_ast::ast::*;
use oxc_ast::visit::walk;
use oxc_ast::Visit;

use rustc_hash::FxHashMap;

use crate::ct::vm::value_handler::ValueError;

pub enum OpcodeError {
    InvalidOpcode,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum GetType {
    E,
    L,
    I,
    Unknown,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Opcode {
    pub left: GetType,
    pub right: GetType,
    pub op_type: OpcodeType,
}

#[derive(PartialEq, Debug, Clone)]
pub enum OpcodeType {
    RShift,
    LShift,
    Not,
    Xor,
    And,
    Or,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulus,
    Get,
    GetProperty,
    StoreGlobal,
    GetGlobal,
    Void,
    Apply,
    None,
}

impl Opcode {
    pub fn new(left: GetType, right: GetType, op_type: OpcodeType) -> Self {
        Self {
            left,
            right,
            op_type,
        }
    }

    // going to be used for logging purposes
    pub fn to_string(&self) -> &str {
        match self.op_type {
            OpcodeType::RShift => ">>",
            OpcodeType::LShift => "<<",
            OpcodeType::Not => "!",
            OpcodeType::Xor => "^",
            OpcodeType::And => "&",
            OpcodeType::Or => "|",
            OpcodeType::Add => "+",
            OpcodeType::Subtract => "-",
            OpcodeType::Multiply => "*",
            OpcodeType::Divide => "/",
            OpcodeType::Modulus => "%",
            OpcodeType::Get => "get",
            OpcodeType::GetProperty => "getProperty",
            OpcodeType::StoreGlobal => "storeGlobal",
            OpcodeType::GetGlobal => "getGlobal",
            OpcodeType::Void => "void",
            OpcodeType::Apply => "apply",
            OpcodeType::None => "none",
        }
    }

    pub fn is_executable(&self) -> bool {
        match self.op_type {
            OpcodeType::RShift
            | OpcodeType::LShift
            | OpcodeType::Xor
            | OpcodeType::And
            | OpcodeType::Or
            | OpcodeType::Add
            | OpcodeType::Subtract
            | OpcodeType::Multiply
            | OpcodeType::Divide
            | OpcodeType::Modulus => true,
            _ => false,
        }
    }

    pub fn execute(&self, x: i64, y: i64) -> Result<i64, ValueError> {
        match self.op_type {
            OpcodeType::RShift => Ok(x >> y),
            OpcodeType::LShift => Ok(x << y),
            OpcodeType::Xor => Ok(x ^ y),
            OpcodeType::And => Ok(x & y),
            OpcodeType::Or => Ok(x | y),
            OpcodeType::Add => Ok(x + y),
            OpcodeType::Subtract => Ok(x - y),
            OpcodeType::Multiply => Ok(x * y),
            OpcodeType::Divide => {
                if y == 0 {
                    Err(ValueError::MathError("Division by zero".to_string()))
                } else {
                    Ok(x / y)
                }
            }
            OpcodeType::Modulus => {
                if y == 0 {
                    Err(ValueError::MathError("Modulus by zero".to_string()))
                } else {
                    Ok(x % y)
                }
            }
            _ => Err(ValueError::MathError("Operation not supported".to_string())),
        }
    }
}

#[derive(Debug, Default)]
pub struct OpcodesHandler {
    list: FxHashMap<usize, Opcode>,
}

impl OpcodesHandler {
    pub fn new() -> Self {
        Self {
            list: FxHashMap::default(),
        }
    }

    pub fn add(&mut self, index: usize, op: Opcode) {
        self.list.insert(index, op);
    }

    pub fn get(&self, index: usize) -> Result<&Opcode, OpcodeError> {
        if let Some(list) = self.list.get(&index) {
            Ok(list)
        } else {
            Err(OpcodeError::InvalidOpcode)
        }
    }

    pub fn get_by_type(&self, op_type: OpcodeType) -> Vec<&Opcode> {
        self.list
            .values()
            .filter(|op| op.op_type == op_type)
            .collect()
    }
}

#[derive(Default)]
pub struct OpcodeVisitor {
    pub opcodes: OpcodesHandler,
    pub array: String,
    pub current_index: usize,
}

impl OpcodeVisitor {
    fn determine_type(expr: &Expression) -> GetType {
        if let Expression::CallExpression(call_expr) = expr {
            if let Expression::Identifier(ident) = &call_expr.callee {
                match &*ident.name.as_str() {
                    "e" => return GetType::E,
                    "l" => return GetType::L,
                    _ => return GetType::Unknown,
                }
            }
        }
        GetType::Unknown
    }
}

impl<'a> Visit<'a> for OpcodeVisitor {
    fn visit_variable_declarator(&mut self, it: &VariableDeclarator<'a>) {
        if let Some(init) = &it.init {
            if let Expression::ArrayExpression(array_lit) = &*init {
                if array_lit.elements.len() == 96 {
                    if let Some(ident) = &it.id.get_identifier() {
                        self.array = ident.as_str().to_string();
                        array_lit
                            .elements
                            .iter()
                            .enumerate()
                            .for_each(|(index, elem)| {
                                if let ArrayExpressionElement::FunctionExpression(expr) = elem {
                                    self.current_index = index;
                                    self.visit_function(&expr, oxc_semantic::ScopeFlags::all());
                                }
                            });
                    }
                }
            }
        }
    }

    fn visit_function(&mut self, it: &Function<'a>, _: oxc_semantic::ScopeFlags) {
        walk::walk_function(self, it, oxc_semantic::ScopeFlags::all());
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        match &it.callee {
            Expression::Identifier(ident) => {
                if &*ident.name.as_str() == "a" {
                    for arg in &it.arguments {
                        match &*arg {
                            Argument::UnaryExpression(unary_expr) => {
                                let arg_type = Self::determine_type(&unary_expr.argument);

                                match unary_expr.operator {
                                    UnaryOperator::BitwiseNot => {
                                        self.opcodes.add(
                                            self.current_index,
                                            Opcode {
                                                left: arg_type,
                                                right: GetType::Unknown,
                                                op_type: OpcodeType::Not,
                                            },
                                        );
                                    }
                                    UnaryOperator::Void => {
                                        self.opcodes.add(
                                            self.current_index,
                                            Opcode {
                                                left: arg_type,
                                                right: GetType::Unknown,
                                                op_type: OpcodeType::Void,
                                            },
                                        );
                                    }
                                    _ => {}
                                };
                            }
                            Argument::BinaryExpression(bin_expr) => {
                                let left_type = Self::determine_type(&bin_expr.left);
                                let right_type = Self::determine_type(&bin_expr.right);
                                let mut opcode = Opcode {
                                    left: left_type,
                                    right: right_type,
                                    op_type: OpcodeType::None,
                                };

                                match bin_expr.operator {
                                    BinaryOperator::ShiftRight => {
                                        opcode.op_type = OpcodeType::RShift;
                                        self.opcodes.add(self.current_index, opcode);
                                    }
                                    BinaryOperator::ShiftLeft => {
                                        opcode.op_type = OpcodeType::LShift;
                                        self.opcodes.add(self.current_index, opcode);
                                    }
                                    BinaryOperator::BitwiseXOR => {
                                        opcode.op_type = OpcodeType::Xor;
                                        self.opcodes.add(self.current_index, opcode);
                                    }
                                    BinaryOperator::BitwiseAnd => {
                                        opcode.op_type = OpcodeType::And;
                                        self.opcodes.add(self.current_index, opcode);
                                    }
                                    BinaryOperator::BitwiseOR => {
                                        opcode.op_type = OpcodeType::Or;
                                        self.opcodes.add(self.current_index, opcode);
                                    }
                                    BinaryOperator::Addition => {
                                        opcode.op_type = OpcodeType::Add;
                                        self.opcodes.add(self.current_index, opcode);
                                    }
                                    BinaryOperator::Subtraction => {
                                        opcode.op_type = OpcodeType::Subtract;
                                        self.opcodes.add(self.current_index, opcode);
                                    }
                                    BinaryOperator::Multiplication => {
                                        opcode.op_type = OpcodeType::Multiply;
                                        self.opcodes.add(self.current_index, opcode);
                                    }
                                    BinaryOperator::Division => {
                                        opcode.op_type = OpcodeType::Divide;
                                        self.opcodes.add(self.current_index, opcode);
                                    }
                                    BinaryOperator::Remainder => {
                                        opcode.op_type = OpcodeType::Modulus;
                                        self.opcodes.add(self.current_index, opcode);
                                    }
                                    _ => (),
                                };
                            }
                            Argument::ComputedMemberExpression(computed) => {
                                let obj_type = Self::determine_type(&computed.object);
                                let prop_type = Self::determine_type(&computed.expression);

                                if computed.object.is_member_expression() {
                                    self.opcodes.add(
                                        self.current_index,
                                        Opcode {
                                            left: GetType::Unknown,
                                            right: GetType::E,
                                            op_type: OpcodeType::GetGlobal,
                                        },
                                    );
                                } else {
                                    if obj_type != GetType::Unknown && prop_type != GetType::Unknown
                                    {
                                        self.opcodes.add(
                                            self.current_index,
                                            Opcode {
                                                left: obj_type,
                                                right: prop_type,
                                                op_type: OpcodeType::GetProperty,
                                            },
                                        );
                                    }
                                }
                            }
                            Argument::CallExpression(call_expr) => {
                                if let Expression::Identifier(ident) = &call_expr.callee {
                                    if &*ident.name.as_str() == "e" {
                                        self.opcodes.add(
                                            self.current_index,
                                            Opcode {
                                                left: GetType::E,
                                                right: GetType::Unknown,
                                                op_type: OpcodeType::Get,
                                            },
                                        );
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            _ => {}
        }

        walk::walk_call_expression(self, it);
    }

    fn visit_assignment_expression(&mut self, it: &AssignmentExpression<'a>) {
        let mut opcode = Opcode {
            left: GetType::E,
            right: GetType::E,
            op_type: OpcodeType::None,
        };

        if let AssignmentTarget::ComputedMemberExpression(computed) = &it.left {
            match &computed.expression {
                Expression::Identifier(ident) => {
                    if &*ident.name.as_str() != "u" {
                        return;
                    }
                }

                Expression::NumericLiteral(num) => {
                    if num.value == 2.0 {
                        if let Expression::StaticMemberExpression(member) = &computed.object {
                            if let Expression::Identifier(_) = &member.object {
                                if let Expression::CallExpression(_) = &it.right {
                                    opcode.op_type = OpcodeType::Apply;
                                    self.opcodes.add(self.current_index, opcode);

                                    return;
                                }
                            }
                        }
                    }
                }

                _ => {}
            }
        }

        match &it.right {
            Expression::Identifier(ident) => {
                if &*ident.name.as_str() == "r" {
                    opcode.op_type = OpcodeType::StoreGlobal;
                    self.opcodes.add(self.current_index, opcode);
                }
            }

            Expression::UnaryExpression(unary) => {
                if unary.operator == UnaryOperator::Void {
                    opcode.right = GetType::Unknown;

                    opcode.op_type = OpcodeType::Void;
                    self.opcodes.add(self.current_index, opcode);

                    return;
                }
            }

            _ => {}
        }
    }
}
