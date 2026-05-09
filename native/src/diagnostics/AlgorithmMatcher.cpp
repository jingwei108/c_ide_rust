#include "AlgorithmMatcher.hpp"
#include "compiler/Ast.hpp"
#include <stack>

namespace cide {

// ============================================================================
// Public API
// ============================================================================

std::vector<AlgorithmMatch> AlgorithmMatcher::Analyze(const ProgramNode& program) {
    matches_.clear();
    linkedListNodeTypes_.clear();

    // Scan struct definitions for linked-list node pattern:
    // struct Node { int val; struct Node* next; }
    for (const auto& st : program.structs) {
        for (const auto& field : st.fields) {
            if (field.type.kind == TypeKind::Pointer &&
                field.type.baseKind == TypeKind::Struct &&
                field.type.name == st.name &&
                (field.name == "next" || field.name == "prev")) {
                linkedListNodeTypes_.push_back(st.name);
                break;
            }
        }
    }

    for (const auto& func : program.funcs) {
        VisitFunc(func);
    }
    return matches_;
}

void AlgorithmMatcher::VisitFunc(const FuncDecl& func) {
    if (!func.body) return;

    auto bubble = DetectBubbleSort(func);
    if (bubble.confidence > 0) {
        matches_.push_back(bubble);
    }
    auto selection = DetectSelectionSort(func);
    if (selection.confidence > 0) {
        matches_.push_back(selection);
    }
    auto insertion = DetectInsertionSort(func);
    if (insertion.confidence > 0) {
        matches_.push_back(insertion);
    }
    auto binarySearch = DetectBinarySearch(func);
    if (binarySearch.confidence > 0) {
        matches_.push_back(binarySearch);
    }
    if (!linkedListNodeTypes_.empty()) {
        auto llTraverse = DetectLinkedListTraversal(func);
        if (llTraverse.confidence > 0) {
            matches_.push_back(llTraverse);
        }
        auto llReverse = DetectLinkedListReverse(func);
        if (llReverse.confidence > 0) {
            matches_.push_back(llReverse);
        }
        auto llInsert = DetectLinkedListInsert(func);
        if (llInsert.confidence > 0) {
            matches_.push_back(llInsert);
        }
        auto llDelete = DetectLinkedListDelete(func);
        if (llDelete.confidence > 0) {
            matches_.push_back(llDelete);
        }
    }
    auto quickSort = DetectQuickSort(func);
    if (quickSort.confidence > 0) {
        matches_.push_back(quickSort);
    }
    auto mergeSort = DetectMergeSort(func);
    if (mergeSort.confidence > 0) {
        matches_.push_back(mergeSort);
    }
}

// ============================================================================
// Bubble sort detection
// Pattern: outer for (i) + inner for (j) + if (arr[j] > arr[j+1]) + swap
// ============================================================================

AlgorithmMatch AlgorithmMatcher::DetectBubbleSort(const FuncDecl& func) {
    if (!func.body) return {};

    // Step 1: Find outer for loop
    const ForStmt* outerFor = nullptr;
    VisitStmt(*func.body, [&](const Stmt& stmt) {
        if (stmt.kind == StmtKind::For && !outerFor) {
            outerFor = static_cast<const ForStmt*>(&stmt);
        }
    });
    if (!outerFor) return {};

    // Step 2: Find inner for loop inside outer loop body
    const ForStmt* innerFor = nullptr;
    if (outerFor->body) {
        VisitStmt(*outerFor->body, [&](const Stmt& stmt) {
            if (stmt.kind == StmtKind::For && !innerFor) {
                innerFor = static_cast<const ForStmt*>(&stmt);
            }
        });
    }
    if (!innerFor) return {};

    // Step 3: Find if statement with array comparison inside inner loop
    const IfStmt* ifStmt = nullptr;
    std::string arrName;
    if (innerFor->body) {
        VisitStmt(*innerFor->body, [&](const Stmt& stmt) {
            if (stmt.kind == StmtKind::If && !ifStmt) {
                auto* candidate = static_cast<const IfStmt*>(&stmt);
                if (candidate->cond && candidate->cond->kind == ExprKind::Binary) {
                    const auto& bin = static_cast<const BinaryExpr&>(*candidate->cond);
                    if (bin.op == BinaryExpr::Op::Gt || bin.op == BinaryExpr::Op::Lt) {
                        // Try to extract array name from left or right side
                        std::string name = ExtractArrayName(bin.left.get());
                        if (!name.empty()) {
                            arrName = name;
                            ifStmt = candidate;
                        } else {
                            name = ExtractArrayName(bin.right.get());
                            if (!name.empty()) {
                                arrName = name;
                                ifStmt = candidate;
                            }
                        }
                    }
                }
            }
        });
    }
    if (!ifStmt || arrName.empty()) return {};

    // Step 4: Check if if-body contains swap pattern
    bool hasSwap = false;
    if (ifStmt->thenStmt) {
        VisitStmt(*ifStmt->thenStmt, [&](const Stmt& stmt) {
            if (IsSwapPattern(stmt, arrName)) {
                hasSwap = true;
            }
        });
    }

    if (!hasSwap) return {};

    // Confidence scoring
    int confidence = 85;
    if (func.params.size() >= 2) confidence += 10; // function takes arr[] and n

    AlgorithmMatch match;
    match.funcName = func.name;
    match.algorithmName = "bubble_sort";
    match.displayName = "冒泡排序";
    match.confidence = std::min(confidence, 100);
    match.suggestion = "检测到冒泡排序模式。外层循环控制轮数，内层循环比较相邻元素并交换。时间复杂度 O(n²)。";
    match.line = outerFor->loc.line;
    if (ifStmt && ifStmt->loc.line > 0) {
        match.visEvents.push_back({ifStmt->loc.line, 1, "j:j+1"}); // Compare
    }
    int swapLine = 0;
    if (ifStmt && ifStmt->thenStmt) {
        if (ifStmt->thenStmt->kind == StmtKind::Block) {
            const auto& block = static_cast<const BlockStmt&>(*ifStmt->thenStmt);
            for (const auto& s : block.stmts) {
                if (s->loc.line > 0) { swapLine = s->loc.line; break; }
            }
        } else {
            swapLine = ifStmt->thenStmt->loc.line;
        }
    }
    if (swapLine > 0) {
        match.visEvents.push_back({swapLine, 2, "j:j+1"}); // Swap
    }
    return match;
}

// ============================================================================
// Selection sort detection
// Pattern: outer for (i) + inner for (j) + if (arr[j] < arr[min]) + swap
// ============================================================================

AlgorithmMatch AlgorithmMatcher::DetectSelectionSort(const FuncDecl& func) {
    if (!func.body) return {};

    // Step 1: Find outer for loop
    const ForStmt* outerFor = nullptr;
    VisitStmt(*func.body, [&](const Stmt& stmt) {
        if (stmt.kind == StmtKind::For && !outerFor) {
            outerFor = static_cast<const ForStmt*>(&stmt);
        }
    });
    if (!outerFor) return {};

    // Step 2: Find inner for loop inside outer loop body
    const ForStmt* innerFor = nullptr;
    if (outerFor->body) {
        VisitStmt(*outerFor->body, [&](const Stmt& stmt) {
            if (stmt.kind == StmtKind::For && !innerFor) {
                innerFor = static_cast<const ForStmt*>(&stmt);
            }
        });
    }
    if (!innerFor) return {};

    // Step 3: Find if with array comparison (arr[j] < arr[min]) inside inner loop
    const IfStmt* ifStmt = nullptr;
    std::string arrName;
    if (innerFor->body) {
        VisitStmt(*innerFor->body, [&](const Stmt& stmt) {
            if (stmt.kind == StmtKind::If && !ifStmt) {
                auto* candidate = static_cast<const IfStmt*>(&stmt);
                if (candidate->cond && candidate->cond->kind == ExprKind::Binary) {
                    const auto& bin = static_cast<const BinaryExpr&>(*candidate->cond);
                    if (bin.op == BinaryExpr::Op::Lt) {
                        std::string name = ExtractArrayName(bin.left.get());
                        if (!name.empty()) {
                            arrName = name;
                            ifStmt = candidate;
                        } else {
                            name = ExtractArrayName(bin.right.get());
                            if (!name.empty()) {
                                arrName = name;
                                ifStmt = candidate;
                            }
                        }
                    }
                }
            }
        });
    }
    if (!ifStmt || arrName.empty()) return {};

    // Step 4: Check if if-body contains min update (min = j or minIdx = j)
    bool hasMinUpdate = false;
    std::string minVarName;
    if (ifStmt->thenStmt) {
        VisitStmt(*ifStmt->thenStmt, [&](const Stmt& stmt) {
            if (stmt.kind == StmtKind::Expr) {
                const auto& exprStmt = static_cast<const ExprStmt&>(stmt);
                if (exprStmt.expr && exprStmt.expr->kind == ExprKind::Assign) {
                    const auto& assign = static_cast<const AssignExpr&>(*exprStmt.expr);
                    if (assign.left && assign.left->kind == ExprKind::Identifier) {
                        if (assign.right && assign.right->kind == ExprKind::Identifier) {
                            hasMinUpdate = true;
                            minVarName = static_cast<const IdentifierExpr&>(*assign.left).name;
                        }
                    }
                }
            }
        });
    }
    if (!hasMinUpdate) return {};

    // Step 5: Check for swap pattern in outer loop body (after inner loop)
    bool hasSwap = false;
    if (outerFor->body) {
        VisitStmt(*outerFor->body, [&](const Stmt& stmt) {
            if (IsSwapPattern(stmt, arrName)) {
                hasSwap = true;
            }
        });
    }
    if (!hasSwap) return {};

    AlgorithmMatch match;
    match.funcName = func.name;
    match.algorithmName = "selection_sort";
    match.displayName = "选择排序";
    match.confidence = 80;
    match.suggestion = "检测到选择排序模式。每轮从未排序区间选出最小元素，放到已排序区间末尾。时间复杂度 O(n²)。";
    match.line = outerFor->loc.line;
    if (ifStmt && ifStmt->loc.line > 0) {
        std::string ctx = minVarName.empty() ? "j:min" : "j:" + minVarName;
        match.visEvents.push_back({ifStmt->loc.line, 1, ctx}); // Compare
    }
    int swapLine = 0;
    if (outerFor->body) {
        const BlockStmt* block = nullptr;
        if (outerFor->body->kind == StmtKind::Block) {
            block = static_cast<const BlockStmt*>(outerFor->body.get());
        }
        if (block) {
            for (const auto& s : block->stmts) {
                if (s->kind == StmtKind::Expr) {
                    const auto& exprStmt = static_cast<const ExprStmt&>(*s);
                    if (exprStmt.expr && exprStmt.expr->kind == ExprKind::Assign) {
                        const auto& assign = static_cast<const AssignExpr&>(*exprStmt.expr);
                        if (assign.left && assign.left->kind == ExprKind::Index) {
                            if (ExtractArrayName(assign.left.get()) == arrName) {
                                swapLine = s->loc.line;
                            }
                        }
                    }
                }
            }
        }
    }
    if (swapLine > 0) {
        std::string ctx = minVarName.empty() ? "i:min" : "i:" + minVarName;
        match.visEvents.push_back({swapLine, 2, ctx}); // Swap
    }
    return match;
}

AlgorithmMatch AlgorithmMatcher::DetectInsertionSort(const FuncDecl& func) {
    if (!func.body) return {};

    // Step 1: Find outer for loop
    const ForStmt* outerFor = nullptr;
    VisitStmt(*func.body, [&](const Stmt& stmt) {
        if (stmt.kind == StmtKind::For && !outerFor) {
            outerFor = static_cast<const ForStmt*>(&stmt);
        }
    });
    if (!outerFor) return {};

    // Step 2: Find while loop inside outer for body
    const WhileStmt* whileLoop = nullptr;
    if (outerFor->body) {
        VisitStmt(*outerFor->body, [&](const Stmt& stmt) {
            if (stmt.kind == StmtKind::While && !whileLoop) {
                whileLoop = static_cast<const WhileStmt*>(&stmt);
            }
        });
    }
    if (!whileLoop) return {};

    // Step 3: Check while condition has array comparison (arr[j] > key)
    std::string arrName;
    if (whileLoop->cond && whileLoop->cond->kind == ExprKind::Binary) {
        const auto* bin = static_cast<const BinaryExpr*>(whileLoop->cond.get());
        if (bin->op == BinaryExpr::Op::Gt) {
            arrName = ExtractArrayName(bin->left.get());
            if (arrName.empty()) arrName = ExtractArrayName(bin->right.get());
        }
    }
    if (arrName.empty()) return {};

    // Step 4: Check while body has shift pattern: arr[j+1] = arr[j]
    bool hasShift = false;
    if (whileLoop->body) {
        VisitStmt(*whileLoop->body, [&](const Stmt& stmt) {
            if (stmt.kind == StmtKind::Expr) {
                const auto& exprStmt = static_cast<const ExprStmt&>(stmt);
                if (exprStmt.expr && exprStmt.expr->kind == ExprKind::Assign) {
                    const auto& assign = static_cast<const AssignExpr&>(*exprStmt.expr);
                    if (assign.left && assign.left->kind == ExprKind::Index &&
                        assign.right && assign.right->kind == ExprKind::Index) {
                        std::string leftArr = ExtractArrayName(assign.left.get());
                        std::string rightArr = ExtractArrayName(assign.right.get());
                        if (leftArr == rightArr && leftArr == arrName) {
                            hasShift = true;
                        }
                    }
                }
            }
        });
    }
    if (!hasShift) return {};

    AlgorithmMatch match;
    match.funcName = func.name;
    match.algorithmName = "insertion_sort";
    match.displayName = "插入排序";
    match.confidence = 75;
    match.suggestion = "检测到插入排序模式。将未排序元素逐个插入已排序区间的正确位置。时间复杂度 O(n²)。";
    match.line = outerFor->loc.line;
    if (whileLoop && whileLoop->loc.line > 0) {
        match.visEvents.push_back({whileLoop->loc.line, 1, "j-1"}); // Compare
    }
    int updateLine = 0;
    if (whileLoop && whileLoop->body) {
        const BlockStmt* block = nullptr;
        if (whileLoop->body->kind == StmtKind::Block) {
            block = static_cast<const BlockStmt*>(whileLoop->body.get());
        }
        if (block) {
            for (const auto& s : block->stmts) {
                if (s->kind == StmtKind::Expr) {
                    const auto& exprStmt = static_cast<const ExprStmt&>(*s);
                    if (exprStmt.expr && exprStmt.expr->kind == ExprKind::Assign) {
                        const auto& assign = static_cast<const AssignExpr&>(*exprStmt.expr);
                        if (assign.left && assign.left->kind == ExprKind::Index &&
                            assign.right && assign.right->kind == ExprKind::Index) {
                            if (ExtractArrayName(assign.left.get()) == arrName &&
                                ExtractArrayName(assign.right.get()) == arrName) {
                                updateLine = s->loc.line;
                            }
                        }
                    }
                }
            }
        }
    }
    if (updateLine > 0) {
        match.visEvents.push_back({updateLine, 3, "j:j+1"}); // Update (shift)
    }
    return match;
}

// ============================================================================
// Binary search detection
// Pattern: while (left <= right) { mid = (left+right)/2; if (arr[mid] == target) ... left/right update }
// ============================================================================

AlgorithmMatch AlgorithmMatcher::DetectBinarySearch(const FuncDecl& func) {
    if (!func.body) return {};

    // Helper: extract identifier name
    auto identName = [](const Expr* expr) -> std::string {
        if (!expr) return "";
        if (expr->kind == ExprKind::Identifier) {
            return static_cast<const IdentifierExpr*>(expr)->name;
        }
        return "";
    };

    // Helper: check if expression contains a given identifier
    std::function<bool(const Expr*, const std::string&)> containsIdent;
    containsIdent = [&](const Expr* expr, const std::string& name) -> bool {
        if (!expr) return false;
        if (expr->kind == ExprKind::Identifier) {
            return static_cast<const IdentifierExpr*>(expr)->name == name;
        }
        if (expr->kind == ExprKind::Binary) {
            const auto* bin = static_cast<const BinaryExpr*>(expr);
            return containsIdent(bin->left.get(), name) || containsIdent(bin->right.get(), name);
        }
        if (expr->kind == ExprKind::Unary) {
            return containsIdent(static_cast<const UnaryExpr*>(expr)->operand.get(), name);
        }
        if (expr->kind == ExprKind::Index) {
            const auto* idx = static_cast<const IndexExpr*>(expr);
            return containsIdent(idx->array.get(), name) || containsIdent(idx->index.get(), name);
        }
        if (expr->kind == ExprKind::Assign) {
            const auto* assign = static_cast<const AssignExpr*>(expr);
            return containsIdent(assign->left.get(), name) || containsIdent(assign->right.get(), name);
        }
        return false;
    };

    // Step 1: Find while loop with (bound1 <= bound2) or (bound1 < bound2)
    const WhileStmt* whileLoop = nullptr;
    std::string bound1, bound2;
    VisitStmt(*func.body, [&](const Stmt& stmt) {
        if (whileLoop) return;
        if (stmt.kind == StmtKind::While) {
            const auto* w = static_cast<const WhileStmt*>(&stmt);
            if (w->cond && w->cond->kind == ExprKind::Binary) {
                const auto* bin = static_cast<const BinaryExpr*>(w->cond.get());
                if (bin->op == BinaryExpr::Op::Le || bin->op == BinaryExpr::Op::Lt) {
                    std::string leftName = identName(bin->left.get());
                    std::string rightName = identName(bin->right.get());
                    if (!leftName.empty() && !rightName.empty() && leftName != rightName) {
                        bound1 = leftName;
                        bound2 = rightName;
                        whileLoop = w;
                    }
                }
            }
        }
    });
    if (!whileLoop) return {};

    // Step 2: Inside while body, look for:
    //   a) variable declaration initialized with expression containing both bounds (mid calc)
    //   b) if statement with arr[mid] comparison
    //   c) assignment updating bound1 or bound2

    bool hasMidCalc = false;
    std::string midVarName;
    const IfStmt* ifStmt = nullptr;
    std::string arrName;
    bool hasBoundUpdate = false;
    int compareLine = 0;
    int updateLine = 0;

    std::function<void(const Stmt&)> scanBody;
    scanBody = [&](const Stmt& stmt) {
        switch (stmt.kind) {
            case StmtKind::Block: {
                const auto& block = static_cast<const BlockStmt&>(stmt);
                for (const auto& s : block.stmts) {
                    scanBody(*s);
                }
                break;
            }
            case StmtKind::VarDecl: {
                const auto& decl = static_cast<const VarDeclStmt&>(stmt);
                if (!hasMidCalc && decl.init) {
                    bool hasB1 = containsIdent(decl.init.get(), bound1);
                    bool hasB2 = containsIdent(decl.init.get(), bound2);
                    if (hasB1 && hasB2) {
                        hasMidCalc = true;
                        midVarName = decl.name;
                    }
                }
                break;
            }
            case StmtKind::If: {
                const auto& ifS = static_cast<const IfStmt&>(stmt);
                if (!ifStmt && ifS.cond) {
                    // Check for arr[mid] comparison
                    const Expr* cond = ifS.cond.get();
                    // Handle nested binary (a == b) inside else-if chains? Keep simple.
                    if (cond->kind == ExprKind::Binary) {
                        const auto* bin = static_cast<const BinaryExpr*>(cond);
                        // left or right side should be arr[mid]
                        auto getArrMidName = [&](const Expr* expr) -> std::string {
                            if (!expr || expr->kind != ExprKind::Index) return "";
                            const auto* idx = static_cast<const IndexExpr*>(expr);
                            if (!idx->index || idx->index->kind != ExprKind::Identifier) return "";
                            if (midVarName.empty()) {
                                // mid not declared yet? maybe mid is assigned before if
                                // Accept any identifier as index for now
                            } else {
                                if (static_cast<const IdentifierExpr*>(idx->index.get())->name != midVarName)
                                    return "";
                            }
                            if (idx->array && idx->array->kind == ExprKind::Identifier) {
                                return static_cast<const IdentifierExpr*>(idx->array.get())->name;
                            }
                            return "";
                        };
                        std::string arrMidLeft = getArrMidName(bin->left.get());
                        std::string arrMidRight = getArrMidName(bin->right.get());
                        if (!arrMidLeft.empty() || !arrMidRight.empty()) {
                            arrName = !arrMidLeft.empty() ? arrMidLeft : arrMidRight;
                            ifStmt = &ifS;
                            compareLine = ifS.loc.line;
                        }
                    }
                }
                if (ifS.thenStmt) scanBody(*ifS.thenStmt);
                if (ifS.elseStmt) scanBody(*ifS.elseStmt);
                break;
            }
            case StmtKind::Expr: {
                const auto& exprStmt = static_cast<const ExprStmt&>(stmt);
                if (exprStmt.expr && exprStmt.expr->kind == ExprKind::Assign) {
                    const auto* assign = static_cast<const AssignExpr*>(exprStmt.expr.get());
                    std::string lhs = identName(assign->left.get());
                    if ((lhs == bound1 || lhs == bound2) && containsIdent(assign->right.get(), midVarName)) {
                        hasBoundUpdate = true;
                        if (updateLine == 0) updateLine = stmt.loc.line;
                    }
                }
                break;
            }
            default:
                break;
        }
    };

    if (whileLoop->body) {
        scanBody(*whileLoop->body);
    }

    // Also accept cases where mid is not declared but assigned (e.g., mid = ...)
    if (!hasMidCalc && midVarName.empty()) {
        // Re-scan for assignment to mid-like variable
        scanBody = [&](const Stmt& stmt) {
            switch (stmt.kind) {
                case StmtKind::Block: {
                    const auto& block = static_cast<const BlockStmt&>(stmt);
                    for (const auto& s : block.stmts) scanBody(*s);
                    break;
                }
                case StmtKind::Expr: {
                    const auto& exprStmt = static_cast<const ExprStmt&>(stmt);
                    if (!hasMidCalc && exprStmt.expr && exprStmt.expr->kind == ExprKind::Assign) {
                        const auto* assign = static_cast<const AssignExpr*>(exprStmt.expr.get());
                        std::string lhs = identName(assign->left.get());
                        if (!lhs.empty() && lhs != bound1 && lhs != bound2 && lhs != arrName) {
                            bool hasB1 = containsIdent(assign->right.get(), bound1);
                            bool hasB2 = containsIdent(assign->right.get(), bound2);
                            if (hasB1 && hasB2) {
                                hasMidCalc = true;
                                midVarName = lhs;
                            }
                        }
                    }
                    break;
                }
                default: break;
            }
        };
        if (whileLoop->body) scanBody(*whileLoop->body);
    }

    if (!hasMidCalc || !ifStmt) return {};
    if (!hasBoundUpdate) return {};

    AlgorithmMatch match;
    match.funcName = func.name;
    match.algorithmName = "binary_search";
    match.displayName = "二分查找";
    match.confidence = 80;
    match.suggestion = "检测到二分查找模式。在有序数组中通过不断折半来缩小搜索范围。时间复杂度 O(log n)。";
    match.line = whileLoop->loc.line;
    if (compareLine > 0) {
        match.visEvents.push_back({compareLine, 1, "mid"}); // Compare
    }
    if (updateLine > 0) {
        match.visEvents.push_back({updateLine, 3, ""}); // Update (range shrink)
    }
    return match;
}

// ============================================================================
// Linked list traversal detection
// Pattern: struct Node* p = head; while (p != NULL) { ... p = p->next; }
// ============================================================================

AlgorithmMatch AlgorithmMatcher::DetectLinkedListTraversal(const FuncDecl& func) {
    if (!func.body || linkedListNodeTypes_.empty()) return {};

    auto isLinkedListPtr = [&](const cide::Type& t) -> bool {
        if (t.kind != TypeKind::Pointer || t.baseKind != TypeKind::Struct) return false;
        for (const auto& nt : linkedListNodeTypes_) {
            if (t.name == nt) return true;
        }
        return false;
    };

    // Step 1: Collect all variables of type struct Node* (params + locals)
    std::vector<std::string> ptrVars;
    for (const auto& p : func.params) {
        if (isLinkedListPtr(p.type)) {
            ptrVars.push_back(p.name);
        }
    }
    // Also scan local variable declarations
    VisitStmt(*func.body, [&](const Stmt& stmt) {
        if (stmt.kind == StmtKind::VarDecl) {
            const auto& decl = static_cast<const VarDeclStmt&>(stmt);
            if (isLinkedListPtr(decl.varType)) {
                ptrVars.push_back(decl.name);
            }
        }
    });
    if (ptrVars.empty()) return {};

    // Step 2: Find while loop with condition involving any ptrVar
    const WhileStmt* whileLoop = nullptr;
    std::string loopVar;
    VisitStmt(*func.body, [&](const Stmt& stmt) {
        if (whileLoop) return;
        if (stmt.kind == StmtKind::While) {
            const auto* w = static_cast<const WhileStmt*>(&stmt);
            if (w->cond) {
                for (const auto& pv : ptrVars) {
                    if (w->cond->kind == ExprKind::Identifier) {
                        if (static_cast<const IdentifierExpr*>(w->cond.get())->name == pv) {
                            whileLoop = w;
                            loopVar = pv;
                            break;
                        }
                    } else if (w->cond->kind == ExprKind::Binary) {
                        const auto* bin = static_cast<const BinaryExpr*>(w->cond.get());
                        if (bin->op == BinaryExpr::Op::Ne) {
                            if (bin->left->kind == ExprKind::Identifier &&
                                static_cast<const IdentifierExpr*>(bin->left.get())->name == pv) {
                                if (bin->right->kind == ExprKind::Literal &&
                                    static_cast<const LiteralExpr*>(bin->right.get())->value == 0) {
                                    whileLoop = w;
                                    loopVar = pv;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    });
    if (!whileLoop) return {};

    // Step 3: Check body has loopVar->next access or loopVar = loopVar->next
    bool hasNextAccess = false;
    bool hasNextAssign = false;

    std::function<void(const Stmt&)> scan;
    scan = [&](const Stmt& stmt) {
        switch (stmt.kind) {
            case StmtKind::Block: {
                const auto& block = static_cast<const BlockStmt&>(stmt);
                for (const auto& s : block.stmts) scan(*s);
                break;
            }
            case StmtKind::If: {
                const auto& ifs = static_cast<const IfStmt&>(stmt);
                if (ifs.thenStmt) scan(*ifs.thenStmt);
                if (ifs.elseStmt) scan(*ifs.elseStmt);
                break;
            }
            case StmtKind::While: {
                const auto& w = static_cast<const WhileStmt&>(stmt);
                if (w.body) scan(*w.body);
                break;
            }
            case StmtKind::Expr: {
                const auto& exprStmt = static_cast<const ExprStmt&>(stmt);
                if (exprStmt.expr) {
                    auto checkExpr = [&](const Expr* expr) {
                        if (!expr) return;
                        if (expr->kind == ExprKind::Member) {
                            const auto* mem = static_cast<const MemberExpr*>(expr);
                            if (mem->member == "next" && mem->object && mem->object->kind == ExprKind::Identifier) {
                                if (static_cast<const IdentifierExpr*>(mem->object.get())->name == loopVar) {
                                    hasNextAccess = true;
                                }
                            }
                        } else if (expr->kind == ExprKind::Assign) {
                            const auto* assign = static_cast<const AssignExpr*>(expr);
                            if (assign->left && assign->left->kind == ExprKind::Identifier) {
                                if (static_cast<const IdentifierExpr*>(assign->left.get())->name == loopVar) {
                                    if (assign->right && assign->right->kind == ExprKind::Member) {
                                        const auto* mem = static_cast<const MemberExpr*>(assign->right.get());
                                        if (mem->member == "next" && mem->object && mem->object->kind == ExprKind::Identifier) {
                                            if (static_cast<const IdentifierExpr*>(mem->object.get())->name == loopVar) {
                                                hasNextAssign = true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    };
                    checkExpr(exprStmt.expr.get());
                }
                break;
            }
            default: break;
        }
    };

    if (whileLoop->body) {
        scan(*whileLoop->body);
    }

    if (!hasNextAccess && !hasNextAssign) return {};

    AlgorithmMatch match;
    match.funcName = func.name;
    match.algorithmName = "linked_list_traversal";
    match.displayName = "链表遍历";
    match.confidence = hasNextAssign ? 85 : 60;
    match.suggestion = "检测到链表遍历模式。从头到尾依次访问每个节点。时间复杂度 O(n)。";
    match.line = whileLoop->loc.line;
    return match;
}

// ============================================================================
// Linked list reverse detection
// Pattern: while(curr) { next=curr->next; curr->next=prev; prev=curr; curr=next; }
// ============================================================================

AlgorithmMatch AlgorithmMatcher::DetectLinkedListReverse(const FuncDecl& func) {
    if (!func.body || linkedListNodeTypes_.empty()) return {};

    auto isLinkedListPtr = [&](const cide::Type& t) -> bool {
        if (t.kind != TypeKind::Pointer || t.baseKind != TypeKind::Struct) return false;
        for (const auto& nt : linkedListNodeTypes_) {
            if (t.name == nt) return true;
        }
        return false;
    };

    // Collect all variables of type struct Node*
    std::vector<std::string> ptrVars;
    for (const auto& p : func.params) {
        if (isLinkedListPtr(p.type)) {
            ptrVars.push_back(p.name);
        }
    }
    VisitStmt(*func.body, [&](const Stmt& stmt) {
        if (stmt.kind == StmtKind::VarDecl) {
            const auto& decl = static_cast<const VarDeclStmt&>(stmt);
            if (isLinkedListPtr(decl.varType)) {
                ptrVars.push_back(decl.name);
            }
        }
    });
    if (ptrVars.empty()) return {};

    const WhileStmt* whileLoop = nullptr;
    VisitStmt(*func.body, [&](const Stmt& stmt) {
        if (whileLoop) return;
        if (stmt.kind == StmtKind::While) {
            const auto* w = static_cast<const WhileStmt*>(&stmt);
            if (w->cond) {
                for (const auto& pv : ptrVars) {
                    if (w->cond->kind == ExprKind::Identifier) {
                        if (static_cast<const IdentifierExpr*>(w->cond.get())->name == pv) {
                            whileLoop = w;
                            break;
                        }
                    } else if (w->cond->kind == ExprKind::Binary) {
                        const auto* bin = static_cast<const BinaryExpr*>(w->cond.get());
                        if (bin->op == BinaryExpr::Op::Ne) {
                            if (bin->left->kind == ExprKind::Identifier &&
                                static_cast<const IdentifierExpr*>(bin->left.get())->name == pv) {
                                if (bin->right->kind == ExprKind::Literal &&
                                    static_cast<const LiteralExpr*>(bin->right.get())->value == 0) {
                                    whileLoop = w;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    });
    if (!whileLoop) return {};

    bool hasLinkFlip = false;
    bool hasCurrAdvance = false;

    std::function<void(const Stmt&)> scan;
    scan = [&](const Stmt& stmt) {
        switch (stmt.kind) {
            case StmtKind::Block: {
                const auto& block = static_cast<const BlockStmt&>(stmt);
                for (const auto& s : block.stmts) scan(*s);
                break;
            }
            case StmtKind::If: {
                const auto& ifs = static_cast<const IfStmt&>(stmt);
                if (ifs.thenStmt) scan(*ifs.thenStmt);
                if (ifs.elseStmt) scan(*ifs.elseStmt);
                break;
            }
            case StmtKind::Expr: {
                const auto& exprStmt = static_cast<const ExprStmt&>(stmt);
                if (exprStmt.expr && exprStmt.expr->kind == ExprKind::Assign) {
                    const auto* assign = static_cast<const AssignExpr*>(exprStmt.expr.get());
                    if (assign->left && assign->left->kind == ExprKind::Member) {
                        const auto* mem = static_cast<const MemberExpr*>(assign->left.get());
                        if (mem->member == "next") {
                            hasLinkFlip = true;
                        }
                    }
                    if (assign->left && assign->left->kind == ExprKind::Identifier) {
                        if (assign->right && assign->right->kind == ExprKind::Identifier) {
                            hasCurrAdvance = true;
                        }
                    }
                }
                break;
            }
            default: break;
        }
    };

    if (whileLoop->body) {
        scan(*whileLoop->body);
    }

    if (!hasLinkFlip) return {};

    AlgorithmMatch match;
    match.funcName = func.name;
    match.algorithmName = "linked_list_reverse";
    match.displayName = "链表反转";
    match.confidence = hasCurrAdvance ? 80 : 60;
    match.suggestion = "检测到链表反转模式。逐个翻转节点的 next 指针方向。时间复杂度 O(n)。";
    match.line = whileLoop->loc.line;
    return match;
}

// ============================================================================
// Linked list insert detection
// Patterns:
//   Head insert:  newNode->next = head; head = newNode;
//   Mid insert:   newNode->next = p->next; p->next = newNode;
// ============================================================================

AlgorithmMatch AlgorithmMatcher::DetectLinkedListInsert(const FuncDecl& func) {
    if (!func.body || linkedListNodeTypes_.empty()) return {};

    bool hasNextLink = false;
    bool hasHeadUpdate = false;
    bool hasMidInsert = false;
    int matchLine = 0;

    std::function<void(const Stmt&)> scan;
    scan = [&](const Stmt& stmt) {
        switch (stmt.kind) {
            case StmtKind::Block: {
                const auto& block = static_cast<const BlockStmt&>(stmt);
                for (const auto& s : block.stmts) scan(*s);
                break;
            }
            case StmtKind::If: {
                const auto& ifs = static_cast<const IfStmt&>(stmt);
                if (ifs.thenStmt) scan(*ifs.thenStmt);
                if (ifs.elseStmt) scan(*ifs.elseStmt);
                break;
            }
            case StmtKind::While: {
                const auto& w = static_cast<const WhileStmt&>(stmt);
                if (w.body) scan(*w.body);
                break;
            }
            case StmtKind::For: {
                const auto& f = static_cast<const ForStmt&>(stmt);
                if (f.body) scan(*f.body);
                break;
            }
            case StmtKind::Expr: {
                const auto& exprStmt = static_cast<const ExprStmt&>(stmt);
                if (exprStmt.expr && exprStmt.expr->kind == ExprKind::Assign) {
                    const auto* assign = static_cast<const AssignExpr*>(exprStmt.expr.get());
                    // Pattern: newNode->next = head (or any other list pointer)
                    if (assign->left && assign->left->kind == ExprKind::Member) {
                        const auto* mem = static_cast<const MemberExpr*>(assign->left.get());
                        if (mem->member == "next") {
                            hasNextLink = true;
                            if (matchLine == 0) matchLine = stmt.loc.line;
                        }
                    }
                    // Pattern: head = newNode (head update after linking)
                    if (assign->left && assign->left->kind == ExprKind::Identifier) {
                        if (assign->right && assign->right->kind == ExprKind::Identifier) {
                            hasHeadUpdate = true;
                            if (matchLine == 0) matchLine = stmt.loc.line;
                        }
                    }
                    // Pattern: p->next = newNode (mid/tail insert)
                    if (assign->left && assign->left->kind == ExprKind::Member) {
                        const auto* mem = static_cast<const MemberExpr*>(assign->left.get());
                        if (mem->member == "next" && assign->right && assign->right->kind == ExprKind::Identifier) {
                            hasMidInsert = true;
                            if (matchLine == 0) matchLine = stmt.loc.line;
                        }
                    }
                }
                break;
            }
            default: break;
        }
    };

    scan(*func.body);

    bool isHeadInsert = hasNextLink && hasHeadUpdate;
    bool isMidInsert = hasNextLink && hasMidInsert && !hasHeadUpdate;

    if (!isHeadInsert && !isMidInsert) return {};

    AlgorithmMatch match;
    match.funcName = func.name;
    match.algorithmName = "linked_list_insert";
    match.displayName = "链表插入";
    match.confidence = isHeadInsert ? 85 : 75;
    match.suggestion = isHeadInsert
        ? "检测到链表头插法。新节点插入到链表头部，时间复杂度 O(1)。"
        : "检测到链表插入操作。新节点插入到链表中间或尾部，注意维护 next 指针的正确性。";
    match.line = matchLine;
    return match;
}

// ============================================================================
// Linked list delete detection
// Patterns:
//   Delete:  p->next = p->next->next;
//   With free: free(temp); p->next = p->next->next;
// ============================================================================

AlgorithmMatch AlgorithmMatcher::DetectLinkedListDelete(const FuncDecl& func) {
    if (!func.body || linkedListNodeTypes_.empty()) return {};

    bool hasSkipNext = false;
    bool hasFree = false;
    int matchLine = 0;

    std::function<void(const Stmt&)> scan;
    scan = [&](const Stmt& stmt) {
        switch (stmt.kind) {
            case StmtKind::Block: {
                const auto& block = static_cast<const BlockStmt&>(stmt);
                for (const auto& s : block.stmts) scan(*s);
                break;
            }
            case StmtKind::If: {
                const auto& ifs = static_cast<const IfStmt&>(stmt);
                if (ifs.thenStmt) scan(*ifs.thenStmt);
                if (ifs.elseStmt) scan(*ifs.elseStmt);
                break;
            }
            case StmtKind::While: {
                const auto& w = static_cast<const WhileStmt&>(stmt);
                if (w.body) scan(*w.body);
                break;
            }
            case StmtKind::For: {
                const auto& f = static_cast<const ForStmt&>(stmt);
                if (f.body) scan(*f.body);
                break;
            }
            case StmtKind::Expr: {
                const auto& exprStmt = static_cast<const ExprStmt&>(stmt);
                if (exprStmt.expr) {
                    // Detect free() call
                    if (exprStmt.expr->kind == ExprKind::Call) {
                        const auto* call = static_cast<const CallExpr*>(exprStmt.expr.get());
                        if (call->name == "free") {
                            hasFree = true;
                            if (matchLine == 0) matchLine = stmt.loc.line;
                        }
                    }
                    // Detect p->next = p->next->next
                    if (exprStmt.expr->kind == ExprKind::Assign) {
                        const auto* assign = static_cast<const AssignExpr*>(exprStmt.expr.get());
                        if (assign->left && assign->left->kind == ExprKind::Member) {
                            const auto* leftMem = static_cast<const MemberExpr*>(assign->left.get());
                            if (leftMem->member == "next") {
                                if (assign->right && assign->right->kind == ExprKind::Member) {
                                    const auto* rightMem = static_cast<const MemberExpr*>(assign->right.get());
                                    if (rightMem->member == "next" && rightMem->object && rightMem->object->kind == ExprKind::Member) {
                                        const auto* innerMem = static_cast<const MemberExpr*>(rightMem->object.get());
                                        if (innerMem->member == "next") {
                                            hasSkipNext = true;
                                            if (matchLine == 0) matchLine = stmt.loc.line;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                break;
            }
            default: break;
        }
    };

    scan(*func.body);

    if (!hasSkipNext) return {};

    AlgorithmMatch match;
    match.funcName = func.name;
    match.algorithmName = "linked_list_delete";
    match.displayName = "链表删除";
    match.confidence = hasFree ? 90 : 75;
    match.suggestion = hasFree
        ? "检测到链表删除操作。跳过目标节点并释放内存，注意防止内存泄漏。时间复杂度 O(n)。"
        : "检测到链表节点跳过操作。虽然跳过了节点，但未释放内存，可能导致内存泄漏。建议调用 free()。";
    match.line = matchLine;
    return match;
}

// ============================================================================
// Quick sort detection
// Pattern: recursive calls + partition loop with array compare + swap
// ============================================================================

AlgorithmMatch AlgorithmMatcher::DetectQuickSort(const FuncDecl& func) {
    if (!func.body) return {};

    auto identName = [](const Expr* expr) -> std::string {
        if (!expr) return "";
        if (expr->kind == ExprKind::Identifier) {
            return static_cast<const IdentifierExpr*>(expr)->name;
        }
        return "";
    };

    // Step 1: Must have recursive call
    bool hasRecursiveCall = false;
    VisitStmt(*func.body, [&](const Stmt& stmt) {
        if (stmt.kind == StmtKind::Expr) {
            const auto& exprStmt = static_cast<const ExprStmt&>(stmt);
            if (exprStmt.expr && exprStmt.expr->kind == ExprKind::Call) {
                const auto* call = static_cast<const CallExpr*>(exprStmt.expr.get());
                if (call->name == func.name) {
                    hasRecursiveCall = true;
                }
            }
        }
    });
    if (!hasRecursiveCall) return {};

    // Step 2: Find partition loop (while or for with two-variable comparison)
    const Stmt* partitionLoop = nullptr;
    VisitStmt(*func.body, [&](const Stmt& stmt) {
        if (partitionLoop) return;
        if (stmt.kind == StmtKind::While || stmt.kind == StmtKind::For) {
            const Expr* cond = nullptr;
            if (stmt.kind == StmtKind::While) {
                cond = static_cast<const WhileStmt&>(stmt).cond.get();
            } else {
                cond = static_cast<const ForStmt&>(stmt).cond.get();
            }
            if (cond && cond->kind == ExprKind::Binary) {
                const auto* bin = static_cast<const BinaryExpr*>(cond);
                if (bin->op == BinaryExpr::Op::Lt || bin->op == BinaryExpr::Op::Le ||
                    bin->op == BinaryExpr::Op::Gt || bin->op == BinaryExpr::Op::Ge) {
                    std::string leftName = identName(bin->left.get());
                    std::string rightName = identName(bin->right.get());
                    if (!leftName.empty() && !rightName.empty() && leftName != rightName) {
                        partitionLoop = &stmt;
                    }
                }
            }
        }
    });

    // Step 3: Scan for array compare, swap, index movement
    bool hasArrayCompare = false;
    bool hasSwap = false;
    bool hasIndexMove = false;
    std::string arrName;
    int compareLine = 0;
    int swapLine = 0;

    std::function<bool(const Expr*, const std::string&)> containsIdent;
    containsIdent = [&](const Expr* expr, const std::string& name) -> bool {
        if (!expr) return false;
        if (expr->kind == ExprKind::Identifier) {
            return static_cast<const IdentifierExpr*>(expr)->name == name;
        }
        if (expr->kind == ExprKind::Binary) {
            const auto* bin = static_cast<const BinaryExpr*>(expr);
            return containsIdent(bin->left.get(), name) || containsIdent(bin->right.get(), name);
        }
        if (expr->kind == ExprKind::Unary) {
            return containsIdent(static_cast<const UnaryExpr*>(expr)->operand.get(), name);
        }
        if (expr->kind == ExprKind::Index) {
            const auto* idx = static_cast<const IndexExpr*>(expr);
            return containsIdent(idx->array.get(), name) || containsIdent(idx->index.get(), name);
        }
        return false;
    };

    // Recursively find array name inside any expression (for nested binary like &&)
    std::function<std::string(const Expr*)> findArrInExpr;
    findArrInExpr = [&](const Expr* expr) -> std::string {
        if (!expr) return "";
        if (expr->kind == ExprKind::Index) {
            const auto* idx = static_cast<const IndexExpr*>(expr);
            if (idx->array && idx->array->kind == ExprKind::Identifier) {
                return static_cast<const IdentifierExpr*>(idx->array.get())->name;
            }
        }
        if (expr->kind == ExprKind::Binary) {
            const auto* bin = static_cast<const BinaryExpr*>(expr);
            std::string left = findArrInExpr(bin->left.get());
            if (!left.empty()) return left;
            return findArrInExpr(bin->right.get());
        }
        if (expr->kind == ExprKind::Unary) {
            return findArrInExpr(static_cast<const UnaryExpr*>(expr)->operand.get());
        }
        return "";
    };

    auto checkCondForArray = [&](const Expr* cond, int line) {
        std::string name = findArrInExpr(cond);
        if (!name.empty()) {
            hasArrayCompare = true;
            arrName = name;
            if (compareLine == 0) compareLine = line;
        }
    };

    std::function<void(const Stmt&)> scan;
    scan = [&](const Stmt& stmt) {
        switch (stmt.kind) {
            case StmtKind::Block: {
                const auto& block = static_cast<const BlockStmt&>(stmt);
                for (const auto& s : block.stmts) scan(*s);
                break;
            }
            case StmtKind::If: {
                const auto& ifs = static_cast<const IfStmt&>(stmt);
                checkCondForArray(ifs.cond.get(), ifs.loc.line);
                if (ifs.thenStmt) scan(*ifs.thenStmt);
                if (ifs.elseStmt) scan(*ifs.elseStmt);
                break;
            }
            case StmtKind::While: {
                const auto& w = static_cast<const WhileStmt&>(stmt);
                checkCondForArray(w.cond.get(), w.loc.line);
                if (w.body) scan(*w.body);
                break;
            }
            case StmtKind::For: {
                const auto& f = static_cast<const ForStmt&>(stmt);
                checkCondForArray(f.cond.get(), f.loc.line);
                if (f.body) scan(*f.body);
                break;
            }
            case StmtKind::Expr: {
                const auto& exprStmt = static_cast<const ExprStmt&>(stmt);
                if (exprStmt.expr) {
                    if (exprStmt.expr->kind == ExprKind::Assign) {
                        const auto* assign = static_cast<const AssignExpr*>(exprStmt.expr.get());
                        if (assign->left && assign->left->kind == ExprKind::Identifier) {
                            std::string lhsName = identName(assign->left.get());
                            if (!lhsName.empty() && containsIdent(assign->right.get(), lhsName)) {
                                hasIndexMove = true;
                            }
                        }
                    } else if (exprStmt.expr->kind == ExprKind::Unary) {
                        const auto* un = static_cast<const UnaryExpr*>(exprStmt.expr.get());
                        if (un->op == UnaryExpr::Op::PreInc || un->op == UnaryExpr::Op::PostInc ||
                            un->op == UnaryExpr::Op::PreDec || un->op == UnaryExpr::Op::PostDec) {
                            hasIndexMove = true;
                        }
                    }
                }
                break;
            }
            default: break;
        }
    };

    if (partitionLoop) {
        if (partitionLoop->kind == StmtKind::While) {
            const auto& w = static_cast<const WhileStmt&>(*partitionLoop);
            if (w.body) scan(*w.body);
        } else if (partitionLoop->kind == StmtKind::For) {
            const auto& f = static_cast<const ForStmt&>(*partitionLoop);
            if (f.body) scan(*f.body);
        }
    }

    // Scan whole body for swap (may be outside the inner partition loop)
    VisitStmt(*func.body, [&](const Stmt& stmt) {
        if (!arrName.empty() && IsSwapPattern(stmt, arrName)) {
            hasSwap = true;
            if (swapLine == 0) {
                if (stmt.kind == StmtKind::Block) {
                    const auto& block = static_cast<const BlockStmt&>(stmt);
                    for (const auto& s : block.stmts) {
                        if (s->loc.line > 0) { swapLine = s->loc.line; break; }
                    }
                } else {
                    swapLine = stmt.loc.line;
                }
            }
        }
    });

    if (!hasArrayCompare && !hasSwap) return {};
    if (!hasSwap) return {};

    int confidence = 75;
    if (partitionLoop) confidence += 10;
    if (hasIndexMove) confidence += 10;

    AlgorithmMatch match;
    match.funcName = func.name;
    match.algorithmName = "quick_sort";
    match.displayName = "快速排序";
    match.confidence = std::min(confidence, 100);
    match.suggestion = "检测到快速排序模式。选取基准元素进行分区，再递归排序子区间。平均时间复杂度 O(n log n)。";
    match.line = partitionLoop ? partitionLoop->loc.line : func.loc.line;
    if (compareLine > 0) match.visEvents.push_back({compareLine, 1, "j"}); // Compare
    if (swapLine > 0) match.visEvents.push_back({swapLine, 2, "i:j"});       // Swap
    return match;
}

// ============================================================================
// Merge sort detection
// Pattern: two recursive calls + temporary storage + merge loop + copy back
// ============================================================================

AlgorithmMatch AlgorithmMatcher::DetectMergeSort(const FuncDecl& func) {
    if (!func.body) return {};

    // Step 1: Must have at least 2 recursive calls
    int recursiveCallCount = 0;
    VisitStmt(*func.body, [&](const Stmt& stmt) {
        if (stmt.kind == StmtKind::Expr) {
            const auto& exprStmt = static_cast<const ExprStmt&>(stmt);
            if (exprStmt.expr && exprStmt.expr->kind == ExprKind::Call) {
                const auto* call = static_cast<const CallExpr*>(exprStmt.expr.get());
                if (call->name == func.name) {
                    recursiveCallCount++;
                }
            }
        }
    });
    if (recursiveCallCount < 2) return {};

    // Step 2: Look for temporary storage (array or pointer local)
    bool hasTempStorage = false;
    VisitStmt(*func.body, [&](const Stmt& stmt) {
        if (stmt.kind == StmtKind::VarDecl) {
            const auto& decl = static_cast<const VarDeclStmt&>(stmt);
            if (decl.varType.isArray() || decl.varType.isPointer()) {
                hasTempStorage = true;
            }
        }
    });

    // Step 3: Look for array compare, merge assignment, copy back
    bool hasArrayCompare = false;
    bool hasMergeAssign = false;
    bool hasCopyBack = false;
    std::string arrName;
    int compareLine = 0;
    int updateLine = 0;

    auto getArrName = [&](const Expr* expr) -> std::string {
        if (!expr || expr->kind != ExprKind::Index) return "";
        const auto* idx = static_cast<const IndexExpr*>(expr);
        if (idx->array && idx->array->kind == ExprKind::Identifier) {
            return static_cast<const IdentifierExpr*>(idx->array.get())->name;
        }
        return "";
    };

    std::function<void(const Stmt&)> scan;
    scan = [&](const Stmt& stmt) {
        switch (stmt.kind) {
            case StmtKind::Block: {
                const auto& block = static_cast<const BlockStmt&>(stmt);
                for (const auto& s : block.stmts) scan(*s);
                break;
            }
            case StmtKind::If: {
                const auto& ifs = static_cast<const IfStmt&>(stmt);
                if (ifs.cond && ifs.cond->kind == ExprKind::Binary) {
                    const auto* bin = static_cast<const BinaryExpr*>(ifs.cond.get());
                    std::string leftArr = getArrName(bin->left.get());
                    std::string rightArr = getArrName(bin->right.get());
                    if (!leftArr.empty() || !rightArr.empty()) {
                        hasArrayCompare = true;
                        arrName = !leftArr.empty() ? leftArr : rightArr;
                        if (compareLine == 0) compareLine = ifs.loc.line;
                    }
                }
                if (ifs.thenStmt) scan(*ifs.thenStmt);
                if (ifs.elseStmt) scan(*ifs.elseStmt);
                break;
            }
            case StmtKind::While:
            case StmtKind::For: {
                const Stmt* body = nullptr;
                if (stmt.kind == StmtKind::While) {
                    body = static_cast<const WhileStmt&>(stmt).body.get();
                } else {
                    body = static_cast<const ForStmt&>(stmt).body.get();
                }
                if (body) scan(*body);
                break;
            }
            case StmtKind::Expr: {
                const auto& exprStmt = static_cast<const ExprStmt&>(stmt);
                if (exprStmt.expr && exprStmt.expr->kind == ExprKind::Assign) {
                    const auto* assign = static_cast<const AssignExpr*>(exprStmt.expr.get());
                    if (assign->left && assign->left->kind == ExprKind::Index &&
                        assign->right && assign->right->kind == ExprKind::Index) {
                        std::string leftArr = getArrName(assign->left.get());
                        std::string rightArr = getArrName(assign->right.get());
                        if (!leftArr.empty() && !rightArr.empty()) {
                            if (leftArr != rightArr) {
                                hasCopyBack = true;
                            } else {
                                hasMergeAssign = true;
                            }
                            if (updateLine == 0) updateLine = stmt.loc.line;
                        }
                    }
                }
                break;
            }
            default: break;
        }
    };

    scan(*func.body);

    if (!hasArrayCompare) return {};
    if (!hasMergeAssign && !hasCopyBack) return {};

    int confidence = 70;
    if (recursiveCallCount >= 2) confidence += 10;
    if (hasTempStorage) confidence += 10;
    if (hasCopyBack) confidence += 10;

    AlgorithmMatch match;
    match.funcName = func.name;
    match.algorithmName = "merge_sort";
    match.displayName = "归并排序";
    match.confidence = std::min(confidence, 100);
    match.suggestion = "检测到归并排序模式。将数组不断二分后递归排序，再合并有序子数组。时间复杂度 O(n log n)。";
    match.line = func.loc.line;
    if (compareLine > 0) match.visEvents.push_back({compareLine, 1, "i:j"}); // Compare
    if (updateLine > 0) match.visEvents.push_back({updateLine, 3, "k:i"});   // Update (merge)
    return match;
}

// ============================================================================
// Helpers
// ============================================================================

void AlgorithmMatcher::VisitStmt(const Stmt& stmt, std::function<void(const Stmt&)> visitor) {
    visitor(stmt);
    switch (stmt.kind) {
        case StmtKind::Block: {
            const auto& block = static_cast<const BlockStmt&>(stmt);
            for (const auto& s : block.stmts) {
                VisitStmt(*s, visitor);
            }
            break;
        }
        case StmtKind::If: {
            const auto& ifStmt = static_cast<const IfStmt&>(stmt);
            if (ifStmt.thenStmt) VisitStmt(*ifStmt.thenStmt, visitor);
            if (ifStmt.elseStmt) VisitStmt(*ifStmt.elseStmt, visitor);
            break;
        }
        case StmtKind::For: {
            const auto& forStmt = static_cast<const ForStmt&>(stmt);
            if (forStmt.init && forStmt.init->kind == StmtKind::VarDecl) {
                // init is a statement in this implementation
            }
            if (forStmt.body) VisitStmt(*forStmt.body, visitor);
            break;
        }
        case StmtKind::While: {
            const auto& whileStmt = static_cast<const WhileStmt&>(stmt);
            if (whileStmt.body) VisitStmt(*whileStmt.body, visitor);
            break;
        }
        case StmtKind::DoWhile: {
            const auto& doWhile = static_cast<const DoWhileStmt&>(stmt);
            if (doWhile.body) VisitStmt(*doWhile.body, visitor);
            break;
        }
        case StmtKind::VarDecl: {
            const auto& decl = static_cast<const VarDeclStmt&>(stmt);
            if (decl.init) VisitExpr(*decl.init, visitor);
            break;
        }
        case StmtKind::Expr: {
            const auto& exprStmt = static_cast<const ExprStmt&>(stmt);
            if (exprStmt.expr) VisitExpr(*exprStmt.expr, visitor);
            break;
        }
        case StmtKind::Return: {
            const auto& ret = static_cast<const ReturnStmt&>(stmt);
            if (ret.value) VisitExpr(*ret.value, visitor);
            break;
        }
        case StmtKind::Switch: {
            const auto& sw = static_cast<const SwitchStmt&>(stmt);
            if (sw.body) VisitStmt(*sw.body, visitor);
            break;
        }
        case StmtKind::Case: {
            const auto& c = static_cast<const CaseStmt&>(stmt);
            if (c.stmt) VisitStmt(*c.stmt, visitor);
            break;
        }
        default:
            break;
    }
}

void AlgorithmMatcher::VisitExpr(const Expr& expr, std::function<void(const Stmt&)> visitor) {
    (void)expr;
    (void)visitor;
    // Expression visitor used to find statements nested in expressions
    // (not needed for current pattern matching)
}

std::string AlgorithmMatcher::ExtractArrayName(const Expr* expr) {
    if (!expr) return "";
    if (expr->kind == ExprKind::Index) {
        const auto& idx = static_cast<const IndexExpr&>(*expr);
        if (idx.array && idx.array->kind == ExprKind::Identifier) {
            return static_cast<const IdentifierExpr&>(*idx.array).name;
        }
    }
    return "";
}

bool AlgorithmMatcher::IsSwapPattern(const Stmt& stmt, const std::string& arrName) {
    // Swap pattern: tmp = arr[i]; arr[i] = arr[j]; arr[j] = tmp;
    // Or: arr[i] = arr[j]; arr[j] = tmp; (with tmp declared before)
    // We check for at least 2 assignments involving the same array
    if (stmt.kind != StmtKind::Block) {
        // Single statement: check if it's an assignment involving arr
        if (stmt.kind == StmtKind::Expr) {
            const auto& exprStmt = static_cast<const ExprStmt&>(stmt);
            if (exprStmt.expr && exprStmt.expr->kind == ExprKind::Assign) {
                const auto& assign = static_cast<const AssignExpr&>(*exprStmt.expr);
                if (assign.left && assign.left->kind == ExprKind::Index) {
                    return ExtractArrayName(assign.left.get()) == arrName;
                }
            }
        }
        return false;
    }

    const auto& block = static_cast<const BlockStmt&>(stmt);
    int assignCount = 0;
    for (const auto& s : block.stmts) {
        if (s->kind == StmtKind::Expr) {
            const auto& exprStmt = static_cast<const ExprStmt&>(*s);
            if (exprStmt.expr && exprStmt.expr->kind == ExprKind::Assign) {
                const auto& assign = static_cast<const AssignExpr&>(*exprStmt.expr);
                if (assign.left && assign.left->kind == ExprKind::Index) {
                    if (ExtractArrayName(assign.left.get()) == arrName) {
                        assignCount++;
                    }
                }
            }
        }
    }
    // Swap requires at least 2 assignments to the same array
    return assignCount >= 2;
}

} // namespace cide
