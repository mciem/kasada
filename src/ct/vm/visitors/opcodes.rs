use oxc_ast::ast::*;
use oxc_ast::visit::walk;
use oxc_ast::Visit;

use rustc_hash::FxHashMap;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum GetType {
    E,
    L,
    I,
    Unknown,
}

#[derive(PartialEq, Debug)]
pub struct Opcode {
    pub left: GetType,
    pub right: GetType,
}

#[derive(Debug)]
pub struct Opcodes {
    pub r_shift: FxHashMap<usize, Opcode>,
    pub l_shift: FxHashMap<usize, Opcode>,
    pub not: FxHashMap<usize, Opcode>,
    pub xor: FxHashMap<usize, Opcode>,
    pub and: FxHashMap<usize, Opcode>,
    pub or: FxHashMap<usize, Opcode>,
    pub add: FxHashMap<usize, Opcode>,
    pub subtract: FxHashMap<usize, Opcode>,
    pub multiply: FxHashMap<usize, Opcode>,
    pub divide: FxHashMap<usize, Opcode>,
    pub modulus: FxHashMap<usize, Opcode>,
    pub get: FxHashMap<usize, Opcode>,
    pub get_property: FxHashMap<usize, Opcode>,
    pub delete: FxHashMap<usize, Opcode>,
}

pub struct OpcodeVisitor {
    pub opcodes: Opcodes,
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

    pub fn new() -> Self {
        Self {
            opcodes: Opcodes {
                r_shift: FxHashMap::default(),
                l_shift: FxHashMap::default(),
                not: FxHashMap::default(),
                xor: FxHashMap::default(),
                and: FxHashMap::default(),
                or: FxHashMap::default(),
                add: FxHashMap::default(),
                subtract: FxHashMap::default(),
                multiply: FxHashMap::default(),
                divide: FxHashMap::default(),
                modulus: FxHashMap::default(),
                get: FxHashMap::default(),
                get_property: FxHashMap::default(),
                delete: FxHashMap::default(),
            },
            array: String::new(),
            current_index: 0,
        }
    }
}

impl<'a> Visit<'a> for OpcodeVisitor {
    fn visit_variable_declarator(&mut self, it: &VariableDeclarator<'a>) {
        if let Some(init) = &it.init {
            self.visit_expression(init);

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

        walk::walk_variable_declarator(self, it);
    }

    fn visit_function(&mut self, it: &Function<'a>, _: oxc_semantic::ScopeFlags) {
        if let Some(body) = &it.body {
            for stmt in &*body.statements {
                if let Statement::ExpressionStatement(expr_stmt) = stmt {
                    if let Expression::CallExpression(call_expr) = &expr_stmt.expression {
                        self.visit_call_expression(call_expr);
                    }
                }
            }
        }

        walk::walk_function(self, it, oxc_semantic::ScopeFlags::all());
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Expression::Identifier(ident) = &it.callee {
            if &*ident.name.as_str() == "a" {
                for arg in &it.arguments {
                    match &*arg {
                        Argument::UnaryExpression(unary_expr) => {
                            let arg_type = Self::determine_type(&unary_expr.argument);
                            let opcode = Opcode {
                                left: arg_type,
                                right: GetType::Unknown,
                            };

                            match unary_expr.operator {
                                UnaryOperator::BitwiseNot => {
                                    self.opcodes.not.insert(self.current_index, opcode);
                                }
                                UnaryOperator::Delete => {
                                    self.opcodes.delete.insert(self.current_index, opcode);
                                }
                                _ => {}
                            };
                        }
                        Argument::BinaryExpression(bin_expr) => {
                            let left_type = Self::determine_type(&bin_expr.left);
                            let right_type = Self::determine_type(&bin_expr.right);
                            let opcode = Opcode {
                                left: left_type,
                                right: right_type,
                            };

                            match bin_expr.operator {
                                BinaryOperator::ShiftRight => {
                                    self.opcodes.r_shift.insert(self.current_index, opcode)
                                }
                                BinaryOperator::ShiftLeft => {
                                    self.opcodes.l_shift.insert(self.current_index, opcode)
                                }
                                BinaryOperator::BitwiseXOR => {
                                    self.opcodes.xor.insert(self.current_index, opcode)
                                }
                                BinaryOperator::BitwiseAnd => {
                                    self.opcodes.and.insert(self.current_index, opcode)
                                }
                                BinaryOperator::BitwiseOR => {
                                    self.opcodes.or.insert(self.current_index, opcode)
                                }
                                BinaryOperator::Addition => {
                                    self.opcodes.add.insert(self.current_index, opcode)
                                }
                                BinaryOperator::Subtraction => {
                                    self.opcodes.subtract.insert(self.current_index, opcode)
                                }
                                BinaryOperator::Multiplication => {
                                    self.opcodes.multiply.insert(self.current_index, opcode)
                                }
                                BinaryOperator::Division => {
                                    self.opcodes.divide.insert(self.current_index, opcode)
                                }
                                BinaryOperator::Remainder => {
                                    self.opcodes.modulus.insert(self.current_index, opcode)
                                }
                                _ => None,
                            };
                        }
                        Argument::ComputedMemberExpression(computed) => {
                            let obj_type = Self::determine_type(&computed.object);
                            let prop_type = Self::determine_type(&computed.expression);

                            if obj_type != GetType::Unknown {
                                self.opcodes.get_property.insert(
                                    self.current_index,
                                    Opcode {
                                        left: obj_type,
                                        right: prop_type,
                                    },
                                );
                            }
                        }
                        Argument::CallExpression(call_expr) => {
                            if let Expression::Identifier(ident) = &call_expr.callee {
                                if &*ident.name.as_str() == "e" {
                                    self.opcodes.get.insert(
                                        self.current_index,
                                        Opcode {
                                            left: GetType::E,
                                            right: GetType::Unknown,
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
        walk::walk_call_expression(self, it);
    }
}
