use swc_common::sync::Lrc;
use swc_common::{FileName, SourceMap};
use swc_ecma_parser::{EsSyntax, Parser, StringInput, Syntax};
use swc_ecma_visit::VisitWith;

use crate::ct::vm::visitors::opcodes::{OpcodeVisitor, Opcodes};

pub fn parse(js_code: &str) -> Opcodes {
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

    let mut visitor = OpcodeVisitor::new();
    module.visit_with(&mut visitor);

    visitor.opcodes
}
