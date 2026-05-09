#pragma once

#include "compiler/BytecodeGen.hpp"
#include "vm/CideVM.hpp"

#include <cstdint>
#include <string>
#include <vector>
#include <unordered_map>

// ============================================================================
// Internal Session Structures (shared between capi and vm layers)
// ============================================================================

struct CideMemoryRegion {
    uint32_t addr;
    int size;
    std::string name;
    std::string type;
    bool isHeap;
    bool isFreed;
};

struct FreeBlock {
    uint32_t addr;
    int size;
};

// Fix kind for structured auto-fixes
enum class CideFixKind : int {
    None = 0,           // No automatic fix available
    ReplaceText = 1,    // Replace text in a range
    InsertText = 2,     // Insert text at a position
    DeleteText = 3,     // Delete text in a range
    ManualHint = 4      // Manual fix required (human-readable hint only)
};

struct CideDiagnostic {
    int line;
    int column;
    int errorCode;
    int severity; // 0=error, 1=warning, 2=hint
    std::string message;
    std::string fixSuggestion;
    // Structured fix data (optional, populated for auto-fixable diagnostics)
    CideFixKind fixKind = CideFixKind::None;
    int replaceStartLine = 0;
    int replaceStartColumn = 0;
    int replaceEndLine = 0;
    int replaceEndColumn = 0;
    std::string replacementText = "";
};

struct CideTraceEntry {
    int line;
    std::string operation;
};

struct CideAlgorithmMatch {
    std::string name;
    std::string displayName;
    std::string funcName;
    int confidence;
    std::string suggestion;
    int line;
    std::vector<std::tuple<int, int, std::string>> visEvents;
};

struct CideCompileUnit {
    std::string filename;
    std::string source;
};

struct CideCompileState {
    std::string errors;
    std::string errorsBuffer; // persistent buffer for cide_get_compile_errors
    std::vector<CideCompileUnit> compileUnits;
    bool compiled = false;
    std::vector<cide::Instruction> bytecode;
    std::vector<int32_t> globalsInit;
    std::vector<CideDiagnostic> diagnostics;
    std::vector<std::pair<uint32_t, cide::SourceLoc>> sourceMap;
    std::unordered_map<std::string, cide::FuncMeta> funcTable;
    std::unordered_map<std::string, int> funcIndex;
    std::vector<std::pair<uint32_t, std::string>> stringData;
    std::vector<cide::VMSymbol> symbols;
    std::vector<CideAlgorithmMatch> algorithmMatches;
    // Struct field layouts: structName -> [(fieldName, offset), ...]
    std::unordered_map<std::string, std::vector<std::pair<std::string, int>>> structFields;
};

struct CideRuntimeState {
    std::string error;
    std::vector<std::string> outputLines;
    bool running = false;
    std::vector<CideTraceEntry> trace;
    int currentLine = 0;
    std::vector<std::string> inputLines;
    size_t inputIndex = 0;

    // Single-step control
    bool stepMode = false;
    int stepCount = 0;

    // Variable snapshot cache (for cide_variable_* APIs)
    std::vector<cide::CideVM::VMVariableSnapshot> variableSnapshot;

    // Vis event cache (for cide_vis_event_* APIs)
    std::vector<cide::CideVM::VisEvent> visEventCache;
};

struct CideMemoryState {
    std::vector<CideMemoryRegion> regions;
    std::vector<FreeBlock> freeList;
    uint32_t heapOffset = 0x5000;
    int allocCounter = 0;
};

struct CideSession {
    CideCompileState compile;
    CideRuntimeState runtime;
    CideMemoryState memory;
    cide::CideVM vm;
};

struct HostCtx {
    CideSession* session;
    cide::CideVM* vm;
};
