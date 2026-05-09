#pragma once

#include "Ast.hpp"
#include "Lexer.hpp"
#include <memory>
#include <string>
#include <vector>
#include <unordered_map>

namespace cide {

struct ParseError {
    std::string message;
    int line;
    int column;
    int code = 0;
};

class Parser {
public:
    explicit Parser(std::vector<Token> tokens);

    std::unique_ptr<ProgramNode> Parse();
    const std::vector<ParseError>& Errors() const { return errors_; }
    bool HasErrors() const { return !errors_.empty(); }

private:
    std::vector<Token> tokens_;
    std::vector<ParseError> errors_;
    std::unordered_map<std::string, Type> typedefNames_;
    size_t pos_ = 0;

    // Token helpers
    const Token& Peek(size_t offset = 0) const;
    const Token& Current() const { return Peek(); }
    const Token& Previous() const;
    bool Check(TokenType type) const;
    bool IsAtEnd() const;
    const Token& Advance();
    bool Match(TokenType type);
    const Token& Consume(TokenType type, const std::string& msg);
    void Synchronize();

    bool IsTypeToken() const;

    // Program
    std::unique_ptr<ProgramNode> ParseProgram();
    StructDecl ParseStructDecl();
    FuncDecl ParseFuncDecl();
    void ParseTypedef();
    void ParseEnumDecl(ProgramNode* program);

    // Type parsing
    Type ParseBaseType();
    std::pair<Type, std::string> ParseTypeAndName();
    Type ParseTypeOnly();

    // Statements
    StmtPtr ParseStatement();
    StmtPtr ParseBlock();
    StmtPtr ParseVarDeclStmt();
    StmtPtr ParseIfStmt();
    StmtPtr ParseWhileStmt();
    StmtPtr ParseDoWhileStmt();
    StmtPtr ParseForStmt();
    StmtPtr ParseReturnStmt();
    StmtPtr ParseBreakStmt();
    StmtPtr ParseContinueStmt();
    StmtPtr ParseSwitchStmt();
    StmtPtr ParseCaseStmt();
    StmtPtr ParseExprStmt();

    // Expressions (precedence climbing)
    ExprPtr ParseExpression();
    ExprPtr ParseAssign();
    ExprPtr ParseOr();
    ExprPtr ParseAnd();
    ExprPtr ParseEquality();
    ExprPtr ParseRelational();
    ExprPtr ParseAdditive();
    ExprPtr ParseMultiplicative();
    ExprPtr ParseUnary();
    ExprPtr ParseSizeof();
    ExprPtr ParsePostfix();
    ExprPtr ParsePrimary();
    ExprPtr ParseInitList();

    // Helpers
    ExprPtr ParseCallExpr(const std::string& name);
    std::vector<ExprPtr> ParseArgList();
    std::vector<Param> ParseParamList();
};

} // namespace cide
