pub mod ir;

pub use ir::StringGenerator;

#[allow(clippy::module_inception)]
pub mod codegen;
pub use codegen::CodeGenerator;
