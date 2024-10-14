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
    pub xor: Vec<Opcode>,
    pub and: Vec<Opcode>,
    pub or: Vec<Opcode>,
    pub add: Vec<Opcode>,
    pub subtract: Vec<Opcode>,
    pub store_global: Vec<Opcode>,
    pub get_property: Vec<Opcode>,
}

pub struct OpcodeVisitor {
    pub opcodes: Opcodes,
    pub array: String,
    pub current_index: usize,
}

impl OpcodeVisitor {
    fn determine_type(expr: &Expr) -> GetType {
        match expr {
            Expr::Call(call_expr) => {
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
            _ => {}
        }
        GetType::Unknown
    }

    pub fn new() -> Self {
        Self {
            opcodes: Opcodes {
                r_shift: Vec::new(),
                l_shift: Vec::new(),
                xor: Vec::new(),
                and: Vec::new(),
                or: Vec::new(),
                add: Vec::new(),
                subtract: Vec::new(),
                store_global: Vec::new(),
                get_property: Vec::new(),
            },
            array: String::new(),
            current_index: 0,
        }
    }
}

impl Visit for OpcodeVisitor {
    fn visit_function(&mut self, node: &Function) {
        if node.params.len() != 4 {
            let mut u = GetType::Unknown;
            let mut r = GetType::Unknown;
            let mut l = GetType::Unknown;

            if let Some(body) = &node.body {
                if body.stmts.len() != 2 {
                    node.visit_children_with(self);
                }

                if let Stmt::Decl(var_decl) = &body.stmts[0] {
                    if let Decl::Var(var_decl) = &*var_decl {
                        if var_decl.decls.len() == 3 {
                            for i in 0..2 {
                                if let Some(init) = &var_decl.decls[i].init {
                                    if let Expr::Call(call_expr) = &**init {
                                        if let Callee::Expr(callee_expr) = &call_expr.callee {
                                            if let Expr::Ident(ident) = &**callee_expr {
                                                match ident.sym.as_ref() {
                                                    "u" => {
                                                        u = Self::determine_type(
                                                            &call_expr.args[0].expr,
                                                        )
                                                    }
                                                    "r" => {
                                                        r = Self::determine_type(
                                                            &call_expr.args[0].expr,
                                                        )
                                                    }
                                                    "l" => {
                                                        l = Self::determine_type(
                                                            &call_expr.args[0].expr,
                                                        )
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if u == GetType::E && r == GetType::E && l == GetType::Unknown {
                    if let Stmt::Expr(expr_stmt) = &body.stmts[1] {
                        if let Expr::Assign(assign_expr) = &*expr_stmt.expr {
                            if let AssignTarget::Simple(SimpleAssignTarget::Member(member_expr)) =
                                &assign_expr.left
                            {
                                if let MemberProp::Computed(computed) = &member_expr.prop {
                                    if let Expr::Ident(ident) = &*member_expr.obj {
                                        if ident.sym == *"u" {
                                            self.opcodes.store_global.push(Opcode {
                                                index: self.current_index,
                                                left: u,
                                                right: r,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        node.visit_children_with(self);
    }

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
                        if let Expr::Bin(bin_expr) = &*arg.expr {
                            let op_vec = match bin_expr.op {
                                BinaryOp::RShift => &mut self.opcodes.r_shift,
                                BinaryOp::LShift => &mut self.opcodes.l_shift,
                                BinaryOp::BitXor => &mut self.opcodes.xor,
                                BinaryOp::BitAnd => &mut self.opcodes.and,
                                BinaryOp::BitOr => &mut self.opcodes.or,
                                BinaryOp::Add => &mut self.opcodes.add,
                                BinaryOp::Sub => &mut self.opcodes.subtract,
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
}
