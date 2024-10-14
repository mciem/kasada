use swc_common::sync::Lrc;
use swc_common::{FileName, SourceMap};
use swc_ecma_parser::{EsSyntax, Parser, StringInput, Syntax};
use swc_ecma_visit::VisitWith;

use lazy_static::lazy_static;
use regex::Regex;

use super::visitors::opcodes::{OpcodeVisitor, Opcodes};
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

pub fn parse(js_code: &str) -> (Opcodes, Values) {
    let cm: Lrc<SourceMap> = Default::default();

    let fm = cm.new_source_file(
        Lrc::new(FileName::Custom("input.js".into())),
        js_code.into(),
    );

    let lexer = swc_ecma_parser::lexer::Lexer::new(
        Syntax::Es(EsSyntax {
            jsx: true,
            ..Default::default()
        }),
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);
    let module = parser.parse_module().expect("Failed to parse module");

    let mut opcode_visitor = OpcodeVisitor::new();
    module.visit_with(&mut opcode_visitor);

    let mut values_visitor = ValuesVisitor::new();
    module.visit_with(&mut values_visitor);

    (opcode_visitor.opcodes, values_visitor.values)
}
