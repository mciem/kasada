use swc_ecma_ast::*;
use swc_ecma_visit::{Visit, VisitWith};

use rustc_hash::FxHashMap;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum GetType {
    E,
    L,
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
    pub store_global: FxHashMap<usize, Opcode>,
    pub get_global: FxHashMap<usize, Opcode>,
    pub get_property: FxHashMap<usize, Opcode>,
    pub delete: FxHashMap<usize, Opcode>,
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
                store_global: FxHashMap::default(),
                get_global: FxHashMap::default(),
                get_property: FxHashMap::default(),
                delete: FxHashMap::default(),
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
                                self.opcodes.not.insert(
                                    self.current_index,
                                    Opcode {
                                        left: arg_type,
                                        right: GetType::Unknown,
                                    },
                                );
                            } else if unary_expr.op == UnaryOp::Delete {
                                let arg_type = Self::determine_type(&unary_expr.arg);
                                self.opcodes.delete.insert(
                                    self.current_index,
                                    Opcode {
                                        left: arg_type,
                                        right: GetType::Unknown,
                                    },
                                );
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

                            op_vec.insert(
                                self.current_index,
                                Opcode {
                                    left: left_type,
                                    right: right_type,
                                },
                            );
                        } else if let Expr::Member(member_expr) = &*arg.expr {
                            if let MemberProp::Computed(computed) = &member_expr.prop {
                                let obj_type = Self::determine_type(&member_expr.obj);
                                let prop_type = Self::determine_type(&computed.expr);

                                if member_expr.obj.is_member() {
                                    self.opcodes.get_global.insert(
                                        self.current_index,
                                        Opcode {
                                            left: GetType::Unknown,
                                            right: GetType::E,
                                        },
                                    );

                                    return;
                                }

                                self.opcodes.get_property.insert(
                                    self.current_index,
                                    Opcode {
                                        left: obj_type,
                                        right: prop_type,
                                    },
                                );
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
            self.opcodes.store_global.insert(self.current_index, opcode);
        }

        assign_expr.visit_children_with(self);
    }
}
