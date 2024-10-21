use swc_ecma_ast::*;
use swc_ecma_visit::{Visit, VisitWith};

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
    fn determine_type(expr: &Expr) -> GetType {
        if let Expr::Call(call_expr) = expr {
            if let Callee::Expr(callee_expr) = &call_expr.callee {
                if let Expr::Ident(ident) = &**callee_expr {
                    match ident.sym.as_ref() {
                        "e" => return GetType::E,
                        "l" => return GetType::L,
                        _ => return GetType::Unknown,
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
                get: FxHashMap::default(),
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
            match &**init {
                Expr::Array(array_lit) if array_lit.elems.len() == 96 => {
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
                _ => {}
            }
        }
    }

    fn visit_call_expr(&mut self, call_expr: &CallExpr) {
        if let Callee::Expr(callee_expr) = &call_expr.callee {
            if let Expr::Ident(ident) = &**callee_expr {
                if ident.sym == "a" {
                    for arg in &call_expr.args {
                        match &*arg.expr {
                            Expr::Unary(unary_expr) => {
                                let arg_type = Self::determine_type(&unary_expr.arg);
                                let opcode = Opcode {
                                    left: arg_type,
                                    right: GetType::Unknown,
                                };
                                match unary_expr.op {
                                    UnaryOp::Tilde => {
                                        self.opcodes.not.insert(self.current_index, opcode);
                                    }
                                    UnaryOp::Delete => {
                                        self.opcodes.delete.insert(self.current_index, opcode);
                                    }
                                    _ => {}
                                }
                            }
                            Expr::Bin(bin_expr) => {
                                let left_type = Self::determine_type(&bin_expr.left);
                                let right_type = Self::determine_type(&bin_expr.right);
                                let opcode = Opcode {
                                    left: left_type,
                                    right: right_type,
                                };
                                match bin_expr.op {
                                    BinaryOp::RShift => {
                                        self.opcodes.r_shift.insert(self.current_index, opcode)
                                    }
                                    BinaryOp::LShift => {
                                        self.opcodes.l_shift.insert(self.current_index, opcode)
                                    }
                                    BinaryOp::BitXor => {
                                        self.opcodes.xor.insert(self.current_index, opcode)
                                    }
                                    BinaryOp::BitAnd => {
                                        self.opcodes.and.insert(self.current_index, opcode)
                                    }
                                    BinaryOp::BitOr => {
                                        self.opcodes.or.insert(self.current_index, opcode)
                                    }
                                    BinaryOp::Add => {
                                        self.opcodes.add.insert(self.current_index, opcode)
                                    }
                                    BinaryOp::Sub => {
                                        self.opcodes.subtract.insert(self.current_index, opcode)
                                    }
                                    BinaryOp::Mul => {
                                        self.opcodes.multiply.insert(self.current_index, opcode)
                                    }
                                    BinaryOp::Div => {
                                        self.opcodes.divide.insert(self.current_index, opcode)
                                    }
                                    BinaryOp::Mod => {
                                        self.opcodes.modulus.insert(self.current_index, opcode)
                                    }
                                    _ => None,
                                };
                            }
                            Expr::Member(member_expr) => {
                                if let MemberProp::Computed(computed) = &member_expr.prop {
                                    if !matches!(*computed.expr, Expr::Lit(_)) {
                                        let obj_type = Self::determine_type(&member_expr.obj);
                                        let prop_type = Self::determine_type(&computed.expr);

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
                                }
                            }
                            Expr::Call(call_expr) => {
                                if let Callee::Expr(callee_expr) = &call_expr.callee {
                                    if let Expr::Ident(ident) = &**callee_expr {
                                        if ident.sym.as_ref() == "e" {
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
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        call_expr.visit_children_with(self);
    }
}
