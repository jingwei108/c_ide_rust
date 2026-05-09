#pragma once

#include "Ast.hpp"
#include "diagnostics/ErrorCodes.hpp"
#include <string>
#include <vector>
#include <unordered_map>

namespace cide {

struct TypeError {
    std::string message;
    int line;
    int column;
    int code = 0;
};

struct VarSymbol {
    Type type;
    bool isGlobal = false;
};

struct FuncSymbol {
    Type returnType;
    std::vector<Type> paramTypes;
};

struct StructSymbol {
    std::vector<std::pair<Type, std::string>> fields;
};

class TypeChecker : public AstVisitor {
public:
    bool Check(ProgramNode& program);
    const std::vector<TypeError>& Errors() const { return errors_; }
    const std::vector<TypeError>& Warnings() const { return warnings_; }
    bool HasErrors() const { return !errors_.empty(); }
    bool HasWarnings() const { return !warnings_.empty(); }

    // Visitor interface
    void VisitProgram(ProgramNode& node) override;
    void VisitFuncDecl(FuncDecl& node) override;
    void VisitBlock(BlockStmt& node) override;
    void VisitVarDecl(VarDeclStmt& node) override;
    void VisitExprStmt(ExprStmt& node) override;
    void VisitIf(IfStmt& node) override;
    void VisitWhile(WhileStmt& node) override;
    void VisitDoWhile(DoWhileStmt& node) override;
    void VisitFor(ForStmt& node) override;
    void VisitReturn(ReturnStmt& node) override;
    void VisitBreak(BreakStmt& node) override;
    void VisitContinue(ContinueStmt& node) override;
    void VisitSwitch(SwitchStmt& node) override;
    void VisitCase(CaseStmt& node) override;
    void VisitBinary(BinaryExpr& node) override;
    void VisitUnary(UnaryExpr& node) override;
    void VisitLiteral(LiteralExpr& node) override;
    void VisitStringLiteral(StringLiteralExpr& node) override;
    void VisitIdentifier(IdentifierExpr& node) override;
    void VisitCall(CallExpr& node) override;
    void VisitIndex(IndexExpr& node) override;
    void VisitMember(MemberExpr& node) override;
    void VisitAssign(AssignExpr& node) override;
    void VisitSizeof(SizeofExpr& node) override;
    void VisitInitList(InitListExpr& node) override;

private:
    std::vector<TypeError> errors_;
    std::vector<TypeError> warnings_;
    bool hasErrors_ = false;

    // Symbol tables
    std::unordered_map<std::string, FuncSymbol> funcs_;
    std::unordered_map<std::string, StructSymbol> structs_;
    std::vector<std::unordered_map<std::string, VarSymbol>> scopes_;

    Type currentFuncReturn_;
    int loopDepth_ = 0;
    int switchDepth_ = 0;

    // Scope management
    void EnterScope();
    void ExitScope();
    void DeclareVar(const std::string& name, const Type& type, bool isGlobal = false);
    std::optional<VarSymbol> LookupVar(const std::string& name);

    // Statement dispatch helper
    void DispatchStmt(Stmt& stmt);

    // Type operations
    void ReportError(const std::string& msg, const SourceLoc& loc, ErrorCode code = ErrorCode::Unknown);
    bool IsInt(const Type& t) const { return t.kind == TypeKind::Int || t.kind == TypeKind::Char; }
    bool IsComparable(const Type& a, const Type& b) const;
    void CheckArrayInitializer(Type& arrType, Expr& init, const SourceLoc& loc);
    void CheckStructInitializer(const Type& structType, Expr& init, const SourceLoc& loc);
    bool ValidateNestedInitList(const std::vector<int>& dims, Expr& init,
                                 const SourceLoc& loc, TypeKind baseKind,
                                 const std::string& structName);
    bool IsAssignable(const Type& target, const Type& value, const SourceLoc& loc);
    void ReportWarning(const std::string& msg, const SourceLoc& loc, ErrorCode code = ErrorCode::Unknown);
    Type ResolveExprType(Expr& expr);
    void CheckCondition(Expr& cond, const std::string& ctx);

    // Struct helpers
    std::optional<Type> GetStructFieldType(const std::string& structName, const std::string& fieldName);
};

} // namespace cide
