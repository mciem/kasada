use swc_ecma_ast::*;
use swc_ecma_visit::{Visit, VisitWith};

#[derive(Debug)]
pub struct Opcodes {
    pub r_shift: Vec<u8>,
    pub l_shift: Vec<u8>,
}

pub struct OpcodeVisitor {
    pub opcodes: Opcodes,
    pub array: String,
    pub current_index: usize,
}

impl OpcodeVisitor {
    pub fn new() -> Self {
        Self {
            opcodes: Opcodes {
                r_shift: Vec::new(),
                l_shift: Vec::new(),
            },
            array: String::new(),
            current_index: 0,
        }
    }

    fn check_shift_operation(&self, left: &Expr, right: &Expr, op: BinaryOp) -> bool {
        match op {
            BinaryOp::RShift => {
                self.is_function_call(left, "l") && self.is_function_call(right, "e")
            }
            BinaryOp::LShift => {
                self.is_function_call(left, "e") && self.is_function_call(right, "l")
            }
            BinaryOp::BitAnd => {
                self.is_function_call(left, "l") && self.is_function_call(right, "e")
            }
            BinaryOp::BitOr => {
                self.is_function_call(left, "l") && self.is_function_call(right, "l")
            }
            _ => false,
        }
    }

    fn is_function_call(&self, expr: &Expr, func_name: &str) -> bool {
        if let Expr::Call(CallExpr { callee, .. }) = expr {
            if let Callee::Expr(callee_expr) = callee {
                if let Expr::Ident(ident) = &**callee_expr {
                    return ident.sym == func_name;
                }
            }
        }
        false
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
                        if let Expr::Bin(bin_expr) = &*arg.expr {
                            let shift_index = match bin_expr.op {
                                BinaryOp::RShift => &mut self.opcodes.r_shift,
                                BinaryOp::LShift => &mut self.opcodes.l_shift,
                                _ => continue,
                            };

                            shift_index.push(self.current_index as u8);

                            /*
                            if self.check_shift_operation(
                                &bin_expr.left,
                                &bin_expr.right,
                                bin_expr.op,
                            ) {
                                println!(
                                    "Found shift operation: {:?}, {:?}",
                                    bin_expr.op, self.current_index
                                );
                                let shift_index = match bin_expr.op {
                                    BinaryOp::RShift => &mut self.opcodes.r_shift,
                                    BinaryOp::LShift => &mut self.opcodes.l_shift,
                                    BinaryOp::BitAnd => &mut self.opcodes.and,
                                    BinaryOp::BitOr => &mut self.opcodes.or,
                                    _ => continue,
                                };

                                *shift_index = self.current_index as u8;
                            }
                            */
                        }
                    }
                }
            }
        }
        call_expr.visit_children_with(self);
    }
}
