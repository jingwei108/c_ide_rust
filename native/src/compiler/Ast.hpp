#pragma once

#include <memory>
#include <string>
#include <vector>
#include <optional>

namespace cide {

// ============================================================================
// Source Location
// ============================================================================

struct SourceLoc {
    int line = 1;
    int column = 1;
};

// ============================================================================
// Type System
// ============================================================================

enum class TypeKind {
    Void,
    Int,
    Char,
    Pointer,
    Array,
    Struct,
};

struct Type {
    TypeKind kind = TypeKind::Void;
    std::string name{}; // struct name for Struct/Pointer, empty for int/char
    int arraySize = 0;  // for Array: total elements (back-compat), or -1 = unspecified
    TypeKind baseKind = TypeKind::Void;  // innermost element type for Pointer/Array
    std::vector<int> dims;  // multi-dimensional sizes; empty = scalar. e.g. [3][3] -> {3,3}

    bool isScalar() const { return kind == TypeKind::Int || kind == TypeKind::Char; }
    bool isPointer() const { return kind == TypeKind::Pointer; }
    bool isArray() const { return kind == TypeKind::Array; }
    bool isStruct() const { return kind == TypeKind::Struct; }
    bool isVoid() const { return kind == TypeKind::Void; }

    int totalElements() const {
        if (kind != TypeKind::Array) return 1;
        if (!dims.empty()) {
            int total = 1;
            for (int d : dims) { total *= (d > 0 ? d : 1); }
            return total;
        }
        return arraySize > 0 ? arraySize : 1;
    }

    // Returns the element type after one subscript.
    // e.g. int[3][3] -> subscript -> int[3]; int[3] -> subscript -> int
    Type subscriptType() const {
        if (!isArray()) return *this;
        if (dims.size() <= 1) {
            return Type{baseKind, name, 0, TypeKind::Void};
        }
        Type t = *this;
        t.dims.erase(t.dims.begin());
        t.arraySize = t.totalElements();
        return t;
    }

    bool operator==(const Type& other) const {
        return kind == other.kind && name == other.name &&
               arraySize == other.arraySize && baseKind == other.baseKind &&
               dims == other.dims;
    }
    bool operator!=(const Type& other) const { return !(*this == other); }

    std::string ToString() const {
        switch (kind) {
            case TypeKind::Void: return "void";
            case TypeKind::Int: return "int";
            case TypeKind::Char: return "char";
            case TypeKind::Pointer: {
                std::string base = (baseKind == TypeKind::Struct) ? ("struct " + name) :
                                   (baseKind == TypeKind::Char) ? "char" : "int";
                return base + "*";
            }
            case TypeKind::Array: {
                std::string base = (baseKind == TypeKind::Struct) ? ("struct " + name) :
                                   (baseKind == TypeKind::Char) ? "char" : "int";
                if (!dims.empty()) {
                    for (int d : dims) {
                        base += "[" + std::to_string(d) + "]";
                    }
                    return base;
                }
                if (arraySize > 0) return base + "[" + std::to_string(arraySize) + "]";
                return base + "[]";
            }
            case TypeKind::Struct: return "struct " + name;
        }
        return "unknown";
    }
};

// ============================================================================
// Forward Declarations
// ============================================================================

struct Expr;
struct Stmt;
struct Decl;
struct ProgramNode;

// ============================================================================
// Expression Nodes
// ============================================================================

enum class ExprKind {
    Binary,
    Unary,
    Literal,
    StringLiteral,
    Identifier,
    Call,
    Index,
    Member,
    Assign,
    Sizeof,      // Phase 2 extension
    InitList,
};

struct Expr {
    ExprKind kind;
    SourceLoc loc;
    Type type;   // computed by TypeChecker

    explicit Expr(ExprKind k) : kind(k) {}
    virtual ~Expr() = default;
};

using ExprPtr = std::unique_ptr<Expr>;

// Binary: + - * / % == != < <= > >= && ||
struct BinaryExpr : Expr {
    enum class Op {
        Add, Sub, Mul, Div, Mod,
        Eq, Ne, Lt, Le, Gt, Ge,
        And, Or
    };
    Op op;
    ExprPtr left;
    ExprPtr right;

    BinaryExpr(Op o, ExprPtr l, ExprPtr r)
        : Expr(ExprKind::Binary), op(o), left(std::move(l)), right(std::move(r)) {}
};

// Unary: - ! & * ++ -- (prefix/postfix)
struct UnaryExpr : Expr {
    enum class Op {
        Neg, Not, Addr, Deref, PreInc, PreDec, PostInc, PostDec
    };
    Op op;
    ExprPtr operand;

    UnaryExpr(Op o, ExprPtr e)
        : Expr(ExprKind::Unary), op(o), operand(std::move(e)) {}
};

// Literal: integer constant
struct LiteralExpr : Expr {
    int32_t value;

    explicit LiteralExpr(int32_t v)
        : Expr(ExprKind::Literal), value(v) {
        type = Type{TypeKind::Int};
    }
};

// String literal: "hello"
struct StringLiteralExpr : Expr {
    std::string value;

    explicit StringLiteralExpr(std::string v)
        : Expr(ExprKind::StringLiteral), value(std::move(v)) {
        type = Type{TypeKind::Pointer, "char", 0, TypeKind::Char};
    }
};

// Identifier: variable name
struct IdentifierExpr : Expr {
    std::string name;

    explicit IdentifierExpr(std::string n)
        : Expr(ExprKind::Identifier), name(std::move(n)) {}
};

// Function call: foo(a, b)
struct CallExpr : Expr {
    std::string name;
    std::vector<ExprPtr> args;

    CallExpr(std::string n, std::vector<ExprPtr> a)
        : Expr(ExprKind::Call), name(std::move(n)), args(std::move(a)) {}
};

// Array index: arr[i]
struct IndexExpr : Expr {
    ExprPtr array;
    ExprPtr index;

    IndexExpr(ExprPtr a, ExprPtr i)
        : Expr(ExprKind::Index), array(std::move(a)), index(std::move(i)) {}
};

// Member access: node.val, node->val (simplified to same)
struct MemberExpr : Expr {
    ExprPtr object;
    std::string member;

    MemberExpr(ExprPtr o, std::string m)
        : Expr(ExprKind::Member), object(std::move(o)), member(std::move(m)) {}
};

// Assignment: a = b, a += b, etc.
struct AssignExpr : Expr {
    enum class Op {
        Assign, AddAssign, SubAssign, MulAssign, DivAssign, ModAssign
    };
    Op op;
    ExprPtr left;
    ExprPtr right;

    AssignExpr(Op o, ExprPtr l, ExprPtr r)
        : Expr(ExprKind::Assign), op(o), left(std::move(l)), right(std::move(r)) {}
};

// Sizeof: sizeof(type) or sizeof expr
struct SizeofExpr : Expr {
    Type targetType;   // valid if isTypeQuery
    ExprPtr operand;   // valid if !isTypeQuery
    bool isTypeQuery;

    explicit SizeofExpr(Type t)
        : Expr(ExprKind::Sizeof), targetType(std::move(t)), isTypeQuery(true) {
        type = Type{TypeKind::Int};
    }
    explicit SizeofExpr(ExprPtr e)
        : Expr(ExprKind::Sizeof), operand(std::move(e)), isTypeQuery(false) {
        type = Type{TypeKind::Int};
    }
};

// InitList: { expr1, expr2, ... }
struct InitListExpr : Expr {
    std::vector<ExprPtr> elements;

    explicit InitListExpr(std::vector<ExprPtr> elems)
        : Expr(ExprKind::InitList), elements(std::move(elems)) {}
};

// ============================================================================
// Statement Nodes
// ============================================================================

enum class StmtKind {
    Block,
    VarDecl,
    Expr,
    If,
    While,
    DoWhile,
    For,
    Return,
    Break,
    Continue,
    Switch,
    Case,
};

struct Stmt {
    StmtKind kind;
    SourceLoc loc;

    explicit Stmt(StmtKind k) : kind(k) {}
    virtual ~Stmt() = default;
};

using StmtPtr = std::unique_ptr<Stmt>;

// Block: { stmt... }
struct BlockStmt : Stmt {
    std::vector<StmtPtr> stmts;

    BlockStmt() : Stmt(StmtKind::Block) {}
};

// Variable declaration: int a = 5;
struct VarDeclStmt : Stmt {
    Type varType;
    std::string name;
    ExprPtr init;  // nullptr if no initializer
    // Multi-variable declaration: int a = 1, b = 2, c;
    std::vector<std::pair<std::string, ExprPtr>> extraVars;

    VarDeclStmt(Type t, std::string n, ExprPtr i)
        : Stmt(StmtKind::VarDecl), varType(std::move(t)), name(std::move(n)), init(std::move(i)) {}
};

// Expression statement: expr;
struct ExprStmt : Stmt {
    ExprPtr expr;

    explicit ExprStmt(ExprPtr e)
        : Stmt(StmtKind::Expr), expr(std::move(e)) {}
};

// If: if (cond) thenStmt else elseStmt
struct IfStmt : Stmt {
    ExprPtr cond;
    StmtPtr thenStmt;
    StmtPtr elseStmt;  // nullptr if no else

    IfStmt(ExprPtr c, StmtPtr t, StmtPtr e)
        : Stmt(StmtKind::If), cond(std::move(c)), thenStmt(std::move(t)), elseStmt(std::move(e)) {}
};

// While: while (cond) body
struct WhileStmt : Stmt {
    ExprPtr cond;
    StmtPtr body;

    WhileStmt(ExprPtr c, StmtPtr b)
        : Stmt(StmtKind::While), cond(std::move(c)), body(std::move(b)) {}
};

struct DoWhileStmt : Stmt {
    StmtPtr body;
    ExprPtr cond;

    DoWhileStmt(StmtPtr b, ExprPtr c)
        : Stmt(StmtKind::DoWhile), body(std::move(b)), cond(std::move(c)) {}
};

struct BreakStmt : Stmt {
    BreakStmt() : Stmt(StmtKind::Break) {}
};

struct ContinueStmt : Stmt {
    ContinueStmt() : Stmt(StmtKind::Continue) {}
};

// Switch: switch (cond) body
struct SwitchStmt : Stmt {
    ExprPtr cond;
    StmtPtr body;

    SwitchStmt(ExprPtr c, StmtPtr b)
        : Stmt(StmtKind::Switch), cond(std::move(c)), body(std::move(b)) {}
};

// Case: case expr: stmt  or  default: stmt
struct CaseStmt : Stmt {
    ExprPtr label;  // nullptr for default
    StmtPtr stmt;

    CaseStmt(ExprPtr l, StmtPtr s)
        : Stmt(StmtKind::Case), label(std::move(l)), stmt(std::move(s)) {}
};

// For: for (init; cond; step) body
struct ForStmt : Stmt {
    StmtPtr init;      // VarDecl or ExprStmt or nullptr
    ExprPtr cond;      // nullptr means true
    ExprPtr step;      // nullptr means no step
    StmtPtr body;

    ForStmt(StmtPtr i, ExprPtr c, ExprPtr s, StmtPtr b)
        : Stmt(StmtKind::For), init(std::move(i)), cond(std::move(c)),
          step(std::move(s)), body(std::move(b)) {}
};

// Return: return expr;
struct ReturnStmt : Stmt {
    ExprPtr value;  // nullptr for "return;"

    explicit ReturnStmt(ExprPtr v)
        : Stmt(StmtKind::Return), value(std::move(v)) {}
};

// ============================================================================
// Declaration Nodes
// ============================================================================

// Function parameter: int a, int arr[], int* p, struct Node* n
struct Param {
    Type type;
    std::string name;
    SourceLoc loc;
};

// Function declaration: int add(int a, int b) { ... }
struct FuncDecl {
    SourceLoc loc;
    Type returnType;
    std::string name;
    std::vector<Param> params;
    StmtPtr body;  // BlockStmt
};

// Struct field: int val; struct Node* next;
struct StructField {
    Type type;
    std::string name;
};

// Struct declaration: struct Node { int val; struct Node* next; };
struct StructDecl {
    SourceLoc loc;
    std::string name;
    std::vector<StructField> fields;
};

// ============================================================================
// Global Variable Declaration
// ============================================================================

struct GlobalDecl {
    SourceLoc loc;
    Type type;
    std::string name;
    ExprPtr init;
};

// ============================================================================
// Program Root
// ============================================================================

struct ProgramNode {
    std::vector<StructDecl> structs;
    std::vector<GlobalDecl> globals;
    std::vector<FuncDecl> funcs;
};

// ============================================================================
// AST Visitor (for CodeGen / TypeChecker)
// ============================================================================

class AstVisitor {
public:
    virtual ~AstVisitor() = default;

    virtual void VisitProgram(ProgramNode& node) = 0;
    virtual void VisitFuncDecl(FuncDecl& node) = 0;

    virtual void VisitBlock(BlockStmt& node) = 0;
    virtual void VisitVarDecl(VarDeclStmt& node) = 0;
    virtual void VisitExprStmt(ExprStmt& node) = 0;
    virtual void VisitIf(IfStmt& node) = 0;
    virtual void VisitWhile(WhileStmt& node) = 0;
    virtual void VisitDoWhile(DoWhileStmt& node) = 0;
    virtual void VisitFor(ForStmt& node) = 0;
    virtual void VisitReturn(ReturnStmt& node) = 0;
    virtual void VisitBreak(BreakStmt& node) = 0;
    virtual void VisitContinue(ContinueStmt& node) = 0;
    virtual void VisitSwitch(SwitchStmt& node) = 0;
    virtual void VisitCase(CaseStmt& node) = 0;

    virtual void VisitBinary(BinaryExpr& node) = 0;
    virtual void VisitUnary(UnaryExpr& node) = 0;
    virtual void VisitLiteral(LiteralExpr& node) = 0;
    virtual void VisitStringLiteral(StringLiteralExpr& node) = 0;
    virtual void VisitIdentifier(IdentifierExpr& node) = 0;
    virtual void VisitCall(CallExpr& node) = 0;
    virtual void VisitIndex(IndexExpr& node) = 0;
    virtual void VisitMember(MemberExpr& node) = 0;
    virtual void VisitAssign(AssignExpr& node) = 0;
    virtual void VisitSizeof(SizeofExpr& node) = 0;
    virtual void VisitInitList(InitListExpr& node) = 0;
};

} // namespace cide
