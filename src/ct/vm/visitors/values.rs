use oxc_ast::ast::{
    ArrayExpression, ArrayExpressionElement, Expression, StringLiteral, VariableDeclarator,
};
use oxc_ast::Visit;

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

    fn process_array(&mut self, array_expr: &ArrayExpression<'_>) {
        if array_expr.elements.len() == 6 {
            for elem in &array_expr.elements {
                if let ArrayExpressionElement::NumericLiteral(lit) = elem {
                    self.values.get_list.push(lit.value as u8);
                }
            }
        }
    }

    fn process_literal(&mut self, lit: &StringLiteral<'_>) {
        let string = lit.to_string();
        if string.len() > 10000 {
            self.values.instructions = string;
        }
    }
}

impl<'a> Visit<'a> for ValuesVisitor {
    fn visit_variable_declarator(&mut self, var_decl: &VariableDeclarator<'_>) {
        if let Some(init) = &var_decl.init {
            match &init {
                Expression::ArrayExpression(array_expr) => self.process_array(array_expr),
                Expression::StringLiteral(lit) => self.process_literal(lit),
                _ => {}
            }
        }
    }
}
