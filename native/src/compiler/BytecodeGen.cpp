#include "BytecodeGen.hpp"
#include <cassert>

namespace cide {

// ============================================================================
// Free helpers
// ============================================================================

// Helper: compute stride (bytes per index step) for multi-dimensional arrays.
// e.g. int[3][3]: first subscript stride = 3*4 = 12, second = 4.
static int ComputeStride(const Type& arrType) {
    if (!arrType.isArray() || arrType.dims.empty()) return 4;
    int stride = 4; // element size
    for (size_t i = 1; i < arrType.dims.size(); ++i) {
        stride *= (arrType.dims[i] > 0 ? arrType.dims[i] : 1);
    }
    return stride;
}

// Helper: flatten nested init list into a 1D sequence of int32_t values.
// Supports multi-dimensional init like { {1,2}, {3,4} } -> {1,2,3,4}.
static std::vector<int32_t> FlattenInitList(const InitListExpr& node) {
    std::vector<int32_t> result;
    for (const auto& elem : node.elements) {
        if (elem->kind == ExprKind::Literal) {
            result.push_back(static_cast<const LiteralExpr&>(*elem).value);
        } else if (elem->kind == ExprKind::InitList) {
            auto sub = FlattenInitList(static_cast<const InitListExpr&>(*elem));
            result.insert(result.end(), sub.begin(), sub.end());
        } else {
            // Non-literal in init list: push 0 as placeholder
            result.push_back(0);
        }
    }
    return result;
}

// ============================================================================
// BytecodeGen Helpers
// ============================================================================

void BytecodeGen::Emit(OpCode op, int32_t operand, const SourceLoc& loc) {
    uint32_t ip = static_cast<uint32_t>(code_.size());
    code_.emplace_back(op, operand, loc);
    if (loc.line > 0) {
        sourceMap_.push_back({ip, loc});
    }
}

void BytecodeGen::PatchJump(size_t ip, size_t target) {
    assert(ip < code_.size() && "PatchJump: IP out of bounds");
    if (ip < code_.size()) {
        code_[ip].operand = static_cast<int32_t>(target);
    }
}

void BytecodeGen::ReportError(const std::string& msg, const SourceLoc& loc) {
    errors_.push_back("第 " + std::to_string(loc.line) + " 行：" + msg);
}

// ============================================================================
// Scope / Symbol Management
// ============================================================================

void BytecodeGen::EnterFunction(const std::string& name, const std::vector<Param>& params) {
    currentFunc_ = name;
    currentFuncArgCount_ = static_cast<int>(params.size());
    localIndices_.clear();
    localTypes_.clear();
    nextLocalIdx_ = 0;

    // Parameters are the first locals
    for (size_t i = 0; i < params.size(); i++) {
        localIndices_[params[i].name] = static_cast<int>(i);
        localTypes_[params[i].name] = params[i].type;
        // Register parameter symbol
        symIndex_[params[i].name] = static_cast<int>(symbols_.size());
        symbols_.push_back({params[i].name, static_cast<uint32_t>(i) * 4u, true, params[i].type, 1});
    }
    nextLocalIdx_ = static_cast<int>(params.size());
    // Temp slots are allocated lazily on first use
    tempSlot0_ = -1;
    tempSlot1_ = -1;
    tempSlot2_ = -1;
}

void BytecodeGen::ExitFunction() {
    if (!currentFunc_.empty()) {
        funcTable_[currentFunc_].localCount = nextLocalIdx_;
    }
    currentFunc_.clear();
    localIndices_.clear();
    localTypes_.clear();
}

int BytecodeGen::ResolveLocal(const std::string& name) {
    auto it = localIndices_.find(name);
    if (it != localIndices_.end()) return it->second;
    return -1;
}

int BytecodeGen::ResolveGlobal(const std::string& name) {
    auto it = globalIndices_.find(name);
    if (it != globalIndices_.end()) return it->second;
    return -1;
}

int BytecodeGen::ResolveSymbolIndex(const std::string& name) {
    auto it = symIndex_.find(name);
    if (it != symIndex_.end()) return it->second;
    return -1;
}

int BytecodeGen::GetTempSlot(int index) {
    int* slot = nullptr;
    if (index == 0) slot = &tempSlot0_;
    else if (index == 1) slot = &tempSlot1_;
    else if (index == 2) slot = &tempSlot2_;
    else slot = &tempSlot0_; // fallback
    if (*slot < 0) *slot = nextLocalIdx_++;
    return *slot;
}

int BytecodeGen::GetMemberOffset(const Type& objectType, const std::string& memberName) {
    std::string structName;
    if (objectType.kind == TypeKind::Struct) {
        structName = objectType.name;
    } else if (objectType.kind == TypeKind::Pointer && objectType.baseKind == TypeKind::Struct) {
        structName = objectType.name;
    } else {
        return 0;
    }
    auto it = structDefs_.find(structName);
    if (it == structDefs_.end()) return 0;
    int offset = 0;
    for (const auto& field : it->second) {
        if (field.name == memberName) return offset;
        offset += 4; // all types are 4 bytes in this subset
    }
    return 0;
}

// ============================================================================
// Entry Point
// ============================================================================

bool BytecodeGen::Generate(ProgramNode& program) {
    code_.clear();
    // Reserve slot 0 for entry jump (patched after wrapper generation).
    // This eliminates the fragile +1 offset adjustment that was previously
    // done by manually shifting all jump targets and function IPs.
    code_.emplace_back(OpCode::Nop, 0, SourceLoc{});
    errors_.clear();
    globalsInit_.clear();
    globalIndices_.clear();
    globalTypes_.clear();
    funcTable_.clear();
    funcIndex_.clear();
    stringData_.clear();
    sourceMap_.clear();
    structDefs_.clear();
    nextFuncIdx_ = 0;
    nextGlobalIdx_ = 0;
    symbols_.clear();
    symIndex_.clear();

    // Register struct definitions
    for (auto& s : program.structs) {
        structDefs_[s.name] = s.fields;
    }

    // Pass 1: Register globals
    for (auto& g : program.globals) {
        globalIndices_[g.name] = nextGlobalIdx_;
        globalTypes_[g.name] = g.type;

        int elemCount = 1;
        if (g.type.isArray()) {
            elemCount = g.type.arraySize;
        } else if (g.type.isStruct()) {
            auto sit = structDefs_.find(g.type.name);
            elemCount = (sit != structDefs_.end()) ? static_cast<int>(sit->second.size()) : 1;
        }
        if (elemCount < 1) {
            // Infer array size from initializer
            if (g.init && g.init->kind == ExprKind::StringLiteral) {
                elemCount = static_cast<int>(static_cast<StringLiteralExpr&>(*g.init).value.size()) + 1;
            } else if (g.init && g.init->kind == ExprKind::InitList) {
                elemCount = static_cast<int>(static_cast<InitListExpr&>(*g.init).elements.size());
            } else {
                elemCount = 1;
            }
            // Update type with inferred size for later use
            globalTypes_[g.name].arraySize = elemCount;
        }

        if (g.init && g.init->kind == ExprKind::InitList) {
            auto values = FlattenInitList(static_cast<InitListExpr&>(*g.init));
            for (size_t i = 0; i < values.size() && i < static_cast<size_t>(elemCount); i++) {
                globalsInit_.push_back(values[i]);
            }
            for (size_t i = values.size(); i < static_cast<size_t>(elemCount); i++) {
                globalsInit_.push_back(0);
            }
        } else if (g.init && g.init->kind == ExprKind::StringLiteral) {
            auto& strLit = static_cast<StringLiteralExpr&>(*g.init);
            for (size_t i = 0; i < strLit.value.size() && i < static_cast<size_t>(elemCount); i++) {
                globalsInit_.push_back(static_cast<int32_t>(static_cast<uint8_t>(strLit.value[i])));
            }
            for (size_t i = strLit.value.size(); i < static_cast<size_t>(elemCount); i++) {
                globalsInit_.push_back(0);
            }
        } else if (g.init && g.init->kind == ExprKind::Literal) {
            globalsInit_.push_back(static_cast<LiteralExpr&>(*g.init).value);
            for (int i = 1; i < elemCount; i++) {
                globalsInit_.push_back(0);
            }
        } else {
            for (int i = 0; i < elemCount; i++) {
                globalsInit_.push_back(0);
            }
        }
        // Register global symbol for runtime diagnostics
        symIndex_[g.name] = static_cast<int>(symbols_.size());
        symbols_.push_back({g.name, 0x1000u + static_cast<uint32_t>(globalIndices_[g.name]) * 4u, false, globalTypes_[g.name], 0});
        nextGlobalIdx_ += elemCount;
    }

    // Set string memory area after all globals (including arrays) are counted
    stringMemOffset_ = 0x1000 + static_cast<uint32_t>(nextGlobalIdx_) * 4;

    // Pass 2: Register function metadata (forward declarations)
    for (auto& f : program.funcs) {
        funcIndex_[f.name] = nextFuncIdx_++;
        FuncMeta meta;
        meta.argCount = static_cast<int>(f.params.size());
        meta.returnType = f.returnType;
        funcTable_[f.name] = meta;
    }

    // Pass 3: Generate function bodies
    // Each function is emitted sequentially into the flat code array.
    for (auto& f : program.funcs) {
        funcTable_[f.name].ip = CurrentIP();
        EnterFunction(f.name, f.params);
        GenStmt(*f.body);

        // Ensure every function ends with a return
        if (f.returnType.isVoid()) {
            Emit(OpCode::RetVoid, 0, f.loc);
        } else {
            // Non-void function without explicit return: push 0 and return
            Emit(OpCode::PushConst, 0, f.loc);
            Emit(OpCode::Ret, 0, f.loc);
        }
        ExitFunction();
    }

    // Entry wrapper: call main, then exit
    auto mainIt = funcTable_.find("main");
    if (mainIt == funcTable_.end()) {
        ReportError("找不到 main 函数", {0, 0});
        return false;
    }

    // Entry wrapper: call main, then exit
    size_t wrapperIP = CurrentIP();
    Emit(OpCode::Call, funcIndex_["main"], {0, 0});
    Emit(OpCode::Ret, 0, {0, 0});

    // Patch entry jump
    code_[0] = Instruction(OpCode::Jump, static_cast<int32_t>(wrapperIP), SourceLoc{});

    return !HasErrors();
}

std::vector<Instruction> BytecodeGen::TakeCode() {
    return std::move(code_);
}

std::vector<int32_t> BytecodeGen::TakeGlobalsInit() {
    return std::move(globalsInit_);
}

// ============================================================================
// Dispatch Helpers
// ============================================================================

void BytecodeGen::GenStmt(Stmt& stmt) {
    if (stmt.loc.line > 0) {
        Emit(OpCode::StepEvent, stmt.loc.line, stmt.loc);
    }
    switch (stmt.kind) {
        case StmtKind::Block:      VisitBlock(static_cast<BlockStmt&>(stmt)); break;
        case StmtKind::VarDecl:    VisitVarDecl(static_cast<VarDeclStmt&>(stmt)); break;
        case StmtKind::Expr:       VisitExprStmt(static_cast<ExprStmt&>(stmt)); break;
        case StmtKind::If:         VisitIf(static_cast<IfStmt&>(stmt)); break;
        case StmtKind::While:      VisitWhile(static_cast<WhileStmt&>(stmt)); break;
        case StmtKind::DoWhile:    VisitDoWhile(static_cast<DoWhileStmt&>(stmt)); break;
        case StmtKind::For:        VisitFor(static_cast<ForStmt&>(stmt)); break;
        case StmtKind::Return:     VisitReturn(static_cast<ReturnStmt&>(stmt)); break;
        case StmtKind::Break:      VisitBreak(static_cast<BreakStmt&>(stmt)); break;
        case StmtKind::Continue:   VisitContinue(static_cast<ContinueStmt&>(stmt)); break;
        case StmtKind::Switch:     VisitSwitch(static_cast<SwitchStmt&>(stmt)); break;
        case StmtKind::Case:       VisitCase(static_cast<CaseStmt&>(stmt)); break;
    }
}

void BytecodeGen::GenExpr(Expr& expr) {
    switch (expr.kind) {
        case ExprKind::Binary:      VisitBinary(static_cast<BinaryExpr&>(expr)); break;
        case ExprKind::Unary:       VisitUnary(static_cast<UnaryExpr&>(expr)); break;
        case ExprKind::Literal:     VisitLiteral(static_cast<LiteralExpr&>(expr)); break;
        case ExprKind::StringLiteral: VisitStringLiteral(static_cast<StringLiteralExpr&>(expr)); break;
        case ExprKind::Identifier:  VisitIdentifier(static_cast<IdentifierExpr&>(expr)); break;
        case ExprKind::Call:        VisitCall(static_cast<CallExpr&>(expr)); break;
        case ExprKind::Index:       VisitIndex(static_cast<IndexExpr&>(expr)); break;
        case ExprKind::Member:      VisitMember(static_cast<MemberExpr&>(expr)); break;
        case ExprKind::Assign:      VisitAssign(static_cast<AssignExpr&>(expr)); break;
        case ExprKind::Sizeof:      VisitSizeof(static_cast<SizeofExpr&>(expr)); break;
        case ExprKind::InitList:    VisitInitList(static_cast<InitListExpr&>(expr)); break;
    }
}

// ============================================================================
// Statements
// ============================================================================

void BytecodeGen::VisitProgram(ProgramNode& /*node*/) {
    // Handled in Generate()
}

void BytecodeGen::VisitFuncDecl(FuncDecl& /*node*/) {
    // Handled in Generate()
}

void BytecodeGen::VisitBlock(BlockStmt& node) {
    for (auto& stmt : node.stmts) {
        GenStmt(*stmt);
    }
}

BytecodeGen::BytecodeGen() = default;

void BytecodeGen::VisitVarDecl(VarDeclStmt& node) {
    // Allocate local(s) — arrays and structs need multiple slots
    int elemCount = 1;
    if (node.varType.isArray()) {
        elemCount = node.varType.arraySize;
    } else if (node.varType.isStruct()) {
        auto sit = structDefs_.find(node.varType.name);
        elemCount = (sit != structDefs_.end()) ? static_cast<int>(sit->second.size()) : 1;
    }

    // Helper lambda to emit allocation + init for one variable
    auto emitOneVar = [&](const std::string& name, ExprPtr& init, const SourceLoc& loc) {
        int localIdx = nextLocalIdx_;
        nextLocalIdx_ += elemCount;
        localIndices_[name] = localIdx;
        localTypes_[name] = node.varType;
        symIndex_[name] = static_cast<int>(symbols_.size());
        symbols_.push_back({name, static_cast<uint32_t>(localIdx) * 4u, true, node.varType, 1});

        if (init) {
            if (node.varType.isArray() && init->kind == ExprKind::InitList) {
                auto values = FlattenInitList(static_cast<InitListExpr&>(*init));
                for (size_t i = 0; i < values.size() && i < static_cast<size_t>(elemCount); i++) {
                    Emit(OpCode::PushConst, values[i], loc);
                    Emit(OpCode::StoreLocal, localIdx + static_cast<int>(i), loc);
                }
                for (size_t i = values.size(); i < static_cast<size_t>(elemCount); i++) {
                    Emit(OpCode::PushConst, 0, loc);
                    Emit(OpCode::StoreLocal, localIdx + static_cast<int>(i), loc);
                }
            } else if (node.varType.isStruct() && init->kind == ExprKind::InitList) {
                auto& initList = static_cast<InitListExpr&>(*init);
                int baseTemp = GetTempSlot(0);
                Emit(OpCode::GetFrameBase, 0, loc);
                Emit(OpCode::PushConst, localIdx * 4, loc);
                Emit(OpCode::Add, 0, loc);
                Emit(OpCode::StoreLocal, baseTemp, loc);
                for (size_t i = 0; i < initList.elements.size() && i < static_cast<size_t>(elemCount); i++) {
                    Emit(OpCode::LoadLocal, baseTemp, loc);
                    Emit(OpCode::PushConst, static_cast<int32_t>(i) * 4, loc);
                    Emit(OpCode::Add, 0, loc);
                    GenExpr(*initList.elements[i]);
                    Emit(OpCode::StoreMem, 0, loc);
                }
            } else if (node.varType.isArray() && init->kind == ExprKind::StringLiteral) {
                auto& strLit = static_cast<StringLiteralExpr&>(*init);
                for (size_t i = 0; i < strLit.value.size() && i < static_cast<size_t>(elemCount); i++) {
                    Emit(OpCode::PushConst, static_cast<int32_t>(static_cast<uint8_t>(strLit.value[i])), loc);
                    Emit(OpCode::StoreLocal, localIdx + static_cast<int>(i), loc);
                }
                for (size_t i = strLit.value.size(); i < static_cast<size_t>(elemCount); i++) {
                    Emit(OpCode::PushConst, 0, loc);
                    Emit(OpCode::StoreLocal, localIdx + static_cast<int>(i), loc);
                }
            } else {
                GenExpr(*init);
                Emit(OpCode::StoreLocal, localIdx, loc);
            }
        } else {
            for (int i = 0; i < elemCount; i++) {
                Emit(OpCode::PushConst, 0, loc);
                Emit(OpCode::StoreLocal, localIdx + i, loc);
            }
        }
    };

    emitOneVar(node.name, node.init, node.loc);
    for (auto& [name, init] : node.extraVars) {
        emitOneVar(name, init, node.loc);
    }
}

void BytecodeGen::VisitExprStmt(ExprStmt& node) {
    if (node.expr) {
        GenExpr(*node.expr);
        // Host functions like printf/print_int already pop their arguments.
        // Void-returning calls leave nothing on the stack.
        if (!node.expr->type.isVoid()) {
            Emit(OpCode::Pop, 0, node.loc);
        }
    }
}

void BytecodeGen::VisitIf(IfStmt& node) {
    GenExpr(*node.cond);

    size_t elseJump = CurrentIP();
    Emit(OpCode::JumpIfZero, 0, node.loc); // placeholder

    GenStmt(*node.thenStmt);

    size_t endJump = CurrentIP();
    Emit(OpCode::Jump, 0, node.loc); // placeholder

    size_t elseIP = CurrentIP();
    PatchJump(elseJump, elseIP);

    if (node.elseStmt) {
        GenStmt(*node.elseStmt);
    }

    size_t endIP = CurrentIP();
    PatchJump(endJump, endIP);
}

void BytecodeGen::VisitWhile(WhileStmt& node) {
    size_t startIP = CurrentIP();
    GenExpr(*node.cond);

    size_t endJump = CurrentIP();
    Emit(OpCode::JumpIfZero, 0, node.loc);

    loopStartIPs_.push_back(startIP);

    size_t breakPatchBase = breakPatches_.size();
    size_t continuePatchBase = continuePatches_.size();

    GenStmt(*node.body);

    Emit(OpCode::Jump, static_cast<int32_t>(startIP), node.loc);

    size_t endIP = CurrentIP();
    PatchJump(endJump, endIP);

    // Patch breaks
    for (size_t i = breakPatchBase; i < breakPatches_.size(); i++) {
        PatchJump(breakPatches_[i], endIP);
    }
    breakPatches_.resize(breakPatchBase);

    // Patch continues
    for (size_t i = continuePatchBase; i < continuePatches_.size(); i++) {
        PatchJump(continuePatches_[i], startIP);
    }
    continuePatches_.resize(continuePatchBase);

    loopStartIPs_.pop_back();
}

void BytecodeGen::VisitDoWhile(DoWhileStmt& node) {
    size_t startIP = CurrentIP();

    loopStartIPs_.push_back(startIP);
    size_t breakPatchBase = breakPatches_.size();
    size_t continuePatchBase = continuePatches_.size();

    GenStmt(*node.body);

    GenExpr(*node.cond);
    Emit(OpCode::JumpIfNotZero, static_cast<int32_t>(startIP), node.loc);

    size_t endIP = CurrentIP();

    for (size_t i = breakPatchBase; i < breakPatches_.size(); i++) {
        PatchJump(breakPatches_[i], endIP);
    }
    breakPatches_.resize(breakPatchBase);

    for (size_t i = continuePatchBase; i < continuePatches_.size(); i++) {
        PatchJump(continuePatches_[i], startIP);
    }
    continuePatches_.resize(continuePatchBase);

    loopStartIPs_.pop_back();
}

void BytecodeGen::VisitFor(ForStmt& node) {
    if (node.init) {
        GenStmt(*node.init);
    }

    size_t startIP = CurrentIP();


    size_t condJump = 0;
    if (node.cond) {
        GenExpr(*node.cond);
        condJump = CurrentIP();
        Emit(OpCode::JumpIfZero, 0, node.loc);
    }

    loopStartIPs_.push_back(startIP);
    size_t breakPatchBase = breakPatches_.size();
    size_t continuePatchBase = continuePatches_.size();

    GenStmt(*node.body);

    size_t continueIP = CurrentIP();
    if (node.step) {
        GenExpr(*node.step);
        Emit(OpCode::Pop, 0, node.loc);
    }
    Emit(OpCode::Jump, static_cast<int32_t>(startIP), node.loc);

    size_t endIP = CurrentIP();
    if (node.cond) {
        PatchJump(condJump, endIP);
    }

    for (size_t i = breakPatchBase; i < breakPatches_.size(); i++) {
        PatchJump(breakPatches_[i], endIP);
    }
    breakPatches_.resize(breakPatchBase);

    for (size_t i = continuePatchBase; i < continuePatches_.size(); i++) {
        PatchJump(continuePatches_[i], continueIP);
    }
    continuePatches_.resize(continuePatchBase);

    loopStartIPs_.pop_back();
}

void BytecodeGen::VisitReturn(ReturnStmt& node) {
    if (node.value) {
        GenExpr(*node.value);
        Emit(OpCode::Ret, 0, node.loc);
    } else {
        Emit(OpCode::RetVoid, 0, node.loc);
    }
}

void BytecodeGen::VisitBreak(BreakStmt& node) {
    size_t ip = CurrentIP();
    Emit(OpCode::Jump, 0, node.loc); // placeholder
    breakPatches_.push_back(ip);
}

void BytecodeGen::VisitContinue(ContinueStmt& node) {
    size_t ip = CurrentIP();
    Emit(OpCode::Jump, 0, node.loc); // placeholder
    continuePatches_.push_back(ip);
}

void BytecodeGen::VisitSwitch(SwitchStmt& node) {
    if (!node.cond || !node.body) return;

    // Collect cases from body
    std::vector<CaseStmt*> cases;
    CaseStmt* defaultCase = nullptr;

    auto collectCases = [&](Stmt& stmt) {
        if (stmt.kind == StmtKind::Case) {
            auto* c = static_cast<CaseStmt*>(&stmt);
            if (c->label) {
                cases.push_back(c);
            } else {
                defaultCase = c;
            }
        }
    };

    if (node.body->kind == StmtKind::Block) {
        auto& block = static_cast<BlockStmt&>(*node.body);
        for (auto& stmt : block.stmts) {
            collectCases(*stmt);
        }
    } else if (node.body->kind == StmtKind::Case) {
        collectCases(*node.body);
    }

    if (cases.empty() && !defaultCase) {
        // Empty switch - just evaluate and discard condition
        GenExpr(*node.cond);
        Emit(OpCode::Pop, 0, node.loc);
        return;
    }

    // Evaluate condition and save to temp local
    GenExpr(*node.cond);
    int condTemp = GetTempSlot(0);
    Emit(OpCode::StoreLocal, condTemp, node.loc);

    std::vector<size_t> caseJumpIPs;

    for (auto* caseStmt : cases) {
        Emit(OpCode::LoadLocal, condTemp, node.loc);
        GenExpr(*caseStmt->label);
        Emit(OpCode::Eq, 0, node.loc);
        // StepEvent for case label comparison (single-step debugging pause point)
        if (caseStmt->label->loc.line > 0) {
            Emit(OpCode::StepEvent, caseStmt->label->loc.line, caseStmt->label->loc);
        }
        size_t jumpIP = CurrentIP();
        Emit(OpCode::JumpIfNotZero, 0, node.loc);
        caseJumpIPs.push_back(jumpIP);
    }

    // Jump to default or end
    size_t defaultOrEndJump = CurrentIP();
    Emit(OpCode::Jump, 0, node.loc);

    size_t breakPatchBase = breakPatches_.size();

    // Generate case bodies in source order (fall-through is natural)
    for (size_t i = 0; i < cases.size(); i++) {
        PatchJump(caseJumpIPs[i], CurrentIP());
        GenStmt(*cases[i]->stmt);
    }

    if (defaultCase) {
        PatchJump(defaultOrEndJump, CurrentIP());
        GenStmt(*defaultCase->stmt);
    } else {
        PatchJump(defaultOrEndJump, CurrentIP());
    }

    size_t endIP = CurrentIP();
    for (size_t i = breakPatchBase; i < breakPatches_.size(); i++) {
        PatchJump(breakPatches_[i], endIP);
    }
    breakPatches_.resize(breakPatchBase);
}

void BytecodeGen::VisitCase(CaseStmt& /*node*/) {
    // Handled in VisitSwitch
}

// ============================================================================
// Expressions
// ============================================================================

void BytecodeGen::VisitLiteral(LiteralExpr& node) {
    Emit(OpCode::PushConst, node.value, node.loc);
}

void BytecodeGen::VisitStringLiteral(StringLiteralExpr& node) {
    uint32_t addr = stringMemOffset_;
    uint32_t newOffset = addr + static_cast<uint32_t>(node.value.size()) + 1;
    newOffset = (newOffset + 3) & ~3; // 4-byte align
    if (newOffset > 0x5000) {
        ReportError("字符串字面量过多，超出内存限制", node.loc);
        Emit(OpCode::PushConst, static_cast<int32_t>(addr), node.loc);
        return;
    }
    stringData_.push_back({addr, node.value});
    stringMemOffset_ = newOffset;
    Emit(OpCode::PushConst, static_cast<int32_t>(addr), node.loc);
}

void BytecodeGen::VisitIdentifier(IdentifierExpr& node) {
    int localIdx = ResolveLocal(node.name);
    if (localIdx >= 0) {
        auto typeIt = localTypes_.find(node.name);
        if (typeIt != localTypes_.end() && typeIt->second.isArray()) {
            if (localIdx < currentFuncArgCount_) {
                // Array parameter: its value is already the base address
                Emit(OpCode::LoadLocal, localIdx, node.loc);
            } else {
                // Local array: push base address
                Emit(OpCode::GetFrameBase, 0, node.loc);
                Emit(OpCode::PushConst, localIdx * 4, node.loc);
                Emit(OpCode::Add, 0, node.loc);
            }
        } else {
            Emit(OpCode::LoadLocal, localIdx, node.loc);
        }
        return;
    }
    int globalIdx = ResolveGlobal(node.name);
    if (globalIdx >= 0) {
        auto typeIt = globalTypes_.find(node.name);
        if (typeIt != globalTypes_.end() && typeIt->second.isArray()) {
            // Global array: push base address
            Emit(OpCode::PushConst, 0x1000 + globalIdx * 4, node.loc);
        } else {
            Emit(OpCode::LoadGlobal, globalIdx, node.loc);
        }
        return;
    }
    ReportError("未声明的标识符 '" + node.name + "'", node.loc);
    Emit(OpCode::PushConst, 0, node.loc);
}

void BytecodeGen::VisitBinary(BinaryExpr& node) {
    GenExpr(*node.left);
    GenExpr(*node.right);

    bool leftIsPointer = node.left->type.isPointer() || node.left->type.isArray();
    bool rightIsPointer = node.right->type.isPointer() || node.right->type.isArray();

    switch (node.op) {
        case BinaryExpr::Op::Add:
            if (leftIsPointer && !rightIsPointer) {
                Emit(OpCode::PushConst, 4, node.loc);
                Emit(OpCode::Mul, 0, node.loc);
                Emit(OpCode::Add, 0, node.loc);
            } else if (!leftIsPointer && rightIsPointer) {
                Emit(OpCode::Swap, 0, node.loc);
                Emit(OpCode::PushConst, 4, node.loc);
                Emit(OpCode::Mul, 0, node.loc);
                Emit(OpCode::Swap, 0, node.loc);
                Emit(OpCode::Add, 0, node.loc);
            } else {
                Emit(OpCode::Add, 0, node.loc);
            }
            break;
        case BinaryExpr::Op::Sub:
            if (leftIsPointer && !rightIsPointer) {
                Emit(OpCode::PushConst, 4, node.loc);
                Emit(OpCode::Mul, 0, node.loc);
                Emit(OpCode::Sub, 0, node.loc);
            } else {
                Emit(OpCode::Sub, 0, node.loc);
            }
            break;
        case BinaryExpr::Op::Mul: Emit(OpCode::Mul, 0, node.loc); break;
        case BinaryExpr::Op::Div: Emit(OpCode::Div, 0, node.loc); break;
        case BinaryExpr::Op::Mod: Emit(OpCode::Mod, 0, node.loc); break;
        case BinaryExpr::Op::Eq:  Emit(OpCode::Eq, 0, node.loc); break;
        case BinaryExpr::Op::Ne:  Emit(OpCode::Ne, 0, node.loc); break;
        case BinaryExpr::Op::Lt:  Emit(OpCode::Lt, 0, node.loc); break;
        case BinaryExpr::Op::Le:  Emit(OpCode::Le, 0, node.loc); break;
        case BinaryExpr::Op::Gt:  Emit(OpCode::Gt, 0, node.loc); break;
        case BinaryExpr::Op::Ge:  Emit(OpCode::Ge, 0, node.loc); break;
        case BinaryExpr::Op::And: Emit(OpCode::And, 0, node.loc); break;
        case BinaryExpr::Op::Or:  Emit(OpCode::Or, 0, node.loc); break;
    }
}

void BytecodeGen::VisitUnary(UnaryExpr& node) {
    switch (node.op) {
        case UnaryExpr::Op::Neg:
            GenExpr(*node.operand);
            Emit(OpCode::Neg, 0, node.loc);
            break;
        case UnaryExpr::Op::Not:
            GenExpr(*node.operand);
            Emit(OpCode::Not, 0, node.loc);
            break;
        case UnaryExpr::Op::Addr: {
            if (node.operand->kind == ExprKind::Identifier) {
                auto& id = static_cast<IdentifierExpr&>(*node.operand);
                auto it = localIndices_.find(id.name);
                if (it != localIndices_.end()) {
                    Emit(OpCode::GetFrameBase, 0, node.loc);
                    Emit(OpCode::PushConst, it->second * 4, node.loc);
                    Emit(OpCode::Add, 0, node.loc);
                    break;
                }
                auto git = globalIndices_.find(id.name);
                if (git != globalIndices_.end()) {
                    Emit(OpCode::PushConst, 0x1000 + git->second * 4, node.loc);
                    break;
                }
            }
            ReportError("取地址暂不支持此表达式", node.loc);
            Emit(OpCode::PushConst, 0, node.loc);
            break;
        }
        case UnaryExpr::Op::Deref:
            GenExpr(*node.operand);
            Emit(OpCode::LoadMem, 0, node.loc);
            break;
        case UnaryExpr::Op::PreInc:
        case UnaryExpr::Op::PostInc:
        case UnaryExpr::Op::PreDec:
        case UnaryExpr::Op::PostDec: {
            bool isInc = (node.op == UnaryExpr::Op::PreInc || node.op == UnaryExpr::Op::PostInc);
            bool isPre = (node.op == UnaryExpr::Op::PreInc || node.op == UnaryExpr::Op::PreDec);

            if (node.operand->kind == ExprKind::Identifier) {
                auto& id = static_cast<IdentifierExpr&>(*node.operand);
                int localIdx = ResolveLocal(id.name);
                if (localIdx >= 0) {
                    Emit(OpCode::LoadLocal, localIdx, node.loc);
                    if (!isPre) {
                        Emit(OpCode::Dup, 0, node.loc); // keep old value for post
                    }
                    Emit(OpCode::PushConst, 1, node.loc);
                    Emit(isInc ? OpCode::Add : OpCode::Sub, 0, node.loc);
                    if (isPre) {
                        Emit(OpCode::Dup, 0, node.loc); // duplicate new value for pre
                    }
                    Emit(OpCode::StoreLocal, localIdx, node.loc);
                    break;
                }
                int globalIdx = ResolveGlobal(id.name);
                if (globalIdx >= 0) {
                    Emit(OpCode::LoadGlobal, globalIdx, node.loc);
                    if (!isPre) {
                        Emit(OpCode::Dup, 0, node.loc);
                    }
                    Emit(OpCode::PushConst, 1, node.loc);
                    Emit(isInc ? OpCode::Add : OpCode::Sub, 0, node.loc);
                    if (isPre) {
                        Emit(OpCode::Dup, 0, node.loc);
                    }
                    Emit(OpCode::StoreGlobal, globalIdx, node.loc);
                    break;
                }
            }
            ReportError("自增/自减暂只支持简单变量", node.loc);
            Emit(OpCode::PushConst, 0, node.loc);
            break;
        }
    }
}

void BytecodeGen::VisitCall(CallExpr& node) {
    // Push arguments (right to left)
    for (auto it = node.args.rbegin(); it != node.args.rend(); ++it) {
        GenExpr(**it);
    }

    auto idxIt = funcIndex_.find(node.name);
    if (idxIt != funcIndex_.end()) {
        Emit(OpCode::Call, idxIt->second, node.loc);
    } else {
        // Built-in host functions
        std::string funcName = node.name;
        if (funcName == "print_int") {
            funcName = "__cide_output";
        } else if (funcName == "printf") {
            funcName = "__cide_printf_n";
        } else if (funcName == "scanf") {
            funcName = "__cide_scanf_1";
        }

        if (funcName == "__cide_output") {
            Emit(OpCode::CallHost, 0, node.loc);
        } else if (funcName == "__cide_step") {
            Emit(OpCode::CallHost, 1, node.loc);
        } else if (funcName == "malloc") {
            Emit(OpCode::CallHost, 2, node.loc);
        } else if (funcName == "free") {
            Emit(OpCode::CallHost, 3, node.loc);
        } else if (funcName == "__cide_printf_0") {
            Emit(OpCode::CallHost, 10, node.loc);
        } else if (funcName == "__cide_printf_1") {
            Emit(OpCode::CallHost, 11, node.loc);
        } else if (funcName == "__cide_printf_n") {
            Emit(OpCode::CallHost, 15, node.loc);
        } else if (funcName == "__cide_scanf_1") {
            Emit(OpCode::CallHost, 20, node.loc);
        } else {
            ReportError("未定义的函数 '" + node.name + "'", node.loc);
            Emit(OpCode::PushConst, 0, node.loc);
        }
    }
}

void BytecodeGen::VisitIndex(IndexExpr& node) {
    // Determine array size for bounds checking
    int boundSize = -1;
    int symIdx = -1;
    if (node.array->kind == ExprKind::Identifier) {
        auto& id = static_cast<IdentifierExpr&>(*node.array);
        auto lit = localTypes_.find(id.name);
        if (lit != localTypes_.end() && lit->second.isArray()) {
            boundSize = lit->second.dims.empty() ? lit->second.arraySize : lit->second.dims[0];
            symIdx = ResolveSymbolIndex(id.name);
        } else {
            auto git = globalTypes_.find(id.name);
            if (git != globalTypes_.end() && git->second.isArray()) {
                boundSize = git->second.dims.empty() ? git->second.arraySize : git->second.dims[0];
                symIdx = ResolveSymbolIndex(id.name);
            }
        }
    } else if (node.array->kind == ExprKind::Index) {
        // Nested index: use the type computed by TypeChecker
        if (node.array->type.isArray() && !node.array->type.dims.empty()) {
            boundSize = node.array->type.dims[0];
        }
    }

    int stride = ComputeStride(node.array->type);

    GenExpr(*node.array);
    GenExpr(*node.index);

    if (boundSize > 0 && symIdx >= 0) {
        int idxTemp = GetTempSlot(0);
        Emit(OpCode::StoreLocal, idxTemp, node.loc);

        // Check index >= 0
        Emit(OpCode::LoadLocal, idxTemp, node.loc);
        Emit(OpCode::PushConst, 0, node.loc);
        Emit(OpCode::Ge, 0, node.loc);
        Emit(OpCode::Not, 0, node.loc);
        size_t jumpNeg = CurrentIP();
        Emit(OpCode::JumpIfZero, 0, node.loc);
        Emit(OpCode::LoadLocal, idxTemp, node.loc);
        Emit(OpCode::TrapBounds, symIdx, node.loc);
        PatchJump(jumpNeg, CurrentIP());

        // Check index < boundSize
        Emit(OpCode::LoadLocal, idxTemp, node.loc);
        Emit(OpCode::PushConst, boundSize, node.loc);
        Emit(OpCode::Lt, 0, node.loc);
        Emit(OpCode::Not, 0, node.loc);
        size_t jumpOk = CurrentIP();
        Emit(OpCode::JumpIfZero, 0, node.loc);
        Emit(OpCode::LoadLocal, idxTemp, node.loc);
        Emit(OpCode::TrapBounds, symIdx, node.loc);
        PatchJump(jumpOk, CurrentIP());

        Emit(OpCode::LoadLocal, idxTemp, node.loc);
    }

    Emit(OpCode::PushConst, stride, node.loc);
    Emit(OpCode::Mul, 0, node.loc);
    Emit(OpCode::Add, 0, node.loc);
    // For multi-dimensional arrays, intermediate index results are addresses (sub-arrays),
    // not values. Only load memory for the final element access.
    if (!node.type.isArray()) {
        Emit(OpCode::LoadMem, 0, node.loc);
    }
}

void BytecodeGen::VisitMember(MemberExpr& node) {
    // Compute base address
    if (node.object->type.isPointer()) {
        GenExpr(*node.object); // pointer value is the address
    } else if (node.object->kind == ExprKind::Identifier) {
        auto& id = static_cast<IdentifierExpr&>(*node.object);
        auto it = localIndices_.find(id.name);
        if (it != localIndices_.end()) {
            Emit(OpCode::GetFrameBase, 0, node.loc);
            Emit(OpCode::PushConst, it->second * 4, node.loc);
            Emit(OpCode::Add, 0, node.loc);
        } else {
            ReportError("全局结构体暂不支持", node.loc);
            Emit(OpCode::PushConst, 0, node.loc);
        }
    } else {
        ReportError("复杂结构体表达式暂不支持", node.loc);
        Emit(OpCode::PushConst, 0, node.loc);
    }
    // Add member offset
    int offset = GetMemberOffset(node.object->type, node.member);
    if (offset > 0) {
        Emit(OpCode::PushConst, offset, node.loc);
        Emit(OpCode::Add, 0, node.loc);
    }
    Emit(OpCode::LoadMem, 0, node.loc);
}

void BytecodeGen::VisitAssign(AssignExpr& node) {
    auto emitCompoundOp = [&](const SourceLoc& loc) {
        switch (node.op) {
            case AssignExpr::Op::AddAssign: Emit(OpCode::Add, 0, loc); break;
            case AssignExpr::Op::SubAssign: Emit(OpCode::Sub, 0, loc); break;
            case AssignExpr::Op::MulAssign: Emit(OpCode::Mul, 0, loc); break;
            case AssignExpr::Op::DivAssign: Emit(OpCode::Div, 0, loc); break;
            case AssignExpr::Op::ModAssign: Emit(OpCode::Mod, 0, loc); break;
            default: break;
        }
    };

    if (node.left->kind == ExprKind::Identifier) {
        auto& id = static_cast<IdentifierExpr&>(*node.left);
        int localIdx = ResolveLocal(id.name);
        if (localIdx >= 0) {
            GenExpr(*node.right);
            if (node.op != AssignExpr::Op::Assign) {
                Emit(OpCode::LoadLocal, localIdx, node.loc);
                emitCompoundOp(node.loc);
            }
            Emit(OpCode::StoreLocal, localIdx, node.loc);
            Emit(OpCode::LoadLocal, localIdx, node.loc);
            return;
        }
        int globalIdx = ResolveGlobal(id.name);
        if (globalIdx >= 0) {
            GenExpr(*node.right);
            if (node.op != AssignExpr::Op::Assign) {
                Emit(OpCode::LoadGlobal, globalIdx, node.loc);
                emitCompoundOp(node.loc);
            }
            Emit(OpCode::StoreGlobal, globalIdx, node.loc);
            Emit(OpCode::LoadGlobal, globalIdx, node.loc);
            return;
        }
    } else if (node.left->kind == ExprKind::Index) {
        if (node.op != AssignExpr::Op::Assign) {
            ReportError("复合赋值暂不支持数组索引", node.loc);
            GenExpr(*node.right);
            Emit(OpCode::Pop, 0, node.loc);
            Emit(OpCode::PushConst, 0, node.loc);
            return;
        }
        auto& idx = static_cast<IndexExpr&>(*node.left);

        // Determine array size for bounds checking
        int boundSize = -1;
        int symIdx = -1;
        if (idx.array->kind == ExprKind::Identifier) {
            auto& id = static_cast<IdentifierExpr&>(*idx.array);
            auto lit = localTypes_.find(id.name);
            if (lit != localTypes_.end() && lit->second.isArray()) {
                boundSize = lit->second.dims.empty() ? lit->second.arraySize : lit->second.dims[0];
                symIdx = ResolveSymbolIndex(id.name);
            } else {
                auto git = globalTypes_.find(id.name);
                if (git != globalTypes_.end() && git->second.isArray()) {
                    boundSize = git->second.dims.empty() ? git->second.arraySize : git->second.dims[0];
                    symIdx = ResolveSymbolIndex(id.name);
                }
            }
        } else if (idx.array->kind == ExprKind::Index) {
            if (idx.array->type.isArray() && !idx.array->type.dims.empty()) {
                boundSize = idx.array->type.dims[0];
            }
        }

        int stride = ComputeStride(idx.array->type);

        // Compute left-hand address first (it uses tempSlot0 internally)
        GenExpr(*idx.array);
        GenExpr(*idx.index);

        if (boundSize > 0 && symIdx >= 0) {
            int idxTemp = GetTempSlot(1);
            Emit(OpCode::StoreLocal, idxTemp, node.loc);

            // Check index >= 0
            Emit(OpCode::LoadLocal, idxTemp, node.loc);
            Emit(OpCode::PushConst, 0, node.loc);
            Emit(OpCode::Ge, 0, node.loc);
            Emit(OpCode::Not, 0, node.loc);
            size_t jumpNeg = CurrentIP();
            Emit(OpCode::JumpIfZero, 0, node.loc);
            Emit(OpCode::LoadLocal, idxTemp, node.loc);
            Emit(OpCode::TrapBounds, symIdx, node.loc);
            PatchJump(jumpNeg, CurrentIP());

            // Check index < boundSize
            Emit(OpCode::LoadLocal, idxTemp, node.loc);
            Emit(OpCode::PushConst, boundSize, node.loc);
            Emit(OpCode::Lt, 0, node.loc);
            Emit(OpCode::Not, 0, node.loc);
            size_t jumpOk = CurrentIP();
            Emit(OpCode::JumpIfZero, 0, node.loc);
            Emit(OpCode::LoadLocal, idxTemp, node.loc);
            Emit(OpCode::TrapBounds, symIdx, node.loc);
            PatchJump(jumpOk, CurrentIP());

            Emit(OpCode::LoadLocal, idxTemp, node.loc);
        }

        Emit(OpCode::PushConst, stride, node.loc);
        Emit(OpCode::Mul, 0, node.loc);
        Emit(OpCode::Add, 0, node.loc);
        int addrTemp = GetTempSlot(2);
        Emit(OpCode::StoreLocal, addrTemp, node.loc);

        // Compute right-hand value after address (so tempSlot0 isn't reused after saving)
        GenExpr(*node.right);
        int valTemp = GetTempSlot(0);
        Emit(OpCode::StoreLocal, valTemp, node.loc);

        Emit(OpCode::LoadLocal, addrTemp, node.loc);
        Emit(OpCode::LoadLocal, valTemp, node.loc);
        Emit(OpCode::StoreMem, 0, node.loc);
        Emit(OpCode::LoadLocal, addrTemp, node.loc);
        Emit(OpCode::LoadMem, 0, node.loc);
        return;
    } else if (node.left->kind == ExprKind::Unary) {
        auto& unary = static_cast<UnaryExpr&>(*node.left);
        if (unary.op == UnaryExpr::Op::Deref) {
            if (node.op != AssignExpr::Op::Assign) {
                ReportError("复合赋值暂不支持指针解引用", node.loc);
                GenExpr(*node.right);
                Emit(OpCode::Pop, 0, node.loc);
                Emit(OpCode::PushConst, 0, node.loc);
                return;
            }
            GenExpr(*node.right);
            int valTemp = GetTempSlot(0);
            Emit(OpCode::StoreLocal, valTemp, node.loc);
            GenExpr(*unary.operand);
            int addrTemp = GetTempSlot(1);
            Emit(OpCode::StoreLocal, addrTemp, node.loc);
            Emit(OpCode::LoadLocal, addrTemp, node.loc);
            Emit(OpCode::LoadLocal, valTemp, node.loc);
            Emit(OpCode::StoreMem, 0, node.loc);
            Emit(OpCode::LoadLocal, addrTemp, node.loc);
            Emit(OpCode::LoadMem, 0, node.loc);
            return;
        }
    } else if (node.left->kind == ExprKind::Member) {
        if (node.op != AssignExpr::Op::Assign) {
            ReportError("复合赋值暂不支持结构体成员", node.loc);
            GenExpr(*node.right);
            Emit(OpCode::Pop, 0, node.loc);
            Emit(OpCode::PushConst, 0, node.loc);
            return;
        }
        auto& member = static_cast<MemberExpr&>(*node.left);
        GenExpr(*node.right);
        int valTemp = GetTempSlot(0);
        Emit(OpCode::StoreLocal, valTemp, node.loc);
        if (member.object->type.isPointer()) {
            GenExpr(*member.object);
        } else if (member.object->kind == ExprKind::Identifier) {
            auto& id = static_cast<IdentifierExpr&>(*member.object);
            auto it = localIndices_.find(id.name);
            if (it != localIndices_.end()) {
                Emit(OpCode::GetFrameBase, 0, node.loc);
                Emit(OpCode::PushConst, it->second * 4, node.loc);
                Emit(OpCode::Add, 0, node.loc);
            } else {
                Emit(OpCode::PushConst, 0, node.loc);
            }
        } else {
            Emit(OpCode::PushConst, 0, node.loc);
        }
        int offset = GetMemberOffset(member.object->type, member.member);
        if (offset > 0) {
            Emit(OpCode::PushConst, offset, node.loc);
            Emit(OpCode::Add, 0, node.loc);
        }
        int addrTemp = GetTempSlot(1);
        Emit(OpCode::StoreLocal, addrTemp, node.loc);
        Emit(OpCode::LoadLocal, addrTemp, node.loc);
        Emit(OpCode::LoadLocal, valTemp, node.loc);
        Emit(OpCode::StoreMem, 0, node.loc);
        Emit(OpCode::LoadLocal, addrTemp, node.loc);
        Emit(OpCode::LoadMem, 0, node.loc);
        return;
    }

    ReportError("赋值目标不支持", node.loc);
    GenExpr(*node.right);
    Emit(OpCode::Pop, 0, node.loc);
    Emit(OpCode::PushConst, 0, node.loc);
}

void BytecodeGen::VisitSizeof(SizeofExpr& node) {
    // All types are 4 bytes in this subset
    Emit(OpCode::PushConst, 4, node.loc);
}

void BytecodeGen::VisitInitList(InitListExpr& node) {
    ReportError("初始化列表只能在变量声明中使用", node.loc);
    Emit(OpCode::PushConst, 0, node.loc);
}

std::vector<cide::VMSymbol> BytecodeGen::TakeSymbols() {
    return std::move(symbols_);
}

} // namespace cide
