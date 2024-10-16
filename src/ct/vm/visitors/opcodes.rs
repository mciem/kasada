use swc_ecma_ast::*;
use swc_ecma_visit::{Visit, VisitWith};

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum GetType {
    E,
    L,
    Unknown,
}

#[derive(PartialEq, Debug)]
pub struct Opcode {
    pub index: usize,
    pub left: GetType,
    pub right: GetType,
}

#[derive(Debug)]
pub struct Opcodes {
    pub r_shift: Vec<Opcode>,
    pub l_shift: Vec<Opcode>,
    pub not: Vec<Opcode>,
    pub xor: Vec<Opcode>,
    pub and: Vec<Opcode>,
    pub or: Vec<Opcode>,
    pub add: Vec<Opcode>,
    pub subtract: Vec<Opcode>,
    pub multiply: Vec<Opcode>,
    pub divide: Vec<Opcode>,
    pub modulus: Vec<Opcode>,
    pub store_global: Vec<Opcode>,
    pub get_global: Vec<Opcode>,
    pub get_property: Vec<Opcode>,
    pub delete: Vec<Opcode>,
}

pub struct OpcodeVisitor {
    pub opcodes: Opcodes,
    pub array: String,
    pub current_index: usize,
}

impl OpcodeVisitor {
    fn determine_type(expr: &Expr) -> GetType {
        if let Expr::Call(call_expr) = expr {
            if let Callee::Expr(callee_expr) = &call_expr.callee {
                if let Expr::Ident(ident) = &**callee_expr {
                    match ident.sym.as_ref() {
                        "e" => return GetType::E,
                        "l" => return GetType::L,
                        _ => {
                            return GetType::Unknown;
                        }
                    }
                }
            }
        }
        GetType::Unknown
    }

    pub fn new() -> Self {
        Self {
            opcodes: Opcodes {
                r_shift: Vec::new(),
                l_shift: Vec::new(),
                not: Vec::new(),
                xor: Vec::new(),
                and: Vec::new(),
                or: Vec::new(),
                add: Vec::new(),
                subtract: Vec::new(),
                multiply: Vec::new(),
                divide: Vec::new(),
                modulus: Vec::new(),
                store_global: Vec::new(),
                get_global: Vec::new(),
                get_property: Vec::new(),
                delete: Vec::new(),
            },
            array: String::new(),
            current_index: 0,
        }
    }
}

impl Visit for OpcodeVisitor {
    fn visit_var_declarator(&mut self, var_declarator: &VarDeclarator) {
        if let Some(init) = &var_declarator.init {
            if let Expr::Array(array_lit) = &**init {
                if array_lit.elems.len() == 96 {
                    if let Pat::Ident(ident) = &var_declarator.name {
                        self.array = ident.id.sym.to_string();
                        array_lit
                            .elems
                            .iter()
                            .enumerate()
                            .for_each(|(index, elem)| {
                                if let Some(expr) = elem {
                                    self.current_index = index;

                                    expr.visit_with(self);
                                }
                            });
                    }
                }
            }
        }
    }

    fn visit_call_expr(&mut self, call_expr: &CallExpr) {
        if let Callee::Expr(callee_expr) = &call_expr.callee {
            if let Expr::Ident(ident) = &**callee_expr {
                if ident.sym == "a" {
                    for arg in &call_expr.args {
                        if let Expr::Unary(unary_expr) = &*arg.expr {
                            if unary_expr.op == UnaryOp::Tilde {
                                let arg_type = Self::determine_type(&unary_expr.arg);
                                self.opcodes.not.push(Opcode {
                                    index: self.current_index,
                                    left: arg_type,
                                    right: GetType::Unknown,
                                });
                            } else if unary_expr.op == UnaryOp::Delete {
                                let arg_type = Self::determine_type(&unary_expr.arg);
                                self.opcodes.delete.push(Opcode {
                                    index: self.current_index,
                                    left: arg_type,
                                    right: GetType::Unknown,
                                });
                            }
                        }

                        if let Expr::Bin(bin_expr) = &*arg.expr {
                            let op_vec = match bin_expr.op {
                                BinaryOp::RShift => &mut self.opcodes.r_shift,
                                BinaryOp::LShift => &mut self.opcodes.l_shift,
                                BinaryOp::BitXor => &mut self.opcodes.xor,
                                BinaryOp::BitAnd => &mut self.opcodes.and,
                                BinaryOp::BitOr => &mut self.opcodes.or,
                                BinaryOp::Add => &mut self.opcodes.add,
                                BinaryOp::Sub => &mut self.opcodes.subtract,
                                BinaryOp::Mul => &mut self.opcodes.multiply,
                                BinaryOp::Div => &mut self.opcodes.divide,
                                BinaryOp::Mod => &mut self.opcodes.modulus,
                                _ => continue,
                            };

                            let left_type = Self::determine_type(&bin_expr.left);
                            let right_type = Self::determine_type(&bin_expr.right);

                            op_vec.push(Opcode {
                                index: self.current_index,
                                left: left_type,
                                right: right_type,
                            });
                        } else if let Expr::Member(member_expr) = &*arg.expr {
                            if let MemberProp::Computed(computed) = &member_expr.prop {
                                let obj_type = Self::determine_type(&member_expr.obj);
                                let prop_type = Self::determine_type(&computed.expr);

                                if member_expr.obj.is_member() {
                                    self.opcodes.get_global.push(Opcode {
                                        index: self.current_index,
                                        left: GetType::Unknown,
                                        right: GetType::E,
                                    });

                                    return;
                                }

                                self.opcodes.get_property.push(Opcode {
                                    index: self.current_index,
                                    left: obj_type,
                                    right: prop_type,
                                });
                            }
                        }
                    }
                }
            }
        }
        call_expr.visit_children_with(self);
    }

    fn visit_assign_expr(&mut self, assign_expr: &AssignExpr) {
        let mut opcode = Opcode {
            index: self.current_index,
            left: GetType::Unknown,
            right: GetType::Unknown,
        };

        if let Some(simple_expr) = assign_expr.left.as_simple() {
            if let Some(member_expr) = simple_expr.as_member() {
                if let MemberProp::Computed(computed) = &member_expr.prop {
                    if let Expr::Ident(ident) = &*computed.expr {
                        if ident.sym == "u" {
                            opcode.left = GetType::E;
                        }
                    }
                }
            }
        }

        if let Expr::Ident(ident) = &*assign_expr.right {
            if ident.sym == "r" {
                opcode.right = GetType::E;
            }
        }

        if opcode.left == GetType::E && opcode.right == GetType::E {
            self.opcodes.store_global.push(opcode);
        }

        assign_expr.visit_children_with(self);
    }
}
