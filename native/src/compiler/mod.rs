#![forbid(unsafe_code)]

pub mod algorithm_detector;
pub use cide_ast as ast;
pub mod cfg;
pub mod codegen;
pub mod cpp_frontend;
pub mod data_flow;
pub mod intent;
pub mod lexer;
pub mod parser;
pub mod typeck;
