use oxc_ast::ast::{
    ArrayExpressionElement, BinaryOperator, Expression, LogicalOperator, VariableDeclarator,
};
use oxc_ast::visit::walk;
use oxc_ast::Visit;

#[derive(Debug, Default, Clone)]
pub struct GetValues {
    pub strings: u8,
}

#[derive(Debug, Default)]
pub struct Values {
    pub instructions: String,
    pub charset: String,
    pub get: GetValues,
}

#[derive(Default)]
pub struct ValuesVisitor {
    pub values: Values,

    current_index: usize,
}

impl<'a> Visit<'a> for ValuesVisitor {
    fn visit_variable_declarator(&mut self, var_decl: &VariableDeclarator<'_>) {
        if let Some(init) = &var_decl.init {
            match &init {
                Expression::ArrayExpression(array_expr) => {
                    if array_expr.elements.len() == 6 {
                        let mut values = Vec::new();
                        for elem in &array_expr.elements {
                            if let ArrayExpressionElement::NumericLiteral(lit) = elem {
                                values.push(lit.value as u8);
                            }
                        }

                        if self.values.get.strings < 10 {
                            self.values.get.strings = values[self.values.get.strings as usize];
                        }
                    }
                }

                _ => {}
            }
        }

        walk::walk_variable_declarator(self, var_decl);
    }

    fn visit_string_literal(&mut self, it: &oxc_ast::ast::StringLiteral<'a>) {
        let string = it.to_string();
        if string.len() > 1000 {
            self.values.instructions = string;
        } else if 60 < string.len() && string.len() < 70 {
            self.values.charset = string;
        }
    }

    fn visit_if_statement(&mut self, it: &oxc_ast::ast::IfStatement<'a>) {
        match &it.test {
            Expression::BinaryExpression(bin_expr) => {
                if bin_expr.operator == BinaryOperator::StrictEquality {
                    if let Expression::ComputedMemberExpression(computed) = &bin_expr.right {
                        if let Expression::NumericLiteral(lit) = &computed.expression {
                            self.current_index = lit.value as usize;
                        }
                    }
                }
            }
            Expression::LogicalExpression(log_expr) => {
                if log_expr.operator == LogicalOperator::And {
                    if let Expression::StaticMemberExpression(static_expr) = &log_expr.right {
                        if let Expression::Identifier(ident) = &static_expr.object {
                            if ident.name == "a" {
                                self.values.get.strings = self.current_index as u8;
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        walk::walk_if_statement(self, it);
    }
}
