#pragma once

#include "OpCode.hpp"
#include "../compiler/Ast.hpp"

namespace cide {

// A single bytecode instruction with source location for precise error mapping.
struct Instruction {
    OpCode op;
    int32_t operand = 0;
    SourceLoc loc;   // source line/col for runtime diagnostics and source map

    Instruction() = default;
    Instruction(OpCode o, int32_t p = 0, SourceLoc l = {})
        : op(o), operand(p), loc(l) {}
};

} // namespace cide
