#pragma once

#include <cstdint>

namespace cide {

// Flat bytecode instruction set for the C subset compiler.
// Only instructions actually used by the compiler are defined.
enum class OpCode : uint8_t {
    // --- Constants & Variables ---
    Nop,            // no operation (used for placeholder patching)
    PushConst,      // operand = int32 constant value
    LoadLocal,      // operand = local variable index
    StoreLocal,     // operand = local variable index
    LoadGlobal,     // operand = global variable index
    StoreGlobal,    // operand = global variable index
    GetFrameBase,   // push current frame base address (in linear memory)
    Pop,            // discard top of stack
    Dup,            // duplicate top of stack
    Swap,

    // --- Memory (linear memory) ---
    LoadMem,        // [addr] -> value (i32, 4 bytes)
    StoreMem,       // value, [addr] -> void
    LoadMemByte,    // [addr] -> value (i8, sign-extended)
    StoreMemByte,   // value, [addr] -> void (i8)

    // --- Arithmetic ---
    Add,            // a, b -> a+b
    Sub,            // a, b -> a-b
    Mul,            // a, b -> a*b
    Div,            // a, b -> a/b  (traps on div-by-zero)
    Mod,            // a, b -> a%b  (traps on div-by-zero)
    Neg,            // a -> -a

    // --- Comparison ---
    Eq,             // a, b -> a==b
    Ne,             // a, b -> a!=b
    Lt,             // a, b -> a<b  (signed)
    Le,             // a, b -> a<=b (signed)
    Gt,             // a, b -> a>b  (signed)
    Ge,             // a, b -> a>=b (signed)

    // --- Logic ---
    And,            // a, b -> a&&b (logical)
    Or,             // a, b -> a||b (logical)
    Not,            // a -> !a

    // --- Control Flow ---
    Jump,           // unconditional: ip = operand
    JumpIfZero,     // conditional: pop value, if 0 then ip = operand
    JumpIfNotZero,  // conditional: pop value, if !=0 then ip = operand
    Call,           // operand = function index
    CallHost,       // operand = host function id
    Ret,            // return from function (value on stack)
    RetVoid,        // return void

    // --- Debugging / Visualization (zero-intrusive hooks) ---
    StepEvent,      // operand = source line number

    // --- Runtime checks ---
    TrapBounds,     // operand = symbol index in symbols_ array
};

} // namespace cide
