#include "TypeChecker.hpp"
#include "diagnostics/ErrorCodes.hpp"

namespace cide {

// ============================================================================
// Helpers
// ============================================================================

void TypeChecker::ReportError(const std::string& msg, const SourceLoc& loc, ErrorCode code) {
    errors_.push_back({msg, loc.line, loc.column, static_cast<int>(code)});
    hasErrors_ = true;
}

void TypeChecker::EnterScope() {
    scopes_.emplace_back();
}

void TypeChecker::ExitScope() {
    if (!scopes_.empty()) scopes_.pop_back();
}

void TypeChecker::DeclareVar(const std::string& name, const Type& type, bool isGlobal) {
    if (scopes_.empty()) {
        scopes_.emplace_back();
    }
    auto& scope = scopes_.back();
    if (scope.find(name) != scope.end()) {
        ReportError("变量 '" + name + "' 已在此作用域中声明", {0, 0});
        return;
    }
    scope[name] = {type, isGlobal};
}

std::optional<VarSymbol> TypeChecker::LookupVar(const std::string& name) {
    // Search from innermost to outermost scope
    for (auto it = scopes_.rbegin(); it != scopes_.rend(); ++it) {
        auto found = it->find(name);
        if (found != it->end()) {
            return found->second;
        }
    }
    return std::nullopt;
}

bool TypeChecker::IsComparable(const Type& a, const Type& b) const {
    // Int vs Int
    if (a.kind == TypeKind::Int && b.kind == TypeKind::Int) return true;
    // Pointer vs Pointer (same or both generic void*)
    if (a.kind == TypeKind::Pointer && b.kind == TypeKind::Pointer) return true;
    // Pointer vs Array (array decays to pointer)
    if (a.kind == TypeKind::Pointer && b.kind == TypeKind::Array) return true;
    if (a.kind == TypeKind::Array && b.kind == TypeKind::Pointer) return true;
    // Pointer vs Int (NULL comparison: ptr == 0, ptr != 0)
    if (a.kind == TypeKind::Pointer && b.kind == TypeKind::Int) return true;
    if (a.kind == TypeKind::Int && b.kind == TypeKind::Pointer) return true;
    return false;
}

void TypeChecker::ReportWarning(const std::string& msg, const SourceLoc& loc, ErrorCode code) {
    warnings_.push_back({msg, loc.line, loc.column, static_cast<int>(code)});
}

bool TypeChecker::IsAssignable(const Type& target, const Type& value, const SourceLoc& loc) {
    if (target == value) return true;
    // Array can decay to pointer
    if (target.kind == TypeKind::Pointer && value.kind == TypeKind::Array
        && target.baseKind == value.baseKind && target.name == value.name) {
        ReportWarning("数组隐式转换为指针。数组名在表达式中会自动退化为指向首元素的指针。", loc);
        return true;
    }
    // Array to array (e.g. int[] parameter accepts int[5], int[3][3] -> int[][3])
    if (target.kind == TypeKind::Array && value.kind == TypeKind::Array
        && target.baseKind == value.baseKind && target.name == value.name) {
        // For multi-dimensional arrays, allow if specified dimensions match
        // and target's unspecified dims (-1) accept any size.
        bool dimsCompatible = true;
        size_t checkCount = std::min(target.dims.size(), value.dims.size());
        for (size_t i = 0; i < checkCount; ++i) {
            if (target.dims[i] > 0 && target.dims[i] != value.dims[i]) {
                dimsCompatible = false;
                break;
            }
        }
        if (dimsCompatible) return true;
    }
    // int/char to int/char (always, with warning for cross-conversion)
    if ((target.kind == TypeKind::Int || target.kind == TypeKind::Char) &&
        (value.kind == TypeKind::Int || value.kind == TypeKind::Char)) {
        if (target.kind != value.kind) {
            std::string from = (value.kind == TypeKind::Char) ? "char" : "int";
            std::string to = (target.kind == TypeKind::Char) ? "char" : "int";
            ReportWarning(from + " 被隐式转换为 " + to + "。不同类型的标量之间赋值可能会丢失精度。", loc);
        }
        return true;
    }
    // NULL pointer assignment: int 0 to any pointer
    if (target.kind == TypeKind::Pointer && value.kind == TypeKind::Int) {
        ReportWarning("整数被隐式转换为指针。建议确保这是有意义的地址值（如 NULL = 0）。", loc);
        return true;
    }
    // Generic pointer (from malloc) can assign to any named pointer
    if (target.kind == TypeKind::Pointer && value.kind == TypeKind::Pointer && value.name.empty()) {
        ReportWarning("void* 指针被隐式转换为具体类型的指针。请确保内存布局正确。", loc);
        return true;
    }
    return false;
}

std::optional<Type> TypeChecker::GetStructFieldType(const std::string& structName, const std::string& fieldName) {
    auto it = structs_.find(structName);
    if (it == structs_.end()) return std::nullopt;
    for (const auto& [ftype, fname] : it->second.fields) {
        if (fname == fieldName) return ftype;
    }
    return std::nullopt;
}

void TypeChecker::CheckStructInitializer(const Type& structType, Expr& init, const SourceLoc& loc) {
    if (init.kind != ExprKind::InitList) {
        auto initType = ResolveExprType(init);
        if (!IsAssignable(structType, initType, loc)) {
            ReportError("类型不匹配：无法将 '" + initType.ToString() +
                        "' 赋值给 '" + structType.ToString() + "'", loc);
        }
        return;
    }
    auto& initList = static_cast<InitListExpr&>(init);
    auto it = structs_.find(structType.name);
    if (it == structs_.end()) {
        ReportError("未知的结构体类型 '" + structType.name + "'", loc);
        return;
    }
    const auto& fields = it->second.fields;
    if (initList.elements.size() > fields.size()) {
        ReportError("初始化列表元素数量超过结构体字段数", loc);
    }
    for (size_t i = 0; i < initList.elements.size() && i < fields.size(); i++) {
        auto eType = ResolveExprType(*initList.elements[i]);
        if (!IsAssignable(fields[i].first, eType, loc)) {
            ReportError("结构体初始化类型不匹配：字段 '" + fields[i].second +
                        "' 期望 '" + fields[i].first.ToString() +
                        "'，实际 '" + eType.ToString() + "'", loc);
        }
    }
}

bool TypeChecker::ValidateNestedInitList(const std::vector<int>& dims, Expr& init,
                                           const SourceLoc& loc, TypeKind baseKind,
                                           const std::string& structName) {
    if (dims.empty()) {
        Type expected = baseKind == TypeKind::Struct ? Type{TypeKind::Struct, structName} :
                        baseKind == TypeKind::Char ? Type{TypeKind::Char} :
                        Type{TypeKind::Int};
        auto eType = ResolveExprType(init);
        if (!IsAssignable(expected, eType, loc)) {
            ReportError("数组初始化元素类型不匹配：期望 '" + expected.ToString() +
                        "'，实际 '" + eType.ToString() + "'", loc);
            return false;
        }
        return true;
    }

    if (init.kind != ExprKind::InitList) {
        ReportError("多维数组初始化需要嵌套初始化列表", loc);
        return false;
    }

    auto& initList = static_cast<InitListExpr&>(init);
    size_t expectedCount = dims[0] > 0 ? static_cast<size_t>(dims[0]) : initList.elements.size();
    if (initList.elements.size() > expectedCount) {
        ReportError("初始化列表元素数量超过数组维度大小", loc);
    }

    std::vector<int> subDims(dims.begin() + 1, dims.end());
    for (auto& elem : initList.elements) {
        if (!ValidateNestedInitList(subDims, *elem, loc, baseKind, structName)) {
            return false;
        }
    }
    return true;
}

void TypeChecker::CheckArrayInitializer(Type& arrType, Expr& init, const SourceLoc& loc) {
    Type elemType = arrType.baseKind == TypeKind::Struct ?
        Type{TypeKind::Struct, arrType.name} :
        Type{arrType.baseKind};

    // Multi-dimensional array init
    if (!arrType.dims.empty() && arrType.dims.size() > 1) {
        if (init.kind == ExprKind::InitList) {
            auto& initList = static_cast<InitListExpr&>(init);
            if (arrType.dims[0] <= 0) {
                arrType.dims[0] = static_cast<int>(initList.elements.size());
                arrType.arraySize = arrType.totalElements();
            }
            ValidateNestedInitList(arrType.dims, init, loc, arrType.baseKind, arrType.name);
        } else {
            auto initType = ResolveExprType(init);
            ReportError("多维数组初始化必须使用嵌套初始化列表，不能是 '" +
                        initType.ToString() + "'", loc);
        }
        return;
    }

    if (init.kind == ExprKind::InitList) {
        auto& initList = static_cast<InitListExpr&>(init);
        int expectedSize = arrType.arraySize;
        if (expectedSize <= 0) {
            expectedSize = static_cast<int>(initList.elements.size());
            arrType.arraySize = expectedSize;
        }
        if (initList.elements.size() > static_cast<size_t>(expectedSize)) {
            ReportError("初始化列表元素数量超过数组大小", loc);
        }
        for (auto& elem : initList.elements) {
            auto eType = ResolveExprType(*elem);
            if (!IsAssignable(elemType, eType, loc)) {
                ReportError("数组初始化元素类型不匹配：期望 '" + elemType.ToString() +
                            "'，实际 '" + eType.ToString() + "'", loc);
            }
        }
    } else if (init.kind == ExprKind::StringLiteral) {
        auto& str = static_cast<StringLiteralExpr&>(init);
        if (elemType.kind != TypeKind::Char) {
            ReportError("字符串字面量只能用于初始化 char 数组", loc);
            return;
        }
        int strLen = static_cast<int>(str.value.size());
        if (arrType.arraySize <= 0) {
            arrType.arraySize = strLen + 1;  // include '\0'
        } else if (strLen + 1 > arrType.arraySize) {
            ReportError("字符串字面量长度超过数组大小", loc);
        }
    } else {
        auto initType = ResolveExprType(init);
        ReportError("数组初始化必须使用初始化列表或字符串字面量，不能是 '" +
                    initType.ToString() + "'", loc);
    }
}

// ============================================================================
// Entry Point
// ============================================================================

bool TypeChecker::Check(ProgramNode& program) {
    // Pass 1: Register structs
    for (auto& s : program.structs) {
        if (structs_.find(s.name) != structs_.end()) {
            ReportError("结构体 '" + s.name + "' 重复定义", s.loc);
            continue;
        }
        StructSymbol sym;
        for (const auto& f : s.fields) {
            sym.fields.push_back({f.type, f.name});
        }
        structs_[s.name] = std::move(sym);
    }

    // Pass 2: Register function signatures
    for (auto& f : program.funcs) {
        if (funcs_.find(f.name) != funcs_.end()) {
            ReportError("函数 '" + f.name + "' 重复定义", f.loc);
            continue;
        }
        FuncSymbol sym;
        sym.returnType = f.returnType;
        for (const auto& p : f.params) {
            sym.paramTypes.push_back(p.type);
        }
        funcs_[f.name] = std::move(sym);
    }

    // Pass 2.5: Register global variables in the outermost scope
    EnterScope();
    for (auto& g : program.globals) {
        DeclareVar(g.name, g.type, true);
    }
    // Check global variable initializers
    for (auto& g : program.globals) {
        if (g.init) {
            if (g.type.isArray()) {
                CheckArrayInitializer(g.type, *g.init, g.loc);
            } else if (g.type.isStruct() && g.init->kind == ExprKind::InitList) {
                CheckStructInitializer(g.type, *g.init, g.loc);
            } else {
                auto initType = ResolveExprType(*g.init);
                if (!IsAssignable(g.type, initType, g.loc)) {
                    ReportError("类型不匹配：无法将 '" + initType.ToString() +
                                "' 赋值给 '" + g.type.ToString() + "'", g.loc);
                }
            }
        }
    }

    // Pass 3: Check function bodies
    for (auto& f : program.funcs) {
        VisitFuncDecl(f);
    }

    ExitScope();  // exit global scope
    return !hasErrors_;
}

void TypeChecker::VisitProgram(ProgramNode& /*node*/) {
    // Handled in Check()
}

void TypeChecker::VisitFuncDecl(FuncDecl& node) {
    currentFuncReturn_ = node.returnType;
    EnterScope();

    // Register parameters
    for (const auto& p : node.params) {
        DeclareVar(p.name, p.type);
    }

    // Check body
    if (node.body) {
        VisitBlock(static_cast<BlockStmt&>(*node.body));
    }

    ExitScope();
}

void TypeChecker::VisitBlock(BlockStmt& node) {
    EnterScope();
    for (auto& stmt : node.stmts) {
        DispatchStmt(*stmt);
    }
    ExitScope();
}

void TypeChecker::VisitVarDecl(VarDeclStmt& node) {
    // Check initializer type for main variable
    if (node.init) {
        if (node.varType.isArray()) {
            CheckArrayInitializer(node.varType, *node.init, node.loc);
        } else if (node.varType.isStruct() && node.init->kind == ExprKind::InitList) {
            CheckStructInitializer(node.varType, *node.init, node.loc);
        } else {
            auto initType = ResolveExprType(*node.init);
            if (!IsAssignable(node.varType, initType, node.loc)) {
                ReportError("类型不匹配：无法将 '" + initType.ToString() +
                            "' 赋值给 '" + node.varType.ToString() + "'", node.loc);
            }
        }
    }
    DeclareVar(node.name, node.varType);

    // Check extra variables in multi-var declaration
    for (auto& [name, init] : node.extraVars) {
        if (init) {
            if (node.varType.isArray()) {
                CheckArrayInitializer(node.varType, *init, node.loc);
            } else if (node.varType.isStruct() && init->kind == ExprKind::InitList) {
                CheckStructInitializer(node.varType, *init, node.loc);
            } else {
                auto initType = ResolveExprType(*init);
                if (!IsAssignable(node.varType, initType, node.loc)) {
                    ReportError("类型不匹配：无法将 '" + initType.ToString() +
                                "' 赋值给 '" + node.varType.ToString() + "'", node.loc);
                }
            }
        }
        DeclareVar(name, node.varType);
    }
}

void TypeChecker::VisitExprStmt(ExprStmt& node) {
    if (node.expr) {
        ResolveExprType(*node.expr);
    }
}

void TypeChecker::VisitIf(IfStmt& node) {
    if (node.cond) {
        CheckCondition(*node.cond, "if 条件");
    }
    if (node.thenStmt) {
        DispatchStmt(*node.thenStmt);
    }
    if (node.elseStmt) {
        DispatchStmt(*node.elseStmt);
    }
}

void TypeChecker::VisitWhile(WhileStmt& node) {
    if (node.cond) {
        CheckCondition(*node.cond, "while 条件");
    }
    loopDepth_++;
    if (node.body) {
        DispatchStmt(*node.body);
    }
    loopDepth_--;
}

void TypeChecker::VisitDoWhile(DoWhileStmt& node) {
    loopDepth_++;
    if (node.body) {
        DispatchStmt(*node.body);
    }
    loopDepth_--;
    if (node.cond) {
        CheckCondition(*node.cond, "do...while 条件");
    }
}

void TypeChecker::VisitBreak(BreakStmt& node) {
    if (loopDepth_ <= 0 && switchDepth_ <= 0) {
        ReportError("break 只能在循环或 switch 体内使用", node.loc, ErrorCode::E3010_BreakOutsideLoop);
    }
}

void TypeChecker::VisitContinue(ContinueStmt& node) {
    if (loopDepth_ <= 0) {
        ReportError("continue 只能在循环体内使用", node.loc, ErrorCode::E3011_ContinueOutsideLoop);
    }
}

void TypeChecker::VisitFor(ForStmt& node) {
    EnterScope();

    if (node.init) {
        switch (node.init->kind) {
            case StmtKind::VarDecl: VisitVarDecl(static_cast<VarDeclStmt&>(*node.init)); break;
            case StmtKind::Expr: VisitExprStmt(static_cast<ExprStmt&>(*node.init)); break;
            default: break;
        }
    }

    if (node.cond) {
        CheckCondition(*node.cond, "for 条件");
        // Warn about <= in loop condition (common off-by-one error for array indexing)
        if (node.cond->kind == ExprKind::Binary) {
            auto* bin = static_cast<BinaryExpr*>(node.cond.get());
            if (bin->op == BinaryExpr::Op::Le) {
                ReportWarning("循环条件中使用了 '<='，如果用于数组索引，可能导致越界（off-by-one 错误）。你是否想使用 '<'？",
                              node.cond->loc, ErrorCode::W3051_ArrayBoundOffByOne);
            }
        }
    }

    if (node.step) {
        ResolveExprType(*node.step);
    }

    loopDepth_++;
    if (node.body) {
        DispatchStmt(*node.body);
    }
    loopDepth_--;

    ExitScope();
}

void TypeChecker::VisitReturn(ReturnStmt& node) {
    if (currentFuncReturn_.isVoid()) {
        if (node.value) {
                ReportError("void 函数不能有返回值", node.loc, ErrorCode::E3012_VoidFuncReturnValue);
        }
        return;
    }

    if (!node.value) {
        ReportError("非 void 函数必须返回一个值", node.loc, ErrorCode::E3013_MissingReturnValue);
        return;
    }

    auto valType = ResolveExprType(*node.value);
    if (!IsAssignable(currentFuncReturn_, valType, node.loc)) {
        ReportError("返回类型不匹配：期望 '" + currentFuncReturn_.ToString() +
                    "'，实际 '" + valType.ToString() + "'", node.loc, ErrorCode::E3014_ReturnTypeMismatch);
    }
}

void TypeChecker::DispatchStmt(Stmt& stmt) {
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

void TypeChecker::CheckCondition(Expr& cond, const std::string& ctx) {
    auto type = ResolveExprType(cond);
    // Any int or pointer can be used as condition
    if (type.kind != TypeKind::Int && type.kind != TypeKind::Pointer && type.kind != TypeKind::Array) {
        ReportError(ctx + " 必须是整数或指针类型", cond.loc);
    }

    // Warn if condition contains assignment (likely meant to be comparison)
    auto isAssignExpr = [](const Expr* expr) -> bool {
        return expr && expr->kind == ExprKind::Assign &&
               static_cast<const AssignExpr*>(expr)->op == AssignExpr::Op::Assign;
    };
    if (isAssignExpr(&cond)) {
        ReportWarning("条件中使用了赋值运算符 '='，你是否想使用比较运算符 '=='？", cond.loc);
    } else if (cond.kind == ExprKind::Binary) {
        const auto* bin = static_cast<const BinaryExpr*>(&cond);
        if (isAssignExpr(bin->left.get()) || isAssignExpr(bin->right.get())) {
            ReportWarning("条件中包含了赋值表达式，你是否想使用比较运算符 '=='？", cond.loc);
        }
    }
}

// ============================================================================
// Expression Type Resolution
// ============================================================================

Type TypeChecker::ResolveExprType(Expr& expr) {
    switch (expr.kind) {
        case ExprKind::Binary: {
            auto& bin = static_cast<BinaryExpr&>(expr);
            VisitBinary(bin);
            return bin.type;
        }
        case ExprKind::Unary: {
            auto& un = static_cast<UnaryExpr&>(expr);
            VisitUnary(un);
            return un.type;
        }
        case ExprKind::Literal: {
            auto& lit = static_cast<LiteralExpr&>(expr);
            VisitLiteral(lit);
            return lit.type;
        }
        case ExprKind::StringLiteral: {
            auto& lit = static_cast<StringLiteralExpr&>(expr);
            VisitStringLiteral(lit);
            return lit.type;
        }
        case ExprKind::Identifier: {
            auto& id = static_cast<IdentifierExpr&>(expr);
            VisitIdentifier(id);
            return id.type;
        }
        case ExprKind::Call: {
            auto& call = static_cast<CallExpr&>(expr);
            VisitCall(call);
            return call.type;
        }
        case ExprKind::Index: {
            auto& idx = static_cast<IndexExpr&>(expr);
            VisitIndex(idx);
            return idx.type;
        }
        case ExprKind::Member: {
            auto& mem = static_cast<MemberExpr&>(expr);
            VisitMember(mem);
            return mem.type;
        }
        case ExprKind::Assign: {
            auto& asgn = static_cast<AssignExpr&>(expr);
            VisitAssign(asgn);
            return asgn.type;
        }
        case ExprKind::Sizeof: {
            auto& sz = static_cast<SizeofExpr&>(expr);
            VisitSizeof(sz);
            return sz.type;
        }
        case ExprKind::InitList: {
            auto& init = static_cast<InitListExpr&>(expr);
            VisitInitList(init);
            return init.type;
        }
    }
    return Type{TypeKind::Void};
}

void TypeChecker::VisitBinary(BinaryExpr& node) {
    auto leftType = ResolveExprType(*node.left);
    auto rightType = ResolveExprType(*node.right);

    switch (node.op) {
        case BinaryExpr::Op::Add:
        case BinaryExpr::Op::Sub:
            if (IsInt(leftType) && IsInt(rightType)) {
                node.type = Type{TypeKind::Int};
            } else if (leftType.isPointer() && IsInt(rightType)) {
                node.type = leftType;
            } else if (IsInt(leftType) && rightType.isPointer() && node.op == BinaryExpr::Op::Add) {
                node.type = rightType;
            } else {
                ReportError("算术运算要求两边都是 int 类型，或指针与整数", node.loc);
                node.type = Type{TypeKind::Int};
            }
            break;

        case BinaryExpr::Op::Mul:
        case BinaryExpr::Op::Div:
        case BinaryExpr::Op::Mod:
            if (!IsInt(leftType) || !IsInt(rightType)) {
                ReportError("算术运算要求两边都是 int 类型", node.loc);
            }
            node.type = Type{TypeKind::Int};
            break;

        case BinaryExpr::Op::Eq:
        case BinaryExpr::Op::Ne:
            if (!IsComparable(leftType, rightType)) {
                ReportError("类型不兼容，无法比较", node.loc);
            }
            node.type = Type{TypeKind::Int};
            break;

        case BinaryExpr::Op::Lt:
        case BinaryExpr::Op::Le:
        case BinaryExpr::Op::Gt:
        case BinaryExpr::Op::Ge:
            if (!IsInt(leftType) || !IsInt(rightType)) {
                ReportError("关系运算要求两边都是 int 类型", node.loc);
            }
            node.type = Type{TypeKind::Int};
            break;

        case BinaryExpr::Op::And:
        case BinaryExpr::Op::Or:
            if (!IsInt(leftType) || !IsInt(rightType)) {
                ReportError("逻辑运算要求两边都是 int 类型", node.loc);
            }
            node.type = Type{TypeKind::Int};
            break;
    }
}

void TypeChecker::VisitUnary(UnaryExpr& node) {
    auto operandType = ResolveExprType(*node.operand);

    switch (node.op) {
        case UnaryExpr::Op::Neg:
        case UnaryExpr::Op::Not:
            if (!IsInt(operandType)) {
                ReportError("一元运算要求操作数是 int 类型", node.loc);
            }
            node.type = Type{TypeKind::Int};
            break;

        case UnaryExpr::Op::Addr:
            // &var -> pointer to var's type
            node.type = Type{TypeKind::Pointer, operandType.name};
            node.type.baseKind = operandType.kind;
            break;

        case UnaryExpr::Op::Deref:
            // *ptr -> base type
            if (operandType.kind != TypeKind::Pointer && operandType.kind != TypeKind::Array) {
                ReportError("解引用要求指针类型", node.loc, ErrorCode::E3021_DerefNonPointer);
                node.type = Type{TypeKind::Int};
            } else {
                node.type = operandType.baseKind == TypeKind::Struct ?
                    Type{TypeKind::Struct, operandType.name} :
                    Type{operandType.baseKind};
            }
            break;

        case UnaryExpr::Op::PreInc:
        case UnaryExpr::Op::PreDec:
        case UnaryExpr::Op::PostInc:
        case UnaryExpr::Op::PostDec:
            if (!IsInt(operandType)) {
                ReportError("自增/自减要求 int 类型", node.loc);
            }
            node.type = Type{TypeKind::Int};
            break;
    }
}

void TypeChecker::VisitLiteral(LiteralExpr& node) {
    node.type = Type{TypeKind::Int};
}

void TypeChecker::VisitStringLiteral(StringLiteralExpr& node) {
    node.type = Type{TypeKind::Pointer, "char", 0, TypeKind::Char};
}

void TypeChecker::VisitIdentifier(IdentifierExpr& node) {
    auto sym = LookupVar(node.name);
    if (!sym) {
        ReportError("未声明的变量 '" + node.name + "'", node.loc, ErrorCode::E3023_UndeclaredVar);
        node.type = Type{TypeKind::Int};
        return;
    }
    node.type = sym->type;
}

void TypeChecker::VisitCall(CallExpr& node) {
    auto it = funcs_.find(node.name);
    if (it == funcs_.end()) {
        // Built-in functions: malloc, free, print_int
        if (node.name == "malloc") {
            if (node.args.size() != 1) {
                ReportError("malloc 需要一个参数", node.loc);
            } else {
                auto argType = ResolveExprType(*node.args[0]);
                if (!IsInt(argType)) {
                    ReportError("malloc 参数必须是 int", node.loc);
                }
            }
            node.type = Type{TypeKind::Pointer};
            return;
        }
        if (node.name == "free") {
            if (node.args.size() != 1) {
                ReportError("free 需要一个参数", node.loc);
            } else {
                auto argType = ResolveExprType(*node.args[0]);
                if (argType.kind != TypeKind::Pointer && argType.kind != TypeKind::Array) {
                    ReportError("free 参数必须是指针", node.loc);
                }
            }
            node.type = Type{TypeKind::Void};
            return;
        }
        if (node.name == "print_int" || node.name == "__cide_output" || node.name == "__cide_step") {
            if (node.args.size() != 1) {
                ReportError(node.name + " 需要一个参数", node.loc);
            } else {
                auto argType = ResolveExprType(*node.args[0]);
                if (!IsInt(argType)) {
                    ReportError(node.name + " 参数必须是 int", node.loc);
                }
            }
            node.type = Type{TypeKind::Void};
            return;
        }
        if (node.name == "printf") {
            if (node.args.empty()) {
                ReportError("printf 至少需要 1 个参数（格式字符串）", node.loc);
            } else {
                auto fmtType = ResolveExprType(*node.args[0]);
                if (fmtType.kind != TypeKind::Pointer && fmtType.kind != TypeKind::Array) {
                    ReportError("printf 的第一个参数必须是字符串", node.loc);
                }
                for (size_t i = 1; i < node.args.size(); i++) {
                    auto argType = ResolveExprType(*node.args[i]);
                    if (!IsInt(argType) && argType.kind != TypeKind::Pointer && argType.kind != TypeKind::Array) {
                        ReportError("printf 的第 " + std::to_string(i + 1) + " 个参数必须是 int、char 或指针", node.loc);
                    }
                }
            }
            node.type = Type{TypeKind::Void};
            return;
        }
        if (node.name == "scanf") {
            if (node.args.size() < 2) {
                ReportError("scanf 至少需要 2 个参数（格式字符串和地址）", node.loc);
            } else {
                auto fmtType = ResolveExprType(*node.args[0]);
                if (fmtType.kind != TypeKind::Pointer && fmtType.kind != TypeKind::Array) {
                    ReportError("scanf 的第一个参数必须是字符串", node.loc);
                }
                for (size_t i = 1; i < node.args.size(); i++) {
                    auto argType = ResolveExprType(*node.args[i]);
                    if (argType.kind != TypeKind::Pointer && argType.kind != TypeKind::Array) {
                        ReportError("scanf 的第 " + std::to_string(i + 1) + " 个参数必须是指针", node.loc);
                    }
                }
            }
            node.type = Type{TypeKind::Void};
            return;
        }

        ReportError("未定义的函数 '" + node.name + "'", node.loc, ErrorCode::E3036_UndefinedFunc);
            node.type = Type{TypeKind::Void};
            return;
        }

    const auto& sym = it->second;
    if (node.args.size() != sym.paramTypes.size()) {
        ReportError("函数 '" + node.name + "' 参数数量不匹配：期望 " +
                    std::to_string(sym.paramTypes.size()) + "，实际 " +
                    std::to_string(node.args.size()), node.loc);
    } else {
        for (size_t i = 0; i < node.args.size() && i < sym.paramTypes.size(); i++) {
            auto argType = ResolveExprType(*node.args[i]);
            if (!IsAssignable(sym.paramTypes[i], argType, node.loc)) {
                ReportError("函数 '" + node.name + "' 第 " + std::to_string(i + 1) +
                            " 个参数类型不匹配", node.loc);
            }
        }
    }
    node.type = sym.returnType;
}

void TypeChecker::VisitIndex(IndexExpr& node) {
    auto arrType = ResolveExprType(*node.array);
    auto idxType = ResolveExprType(*node.index);

    if (!IsInt(idxType)) {
        ReportError("数组索引必须是 int 类型", node.loc, ErrorCode::E3039_ArrayIndexType);
        node.type = Type{TypeKind::Int};
        return;
    }
    if (!arrType.isArray() && !arrType.isPointer()) {
        ReportError("不能对非数组/指针类型进行索引", node.loc, ErrorCode::E3040_IndexNonArray);
        node.type = Type{TypeKind::Int};
        return;
    }

    // Multi-dimensional array: arr[i] returns sub-array type if more dims remain
    if (arrType.isArray() && !arrType.dims.empty() && arrType.dims.size() > 1) {
        node.type = arrType.subscriptType();
        return;
    }

    // Set element type based on array/pointer base type
    if (arrType.baseKind == TypeKind::Struct) {
        node.type = Type{TypeKind::Struct, arrType.name};
    } else if (arrType.baseKind == TypeKind::Char) {
        node.type = Type{TypeKind::Char};
    } else {
        node.type = Type{TypeKind::Int};
    }
}

void TypeChecker::VisitMember(MemberExpr& node) {
    auto objType = ResolveExprType(*node.object);

    std::string structName;
    if (objType.kind == TypeKind::Struct) {
        structName = objType.name;
    } else if (objType.kind == TypeKind::Pointer && !objType.name.empty()) {
        // struct pointer -> auto-deref
        structName = objType.name;
    } else {
        ReportError("'.' 和 '->' 只能用于结构体类型", node.loc);
        node.type = Type{TypeKind::Int};
        return;
    }

    auto fieldType = GetStructFieldType(structName, node.member);
    if (!fieldType) {
        ReportError("结构体 '" + structName + "' 没有成员 '" + node.member + "'", node.loc);
        node.type = Type{TypeKind::Int};
        return;
    }
    node.type = *fieldType;
}

void TypeChecker::VisitAssign(AssignExpr& node) {
    auto rightType = ResolveExprType(*node.right);
    auto leftType = ResolveExprType(*node.left);

    // Check left is assignable (identifier, index, member, deref)
    bool isLValue = false;
    if (node.left->kind == ExprKind::Identifier) isLValue = true;
    if (node.left->kind == ExprKind::Index) isLValue = true;
    if (node.left->kind == ExprKind::Member) isLValue = true;
    if (node.left->kind == ExprKind::Unary) {
        auto& un = static_cast<UnaryExpr&>(*node.left);
        if (un.op == UnaryExpr::Op::Deref) isLValue = true;
    }

    if (!isLValue) {
        ReportError("赋值左边必须是可修改的左值", node.loc);
    }

    if (!IsAssignable(leftType, rightType, node.loc)) {
        ReportError("类型不匹配：无法将 '" + rightType.ToString() +
                    "' 赋值给 '" + leftType.ToString() + "'", node.loc);
    }

    // For compound assignments, also check arithmetic compatibility
    if (node.op != AssignExpr::Op::Assign) {
        if (!IsInt(leftType) || !IsInt(rightType)) {
            ReportError("复合赋值要求两边都是 int 类型", node.loc);
        }
    }

    node.type = leftType;
}

void TypeChecker::VisitSizeof(SizeofExpr& node) {
    node.type = Type{TypeKind::Int};
    if (!node.isTypeQuery && node.operand) {
        ResolveExprType(*node.operand);
    }
}

void TypeChecker::VisitInitList(InitListExpr& node) {
    // InitList type depends on context; mark as Void here
    // Actual type checking is done in CheckArrayInitializer
    for (auto& elem : node.elements) {
        ResolveExprType(*elem);
    }
    node.type = Type{TypeKind::Void};
}

void TypeChecker::VisitSwitch(SwitchStmt& node) {
    switchDepth_++;
    if (node.cond) {
        auto condType = ResolveExprType(*node.cond);
        if (!IsInt(condType)) {
            ReportError("switch 条件必须是整数类型", node.cond->loc);
        }
    }
    if (node.body) {
        DispatchStmt(*node.body);
    }
    switchDepth_--;
}

void TypeChecker::VisitCase(CaseStmt& node) {
    if (node.label) {
        auto labelType = ResolveExprType(*node.label);
        if (!IsInt(labelType)) {
            ReportError("case 标签必须是整数常量", node.label->loc);
        }
    }
    if (node.stmt) {
        DispatchStmt(*node.stmt);
    }
}

} // namespace cide
