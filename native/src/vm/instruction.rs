use super::opcode::OpCode;

#[derive(Debug, Clone, Copy, Default)]
pub struct SourceLoc {
    pub line: i32,
    pub column: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    pub op: OpCode,
    pub operand: i32,
    pub loc: SourceLoc,
}

impl Instruction {
    pub fn new(op: OpCode, operand: i32, loc: SourceLoc) -> Self {
        Self { op, operand, loc }
    }
}
