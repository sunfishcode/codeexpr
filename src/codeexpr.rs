//! Code Expressions.

use std::fmt;
use std::rc::Rc;

// TODO: Use real things instead of scaffolding.
//use crate::ir::immediates::{Imm64, Offset32};
//use crate::ir::{ExternalName, Type};
//use crate::isa::TargetIsa;
#[derive(Clone, Debug, Hash, Eq, PartialEq, Copy)]
pub enum Type {
    I32,
    I64,
}
pub type Offset32 = i32;
pub type Imm64 = i64;
pub type ExternalName = String;
pub trait TargetIsa {
    fn pointer_type(&self) -> Type;
}
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::I32 => write!(f, "i32"),
            Self::I64 => write!(f, "i64"),
        }
    }
}

/// A "code expression", which is an expression which can be expanded into code
/// or interpreted directly.
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum CodeExpr {
    /// Value is the address of the VM context struct.
    VMContext,

    /// The value of a symbol, which is a name which will be resolved to an
    /// actual value later (eg. by linking).
    ///
    /// For now, symbolic values always have pointer type, and represent
    /// addresses, however in the future they could be used to represent other
    /// things as well.
    Symbol {
        /// The symbolic name.
        name: ExternalName,

        /// Offset from the symbol. This can be used instead of IAddImm to represent folding an
        /// offset into a symbol.
        offset: Imm64,

        /// Will this symbol be defined nearby, such that it will always be a certain distance
        /// away, after linking? If so, references to it can avoid going through a GOT. Note that
        /// symbols meant to be preemptible cannot be colocated.
        colocated: bool,
    },

    /// Add an immediate constant to a value.
    IAddImm {
        /// The base value.
        base: Rc<CodeExpr>,

        /// Offset to be added to the value.
        offset: Imm64,

        /// The type of the iadd.
        result_type: Type,
    },

    /// Load a value from memory.
    ///
    /// The `base` expression is the address of a memory location to load from.
    /// The memory must be accessible, and naturally aligned to hold a value of
    /// the type.
    Load {
        /// The base pointer.
        base: Rc<CodeExpr>,

        /// Offset added to the base pointer before doing the load.
        offset: Offset32,

        /// Type of the loaded value.
        result_type: Type,

        /// Specifies whether the memory that this refers to is readonly, allowing for the
        /// elimination of redundant loads.
        readonly: bool,
    },

    /// A function call.
    Call {
        /// The expression to call. May be a `Symbol` for a direct call, or
        /// other kinds of expression for indirect calls.
        callee: Rc<CodeExpr>,

        /// Arguments to pass to the call.
        args: Vec<CodeExpr>,

        /// The result type of the call.
        result_type: Type,
    },

    /// An "if-then-else".
    IfElse {
        /// The boolean condition.
        condition: Rc<CodeExpr>,
        /// Expression to execute if `condition` is true.
        then: Rc<CodeExpr>,
        /// Expression to execute if `condition` is false.
        else_: Rc<CodeExpr>,
    },
}

impl CodeExpr {
    /// Assume that `self` is an `CodeExpr::Symbol` and return its name.
    pub fn symbol_name(&self) -> &ExternalName {
        match *self {
            Self::Symbol { ref name, .. } => name,
            _ => panic!("only symbols have names"),
        }
    }

    /// Return the type of this expression.
    pub fn result_type(&self, isa: &dyn TargetIsa) -> Type {
        match *self {
            Self::VMContext { .. } | Self::Symbol { .. } => isa.pointer_type(),
            Self::IAddImm { result_type, .. }
            | Self::Load { result_type, .. }
            | Self::Call { result_type, .. } => result_type,
            Self::IfElse {
                condition: _,
                ref then,
                ref else_,
            } => {
                let result = then.result_type(isa);
                assert_eq!(result, else_.result_type(isa));
                result
            }
        }
    }
}

impl fmt::Display for CodeExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::VMContext => write!(f, "vmctx"),
            Self::Symbol {
                ref name,
                offset,
                colocated,
            } => {
                write!(
                    f,
                    "symbol {}{}",
                    if colocated { "colocated " } else { "" },
                    name
                )?;
                let offset_val: i64 = offset.into();
                if offset_val > 0 {
                    write!(f, "+")?;
                }
                if offset_val != 0 {
                    write!(f, "{}", offset)?;
                }
                Ok(())
            }
            Self::IAddImm {
                result_type,
                ref base,
                offset,
            } => write!(f, "iadd_imm.{}({}, {})", result_type, base, offset),
            Self::Load {
                ref base,
                offset,
                result_type,
                readonly,
            } => write!(
                f,
                "load.{} notrap aligned {}({}+{})",
                result_type,
                if readonly { "readonly " } else { "" },
                base,
                offset
            ),
            Self::Call {
                ref callee,
                ref args,
                result_type,
            } => write!(
                f,
                "call.{}({}, {})",
                result_type,
                callee,
                args.iter()
                    .map(|arg| format!("{}", arg))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::IfElse {
                ref condition,
                ref then,
                ref else_,
            } => write!(f, "if {} {{ {} }} else {{ {} }}", condition, then, else_),
        }
    }
}
