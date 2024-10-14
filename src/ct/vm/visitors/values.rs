use swc_ecma_ast::*;
use swc_ecma_visit::Visit;

#[derive(Debug)]
pub struct Values {
    pub instructions: String,
    pub get_list: Vec<u8>,
}

pub struct ValuesVisitor {
    pub values: Values,
}

impl ValuesVisitor {
    pub fn new() -> Self {
        Self {
            values: Values {
                instructions: String::new(),
                get_list: Vec::new(),
            },
        }
    }
}

impl Visit for ValuesVisitor {
    fn visit_var_declarator(&mut self, var_declarator: &VarDeclarator) {
        if let Some(init) = &var_declarator.init {
            if let Expr::Array(array_lit) = &**init {
                if array_lit.elems.len() == 6 {
                    for elem in &array_lit.elems {
                        if let Some(expr) = elem {
                            let expr = &expr.expr;
                            if let Expr::Lit(lit) = &**expr {
                                match lit {
                                    Lit::Num(num) => {
                                        self.values.get_list.push(num.value as u8);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            } else if let Expr::Lit(lit) = &**init {
                if let Lit::Str(str_lit) = lit {
                    let string = str_lit.value.to_string();

                    if string.len() > 10000 {
                        self.values.instructions = string;
                    }
                }
            }
        }
    }
}
