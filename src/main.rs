mod codeexpr;

use std::rc::Rc;
use codeexpr::{CodeExpr, Type};

fn main() {
    let g = CodeExpr::IfElse {
        condition: Rc::new(CodeExpr::VMContext),
        then: Rc::new(CodeExpr::Symbol {
            name: "foo".to_string(),
            offset: 0,
            colocated: false,
        }),
        else_: Rc::new(CodeExpr::Load {
            base: Rc::new(CodeExpr::VMContext),
            offset: 32,
            readonly: false,
            result_type: Type::I32,
        }),
    };
    println!("hello {}", g);
}
