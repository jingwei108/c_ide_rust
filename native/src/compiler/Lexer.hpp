#pragma once

#include <string>
#include <vector>
#include <cstdint>
#include <unordered_map>

namespace cide {

enum class TokenType {
    // Keywords
    Int, Void, Char, If, Else, While, Do, For, Return, Break, Continue, Struct, Sizeof, Switch, Case, Default, Typedef, Enum, Unsigned,

    // Identifiers & Literals
    Identifier, Number, String,

    // Operators
    Plus, Minus, Star, Slash, Percent,
    Eq, Ne, Lt, Le, Gt, Ge,
    AndAnd, OrOr, Not,
    Assign, PlusAssign, MinusAssign, StarAssign, SlashAssign, PercentAssign,
    Ampersand,     // & (address-of)
    Increment, Decrement,

    // Separators
    Semicolon, Comma,
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Dot, Arrow,    // . -> (both map to member access)
    Colon,         // :

    // Special
    Eof,
    Unknown,
};

struct Token {
    TokenType type;
    std::string text;
    int line;
    int column;
};

struct LexerError {
    std::string message;
    int line;
    int column;
    int code = 0;
};

class Lexer {
public:
    explicit Lexer(std::string source);

    std::vector<Token> Tokenize();
    const std::vector<LexerError>& Errors() const { return errors_; }
    bool HasErrors() const { return !errors_.empty(); }

private:
    std::string source_;
    std::vector<LexerError> errors_;
    size_t pos_ = 0;
    int line_ = 1;
    int column_ = 1;

    // Simple macro table: #define NAME replacement-tokens
    std::unordered_map<std::string, std::vector<Token>> macros_;

    Token NextToken();
    Token MakeToken(TokenType type, std::string text);
    Token MakeToken(TokenType type, size_t start, size_t len);

    char Peek(size_t offset = 0) const;
    char Advance();
    bool Match(char expected);
    void SkipWhitespace();
    void SkipComment();
    void SkipPreprocessorDirective();
    void ParseDefineDirective();

    Token IdentifierOrKeyword();
    Token Number();
    Token StringLiteral();
    std::vector<Token> ExpandMacros(const std::vector<Token>& tokens);
};

} // namespace cide
