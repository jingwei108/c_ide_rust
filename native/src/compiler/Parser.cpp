#include "Parser.hpp"
#include "diagnostics/ErrorCodes.hpp"
#include <cstdlib>
namespace cide {

// ============================================================================
// Constructor
// ============================================================================

Parser::Parser(std::vector<Token> tokens) : tokens_(std::move(tokens)) {}

// ============================================================================
// Token Helpers
// ============================================================================

const Token& Parser::Peek(size_t offset) const {
    if (pos_ + offset >= tokens_.size()) {
        static const Token eof{TokenType::Eof, "", -1, -1};
        return eof;
    }
    return tokens_[pos_ + offset];
}

const Token& Parser::Previous() const {
    if (pos_ == 0) return Peek();
    return tokens_[pos_ - 1];
}

bool Parser::Check(TokenType type) const {
    if (IsAtEnd()) return false;
    return Current().type == type;
}

bool Parser::IsAtEnd() const {
    return Current().type == TokenType::Eof;
}

const Token& Parser::Advance() {
    if (!IsAtEnd()) pos_++;
    return tokens_[pos_ - 1];
}

bool Parser::Match(TokenType type) {
    if (Check(type)) {
        Advance();
        return true;
    }
    return false;
}

const Token& Parser::Consume(TokenType type, const std::string& msg) {
    if (Check(type)) return Advance();
    ErrorCode code = ErrorCode::E2005_ExpectedSemicolon;
    if (type == TokenType::RBrace) code = ErrorCode::E2006_ExpectedClosingBrace;
    else if (type == TokenType::RParen) code = ErrorCode::E2007_ExpectedClosingParen;
    else if (type == TokenType::RBracket) code = ErrorCode::E2008_ExpectedClosingBracket;
    errors_.push_back({msg, Current().line, Current().column, static_cast<int>(code)});
    return Peek(); // Return current (wrong) token
}

void Parser::Synchronize() {
    while (!IsAtEnd()) {
        if (Previous().type == TokenType::Semicolon) return;
        switch (Current().type) {
            case TokenType::Int:
            case TokenType::Void:
            case TokenType::Char:
            case TokenType::If:
            case TokenType::While:
            case TokenType::Do:
            case TokenType::For:
            case TokenType::Return:
            case TokenType::Break:
            case TokenType::Continue:
            case TokenType::Struct:
            case TokenType::Switch:
            case TokenType::Case:
            case TokenType::Default:
            case TokenType::Typedef:
            case TokenType::Enum:
            case TokenType::Unsigned:
            case TokenType::RBrace:
                return;
            default:
                Advance();
        }
    }
}

bool Parser::IsTypeToken() const {
    if (Check(TokenType::Int) || Check(TokenType::Void) ||
        Check(TokenType::Char) || Check(TokenType::Struct) ||
        Check(TokenType::Unsigned)) {
        return true;
    }
    if (Check(TokenType::Identifier)) {
        return typedefNames_.find(Current().text) != typedefNames_.end();
    }
    return false;
}

// ============================================================================
// Entry Point
// ============================================================================

std::unique_ptr<ProgramNode> Parser::Parse() {
    return ParseProgram();
}

// ============================================================================
// Program
// ============================================================================

std::unique_ptr<ProgramNode> Parser::ParseProgram() {
    auto program = std::make_unique<ProgramNode>();

    while (!IsAtEnd()) {
        if (Check(TokenType::Typedef)) {
            ParseTypedef();
        } else if (Check(TokenType::Enum)) {
            ParseEnumDecl(program.get());
        } else if (Check(TokenType::Struct)) {
            // Peek ahead to distinguish struct declaration vs function/global with struct return type
            auto checkpoint = pos_;
            Advance(); // consume 'struct'
            Consume(TokenType::Identifier, "预期结构体名称");
            bool isStructDecl = Check(TokenType::LBrace);
            pos_ = checkpoint;
            if (isStructDecl) {
                program->structs.push_back(ParseStructDecl());
            } else {
                // Function or global variable with struct return type
                auto [type, name] = ParseTypeAndName();
                auto nameTok = Previous();
                if (Check(TokenType::LParen)) {
                    pos_ = checkpoint;
                    program->funcs.push_back(ParseFuncDecl());
                } else {
                    ExprPtr init = nullptr;
                    if (Match(TokenType::Assign)) {
                        if (Check(TokenType::LBrace)) {
                            init = ParseInitList();
                        } else {
                            init = ParseExpression();
                        }
                    }
                    Consume(TokenType::Semicolon, "全局变量声明后预期 ';'");
                    program->globals.push_back({{nameTok.line, nameTok.column}, type, name, std::move(init)});
                }
            }
        } else if (IsTypeToken()) {
            // Peek ahead to distinguish function declaration vs global variable
            auto checkpoint = pos_;
            auto [type, name] = ParseTypeAndName();
            auto nameTok = Previous();  // name token consumed by ParseTypeAndName
            if (Check(TokenType::LParen)) {
                // Function declaration
                pos_ = checkpoint;
                program->funcs.push_back(ParseFuncDecl());
            } else {
                // Global variable declaration (supports multi-var: int a = 1, b = 2;)
                ExprPtr init = nullptr;
                if (Match(TokenType::Assign)) {
                    if (Check(TokenType::LBrace)) {
                        init = ParseInitList();
                    } else {
                        init = ParseExpression();
                    }
                }
                program->globals.push_back({{nameTok.line, nameTok.column}, type, name, std::move(init)});
                while (Match(TokenType::Comma)) {
                    auto extraNameTok = Consume(TokenType::Identifier, "预期标识符名称");
                    ExprPtr extraInit = nullptr;
                    if (Match(TokenType::Assign)) {
                        if (Check(TokenType::LBrace)) {
                            extraInit = ParseInitList();
                        } else {
                            extraInit = ParseExpression();
                        }
                    }
                    program->globals.push_back({{extraNameTok.line, extraNameTok.column}, type, extraNameTok.text, std::move(extraInit)});
                }
                Consume(TokenType::Semicolon, "全局变量声明后预期 ';'");
            }
        } else {
            errors_.push_back({
                "预期 struct、函数或全局变量声明，找到: " + Current().text,
                Current().line, Current().column,
                static_cast<int>(ErrorCode::E2005_ExpectedSemicolon)
            });
            Advance();
        }
    }

    return program;
}

StructDecl Parser::ParseStructDecl() {
    Consume(TokenType::Struct, "预期 'struct'");
    auto nameTok = Consume(TokenType::Identifier, "预期结构体名称");
    Consume(TokenType::LBrace, "预期 '{'");

    StructDecl decl;
    decl.loc = {nameTok.line, nameTok.column};
    decl.name = nameTok.text;

    while (!Check(TokenType::RBrace) && !IsAtEnd()) {
        auto fieldCheckpoint = pos_;
        auto [ftype, fname] = ParseTypeAndName();
        if (pos_ == fieldCheckpoint) {
            // Failed to parse a field, skip one token to avoid infinite loop
            Advance();
            break;
        }
        Consume(TokenType::Semicolon, "预期 ';'");
        decl.fields.push_back({ftype, fname});
    }

    Consume(TokenType::RBrace, "预期 '}'");
    Consume(TokenType::Semicolon, "结构体声明后预期 ';'");
    return decl;
}

FuncDecl Parser::ParseFuncDecl() {
    auto baseType = ParseBaseType();

    // Check for pointer return type
    Type retType = baseType;
    if (Match(TokenType::Star)) {
        retType = Type{TypeKind::Pointer, baseType.name, 0, baseType.kind};
    }

    auto nameTok = Consume(TokenType::Identifier, "预期函数名称");
    Consume(TokenType::LParen, "预期 '('");

    FuncDecl decl;
    decl.loc = {nameTok.line, nameTok.column};
    decl.returnType = retType;
    decl.name = nameTok.text;
    decl.params = ParseParamList();

    Consume(TokenType::RParen, "预期 ')'");
    decl.body = ParseBlock();

    return decl;
}

// ============================================================================
// Type Parsing
// ============================================================================

Type Parser::ParseBaseType() {
    if (Match(TokenType::Int)) {
        return Type{TypeKind::Int};
    }
    if (Match(TokenType::Unsigned)) {
        // Support "unsigned" and "unsigned int"
        Match(TokenType::Int);
        return Type{TypeKind::Int};
    }
    if (Match(TokenType::Void)) {
        return Type{TypeKind::Void};
    }
    if (Match(TokenType::Char)) {
        return Type{TypeKind::Char};
    }
    if (Match(TokenType::Struct)) {
        auto nameTok = Consume(TokenType::Identifier, "预期结构体名称");
        return Type{TypeKind::Struct, nameTok.text};
    }
    if (Check(TokenType::Identifier)) {
        auto it = typedefNames_.find(Current().text);
        if (it != typedefNames_.end()) {
            Advance();
            return it->second;
        }
    }
    errors_.push_back({"预期类型名称 (int, char, void, struct)", Current().line, Current().column, static_cast<int>(ErrorCode::E2001_ExpectedType)});
    return Type{TypeKind::Void};
}

std::pair<Type, std::string> Parser::ParseTypeAndName() {
    auto baseType = ParseBaseType();

    // Pointer before name: int* p, struct Node* n
    if (Match(TokenType::Star)) {
        auto nameTok = Consume(TokenType::Identifier, "预期标识符名称");
        return {Type{TypeKind::Pointer, baseType.name, 0, baseType.kind}, nameTok.text};
    }

    auto nameTok = Consume(TokenType::Identifier, "预期标识符名称");

    // Array after name: int arr[10], int arr[3][3], int arr[][3]
    std::vector<int> dims;
    while (Match(TokenType::LBracket)) {
        if (Check(TokenType::Number)) {
            auto sizeTok = Advance();
            int size = std::atoi(sizeTok.text.c_str());
            dims.push_back(size);
        } else if (Check(TokenType::RBracket)) {
            dims.push_back(-1); // unspecified dimension
        } else {
            errors_.push_back({"预期数组大小或 ']'", Current().line, Current().column, static_cast<int>(ErrorCode::E2002_ExpectedArraySize)});
        }
        Consume(TokenType::RBracket, "预期 ']'");
    }

    if (!dims.empty()) {
        int total = 1;
        for (int d : dims) { total *= (d > 0 ? d : 1); }
        Type arrType{TypeKind::Array, baseType.name, total, baseType.kind};
        arrType.dims = std::move(dims);
        return {arrType, nameTok.text};
    }

    return {baseType, nameTok.text};
}

std::vector<Param> Parser::ParseParamList() {
    std::vector<Param> params;

    if (Check(TokenType::RParen)) {
        return params; // Empty params: foo()
    }

    // Special case: void foo(void)
    if (Check(TokenType::Void) && Peek(1).type == TokenType::RParen) {
        Advance(); // consume void
        return params;
    }

    while (true) {
        auto [ptype, pname] = ParseTypeAndName();
        params.push_back({ptype, pname, {Current().line, Current().column}});

        if (!Match(TokenType::Comma)) break;
    }

    return params;
}

// ============================================================================
// Statements
// ============================================================================

StmtPtr Parser::ParseStatement() {
    if (Check(TokenType::LBrace)) {
        return ParseBlock();
    }
    if (Check(TokenType::If)) {
        return ParseIfStmt();
    }
    if (Check(TokenType::While)) {
        return ParseWhileStmt();
    }
    if (Check(TokenType::Do)) {
        return ParseDoWhileStmt();
    }
    if (Check(TokenType::For)) {
        return ParseForStmt();
    }
    if (Check(TokenType::Return)) {
        return ParseReturnStmt();
    }
    if (Check(TokenType::Break)) {
        return ParseBreakStmt();
    }
    if (Check(TokenType::Continue)) {
        return ParseContinueStmt();
    }
    if (Check(TokenType::Switch)) {
        return ParseSwitchStmt();
    }
    if (Check(TokenType::Case) || Check(TokenType::Default)) {
        return ParseCaseStmt();
    }
    if (IsTypeToken()) {
        return ParseVarDeclStmt();
    }
    return ParseExprStmt();
}

StmtPtr Parser::ParseBlock() {
    Consume(TokenType::LBrace, "预期 '{'");
    auto block = std::make_unique<BlockStmt>();
    while (!Check(TokenType::RBrace) && !IsAtEnd()) {
        auto stmtCheckpoint = pos_;
        block->stmts.push_back(ParseStatement());
        if (pos_ == stmtCheckpoint) {
            // Failed to parse a statement, skip one token to avoid infinite loop
            Advance();
        }
    }
    Consume(TokenType::RBrace, "预期 '}'");
    return block;
}

StmtPtr Parser::ParseVarDeclStmt() {
    auto loc = Current();
    auto [varType, name] = ParseTypeAndName();
    ExprPtr init = nullptr;
    if (Match(TokenType::Assign)) {
        if (Check(TokenType::LBrace)) {
            init = ParseInitList();
        } else {
            init = ParseExpression();
        }
    }

    auto stmt = std::make_unique<VarDeclStmt>(varType, name, std::move(init));
    stmt->loc = {loc.line, loc.column};

    // Multi-variable declaration: int a = 1, b = 2, c;
    while (Match(TokenType::Comma)) {
        auto extraName = Consume(TokenType::Identifier, "预期标识符名称");
        ExprPtr extraInit = nullptr;
        if (Match(TokenType::Assign)) {
            if (Check(TokenType::LBrace)) {
                extraInit = ParseInitList();
            } else {
                extraInit = ParseExpression();
            }
        }
        stmt->extraVars.push_back({extraName.text, std::move(extraInit)});
    }

    Consume(TokenType::Semicolon, "变量声明后预期 ';'");
    return stmt;
}

StmtPtr Parser::ParseIfStmt() {
    auto loc = Current();
    Consume(TokenType::If, "预期 'if'");
    Consume(TokenType::LParen, "预期 '('");
    auto cond = ParseExpression();
    Consume(TokenType::RParen, "预期 ')'");
    auto thenStmt = ParseStatement();
    StmtPtr elseStmt = nullptr;
    if (Match(TokenType::Else)) {
        elseStmt = ParseStatement();
    }
    auto stmt = std::make_unique<IfStmt>(std::move(cond), std::move(thenStmt), std::move(elseStmt));
    stmt->loc = {loc.line, loc.column};
    return stmt;
}

StmtPtr Parser::ParseWhileStmt() {
    auto loc = Current();
    Consume(TokenType::While, "预期 'while'");
    Consume(TokenType::LParen, "预期 '('");
    auto cond = ParseExpression();
    Consume(TokenType::RParen, "预期 ')'");
    auto body = ParseStatement();
    auto stmt = std::make_unique<WhileStmt>(std::move(cond), std::move(body));
    stmt->loc = {loc.line, loc.column};
    return stmt;
}

StmtPtr Parser::ParseDoWhileStmt() {
    auto loc = Current();
    Consume(TokenType::Do, "预期 'do'");
    auto body = ParseStatement();
    Consume(TokenType::While, "预期 'while'");
    Consume(TokenType::LParen, "预期 '('");
    auto cond = ParseExpression();
    Consume(TokenType::RParen, "预期 ')'");
    Consume(TokenType::Semicolon, "do...while 后预期 ';'");
    auto stmt = std::make_unique<DoWhileStmt>(std::move(body), std::move(cond));
    stmt->loc = {loc.line, loc.column};
    return stmt;
}

StmtPtr Parser::ParseBreakStmt() {
    auto loc = Current();
    Consume(TokenType::Break, "预期 'break'");
    Consume(TokenType::Semicolon, "break 后预期 ';'");
    auto stmt = std::make_unique<BreakStmt>();
    stmt->loc = {loc.line, loc.column};
    return stmt;
}

StmtPtr Parser::ParseContinueStmt() {
    auto loc = Current();
    Consume(TokenType::Continue, "预期 'continue'");
    Consume(TokenType::Semicolon, "continue 后预期 ';'");
    auto stmt = std::make_unique<ContinueStmt>();
    stmt->loc = {loc.line, loc.column};
    return stmt;
}

StmtPtr Parser::ParseForStmt() {
    auto loc = Current();
    Consume(TokenType::For, "预期 'for'");
    Consume(TokenType::LParen, "预期 '('");

    // Init: var decl or expr (no trailing semicolon here)
    StmtPtr init = nullptr;
    if (IsTypeToken()) {
        auto varLoc = Current();
        auto [varType, name] = ParseTypeAndName();
        ExprPtr initExpr = nullptr;
        if (Match(TokenType::Assign)) {
            if (Check(TokenType::LBrace)) {
                initExpr = ParseInitList();
            } else {
                initExpr = ParseExpression();
            }
        }
        auto vd = std::make_unique<VarDeclStmt>(varType, name, std::move(initExpr));
        vd->loc = {varLoc.line, varLoc.column};
        while (Match(TokenType::Comma)) {
            auto extraNameTok = Consume(TokenType::Identifier, "预期标识符名称");
            ExprPtr extraInit = nullptr;
            if (Match(TokenType::Assign)) {
                if (Check(TokenType::LBrace)) {
                    extraInit = ParseInitList();
                } else {
                    extraInit = ParseExpression();
                }
            }
            vd->extraVars.push_back({extraNameTok.text, std::move(extraInit)});
        }
        init = std::move(vd);
    } else if (!Check(TokenType::Semicolon)) {
        auto esLoc = Current();
        auto es = std::make_unique<ExprStmt>(ParseExpression());
        es->loc = {esLoc.line, esLoc.column};
        init = std::move(es);
    }
    Consume(TokenType::Semicolon, "预期 ';'");

    ExprPtr cond = nullptr;
    if (!Check(TokenType::Semicolon)) {
        cond = ParseExpression();
    }
    Consume(TokenType::Semicolon, "预期 ';'");

    ExprPtr step = nullptr;
    if (!Check(TokenType::RParen)) {
        step = ParseExpression();
    }
    Consume(TokenType::RParen, "预期 ')'");

    auto body = ParseStatement();
    auto stmt = std::make_unique<ForStmt>(std::move(init), std::move(cond), std::move(step), std::move(body));
    stmt->loc = {loc.line, loc.column};
    return stmt;
}

StmtPtr Parser::ParseReturnStmt() {
    auto loc = Current();
    Consume(TokenType::Return, "预期 'return'");
    ExprPtr value = nullptr;
    if (!Check(TokenType::Semicolon)) {
        value = ParseExpression();
    }
    Consume(TokenType::Semicolon, "return 后预期 ';'");
    auto stmt = std::make_unique<ReturnStmt>(std::move(value));
    stmt->loc = {loc.line, loc.column};
    return stmt;
}

StmtPtr Parser::ParseExprStmt() {
    auto loc = Current();
    auto expr = ParseExpression();
    Consume(TokenType::Semicolon, "预期 ';'");
    auto stmt = std::make_unique<ExprStmt>(std::move(expr));
    stmt->loc = {loc.line, loc.column};
    return stmt;
}

// ============================================================================
// Expressions (precedence climbing)
// ============================================================================

ExprPtr Parser::ParseExpression() {
    return ParseAssign();
}

ExprPtr Parser::ParseAssign() {
    auto left = ParseOr();

    if (Match(TokenType::Assign)) {
        auto right = ParseAssign(); // Right-associative
        return std::make_unique<AssignExpr>(AssignExpr::Op::Assign, std::move(left), std::move(right));
    }
    if (Match(TokenType::PlusAssign)) {
        auto right = ParseAssign();
        return std::make_unique<AssignExpr>(AssignExpr::Op::AddAssign, std::move(left), std::move(right));
    }
    if (Match(TokenType::MinusAssign)) {
        auto right = ParseAssign();
        return std::make_unique<AssignExpr>(AssignExpr::Op::SubAssign, std::move(left), std::move(right));
    }
    if (Match(TokenType::StarAssign)) {
        auto right = ParseAssign();
        return std::make_unique<AssignExpr>(AssignExpr::Op::MulAssign, std::move(left), std::move(right));
    }
    if (Match(TokenType::SlashAssign)) {
        auto right = ParseAssign();
        return std::make_unique<AssignExpr>(AssignExpr::Op::DivAssign, std::move(left), std::move(right));
    }
    if (Match(TokenType::PercentAssign)) {
        auto right = ParseAssign();
        return std::make_unique<AssignExpr>(AssignExpr::Op::ModAssign, std::move(left), std::move(right));
    }

    return left;
}

ExprPtr Parser::ParseOr() {
    auto left = ParseAnd();
    while (Match(TokenType::OrOr)) {
        auto right = ParseAnd();
        left = std::make_unique<BinaryExpr>(BinaryExpr::Op::Or, std::move(left), std::move(right));
    }
    return left;
}

ExprPtr Parser::ParseAnd() {
    auto left = ParseEquality();
    while (Match(TokenType::AndAnd)) {
        auto right = ParseEquality();
        left = std::make_unique<BinaryExpr>(BinaryExpr::Op::And, std::move(left), std::move(right));
    }
    return left;
}

ExprPtr Parser::ParseEquality() {
    auto left = ParseRelational();
    while (true) {
        if (Match(TokenType::Eq)) {
            auto right = ParseRelational();
            left = std::make_unique<BinaryExpr>(BinaryExpr::Op::Eq, std::move(left), std::move(right));
        } else if (Match(TokenType::Ne)) {
            auto right = ParseRelational();
            left = std::make_unique<BinaryExpr>(BinaryExpr::Op::Ne, std::move(left), std::move(right));
        } else break;
    }
    return left;
}

ExprPtr Parser::ParseRelational() {
    auto left = ParseAdditive();
    while (true) {
        if (Match(TokenType::Lt)) {
            auto right = ParseAdditive();
            left = std::make_unique<BinaryExpr>(BinaryExpr::Op::Lt, std::move(left), std::move(right));
        } else if (Match(TokenType::Le)) {
            auto right = ParseAdditive();
            left = std::make_unique<BinaryExpr>(BinaryExpr::Op::Le, std::move(left), std::move(right));
        } else if (Match(TokenType::Gt)) {
            auto right = ParseAdditive();
            left = std::make_unique<BinaryExpr>(BinaryExpr::Op::Gt, std::move(left), std::move(right));
        } else if (Match(TokenType::Ge)) {
            auto right = ParseAdditive();
            left = std::make_unique<BinaryExpr>(BinaryExpr::Op::Ge, std::move(left), std::move(right));
        } else break;
    }
    return left;
}

ExprPtr Parser::ParseAdditive() {
    auto left = ParseMultiplicative();
    while (true) {
        if (Match(TokenType::Plus)) {
            auto right = ParseMultiplicative();
            left = std::make_unique<BinaryExpr>(BinaryExpr::Op::Add, std::move(left), std::move(right));
        } else if (Match(TokenType::Minus)) {
            auto right = ParseMultiplicative();
            left = std::make_unique<BinaryExpr>(BinaryExpr::Op::Sub, std::move(left), std::move(right));
        } else break;
    }
    return left;
}

ExprPtr Parser::ParseMultiplicative() {
    auto left = ParseUnary();
    while (true) {
        if (Match(TokenType::Star)) {
            auto right = ParseUnary();
            left = std::make_unique<BinaryExpr>(BinaryExpr::Op::Mul, std::move(left), std::move(right));
        } else if (Match(TokenType::Slash)) {
            auto right = ParseUnary();
            left = std::make_unique<BinaryExpr>(BinaryExpr::Op::Div, std::move(left), std::move(right));
        } else if (Match(TokenType::Percent)) {
            auto right = ParseUnary();
            left = std::make_unique<BinaryExpr>(BinaryExpr::Op::Mod, std::move(left), std::move(right));
        } else break;
    }
    return left;
}

ExprPtr Parser::ParseUnary() {
    if (Match(TokenType::Sizeof)) {
        return ParseSizeof();
    }
    if (Match(TokenType::Minus)) {
        auto operand = ParseUnary();
        return std::make_unique<UnaryExpr>(UnaryExpr::Op::Neg, std::move(operand));
    }
    if (Match(TokenType::Not)) {
        auto operand = ParseUnary();
        return std::make_unique<UnaryExpr>(UnaryExpr::Op::Not, std::move(operand));
    }
    if (Match(TokenType::Ampersand)) {
        auto operand = ParseUnary();
        return std::make_unique<UnaryExpr>(UnaryExpr::Op::Addr, std::move(operand));
    }
    if (Match(TokenType::Star)) {
        auto operand = ParseUnary();
        return std::make_unique<UnaryExpr>(UnaryExpr::Op::Deref, std::move(operand));
    }
    if (Match(TokenType::Increment)) {
        auto operand = ParseUnary();
        return std::make_unique<UnaryExpr>(UnaryExpr::Op::PreInc, std::move(operand));
    }
    if (Match(TokenType::Decrement)) {
        auto operand = ParseUnary();
        return std::make_unique<UnaryExpr>(UnaryExpr::Op::PreDec, std::move(operand));
    }
    return ParsePostfix();
}

ExprPtr Parser::ParseSizeof() {
    SourceLoc loc = {Previous().line, Previous().column};
    if (Match(TokenType::LParen)) {
        // Try to parse as sizeof(type)
        auto checkpoint = pos_;
        bool isType = false;
        Type t;
        if (Check(TokenType::Int) || Check(TokenType::Void) ||
            Check(TokenType::Char) || Check(TokenType::Struct)) {
            t = ParseBaseType();
            if (Match(TokenType::Star)) {
                t = Type{TypeKind::Pointer, t.name};
            }
            if (Check(TokenType::RParen)) {
                isType = true;
            }
        }
        if (isType) {
            Consume(TokenType::RParen, "sizeof(type) 后预期 ')'");
            auto node = std::make_unique<SizeofExpr>(std::move(t));
            node->loc = loc;
            return node;
        }
        // Not a type query: rollback and parse as expression
        pos_ = checkpoint;
        auto expr = ParseExpression();
        Consume(TokenType::RParen, "sizeof(expr) 后预期 ')'");
        auto node = std::make_unique<SizeofExpr>(std::move(expr));
        node->loc = loc;
        return node;
    }
    // sizeof expr without parens
    auto expr = ParseUnary();
    auto node = std::make_unique<SizeofExpr>(std::move(expr));
    node->loc = loc;
    return node;
}

Type Parser::ParseTypeOnly() {
    auto base = ParseBaseType();
    if (Match(TokenType::Star)) {
        return Type{TypeKind::Pointer, base.name, 0, base.kind};
    }
    return base;
}

ExprPtr Parser::ParsePostfix() {
    auto expr = ParsePrimary();

    while (true) {
        if (Match(TokenType::LBracket)) {
            auto index = ParseExpression();
            Consume(TokenType::RBracket, "预期 ']'");
            expr = std::make_unique<IndexExpr>(std::move(expr), std::move(index));
        } else if (Match(TokenType::Dot) || Match(TokenType::Arrow)) {
            auto memberTok = Consume(TokenType::Identifier, "预期成员名称");
            expr = std::make_unique<MemberExpr>(std::move(expr), memberTok.text);
        } else if (Match(TokenType::Increment)) {
            expr = std::make_unique<UnaryExpr>(UnaryExpr::Op::PostInc, std::move(expr));
        } else if (Match(TokenType::Decrement)) {
            expr = std::make_unique<UnaryExpr>(UnaryExpr::Op::PostDec, std::move(expr));
        } else {
            break;
        }
    }

    return expr;
}

ExprPtr Parser::ParseInitList() {
    auto loc = Current();
    Consume(TokenType::LBrace, "初始化列表预期 '{'");
    std::vector<ExprPtr> elements;
    if (!Check(TokenType::RBrace)) {
        while (true) {
            if (Check(TokenType::LBrace)) {
                elements.push_back(ParseInitList());
            } else {
                elements.push_back(ParseExpression());
            }
            if (!Match(TokenType::Comma)) break;
        }
    }
    Consume(TokenType::RBrace, "初始化列表预期 '}'");
    auto node = std::make_unique<InitListExpr>(std::move(elements));
    node->loc = {loc.line, loc.column};
    return node;
}

ExprPtr Parser::ParsePrimary() {
    if (Match(TokenType::Number)) {
        int value = std::atoi(tokens_[pos_ - 1].text.c_str());
        return std::make_unique<LiteralExpr>(value);
    }

    if (Match(TokenType::String)) {
        return std::make_unique<StringLiteralExpr>(tokens_[pos_ - 1].text);
    }

    if (Check(TokenType::Identifier)) {
        auto nameTok = Advance();
        if (Check(TokenType::LParen)) {
            return ParseCallExpr(nameTok.text);
        }
        return std::make_unique<IdentifierExpr>(nameTok.text);
    }

    if (Match(TokenType::LParen)) {
        auto expr = ParseExpression();
        Consume(TokenType::RParen, "预期 ')'");
        return expr;
    }

    errors_.push_back({"预期表达式", Current().line, Current().column, static_cast<int>(ErrorCode::E2003_ExpectedExpr)});
    return std::make_unique<LiteralExpr>(0);
}

ExprPtr Parser::ParseCallExpr(const std::string& name) {
    Consume(TokenType::LParen, "预期 '('");
    auto args = ParseArgList();
    Consume(TokenType::RParen, "预期 ')'");
    return std::make_unique<CallExpr>(name, std::move(args));
}

std::vector<ExprPtr> Parser::ParseArgList() {
    std::vector<ExprPtr> args;
    if (Check(TokenType::RParen)) {
        return args;
    }
    while (true) {
        args.push_back(ParseExpression());
        if (!Match(TokenType::Comma)) break;
    }
    return args;
}

StmtPtr Parser::ParseSwitchStmt() {
    SourceLoc loc = {Current().line, Current().column};
    Advance(); // consume 'switch'
    Consume(TokenType::LParen, "switch 后预期 '('");
    auto cond = ParseExpression();
    Consume(TokenType::RParen, "switch 条件后预期 ')'");
    auto body = ParseStatement();
    auto stmt = std::make_unique<SwitchStmt>(std::move(cond), std::move(body));
    stmt->loc = loc;
    return stmt;
}

StmtPtr Parser::ParseCaseStmt() {
    SourceLoc loc = {Current().line, Current().column};
    ExprPtr label = nullptr;
    if (Match(TokenType::Case)) {
        label = ParseExpression();
    } else if (Match(TokenType::Default)) {
        label = nullptr;
    } else {
        errors_.push_back({"预期 'case' 或 'default'", Current().line, Current().column, static_cast<int>(ErrorCode::E2004_ExpectedCaseOrDefault)});
        return nullptr;
    }
    Consume(TokenType::Colon, "case/default 后预期 ':'");
    // Parse all statements until next case/default/}
    std::vector<StmtPtr> stmts;
    while (!Check(TokenType::Case) && !Check(TokenType::Default) && !Check(TokenType::RBrace) && !IsAtEnd()) {
        stmts.push_back(ParseStatement());
    }
    StmtPtr body;
    if (stmts.empty()) {
        body = std::make_unique<BlockStmt>();
    } else if (stmts.size() == 1) {
        body = std::move(stmts[0]);
    } else {
        auto block = std::make_unique<BlockStmt>();
        block->stmts = std::move(stmts);
        body = std::move(block);
    }
    auto caseStmt = std::make_unique<CaseStmt>(std::move(label), std::move(body));
    caseStmt->loc = loc;
    return caseStmt;
}

void Parser::ParseTypedef() {
    Advance(); // consume 'typedef'
    auto type = ParseTypeOnly();
    auto nameTok = Consume(TokenType::Identifier, "typedef 后预期标识符名称");
    Consume(TokenType::Semicolon, "typedef 后预期 ';'");
    typedefNames_[nameTok.text] = type;
}

void Parser::ParseEnumDecl(ProgramNode* program) {
    SourceLoc loc = {Current().line, Current().column};
    Advance(); // consume 'enum'
    std::string enumName;
    if (Check(TokenType::Identifier)) {
        enumName = Current().text;
        Advance();
    }
    Consume(TokenType::LBrace, "enum 后预期 '{'");
    int nextValue = 0;
    while (!Check(TokenType::RBrace) && !IsAtEnd()) {
        auto memberTok = Consume(TokenType::Identifier, "enum 成员预期标识符");
        if (Match(TokenType::Assign)) {
            auto valExpr = ParseExpression();
            if (valExpr && valExpr->kind == ExprKind::Literal) {
                nextValue = static_cast<LiteralExpr&>(*valExpr).value;
            }
        }
        // Add enum constant as a global int variable
        program->globals.push_back({loc, Type{TypeKind::Int}, memberTok.text,
            std::make_unique<LiteralExpr>(nextValue)});
        nextValue++;
        if (!Match(TokenType::Comma)) break;
    }
    Consume(TokenType::RBrace, "enum 成员后预期 '}'");
    Consume(TokenType::Semicolon, "enum 声明后预期 ';'");
    // Optionally register enum name as a type alias (simplified: always int)
    if (!enumName.empty()) {
        typedefNames_[enumName] = Type{TypeKind::Int};
    }
}

} // namespace cide
