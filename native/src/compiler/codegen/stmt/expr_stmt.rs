//! 表达式语句代码生成。

use crate::compiler::ast::{Expr, SourceLoc};
use crate::compiler::codegen::expr::ExprGen;
use crate::vm::opcode::OpCode;

use super::super::BytecodeGen;

impl BytecodeGen {
    pub(crate) fn gen_expr_stmt(&mut self, expr: &mut Expr, loc: &SourceLoc) {
        self.gen_expr(expr);
        if !expr.ty().is_void() && !expr.ty().is_struct() && !expr.ty().is_class() {
            self.emit(OpCode::Pop, 0, loc);
        }
    }
}
