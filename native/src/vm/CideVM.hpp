#pragma once

#include "Instruction.hpp"
#include "../compiler/Ast.hpp"

#include <cstdint>
#include <string>
#include <vector>
#include <unordered_map>
#include <functional>
#include <unordered_set>

namespace cide {

// ============================================================================
// Host Function Interface
// ============================================================================

class CideVM;
using HostFunction = std::function<void(std::vector<int32_t>&, CideVM*, void*)>;

// ============================================================================
// Symbol Table (for runtime diagnostics & visualization)
// ============================================================================

struct VMSymbol {
    std::string name;
    uint32_t addr;      // global addr or stack offset
    bool isLocal;
    Type type;
    int scopeDepth;     // 0 = global
};

// ============================================================================
// CideVM: Lightweight bytecode interpreter for the C subset.
// ============================================================================

class CideVM {
public:
    CideVM();

    // ------------------------------------------------------------------------
    // Program loading
    // ------------------------------------------------------------------------

    void Reset();
    struct FuncMeta {
        size_t ip = 0;
        int argCount = 0;
        int localCount = 0;
    };

    void LoadProgram(const std::vector<Instruction>& code);
    void SetGlobals(const std::vector<int32_t>& globals);
    void RegisterFunction(uint32_t idx, const FuncMeta& meta);
    void RegisterFunctionName(uint32_t idx, const std::string& name);
    void RegisterHostFunction(uint32_t id, HostFunction fn);
    void SetUserData(void* userdata);
    void SetSymbols(const std::vector<VMSymbol>& symbols);
    void SetHeapLimitCallback(std::function<uint32_t()> cb) { getHeapLimit_ = std::move(cb); }

    // ------------------------------------------------------------------------
    // Execution
    // ------------------------------------------------------------------------

    // Run until completion or trap. Returns main's return value.
    int32_t Run();

    // Execute a single instruction. Returns true if OK, false if paused/trapped.
    enum class StepResult { OK, Paused, Finished, Trap };
    StepResult Step();

    // ------------------------------------------------------------------------
    // Control
    // ------------------------------------------------------------------------

    void Pause() { paused_ = true; }
    void Resume() { paused_ = false; stepEventHit_ = false; }
    bool WasStepEventHit() const { return stepEventHit_; }
    void Cancel() { cancelled_ = true; }
    void SetMaxSteps(int maxSteps) { maxSteps_ = maxSteps; }

    // ------------------------------------------------------------------------
    // Breakpoints
    // ------------------------------------------------------------------------
    void AddBreakpoint(int line) { breakpoints_.insert(line); }
    void RemoveBreakpoint(int line) { breakpoints_.erase(line); }
    void ClearBreakpoints() { breakpoints_.clear(); }
    bool HasBreakpoint(int line) const { return breakpoints_.count(line) > 0; }

    // ------------------------------------------------------------------------
    // Diagnostics
    // ------------------------------------------------------------------------

    bool HasError() const { return !error_.empty(); }
    const std::string& GetError() const { return error_; }
    int GetCurrentLine() const { return currentLine_; }
    int GetExecutedSteps() const { return stepCount_; }

    // Memory access for views
    uint8_t* GetMemory() { return memory_.data(); }
    uint32_t GetMemorySize() const { return static_cast<uint32_t>(memory_.size()); }

    // Safe memory access (available for host functions)
    int32_t LoadI32(uint32_t addr, const SourceLoc& loc = {});
    void StoreI32(uint32_t addr, int32_t val, const SourceLoc& loc = {});
    int32_t LoadI8(uint32_t addr, const SourceLoc& loc = {});
    void StoreI8(uint32_t addr, int32_t val, const SourceLoc& loc = {});

    // Stack inspection
    const std::vector<int32_t>& GetStack() const { return stack_; }
    const std::vector<VMSymbol>& GetSymbols() const { return symbols_; }

    // Variable snapshot for debug view
    struct VMVariableSnapshot {
        std::string name;
        uint32_t addr;
        bool isLocal;
        Type type;
        int32_t value;
    };
    std::vector<VMVariableSnapshot> GetVariableSnapshot() const;

    // Call stack for debug view
    struct CallFrame {
        size_t returnIP;
        uint32_t localsBase;
        int localCount;
        std::string funcName;
    };
    const std::vector<CallFrame>& GetCallStack() const { return callStack_; }

    // Runtime visualization events
    struct VisEvent {
        enum Type : int {
            Compare    = 1,
            Swap       = 2,
            Update     = 3,
            NodeCreate = 4,
            EdgeConnect= 5,
            NodeAccess = 6,
            NodeDelete = 7,
        };
        int type;   // VisEvent::Type
        int line;
        int extra[3]; // extensible payload: addr/target/value/etc.
    };
    void SetVisEventLines(const std::vector<std::pair<int, int>>& lines);
    std::vector<VisEvent> TakeVisEvents();

private:
    // Program
    std::vector<Instruction> code_;
    size_t ip_ = 0;

    // Memory (linear memory model)
    std::vector<uint8_t> memory_;
    static constexpr uint32_t kMemSize = 256 * 1024; // 256KB
    static constexpr uint32_t kNullTrapSize = 0x1000;
    static constexpr uint32_t kGlobalStart = 0x1000;
    static constexpr uint32_t kHeapStart = 0x5000;
    static constexpr uint32_t kStackStart = 0x10000;

    // Registers / Stack
    std::vector<int32_t> stack_;
    uint32_t memStackTop_ = kStackStart;
    size_t globalCount_ = 0;

    std::vector<CallFrame> callStack_;

    // Functions
    std::vector<FuncMeta> funcTable_;
    std::vector<std::string> funcNames_;
    std::unordered_map<uint32_t, HostFunction> hostFuncs_;
    void* userdata_ = nullptr;

    // Symbols
    std::vector<VMSymbol> symbols_;

    // Runtime visualization events
    std::vector<std::pair<int, int>> visEventLines_; // (line, type)
    std::vector<VisEvent> visEventQueue_;

    // Control
    bool paused_ = false;
    bool cancelled_ = false;
    bool stepEventHit_ = false;
    int stepCount_ = 0;
    int maxSteps_ = 10000000;
    int currentLine_ = 0;
    std::string error_;

    // Breakpoints
    std::unordered_set<int> breakpoints_;

    // Infinite-loop detection snapshot
    static constexpr int kSnapshotInterval = 100000;
    int lastSnapshotStep_ = 0;
    std::unordered_map<std::string, int32_t> snapshotVars_;

    // Heap limit callback for stack-heap collision detection
    std::function<uint32_t()> getHeapLimit_;

    // Helpers
    int32_t Pop();
    void Push(int32_t val);

    void Trap(const std::string& msg, const SourceLoc& loc = {});
    uint32_t GetHeapLimit() const { return getHeapLimit_ ? getHeapLimit_() : kHeapStart; }
    std::string FormatBoundsError(uint32_t addr);
    std::string FormatDivZeroError(int32_t a, int32_t b);
    std::string FormatInfiniteLoopError();

    void TakeVariableSnapshot();
    int32_t ReadVariable(const VMSymbol& sym);
};

} // namespace cide
