#pragma once

#include "Ast.hpp"
#include "../vm/Instruction.hpp"
#include "../vm/CideVM.hpp"

#include <string>
#include <vector>
#include <unordered_map>
#include <unordered_set>

namespace cide {

// ============================================================================
// BytecodeGen: AST -> Custom Bytecode (CideVM instruction set)
// ============================================================================

// Function metadata (used by both BytecodeGen and CideVM)
struct FuncMeta {
    size_t ip = 0;
    int argCount = 0;
    int localCount = 0;
    Type returnType;
};

class BytecodeGen : public AstVisitor {
public:
    BytecodeGen();

    bool Generate(ProgramNode& program);
    std::vector<Instruction> TakeCode();
    std::vector<int32_t> TakeGlobalsInit();
    const std::vector<std::string>& Errors() const { return errors_; }
    bool HasErrors() const { return !errors_.empty(); }

    const std::unordered_map<std::string, FuncMeta>& GetFuncTable() const { return funcTable_; }
    const std::unordered_map<std::string, int>& GetFuncIndex() const { return funcIndex_; }
    const std::vector<std::pair<uint32_t, std::string>>& GetStringData() const { return stringData_; }
    const std::vector<std::pair<uint32_t, SourceLoc>>& GetSourceMap() const { return sourceMap_; }
    std::vector<cide::VMSymbol> TakeSymbols();
    const std::unordered_map<std::string, std::vector<StructField>>& GetStructDefs() const { return structDefs_; }

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
    std::vector<Instruction> code_;
    std::vector<std::string> errors_;

    std::unordered_map<std::string, FuncMeta> funcTable_;
    std::unordered_map<std::string, int> funcIndex_; // name -> func idx
    int nextFuncIdx_ = 0;
    std::string currentFunc_;
    int currentFuncArgCount_ = 0;

    // Symbol tables
    std::unordered_map<std::string, int> globalIndices_; // name -> global idx
    std::unordered_map<std::string, Type> globalTypes_;
    std::unordered_map<std::string, int> localIndices_;  // name -> local idx (relative to frame)
    std::unordered_map<std::string, Type> localTypes_;
    int nextLocalIdx_ = 0;

    // Fixed temp slots (max 3 concurrently needed in current subset)
    int tempSlot0_ = -1, tempSlot1_ = -1, tempSlot2_ = -1;

    // Globals init values (for VM SetGlobals)
    std::vector<int32_t> globalsInit_;
    int nextGlobalIdx_ = 0;

    // Symbol table for runtime diagnostics
    std::vector<cide::VMSymbol> symbols_;
    std::unordered_map<std::string, int> symIndex_; // name -> index in symbols_

    // Struct definitions for member offset lookup
    std::unordered_map<std::string, std::vector<StructField>> structDefs_;

    // String literals -> linear memory
    std::vector<std::pair<uint32_t, std::string>> stringData_;
    uint32_t stringMemOffset_ = 0x1000;

    // Source map: bytecode IP -> source location
    std::vector<std::pair<uint32_t, SourceLoc>> sourceMap_;

    // Break/continue patch lists (ip locations needing back-patching)
    std::vector<size_t> breakPatches_;
    std::vector<size_t> continuePatches_;
    std::vector<size_t> loopStartIPs_;   // start label for current loop

    // Helpers
    void Emit(OpCode op, int32_t operand = 0, const SourceLoc& loc = {});
    size_t CurrentIP() const { return code_.size(); }
    void PatchJump(size_t ip, size_t target);

    void EnterFunction(const std::string& name, const std::vector<Param>& params);
    void ExitFunction();

    int ResolveLocal(const std::string& name);
    int ResolveGlobal(const std::string& name);
    int ResolveSymbolIndex(const std::string& name);

    void ReportError(const std::string& msg, const SourceLoc& loc);

    int GetMemberOffset(const Type& objectType, const std::string& memberName);

    // Lazy temp slot allocation (max 3 concurrent slots needed)
    int GetTempSlot(int index);

    // Dispatch helpers
    void GenStmt(Stmt& stmt);
    void GenExpr(Expr& expr);
};

} // namespace cide
