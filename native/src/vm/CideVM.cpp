#include "CideVM.hpp"
#include <cstring>
#include <sstream>
#include <iomanip>
#include <limits>

namespace cide {

// ============================================================================
// Helpers
// ============================================================================

static int32_t ReadI32LE(const uint8_t* p) {
    return static_cast<int32_t>(p[0])
         | (static_cast<int32_t>(p[1]) << 8)
         | (static_cast<int32_t>(p[2]) << 16)
         | (static_cast<int32_t>(p[3]) << 24);
}

static std::string FormatHex(uint32_t addr) {
    std::ostringstream oss;
    oss << std::hex << std::uppercase << std::setfill('0') << std::setw(4) << addr;
    return oss.str();
}

static void WriteI32LE(uint8_t* p, int32_t v) {
    p[0] = static_cast<uint8_t>(v);
    p[1] = static_cast<uint8_t>(v >> 8);
    p[2] = static_cast<uint8_t>(v >> 16);
    p[3] = static_cast<uint8_t>(v >> 24);
}

// ============================================================================
// CideVM
// ============================================================================

CideVM::CideVM() {
    memory_.resize(kMemSize, 0);
}

void CideVM::Reset() {
    code_.clear();
    ip_ = 0;
    stack_.clear();
    callStack_.clear();
    funcTable_.clear();
    funcNames_.clear();
    hostFuncs_.clear();
    userdata_ = nullptr;
    symbols_.clear();
    visEventLines_.clear();
    visEventQueue_.clear();
    breakpoints_.clear();
    paused_ = false;
    cancelled_ = false;
    stepEventHit_ = false;
    stepCount_ = 0;
    maxSteps_ = 10000000;
    currentLine_ = 0;
    error_.clear();
    globalCount_ = 0;
    lastSnapshotStep_ = 0;
    snapshotVars_.clear();
    std::fill(memory_.begin(), memory_.end(), 0);
    memStackTop_ = kStackStart;
}

void CideVM::LoadProgram(const std::vector<Instruction>& code) {
    code_ = code;
    ip_ = 0;
}

void CideVM::SetGlobals(const std::vector<int32_t>& globals) {
    globalCount_ = globals.size();
    for (size_t i = 0; i < globals.size(); i++) {
        uint32_t addr = kGlobalStart + static_cast<uint32_t>(i) * 4;
        if (addr + 4 <= kMemSize) {
            WriteI32LE(memory_.data() + addr, globals[i]);
        }
    }
}

void CideVM::RegisterFunction(uint32_t idx, const FuncMeta& meta) {
    if (idx >= funcTable_.size()) funcTable_.resize(idx + 1);
    funcTable_[idx] = meta;
}

void CideVM::RegisterFunctionName(uint32_t idx, const std::string& name) {
    if (idx >= funcNames_.size()) funcNames_.resize(idx + 1);
    funcNames_[idx] = name;
}

void CideVM::RegisterHostFunction(uint32_t id, HostFunction fn) {
    hostFuncs_[id] = std::move(fn);
}

void CideVM::SetUserData(void* userdata) {
    userdata_ = userdata;
}

void CideVM::SetSymbols(const std::vector<VMSymbol>& symbols) {
    symbols_ = symbols;
}

// ============================================================================
// Stack Helpers
// ============================================================================

int32_t CideVM::Pop() {
    if (stack_.empty()) {
        Trap("运行时错误：栈下溢");
        return 0;
    }
    int32_t v = stack_.back();
    stack_.pop_back();
    return v;
}

void CideVM::Push(int32_t val) {
    stack_.push_back(val);
}

// ============================================================================
// Memory Helpers (with bounds checking & NULL trap)
// ============================================================================

int32_t CideVM::LoadI32(uint32_t addr, const SourceLoc& loc) {
    if (addr < kNullTrapSize) {
        Trap("访问了 NULL 指针区域（地址 0x" + FormatHex(addr) +
             "）。NULL 指针不能解引用。请确认指针已被正确初始化。", loc);
        return 0;
    }
    if (addr + 4 > kMemSize) {
        Trap(FormatBoundsError(addr), loc);
        return 0;
    }
    return ReadI32LE(memory_.data() + addr);
}

void CideVM::StoreI32(uint32_t addr, int32_t val, const SourceLoc& loc) {
    if (addr < kNullTrapSize) {
        Trap("向 NULL 指针区域写入（地址 0x" + FormatHex(addr) +
             "）。请确认指针已被正确初始化。", loc);
        return;
    }
    if (addr + 4 > kMemSize) {
        Trap(FormatBoundsError(addr), loc);
        return;
    }
    WriteI32LE(memory_.data() + addr, val);
}

int32_t CideVM::LoadI8(uint32_t addr, const SourceLoc& loc) {
    if (addr < kNullTrapSize) {
        Trap("访问了 NULL 指针区域（地址 0x" + FormatHex(addr) + "）", loc);
        return 0;
    }
    if (addr >= kMemSize) {
        Trap(FormatBoundsError(addr), loc);
        return 0;
    }
    return static_cast<int8_t>(memory_[addr]);
}

void CideVM::StoreI8(uint32_t addr, int32_t val, const SourceLoc& loc) {
    if (addr < kNullTrapSize) {
        Trap("向 NULL 指针区域写入（地址 0x" + FormatHex(addr) + "）。请确认指针已被正确初始化。", loc);
        return;
    }
    if (addr >= kMemSize) {
        Trap(FormatBoundsError(addr), loc);
        return;
    }
    memory_[addr] = static_cast<uint8_t>(val);
}

// ============================================================================
// Error Formatting
// ============================================================================

std::string CideVM::FormatBoundsError(uint32_t addr) {
    // Try to find the closest array symbol for precise diagnostics
    const VMSymbol* bestSym = nullptr;
    uint32_t bestBase = 0;
    int bestDist = std::numeric_limits<int>::max();

    for (const auto& sym : symbols_) {
        if (!sym.type.isArray() || sym.type.arraySize <= 0) continue;

        uint32_t base = sym.addr;
        if (sym.isLocal) {
            if (callStack_.empty()) continue;
            base = callStack_.back().localsBase + sym.addr;
        }
        uint32_t size = static_cast<uint32_t>(sym.type.arraySize) * 4u;

        int dist = std::numeric_limits<int>::max();
        if (addr >= base && addr < base + size) {
            // Address is within the array but hit global memory limit
            dist = 0;
        } else if (addr >= base + size && addr < base + size + 64) {
            dist = static_cast<int>(addr - (base + size));
        } else if (addr + 64 >= base && addr < base) {
            dist = static_cast<int>(base - addr);
        } else {
            continue;
        }

        if (dist < bestDist) {
            bestDist = dist;
            bestSym = &sym;
            bestBase = base;
        }
    }

    std::string diag;
    if (bestSym) {
        int index = static_cast<int>((static_cast<int64_t>(addr) - static_cast<int64_t>(bestBase)) / 4);
        diag = "🚫 数组越界：你访问了 " + bestSym->name + "[" + std::to_string(index) + "]，";
        diag += "但数组 '" + bestSym->name + "' 只有 " + std::to_string(bestSym->type.arraySize) + " 个元素，";
        diag += "有效索引是 0~" + std::to_string(bestSym->type.arraySize - 1) + "。\n\n";
        diag += "📍 发生在第 " + std::to_string(currentLine_) + " 行\n";
        diag += "💡 原因：数组索引超出了合法范围。\n";
        diag += "✅ 检查方法：确认索引变量值在 0 到 " + std::to_string(bestSym->type.arraySize - 1) + " 之间。";
    } else {
        diag = "🚫 内存访问越界：你访问了地址 0x" + FormatHex(addr) +
               "，但合法内存范围是 0x" + FormatHex(kNullTrapSize) +
               " ~ 0x" + FormatHex(kMemSize) + "。\n\n";
        diag += "✅ 检查方法：\n";
        diag += "  • 确认数组索引小于数组大小\n";
        diag += "  • 确认指针已经指向有效的内存地址\n";
        diag += "  • 确认没有使用已经 free 的指针";
    }
    return diag;
}

std::string CideVM::FormatDivZeroError(int32_t a, int32_t b) {
    (void)b;
    std::string diag = "😵 除零错误：你试图用 " + std::to_string(a) + " 除以 0。\n\n";

    // Try to find which variable(s) currently hold zero
    std::vector<std::string> zeroVars;
    for (const auto& sym : symbols_) {
        if (sym.type.isArray()) continue;
        uint32_t vaddr = sym.addr;
        if (sym.isLocal) {
            if (callStack_.empty()) continue;
            vaddr = callStack_.back().localsBase + sym.addr;
        }
        if (vaddr + 4 <= kMemSize && vaddr >= kNullTrapSize) {
            int32_t val = ReadI32LE(memory_.data() + vaddr);
            if (val == 0) {
                zeroVars.push_back(sym.name);
            }
        }
    }

    if (!zeroVars.empty()) {
        diag += "🔍 当前作用域内值为 0 的变量：";
        for (size_t i = 0; i < zeroVars.size(); i++) {
            if (i > 0) diag += "、";
            diag += zeroVars[i];
        }
        diag += "。请检查除法表达式中是否使用了这些变量。\n\n";
    }

    diag += "💡 原因：除数不能为 0。\n";
    diag += "✅ 检查你的除法表达式，确保除数在被除之前不是 0。\n";
    diag += "📝 示例：如果变量 b 可能为 0，先用 if 判断：\n";
    diag += "    if (b != 0) {\n";
    diag += "        result = a / b;\n";
    diag += "    }";
    return diag;
}

void CideVM::Trap(const std::string& msg, const SourceLoc& loc) {
    if (error_.empty()) {
        error_ = msg;
        int line = loc.line > 0 ? loc.line : currentLine_;
        if (line > 0) {
            error_ += "\n📍 发生在第 " + std::to_string(line) + " 行";
            if (loc.column > 0) {
                error_ += " 第 " + std::to_string(loc.column) + " 列";
            }
        }
    }
}

int32_t CideVM::ReadVariable(const VMSymbol& sym) {
    uint32_t vaddr = sym.addr;
    if (sym.isLocal) {
        if (callStack_.empty()) return 0;
        vaddr = callStack_.back().localsBase + sym.addr;
    }
    if (vaddr + 4 > kMemSize || vaddr < kNullTrapSize) return 0;
    return ReadI32LE(memory_.data() + vaddr);
}

void CideVM::TakeVariableSnapshot() {
    // This will be called from Step() at intervals.
    // Implementation is inline in Step() to avoid storing large history.
}

std::vector<CideVM::VMVariableSnapshot> CideVM::GetVariableSnapshot() const {
    std::vector<VMVariableSnapshot> result;
    for (const auto& sym : symbols_) {
        uint32_t vaddr = sym.addr;
        if (sym.isLocal) {
            if (callStack_.empty()) continue;
            vaddr = callStack_.back().localsBase + sym.addr;
        }
        if (vaddr + 4 > kMemSize || vaddr < kNullTrapSize) continue;
        int32_t val = ReadI32LE(memory_.data() + vaddr);
        result.push_back({sym.name, vaddr, sym.isLocal, sym.type, val});
    }
    return result;
}

void CideVM::SetVisEventLines(const std::vector<std::pair<int, int>>& lines) {
    visEventLines_ = lines;
}

std::vector<CideVM::VisEvent> CideVM::TakeVisEvents() {
    auto result = std::move(visEventQueue_);
    visEventQueue_.clear();
    return result;
}

std::string CideVM::FormatInfiniteLoopError() {
    std::string diag = "🔄 程序执行步数超过限制（" + std::to_string(maxSteps_) +
                       " 步），可能包含无限循环。\n\n";

    // Analyze variables that haven't changed since last snapshot
    std::vector<std::string> staleVars;
    std::vector<std::string> changedVars;
    for (const auto& sym : symbols_) {
        if (sym.type.isArray()) continue;
        int32_t curVal = ReadVariable(sym);
        auto it = snapshotVars_.find(sym.name);
        if (it != snapshotVars_.end()) {
            if (it->second == curVal) {
                staleVars.push_back(sym.name + " = " + std::to_string(curVal));
            } else {
                changedVars.push_back(sym.name + " = " + std::to_string(curVal));
            }
        }
    }

    if (!staleVars.empty()) {
        diag += "🔍 在最近 " + std::to_string(kSnapshotInterval) + " 步内没有变化的变量：";
        for (size_t i = 0; i < staleVars.size() && i < 6; i++) {
            if (i > 0) diag += "，";
            diag += staleVars[i];
        }
        if (staleVars.size() > 6) diag += " 等";
        diag += "。\n\n";
    }

    if (!changedVars.empty()) {
        diag += "🔍 发生变化的变量：";
        for (size_t i = 0; i < changedVars.size() && i < 4; i++) {
            if (i > 0) diag += "，";
            diag += changedVars[i];
        }
        if (changedVars.size() > 4) diag += " 等";
        diag += "。\n\n";
    }

    diag += "💡 原因：程序执行了太多步数但没有结束。常见原因：\n";
    diag += "  • 循环条件永远为真（如 while(1)）\n";
    diag += "  • 循环变量没有更新（如忘了写 i++）\n";
    diag += "  • 递归函数没有正确的终止条件\n";
    diag += "✅ 检查方法：确认循环体中有改变循环条件的语句。";
    return diag;
}

// ============================================================================
// Step (execute one instruction)
// ============================================================================

CideVM::StepResult CideVM::Step() {
    if (!error_.empty()) return StepResult::Trap;
    if (ip_ >= code_.size()) return StepResult::Finished;

    // Step limit / cancellation
    stepCount_++;
    if (stepCount_ % kSnapshotInterval == 0) {
        // Take periodic snapshot for infinite-loop analysis
        snapshotVars_.clear();
        for (const auto& sym : symbols_) {
            if (sym.type.isArray()) continue;
            snapshotVars_[sym.name] = ReadVariable(sym);
        }
        lastSnapshotStep_ = stepCount_;
    }
    if (stepCount_ >= maxSteps_) {
        Trap(FormatInfiniteLoopError());
        return StepResult::Trap;
    }
    if (cancelled_) {
        Trap("执行已取消。");
        return StepResult::Trap;
    }

    const Instruction& inst = code_[ip_];
    ip_++;

    // Only update currentLine_ on StepEvent to avoid expression
    // instructions (PushConst, Add, etc.) overwriting the statement line.
    if (inst.op == OpCode::StepEvent) {
        currentLine_ = inst.operand;
        // Check breakpoint: if the current line has a breakpoint, pause execution.
        if (breakpoints_.count(currentLine_) > 0) {
            paused_ = true;
            stepEventHit_ = true;
        }
    }

    switch (inst.op) {
        // --- No-op ---
        case OpCode::Nop:
            break;

        // --- Constants & Variables ---
        case OpCode::PushConst:
            Push(inst.operand);
            break;

        case OpCode::LoadLocal: {
            if (callStack_.empty()) { Trap("LoadLocal: 无调用帧", inst.loc); break; }
            const auto& frame = callStack_.back();
            uint64_t addr64 = static_cast<uint64_t>(frame.localsBase) + static_cast<uint64_t>(inst.operand) * 4;
            if (addr64 + 4 > kMemSize || addr64 < kNullTrapSize) { Trap("LoadLocal: 地址越界", inst.loc); break; }
            Push(ReadI32LE(memory_.data() + static_cast<uint32_t>(addr64)));
            break;
        }

        case OpCode::StoreLocal: {
            if (callStack_.empty()) { Trap("StoreLocal: 无调用帧", inst.loc); break; }
            const auto& frame = callStack_.back();
            uint64_t addr64 = static_cast<uint64_t>(frame.localsBase) + static_cast<uint64_t>(inst.operand) * 4;
            if (addr64 + 4 > kMemSize || addr64 < kNullTrapSize) { Trap("StoreLocal: 地址越界", inst.loc); break; }
            WriteI32LE(memory_.data() + static_cast<uint32_t>(addr64), Pop());
            break;
        }

        case OpCode::GetFrameBase: {
            if (callStack_.empty()) { Trap("GetFrameBase: 无调用帧", inst.loc); break; }
            Push(static_cast<int32_t>(callStack_.back().localsBase));
            break;
        }

        case OpCode::LoadGlobal: {
            if (inst.operand < 0 || static_cast<size_t>(inst.operand) >= globalCount_) {
                Trap("LoadGlobal: 索引越界", inst.loc);
                break;
            }
            uint32_t addr = kGlobalStart + static_cast<uint32_t>(inst.operand) * 4;
            Push(ReadI32LE(memory_.data() + addr));
            break;
        }

        case OpCode::StoreGlobal: {
            if (inst.operand < 0 || static_cast<size_t>(inst.operand) >= globalCount_) {
                Trap("StoreGlobal: 索引越界", inst.loc);
                break;
            }
            uint32_t addr = kGlobalStart + static_cast<uint32_t>(inst.operand) * 4;
            WriteI32LE(memory_.data() + addr, Pop());
            break;
        }

        case OpCode::Pop:
            Pop();
            break;

        case OpCode::Dup: {
            if (stack_.empty()) { Trap("Dup: 栈空", inst.loc); break; }
            Push(stack_.back());
            break;
        }

        case OpCode::Swap: {
            if (stack_.size() < 2) { Trap("Swap: 栈不足", inst.loc); break; }
            std::swap(stack_[stack_.size() - 1], stack_[stack_.size() - 2]);
            break;
        }

        // --- Memory ---
        case OpCode::LoadMem: {
            uint32_t addr = static_cast<uint32_t>(Pop());
            Push(LoadI32(addr, inst.loc));
            break;
        }

        case OpCode::StoreMem: {
            int32_t val = Pop();
            uint32_t addr = static_cast<uint32_t>(Pop());
            StoreI32(addr, val, inst.loc);
            break;
        }

        case OpCode::LoadMemByte: {
            uint32_t addr = static_cast<uint32_t>(Pop());
            Push(LoadI8(addr, inst.loc));
            break;
        }

        case OpCode::StoreMemByte: {
            int32_t val = Pop();
            uint32_t addr = static_cast<uint32_t>(Pop());
            StoreI8(addr, val, inst.loc);
            break;
        }

        // --- Arithmetic ---
        case OpCode::Add: {
            int b = Pop(); int a = Pop();
            int64_t result64 = static_cast<int64_t>(a) + static_cast<int64_t>(b);
            if (result64 > INT32_MAX || result64 < INT32_MIN) {
                Trap("整数加法溢出。两个很大的正数（或很小的负数）相加超出了 int 能表示的范围。", inst.loc);
                break;
            }
            Push(static_cast<int32_t>(result64));
            break;
        }
        case OpCode::Sub: {
            int b = Pop(); int a = Pop();
            int64_t result64 = static_cast<int64_t>(a) - static_cast<int64_t>(b);
            if (result64 > INT32_MAX || result64 < INT32_MIN) {
                Trap("整数减法溢出。被减数太小而减数太大，结果超出了 int 能表示的范围。", inst.loc);
                break;
            }
            Push(static_cast<int32_t>(result64));
            break;
        }
        case OpCode::Mul: {
            int b = Pop(); int a = Pop();
            int64_t result64 = static_cast<int64_t>(a) * static_cast<int64_t>(b);
            if (result64 > INT32_MAX || result64 < INT32_MIN) {
                Trap("整数乘法溢出。乘积太大，超出了 int 能表示的范围。", inst.loc);
                break;
            }
            Push(static_cast<int32_t>(result64));
            break;
        }
        case OpCode::Div: {
            int b = Pop();
            int a = Pop();
            if (b == 0) { Trap(FormatDivZeroError(a, b), inst.loc); break; }
            if (a == INT32_MIN && b == -1) {
                Trap("整数除法溢出。INT_MIN / -1 的结果超出了 int 能表示的范围。", inst.loc);
                break;
            }
            Push(a / b);
            break;
        }
        case OpCode::Mod: {
            int b = Pop();
            int a = Pop();
            if (b == 0) { Trap(FormatDivZeroError(a, b), inst.loc); break; }
            Push(a % b);
            break;
        }
        case OpCode::Neg: {
            int a = Pop();
            if (a == INT32_MIN) {
                Trap("整数取反溢出。-INT_MIN 的结果超出了 int 能表示的范围。", inst.loc);
                break;
            }
            Push(-a);
            break;
        }

        // --- Comparison ---
        case OpCode::Eq:  { int b = Pop(); int a = Pop(); Push(a == b ? 1 : 0); break; }
        case OpCode::Ne:  { int b = Pop(); int a = Pop(); Push(a != b ? 1 : 0); break; }
        case OpCode::Lt:  { int b = Pop(); int a = Pop(); Push(a < b  ? 1 : 0); break; }
        case OpCode::Le:  { int b = Pop(); int a = Pop(); Push(a <= b ? 1 : 0); break; }
        case OpCode::Gt:  { int b = Pop(); int a = Pop(); Push(a > b  ? 1 : 0); break; }
        case OpCode::Ge:  { int b = Pop(); int a = Pop(); Push(a >= b ? 1 : 0); break; }

        // --- Logic ---
        case OpCode::And: { int b = Pop(); int a = Pop(); Push((a && b) ? 1 : 0); break; }
        case OpCode::Or:  { int b = Pop(); int a = Pop(); Push((a || b) ? 1 : 0); break; }
        case OpCode::Not: { Push(Pop() ? 0 : 1); break; }

        // --- Control Flow ---
        case OpCode::Jump:
            ip_ = static_cast<size_t>(inst.operand);
            break;

        case OpCode::JumpIfZero: {
            int val = Pop();
            if (val == 0) ip_ = static_cast<size_t>(inst.operand);
            break;
        }

        case OpCode::JumpIfNotZero: {
            int val = Pop();
            if (val != 0) ip_ = static_cast<size_t>(inst.operand);
            break;
        }

        case OpCode::Call: {
            uint32_t funcIdx = static_cast<uint32_t>(inst.operand);
            if (funcIdx >= funcTable_.size() || funcTable_[funcIdx].ip == 0) {
                Trap("Call: 未知函数索引 " + std::to_string(funcIdx), inst.loc);
                break;
            }
            const auto& meta = funcTable_[funcIdx];
            // Allocate frame in linear memory
            uint64_t frameSize64 = static_cast<uint64_t>(meta.localCount) * 4;
            if (frameSize64 > kMemSize || frameSize64 > memStackTop_) {
                Trap("Call: 栈溢出", inst.loc);
                break;
            }
            uint32_t frameSize = static_cast<uint32_t>(frameSize64);
            if (memStackTop_ < kNullTrapSize + frameSize) {
                Trap("Call: 栈溢出", inst.loc);
                break;
            }
            uint32_t heapLimit = getHeapLimit_ ? getHeapLimit_() : kHeapStart;
            if (memStackTop_ - frameSize < heapLimit) {
                Trap("Call: 栈溢出（栈与堆发生碰撞）。请减少递归深度或动态内存分配。", inst.loc);
                break;
            }
            memStackTop_ -= frameSize;
            uint32_t localsBase = memStackTop_;
            // Pop arguments from stack into memory frame (reverse order)
            for (int i = meta.argCount - 1; i >= 0; i--) {
                int32_t arg = Pop();
                uint64_t argAddr64 = static_cast<uint64_t>(localsBase) + static_cast<uint64_t>(meta.argCount - 1 - i) * 4;
                WriteI32LE(memory_.data() + static_cast<uint32_t>(argAddr64), arg);
            }
            // Zero-initialize remaining locals
            for (int i = meta.argCount; i < meta.localCount; i++) {
                uint64_t localAddr64 = static_cast<uint64_t>(localsBase) + static_cast<uint64_t>(i) * 4;
                WriteI32LE(memory_.data() + static_cast<uint32_t>(localAddr64), 0);
            }
            std::string funcName = (funcIdx < funcNames_.size()) ? funcNames_[funcIdx] : ("func_" + std::to_string(funcIdx));
            callStack_.push_back({ip_, localsBase, meta.localCount, funcName});
            ip_ = meta.ip;
            break;
        }

        case OpCode::CallHost: {
            auto it = hostFuncs_.find(inst.operand);
            if (it != hostFuncs_.end()) {
                it->second(stack_, this, userdata_);
            } else {
                Trap("CallHost: 未知宿主函数 " + std::to_string(inst.operand), inst.loc);
            }
            break;
        }

        case OpCode::Ret: {
            if (callStack_.empty()) {
                // Return from main
                return StepResult::Finished;
            }
            int32_t retVal = Pop();
            const auto& frame = callStack_.back();
            ip_ = frame.returnIP;
            // Free frame memory: restore stack top to before the frame
            memStackTop_ = frame.localsBase;
            Push(retVal);
            callStack_.pop_back();
            break;
        }

        case OpCode::RetVoid: {
            if (callStack_.empty()) {
                return StepResult::Finished;
            }
            const auto& frame = callStack_.back();
            ip_ = frame.returnIP;
            // Free frame memory: restore stack top to before the frame
            memStackTop_ = frame.localsBase;
            callStack_.pop_back();
            break;
        }

        // --- Debugging ---
        case OpCode::StepEvent: {
            stepEventHit_ = true;
            // Emit vis events for algorithm visualization
            for (const auto& ev : visEventLines_) {
                if (ev.first == inst.operand) {
                    visEventQueue_.push_back({ev.second, inst.operand, {0, 0, 0}});
                }
            }
            if (paused_) {
                return StepResult::Paused;
            }
            break;
        }

        // --- Runtime checks ---
        case OpCode::TrapBounds: {
            int symIdx = inst.operand;
            std::string name = "数组";
            int size = 0;
            int index = 0;
            if (symIdx >= 0 && static_cast<size_t>(symIdx) < symbols_.size()) {
                const auto& sym = symbols_[symIdx];
                name = sym.name;
                size = sym.type.arraySize;
            }
            if (!stack_.empty()) {
                index = Pop();
            }
            std::string diag = "🚫 数组越界：你访问了 " + name + "[" + std::to_string(index) + "]，";
            diag += "但数组 '" + name + "' 只有 " + std::to_string(size) + " 个元素，";
            diag += "有效索引是 0~" + std::to_string(size - 1) + "。\n\n";
            diag += "💡 原因：数组索引超出了合法范围。\n";
            diag += "✅ 检查方法：确认索引变量值在 0 到 " + std::to_string(size - 1) + " 之间。";
            Trap(diag, inst.loc);
            break;
        }
    }

    if (!error_.empty()) return StepResult::Trap;
    return StepResult::OK;
}

// ============================================================================
// Run (execute until completion or trap)
// ============================================================================

int32_t CideVM::Run() {
    while (true) {
        auto result = Step();
        if (result == StepResult::Finished) {
            if (!stack_.empty()) return stack_.back();
            return 0;
        }
        if (result == StepResult::Trap) {
            return 0;
        }
        if (result == StepResult::Paused) {
            // In Run() mode, we don't pause; just continue
            paused_ = false;
        }
    }
}

} // namespace cide
