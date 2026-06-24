#![forbid(unsafe_code)]

pub mod algorithm_detector;
pub use cide_ast as ast;
pub mod cfg;
pub use cide_codegen as codegen;
pub use cide_cpp_frontend as cpp_frontend;
pub mod data_flow;
pub mod intent;
pub use cide_lexer as lexer;
pub use cide_parser as parser;
pub use cide_typeck as typeck;
