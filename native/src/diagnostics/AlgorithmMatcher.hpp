#pragma once

#include <functional>
#include <string>
#include <vector>

namespace cide {

struct ProgramNode;
struct FuncDecl;
struct ForStmt;
struct IfStmt;
struct WhileStmt;
struct Stmt;
struct Expr;

struct AlgorithmMatch {
    std::string algorithmName;      // e.g. "bubble_sort"
    std::string displayName;        // e.g. "冒泡排序"
    std::string funcName;           // function name in source code
    int confidence = 0;             // 0-100 match confidence
    std::string suggestion;         // diagnosis / hint
    int line = 0;                   // relevant source line
    // Runtime vis events: (line, type, context) type: 1=Compare, 2=Swap, 3=Update
    // context = index expressions separated by ':', e.g. "j:j+1" or "mid"
    std::vector<std::tuple<int, int, std::string>> visEvents;
};

/// Simple AST-based algorithm pattern matcher.
/// Detects common algorithms (bubble sort, selection sort, etc.) from student code.
class AlgorithmMatcher {
public:
    AlgorithmMatcher() = default;

    /// Analyze the whole program and return any detected algorithm patterns.
    std::vector<AlgorithmMatch> Analyze(const ProgramNode& program);

private:
    void VisitFunc(const FuncDecl& func);
    void VisitStmt(const Stmt& stmt, std::function<void(const Stmt&)> visitor);
    void VisitExpr(const Expr& expr, std::function<void(const Stmt&)> visitor);

    // Pattern detectors
    AlgorithmMatch DetectBubbleSort(const FuncDecl& func);
    AlgorithmMatch DetectSelectionSort(const FuncDecl& func);
    AlgorithmMatch DetectInsertionSort(const FuncDecl& func);
    AlgorithmMatch DetectBinarySearch(const FuncDecl& func);
    AlgorithmMatch DetectLinkedListTraversal(const FuncDecl& func);
    AlgorithmMatch DetectLinkedListReverse(const FuncDecl& func);
    AlgorithmMatch DetectLinkedListInsert(const FuncDecl& func);
    AlgorithmMatch DetectLinkedListDelete(const FuncDecl& func);
    AlgorithmMatch DetectQuickSort(const FuncDecl& func);
    AlgorithmMatch DetectMergeSort(const FuncDecl& func);

    // Helpers
    std::string ExtractArrayName(const Expr* expr);
    bool IsSwapPattern(const Stmt& stmt, const std::string& arrName);

    std::vector<std::string> linkedListNodeTypes_;
    std::vector<AlgorithmMatch> matches_;
};

} // namespace cide
