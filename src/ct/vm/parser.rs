use oxc_allocator::Allocator;
use oxc_ast::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;

use lazy_static::lazy_static;
use regex::Regex;

use super::visitors::opcodes::{OpcodeVisitor, OpcodesHandler};
use super::visitors::values::{Values, ValuesVisitor};

lazy_static! {
    static ref LIST_REGEX: Regex = Regex::new(r"\[\d+(?:,\d+){50,}\]").unwrap();
}

pub fn find_list(string: String) -> Vec<i64> {
    if let Some(matched_list) = LIST_REGEX.find(string.as_str()) {
        let list_str = matched_list.as_str();

        list_str[1..list_str.len() - 1]
            .split(',')
            .map(|num| num.trim().parse::<i64>().unwrap())
            .collect()
    } else {
        Vec::new()
    }
}

pub fn parse(js_code: &str) -> (OpcodesHandler, Values) {
    let allocator = Allocator::default();

    let ret = Parser::new(
        &allocator,
        js_code,
        SourceType::from_path("aaa.js").unwrap(),
    )
    .parse();

    let program = ret.program;

    let mut opcode = OpcodeVisitor::default();
    opcode.visit_program(&program);

    let mut value = ValuesVisitor::default();
    value.visit_program(&program);

    (opcode.opcodes, value.values)
}
