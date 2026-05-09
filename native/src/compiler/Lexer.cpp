#include "Lexer.hpp"
#include "diagnostics/ErrorCodes.hpp"
#include <cctype>
#include <unordered_map>

namespace cide {

static const std::unordered_map<std::string, TokenType> kKeywords = {
    {"int",     TokenType::Int},
    {"void",    TokenType::Void},
    {"char",    TokenType::Char},
    {"if",      TokenType::If},
    {"else",    TokenType::Else},
    {"while",   TokenType::While},
    {"do",      TokenType::Do},
    {"for",     TokenType::For},
    {"return",  TokenType::Return},
    {"break",   TokenType::Break},
    {"continue",TokenType::Continue},
    {"struct",  TokenType::Struct},
    {"sizeof",  TokenType::Sizeof},
    {"switch",  TokenType::Switch},
    {"case",    TokenType::Case},
    {"default", TokenType::Default},
    {"typedef", TokenType::Typedef},
    {"enum",    TokenType::Enum},
    {"unsigned",TokenType::Unsigned},
};

Lexer::Lexer(std::string source) : source_(std::move(source)) {}

std::vector<Token> Lexer::Tokenize() {
    std::vector<Token> tokens;
    while (true) {
        Token t = NextToken();
        tokens.push_back(t);
        if (t.type == TokenType::Eof) break;
    }
    return ExpandMacros(tokens);
}

Token Lexer::NextToken() {
    SkipWhitespace();

    if (pos_ >= source_.size()) {
        return MakeToken(TokenType::Eof, "");
    }

    char c = Peek();

    // Identifiers & keywords
    if (std::isalpha(c) || c == '_') {
        return IdentifierOrKeyword();
    }

    // Numbers
    if (std::isdigit(c)) {
        return Number();
    }

    // String literals
    if (c == '"') {
        return StringLiteral();
    }

    // Comments
    if (c == '/' && Peek(1) == '/') {
        SkipComment();
        return NextToken();
    }

    // Preprocessor directives (#include, #define, etc.)
    if (c == '#') {
        SkipPreprocessorDirective();
        return NextToken();
    }

    // Multi-char operators (check order matters)
    switch (c) {
        case '+':
            if (Match('+')) return MakeToken(TokenType::Increment, "++");
            if (Match('=')) return MakeToken(TokenType::PlusAssign, "+=");
            Advance();
            return MakeToken(TokenType::Plus, "+");
        case '-':
            if (Match('>')) return MakeToken(TokenType::Arrow, "->");
            if (Match('-')) return MakeToken(TokenType::Decrement, "--");
            if (Match('=')) return MakeToken(TokenType::MinusAssign, "-=");
            Advance();
            return MakeToken(TokenType::Minus, "-");
        case '*':
            if (Match('=')) return MakeToken(TokenType::StarAssign, "*=");
            Advance();
            return MakeToken(TokenType::Star, "*");
        case '/':
            if (Match('=')) return MakeToken(TokenType::SlashAssign, "/=");
            Advance();
            return MakeToken(TokenType::Slash, "/");
        case '%':
            if (Match('=')) return MakeToken(TokenType::PercentAssign, "%=");
            Advance();
            return MakeToken(TokenType::Percent, "%");
        case '=':
            if (Match('=')) return MakeToken(TokenType::Eq, "==");
            Advance();
            return MakeToken(TokenType::Assign, "=");
        case '!':
            if (Match('=')) return MakeToken(TokenType::Ne, "!=");
            Advance();
            return MakeToken(TokenType::Not, "!");
        case '<':
            if (Match('=')) return MakeToken(TokenType::Le, "<=");
            Advance();
            return MakeToken(TokenType::Lt, "<");
        case '>':
            if (Match('=')) return MakeToken(TokenType::Ge, ">=");
            Advance();
            return MakeToken(TokenType::Gt, ">");
        case '&':
            if (Match('&')) return MakeToken(TokenType::AndAnd, "&&");
            Advance();
            return MakeToken(TokenType::Ampersand, "&");
        case '|':
            if (Match('|')) return MakeToken(TokenType::OrOr, "||");
            // Single | is not supported in our subset
            Advance();
            errors_.push_back({"暂不支持单竖线 '|'，逻辑或请使用 '||'", line_, column_, static_cast<int>(ErrorCode::E1004_UnsupportedOp)});
            return MakeToken(TokenType::Unknown, "|");
        case ';': Advance(); return MakeToken(TokenType::Semicolon, ";");
        case ',': Advance(); return MakeToken(TokenType::Comma, ",");
        case '(': Advance(); return MakeToken(TokenType::LParen, "(");
        case ')': Advance(); return MakeToken(TokenType::RParen, ")");
        case '{': Advance(); return MakeToken(TokenType::LBrace, "{");
        case '}': Advance(); return MakeToken(TokenType::RBrace, "}");
        case '[': Advance(); return MakeToken(TokenType::LBracket, "[");
        case ']': Advance(); return MakeToken(TokenType::RBracket, "]");
        case '.': Advance(); return MakeToken(TokenType::Dot, ".");
        case ':': Advance(); return MakeToken(TokenType::Colon, ":");
        default:
            Advance();
            errors_.push_back({std::string("无法识别的字符: '") + c + "'", line_, column_, static_cast<int>(ErrorCode::E1001_UnknownChar)});
            return MakeToken(TokenType::Unknown, std::string(1, c));
    }
}

Token Lexer::IdentifierOrKeyword() {
    size_t start = pos_;
    while (pos_ < source_.size() && (std::isalnum(Peek()) || Peek() == '_')) {
        Advance();
    }
    std::string text = source_.substr(start, pos_ - start);

    auto it = kKeywords.find(text);
    if (it != kKeywords.end()) {
        return MakeToken(it->second, text);
    }
    return MakeToken(TokenType::Identifier, text);
}

Token Lexer::Number() {
    size_t start = pos_;
    while (pos_ < source_.size() && std::isdigit(Peek())) {
        Advance();
    }
    return MakeToken(TokenType::Number, start, pos_ - start);
}

Token Lexer::StringLiteral() {
    size_t start = pos_;
    Advance(); // consume opening "
    std::string value;
    while (pos_ < source_.size() && Peek() != '"') {
        if (Peek() == '\n') {
            errors_.push_back({"字符串不能跨行", line_, column_, static_cast<int>(ErrorCode::E1003_StringCrossLine)});
            break;
        }
        if (Peek() == '\\' && pos_ + 1 < source_.size()) {
            char next = source_[pos_ + 1];
            switch (next) {
                case 'n': value += '\n'; break;
                case 't': value += '\t'; break;
                case '\\': value += '\\'; break;
                case '"': value += '"'; break;
                case '0': value += '\0'; break;
                default: value += next; break;
            }
            Advance(); Advance();
        } else {
            value += Peek();
            Advance();
        }
    }
    if (pos_ >= source_.size() || Peek() != '"') {
        errors_.push_back({"字符串未闭合", line_, column_, static_cast<int>(ErrorCode::E1002_UnterminatedString)});
    } else {
        Advance(); // consume closing "
    }
    Token tok = MakeToken(TokenType::String, start, pos_ - start);
    tok.text = std::move(value);
    return tok;
}

void Lexer::SkipWhitespace() {
    while (pos_ < source_.size() && std::isspace(Peek())) {
        Advance();
    }
}

void Lexer::SkipComment() {
    // // style comment
    while (pos_ < source_.size() && Peek() != '\n') {
        Advance();
    }
}

void Lexer::SkipPreprocessorDirective() {
    Advance(); // consume '#'
    SkipWhitespace();

    // Check for #define
    if (pos_ + 6 <= source_.size() && source_.substr(pos_, 6) == "define") {
        pos_ += 6;
        column_ += 6;
        ParseDefineDirective();
        return;
    }

    // Other directives (#include, #pragma, etc.): skip to end of line
    while (pos_ < source_.size() && Peek() != '\n') {
        Advance();
    }
}

void Lexer::ParseDefineDirective() {
    SkipWhitespace();
    if (pos_ >= source_.size() || (!std::isalpha(Peek()) && Peek() != '_')) {
        errors_.push_back({"#define 后预期宏名称", line_, column_, static_cast<int>(ErrorCode::E1005_InvalidDefine)});
        while (pos_ < source_.size() && Peek() != '\n') Advance();
        return;
    }

    // Read macro name
    size_t nameStart = pos_;
    while (pos_ < source_.size() && (std::isalnum(Peek()) || Peek() == '_')) {
        Advance();
    }
    std::string name = source_.substr(nameStart, pos_ - nameStart);

    SkipWhitespace();

    // Read replacement text to end of line
    size_t bodyStart = pos_;
    while (pos_ < source_.size() && Peek() != '\n') {
        Advance();
    }
    std::string body = source_.substr(bodyStart, pos_ - bodyStart);

    // Tokenize replacement body
    Lexer bodyLexer(body);
    std::vector<Token> bodyTokens = bodyLexer.Tokenize();
    // Remove EOF token from macro body
    if (!bodyTokens.empty() && bodyTokens.back().type == TokenType::Eof) {
        bodyTokens.pop_back();
    }

    macros_[name] = std::move(bodyTokens);
}

std::vector<Token> Lexer::ExpandMacros(const std::vector<Token>& tokens) {
    std::vector<Token> result;
    for (const auto& tok : tokens) {
        if (tok.type == TokenType::Identifier) {
            auto it = macros_.find(tok.text);
            if (it != macros_.end()) {
                for (const auto& mt : it->second) {
                    result.push_back(mt);
                    result.back().line = tok.line;
                    result.back().column = tok.column;
                }
                continue;
            }
        }
        result.push_back(tok);
    }
    return result;
}

char Lexer::Peek(size_t offset) const {
    if (pos_ + offset >= source_.size()) return '\0';
    return source_[pos_ + offset];
}

char Lexer::Advance() {
    if (pos_ >= source_.size()) return '\0';
    char c = source_[pos_++];
    if (c == '\n') {
        line_++;
        column_ = 1;
    } else {
        column_++;
    }
    return c;
}

bool Lexer::Match(char expected) {
    if (Peek(1) != expected) return false;
    Advance(); // consume current char
    Advance(); // consume expected char
    return true;
}

Token Lexer::MakeToken(TokenType type, std::string text) {
    return Token{type, std::move(text), line_, static_cast<int>(column_ - text.length())};
}

Token Lexer::MakeToken(TokenType type, size_t start, size_t len) {
    return Token{type, source_.substr(start, len), line_, static_cast<int>(column_ - len)};
}

} // namespace cide
