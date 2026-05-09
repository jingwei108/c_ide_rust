#include "cide_capi.h"

#include <algorithm>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <sstream>
#include <string>
#include <vector>

#include "capi/CideSession.hpp"
#include "compiler/Lexer.hpp"
#include "compiler/Parser.hpp"
#include "compiler/TypeChecker.hpp"
#include "compiler/BytecodeGen.hpp"
#include "diagnostics/AlgorithmMatcher.hpp"
#include "diagnostics/ErrorCodes.hpp"
#include "vm/CideVM.hpp"
#include "vm/HostFunctions.hpp"

// ============================================================================
// Host Functions Registration
// ============================================================================

static void SetupVM(CideSession* s) {
    s->vm.Reset();
    s->vm.LoadProgram(s->compile.bytecode);
    s->vm.SetGlobals(s->compile.globalsInit);
    s->vm.SetMaxSteps(10000000);

    // Register compiled functions
    for (const auto& kv : s->compile.funcTable) {
        auto idxIt = s->compile.funcIndex.find(kv.first);
        if (idxIt != s->compile.funcIndex.end()) {
            cide::CideVM::FuncMeta meta;
            meta.ip = kv.second.ip;
            meta.argCount = kv.second.argCount;
            meta.localCount = kv.second.localCount;
            uint32_t idx = static_cast<uint32_t>(idxIt->second);
            s->vm.RegisterFunction(idx, meta);
            s->vm.RegisterFunctionName(idx, kv.first);
        }
    }

    // Load symbol table for runtime diagnostics
    s->vm.SetSymbols(s->compile.symbols);

    // Load vis event lines for algorithm visualization
    std::vector<std::pair<int, int>> visLines;
    for (const auto& m : s->compile.algorithmMatches) {
        for (const auto& ev : m.visEvents) {
            visLines.push_back({std::get<0>(ev), std::get<1>(ev)});
        }
    }
    s->vm.SetVisEventLines(visLines);

    // Register heap limit callback for stack-heap collision detection
    s->vm.SetHeapLimitCallback([s]() { return s->memory.heapOffset; });

    // Copy string literals into VM linear memory
    uint8_t* mem = s->vm.GetMemory();
    uint32_t memSize = s->vm.GetMemorySize();
    for (const auto& kv : s->compile.stringData) {
        uint32_t addr = kv.first;
        const std::string& str = kv.second;
        if (mem && addr + str.size() + 1 <= memSize) {
            std::memcpy(mem + addr, str.c_str(), str.size() + 1);
        }
    }
}

// Host function registration is now in vm/HostFunctions.cpp

// ============================================================================
// Session Management
// ============================================================================

extern "C" CideSession* cide_session_create() {
    auto* s = new CideSession();
    return s;
}

extern "C" void cide_session_destroy(CideSession* s) {
    if (!s) return;
    delete s;
}

// ============================================================================
// Compilation
// ============================================================================

static std::string BeautifyCompileError(int code, const std::string& raw) {
    // Phase 1: exact code-based matching (robust)
    switch (code) {
        case static_cast<int>(cide::ErrorCode::E1001_UnknownChar): return "🤔 " + raw + "\n💡 提示：本 IDE 只支持标准的 C 语言字符。请检查是否有特殊符号或中文标点。";
        case static_cast<int>(cide::ErrorCode::E1002_UnterminatedString): return "😵 " + raw + "\n💡 提示：字符串缺少右引号。请确保字符串用双引号闭合。";
        case static_cast<int>(cide::ErrorCode::E1003_StringCrossLine): return "😵 " + raw + "\n💡 提示：字符串不能跨越多行。如需换行，请使用 \\n。";
        case static_cast<int>(cide::ErrorCode::E1004_UnsupportedOp): return "🤔 " + raw + "\n💡 提示：逻辑或请使用 '||'，逻辑与请使用 '&&'。";
        case static_cast<int>(cide::ErrorCode::E2001_ExpectedType): return "🤔 " + raw + "\n💡 提示：类型声明应该是 int、char、void 或 struct。";
        case static_cast<int>(cide::ErrorCode::E2002_ExpectedArraySize): return "🤔 " + raw + "\n💡 提示：数组声明需要大小，例如 int arr[10]。";
        case static_cast<int>(cide::ErrorCode::E2003_ExpectedExpr): return "🤔 " + raw + "\n💡 提示：这里需要一个表达式（如变量、数字或计算）。请检查是否有遗漏。";
        case static_cast<int>(cide::ErrorCode::E2004_ExpectedCaseOrDefault): return "🤔 " + raw + "\n💡 提示：switch 语句中请使用 case 或 default 标签。";
        case static_cast<int>(cide::ErrorCode::E2005_ExpectedSemicolon): return "😵 " + raw + "\n💡 提示：你是不是忘了写分号？C 语言中每条语句末尾都需要 ';'。";
        case static_cast<int>(cide::ErrorCode::E2006_ExpectedClosingBrace): return "😵 " + raw + "\n💡 提示：你是不是忘了写右大括号 '}'？请检查每个 '{' 都有对应的 '}'。";
        case static_cast<int>(cide::ErrorCode::E2007_ExpectedClosingParen): return "😵 " + raw + "\n💡 提示：你是不是忘了写右括号 ')'？请检查每个 '(' 都有对应的 ')'。";
        case static_cast<int>(cide::ErrorCode::E2008_ExpectedClosingBracket): return "😵 " + raw + "\n💡 提示：你是不是忘了写右方括号 ']'？请检查每个 '[' 都有对应的 ']'。";
        case static_cast<int>(cide::ErrorCode::E3010_BreakOutsideLoop): return "🚫 " + raw + "\n💡 提示：break 只能用在循环或 switch 语句内部。";
        case static_cast<int>(cide::ErrorCode::E3011_ContinueOutsideLoop): return "🚫 " + raw + "\n💡 提示：continue 只能用在循环语句内部。";
        case static_cast<int>(cide::ErrorCode::E3012_VoidFuncReturnValue): return "⚠️ " + raw + "\n💡 提示：void 函数不需要 return 值。直接写 return; 即可。";
        case static_cast<int>(cide::ErrorCode::E3013_MissingReturnValue): return "⚠️ " + raw + "\n💡 提示：非 void 函数必须返回一个值。请检查所有分支都有 return。";
        case static_cast<int>(cide::ErrorCode::E3014_ReturnTypeMismatch): return "⚠️ " + raw + "\n💡 提示：return 的类型与函数声明不匹配。";
        case static_cast<int>(cide::ErrorCode::E3021_DerefNonPointer): return "😵 " + raw + "\n💡 提示：'*' 只能用于指针类型。请确认变量已经声明为指针（如 int* p）。";
        case static_cast<int>(cide::ErrorCode::E3023_UndeclaredVar): return "🤔 " + raw + "\n💡 提示：使用变量前需要先声明。例如：int a = 5;";
        case static_cast<int>(cide::ErrorCode::E3036_UndefinedFunc): return "🤔 " + raw + "\n💡 提示：调用函数前需要先定义。检查函数名拼写是否正确。";
        case static_cast<int>(cide::ErrorCode::E3039_ArrayIndexType): return "🚫 " + raw + "\n💡 提示：数组索引必须是整数，例如 arr[0]、arr[i]。不能用小数或其他类型。";
        case static_cast<int>(cide::ErrorCode::E3040_IndexNonArray): return "🚫 " + raw + "\n💡 提示：只有数组和指针才能用 [] 索引。请确认变量类型正确。";
    }

    // Phase 2: fallback to substring matching (for errors without codes yet)
    if (raw.find("类型不匹配") != std::string::npos) {
        return "⚠️ " + raw + "\n💡 提示：赋值或传参时类型不一致。例如不能把指针赋值给整数。";
    }
    if (raw.find("除零") != std::string::npos) {
        return "😵 " + raw + "\n💡 提示：除数不能为 0。请确保除法运算前检查除数。";
    }
    if (raw.find("取地址") != std::string::npos) {
        return "🤔 " + raw + "\n💡 提示：'&' 用于获取变量的地址。目前不支持对局部标量取地址，请使用数组或 malloc。";
    }
    if (raw.find("赋值运算符") != std::string::npos && raw.find("==") != std::string::npos) {
        return "⚠️ " + raw + "\n💡 提示：在 if/while/for 的条件中，'=' 是赋值，'==' 才是比较是否相等。这是一个非常常见的错误！";
    }
    if (raw.find("off-by-one") != std::string::npos || raw.find("'<='") != std::string::npos) {
        return "⚠️ " + raw + "\n💡 提示：数组索引从 0 开始，最后一个元素的下标是 '大小-1'。如果循环条件是 i <= n，当 i=n 时就会访问 arr[n]，造成越界。";
    }
    return "⚠️ " + raw;
}

static std::string GenerateFixSuggestion(int code, const std::string& raw) {
    switch (code) {
        case static_cast<int>(cide::ErrorCode::E2005_ExpectedSemicolon): return "在行尾添加分号 ';'";
        case static_cast<int>(cide::ErrorCode::E2006_ExpectedClosingBrace): return "添加右大括号 '}'";
        case static_cast<int>(cide::ErrorCode::E2007_ExpectedClosingParen): return "添加右括号 ')'";
        case static_cast<int>(cide::ErrorCode::E2008_ExpectedClosingBracket): return "添加右方括号 ']'";
        case static_cast<int>(cide::ErrorCode::E3023_UndeclaredVar): return "在使用前声明变量，例如：int x = 0;";
        case static_cast<int>(cide::ErrorCode::E3036_UndefinedFunc): return "检查函数名拼写，或在使用前定义该函数";
        case static_cast<int>(cide::ErrorCode::E3039_ArrayIndexType): return "使用整数作为数组索引，例如 arr[0]";
        case static_cast<int>(cide::ErrorCode::E3040_IndexNonArray): return "确保变量是数组或指针类型";
        case static_cast<int>(cide::ErrorCode::E3021_DerefNonPointer): return "确保变量声明为指针类型，例如 int* p";
    }
    // Fallback to substring matching
    if (raw.find("类型不匹配") != std::string::npos) {
        return "确保赋值或传参时类型一致";
    }
    if (raw.find("除零") != std::string::npos) {
        return "在除法前检查除数是否为 0";
    }
    if (raw.find("取地址") != std::string::npos) {
        return "使用数组或 malloc 分配内存来获取地址";
    }
    if (raw.find("赋值运算符") != std::string::npos && raw.find("==") != std::string::npos) {
        return "将条件中的 '=' 改为 '=='";
    }
    if (raw.find("off-by-one") != std::string::npos || raw.find("'<='") != std::string::npos) {
        return "将循环条件中的 '<=' 改为 '<'";
    }
    return "";
}

// ============================================================================
// Structured Fix Helpers
// ============================================================================

static std::vector<std::string> SplitSourceLines(const std::string& source) {
    std::vector<std::string> lines;
    std::stringstream ss(source);
    std::string line;
    while (std::getline(ss, line)) {
        if (!line.empty() && line.back() == '\r')
            line.pop_back();
        lines.push_back(line);
    }
    // If source ends with a newline, std::getline drops the trailing empty line.
    // Add it back so EOF tokens on the "last+1" line are within bounds.
    if (!source.empty() && source.back() == '\n') {
        lines.push_back("");
    }
    return lines;
}

static void PopulateStructuredFix(CideDiagnostic& d, const std::string& source) {
    auto lines = SplitSourceLines(source);
    if (d.line <= 0 || d.line > static_cast<int>(lines.size())) {
        return;
    }

    const std::string& srcLine = lines[d.line - 1];

    switch (d.errorCode) {
        case static_cast<int>(cide::ErrorCode::E2005_ExpectedSemicolon):
            d.fixKind = CideFixKind::InsertText;
            if (d.line > 1) {
                const std::string& prev = lines[d.line - 2];
                if (!prev.empty()) {
                    size_t end = prev.size();
                    while (end > 0 && (prev[end - 1] == ' ' || prev[end - 1] == '\t'))
                        --end;
                    if (end > 0) {
                        char last = prev[end - 1];
                        if (last != ';' && last != '{' && last != '}') {
                            d.replaceStartLine = d.line - 1;
                            d.replaceStartColumn = static_cast<int>(end);
                            d.replaceEndLine = d.line - 1;
                            d.replaceEndColumn = static_cast<int>(end);
                            d.replacementText = ";";
                            break;
                        }
                    }
                }
            }
            d.replaceStartLine = d.line;
            d.replaceStartColumn = d.column > 1 ? d.column - 1 : 0;
            d.replaceEndLine = d.line;
            d.replaceEndColumn = d.replaceStartColumn;
            d.replacementText = ";";
            break;

        case static_cast<int>(cide::ErrorCode::E2006_ExpectedClosingBrace):
        case static_cast<int>(cide::ErrorCode::E2007_ExpectedClosingParen):
        case static_cast<int>(cide::ErrorCode::E2008_ExpectedClosingBracket):
            d.fixKind = CideFixKind::InsertText;
            if (d.column > 1) {
                d.replaceStartLine = d.line;
                d.replaceStartColumn = d.column - 1;
                d.replaceEndLine = d.line;
                d.replaceEndColumn = d.column - 1;
            } else if (d.line > 1) {
                const std::string& prevLine = lines[d.line - 2];
                d.replaceStartLine = d.line - 1;
                d.replaceStartColumn = static_cast<int>(prevLine.size());
                d.replaceEndLine = d.line - 1;
                d.replaceEndColumn = static_cast<int>(prevLine.size());
            } else {
                d.replaceStartLine = d.line;
                d.replaceStartColumn = 0;
                d.replaceEndLine = d.line;
                d.replaceEndColumn = 0;
            }
            if (d.errorCode == static_cast<int>(cide::ErrorCode::E2006_ExpectedClosingBrace))
                d.replacementText = "}";
            else if (d.errorCode == static_cast<int>(cide::ErrorCode::E2007_ExpectedClosingParen))
                d.replacementText = ")";
            else
                d.replacementText = "]";
            break;

        case static_cast<int>(cide::ErrorCode::E1004_UnsupportedOp): {
            // Lexer column points AFTER the unsupported char (Advance already consumed it)
            int col = d.column - 1; // 0-based position after the char
            if (col > 0 && col <= static_cast<int>(srcLine.size()) + 1) {
                if (col > 0 && srcLine[col - 1] == '|') {
                    d.fixKind = CideFixKind::ReplaceText;
                    d.replaceStartLine = d.line;
                    d.replaceStartColumn = col - 1;
                    d.replaceEndLine = d.line;
                    d.replaceEndColumn = col;
                    d.replacementText = "||";
                } else if (col > 0 && srcLine[col - 1] == '&') {
                    d.fixKind = CideFixKind::ReplaceText;
                    d.replaceStartLine = d.line;
                    d.replaceStartColumn = col - 1;
                    d.replaceEndLine = d.line;
                    d.replaceEndColumn = col;
                    d.replacementText = "&&";
                }
            }
            break;
        }

        case static_cast<int>(cide::ErrorCode::E1001_UnknownChar):
            // Lexer column points AFTER the unknown char (Advance already consumed it)
            if (d.column > 1) {
                int col = d.column - 2; // 0-based position of the unknown char
                if (col >= 0 && col < static_cast<int>(srcLine.size())) {
                    d.fixKind = CideFixKind::DeleteText;
                    d.replaceStartLine = d.line;
                    d.replaceStartColumn = col;
                    d.replaceEndLine = d.line;
                    d.replaceEndColumn = col + 1;
                    d.replacementText = "";
                }
            }
            break;

        case static_cast<int>(cide::ErrorCode::E1002_UnterminatedString):
            d.fixKind = CideFixKind::InsertText;
            d.replaceStartLine = d.line;
            d.replaceStartColumn = static_cast<int>(srcLine.size());
            d.replaceEndLine = d.line;
            d.replaceEndColumn = static_cast<int>(srcLine.size());
            d.replacementText = "\"";
            break;

        case static_cast<int>(cide::ErrorCode::W3051_ArrayBoundOffByOne): {
            size_t pos = srcLine.find("<=");
            if (pos != std::string::npos) {
                d.fixKind = CideFixKind::ReplaceText;
                d.replaceStartLine = d.line;
                d.replaceStartColumn = static_cast<int>(pos);
                d.replaceEndLine = d.line;
                d.replaceEndColumn = static_cast<int>(pos + 2);
                d.replacementText = "<";
            }
            break;
        }

        case static_cast<int>(cide::ErrorCode::W3050_AssignInCondition): {
            for (size_t i = 0; i < srcLine.size(); ++i) {
                if (srcLine[i] == '=') {
                    bool precededByOp = (i > 0 && (srcLine[i-1] == '=' || srcLine[i-1] == '!' || srcLine[i-1] == '<' || srcLine[i-1] == '>'));
                    bool followedByEq = (i + 1 < srcLine.size() && srcLine[i+1] == '=');
                    if (!precededByOp && !followedByEq) {
                        d.fixKind = CideFixKind::ReplaceText;
                        d.replaceStartLine = d.line;
                        d.replaceStartColumn = static_cast<int>(i);
                        d.replaceEndLine = d.line;
                        d.replaceEndColumn = static_cast<int>(i + 1);
                        d.replacementText = "==";
                        break;
                    }
                }
            }
            break;
        }
    }
}

static CideDiagnostic MakeDiagnostic(int line, int column, int code, int severity,
    const std::string& message, const std::string& fixSuggestion,
    const std::string& source)
{
    CideDiagnostic d;
    d.line = line;
    d.column = column;
    d.errorCode = code;
    d.severity = severity;
    d.message = message;
    d.fixSuggestion = fixSuggestion;
    PopulateStructuredFix(d, source);
    return d;
}

static std::string FormatDiagnostics(const std::vector<cide::LexerError>& lexerErrors,
                                      const std::vector<cide::ParseError>& parseErrors,
                                      const std::vector<cide::TypeError>& typeErrors) {
    std::string result;
    for (const auto& e : lexerErrors) {
        result += "第 " + std::to_string(e.line) + " 行：" + BeautifyCompileError(e.code, e.message) + "\n";
    }
    for (const auto& e : parseErrors) {
        result += "第 " + std::to_string(e.line) + " 行：" + BeautifyCompileError(e.code, e.message) + "\n";
    }
    for (const auto& e : typeErrors) {
        result += "第 " + std::to_string(e.line) + " 行：" + BeautifyCompileError(e.code, e.message) + "\n";
    }
    return result;
}

extern "C" int cide_compile_unit(CideSession* s, const char* filename, const char* source) {
    if (!s || !filename || !source) return -1;
    s->compile.compileUnits.push_back({filename, source});
    return 0;
}

extern "C" int cide_compile_all(CideSession* s) {
    if (!s) return -1;
    if (s->compile.compileUnits.empty()) return -1;

    s->compile.errors.clear();
    s->compile.diagnostics.clear();
    s->compile.bytecode.clear();
    s->compile.globalsInit.clear();
    s->compile.compiled = false;

    // Merge all compile units into a single AST
    cide::ProgramNode merged;
    std::vector<cide::LexerError> allLexerErrors;
    std::vector<cide::ParseError> allParseErrors;

    for (auto& unit : s->compile.compileUnits) {
        cide::Lexer lexer(unit.source.c_str());
        auto tokens = lexer.Tokenize();
        if (lexer.HasErrors()) {
            for (const auto& e : lexer.Errors()) {
                allLexerErrors.push_back(e);
                s->compile.diagnostics.push_back(MakeDiagnostic(e.line, e.column, e.code, 0, e.message, GenerateFixSuggestion(e.code, e.message), unit.source));
            }
            continue;
        }

        cide::Parser parser(std::move(tokens));
        auto ast = parser.Parse();
        if (parser.HasErrors()) {
            for (const auto& e : parser.Errors()) {
                allParseErrors.push_back(e);
                s->compile.diagnostics.push_back(MakeDiagnostic(e.line, e.column, e.code, 0, e.message, GenerateFixSuggestion(e.code, e.message), unit.source));
            }
            continue;
        }

        merged.structs.insert(merged.structs.end(),
            std::make_move_iterator(ast->structs.begin()),
            std::make_move_iterator(ast->structs.end()));
        merged.globals.insert(merged.globals.end(),
            std::make_move_iterator(ast->globals.begin()),
            std::make_move_iterator(ast->globals.end()));
        merged.funcs.insert(merged.funcs.end(),
            std::make_move_iterator(ast->funcs.begin()),
            std::make_move_iterator(ast->funcs.end()));
    }

    if (!allLexerErrors.empty()) {
        s->compile.errors = FormatDiagnostics(allLexerErrors, {}, {});
        return -1;
    }
    if (!allParseErrors.empty()) {
        s->compile.errors = FormatDiagnostics({}, allParseErrors, {});
        return -1;
    }

    // 3. TypeChecker
    cide::TypeChecker checker;
    bool typeCheckOk = checker.Check(merged);
    for (const auto& w : checker.Warnings()) {
        s->compile.diagnostics.push_back(MakeDiagnostic(w.line, w.column, 0, 1, w.message, GenerateFixSuggestion(0, w.message), s->compile.compileUnits[0].source));
    }
    if (!typeCheckOk) {
        for (const auto& e : checker.Errors()) {
            s->compile.diagnostics.push_back(MakeDiagnostic(e.line, e.column, e.code, 0, e.message, GenerateFixSuggestion(e.code, e.message), s->compile.compileUnits[0].source));
        }
        s->compile.errors = FormatDiagnostics({}, {}, checker.Errors());
        return -1;
    }

    // 4. BytecodeGen
    cide::BytecodeGen codegen;
    if (!codegen.Generate(merged)) {
        for (const auto& e : codegen.Errors()) {
            s->compile.diagnostics.push_back({0, 0, 0, 0, e, GenerateFixSuggestion(0, e)});
        }
        s->compile.errors = "代码生成失败：\n";
        for (const auto& e : codegen.Errors()) {
            s->compile.errors += e + "\n";
        }
        return -1;
    }

    auto code = codegen.TakeCode();
    auto funcTable = codegen.GetFuncTable();
    s->compile.globalsInit = codegen.TakeGlobalsInit();
    s->compile.funcIndex = codegen.GetFuncIndex();
    s->compile.stringData = codegen.GetStringData();
    s->compile.sourceMap = codegen.GetSourceMap();
    s->compile.symbols = codegen.TakeSymbols();

    // Phase 4: Algorithm pattern recognition
    cide::AlgorithmMatcher matcher;
    auto matches = matcher.Analyze(merged);
    for (const auto& m : matches) {
        s->compile.algorithmMatches.push_back({m.algorithmName, m.displayName, m.funcName, m.confidence, m.suggestion, m.line, m.visEvents});
    }

    s->compile.bytecode = std::move(code);
    s->compile.funcTable = std::move(funcTable);

    // Save struct field layouts for runtime introspection
    s->compile.structFields.clear();
    for (const auto& [name, fields] : codegen.GetStructDefs()) {
        int offset = 0;
        for (const auto& f : fields) {
            s->compile.structFields[name].push_back({f.name, offset});
            offset += 4; // all fields are 4 bytes in this subset
        }
    }

    s->compile.compiled = true;
    return 0;
}

extern "C" int cide_compile(CideSession* s, const char* source) {
    if (!s || !source) return -1;
    s->compile.compileUnits.clear();
    s->compile.compileUnits.push_back({"main.c", source});
    return cide_compile_all(s);
}

extern "C" const char* cide_get_compile_errors(CideSession* s) {
    if (!s || s->compile.errors.empty()) return nullptr;
    // Only reassign if content changed to avoid unnecessary reallocations
    if (s->compile.errorsBuffer != s->compile.errors) {
        s->compile.errorsBuffer.assign(s->compile.errors);
    }
    return s->compile.errorsBuffer.c_str();
}

extern "C" int cide_get_compile_errors_buf(CideSession* s, char* buf, int max_len) {
    if (!s || !buf || max_len <= 0) return -1;
    if (s->compile.errors.empty()) {
        buf[0] = '\0';
        return 0;
    }
    int copyLen = static_cast<int>(s->compile.errors.size());
    if (copyLen >= max_len) copyLen = max_len - 1;
    std::memcpy(buf, s->compile.errors.c_str(), copyLen);
    buf[copyLen] = '\0';
    return copyLen;
}

// ============================================================================
// Execution
// ============================================================================

extern "C" int cide_run(CideSession* s) {
    if (!s || !s->compile.compiled) {
        if (s) s->runtime.error = "程序尚未编译。请先编译代码。";
        return -1;
    }

    s->runtime.outputLines.clear();
    s->runtime.error.clear();
    s->runtime.trace.clear();
    s->memory.regions.clear();
    s->memory.freeList.clear();
    s->memory.heapOffset = 0x5000;
    s->memory.allocCounter = 0;
    s->runtime.running = true;
    s->runtime.stepMode = false;

    SetupVM(s);
    HostCtx ctx{s, &s->vm};
    cide::HostFunctions::RegisterAll(s, &s->vm, &ctx);

    // Register functions in VM
    // Note: BytecodeGen embeds function IPs directly in Call instructions.
    // The VM doesn't need a separate func table for user functions if IPs are absolute.
    // But we keep the func table mechanism for future use.

    // Run
    int32_t retValue = s->vm.Run();

    if (s->vm.HasError()) {
        s->runtime.error = s->vm.GetError();
        s->runtime.running = false;
        return -1;
    }

    s->runtime.outputLines.push_back("程序运行完成，返回值：" + std::to_string(retValue) + "\n");
    s->runtime.running = false;
    return 0;
}

extern "C" int cide_step_next(CideSession* s) {
    if (!s || !s->compile.compiled) {
        if (s) s->runtime.error = "程序尚未编译。";
        return -1;
    }

    if (!s->runtime.running) {
        // First step: setup VM
        s->runtime.outputLines.clear();
        s->runtime.error.clear();
        s->runtime.trace.clear();
        s->memory.regions.clear();
        s->memory.freeList.clear();
        s->memory.heapOffset = 0x5000;
        s->memory.allocCounter = 0;
        s->runtime.stepCount = 0;
        s->runtime.stepMode = true;
        s->runtime.running = true;

        SetupVM(s);
        s->vm.Pause(); // Start paused so first Step() hits StepEvent and returns Paused

        HostCtx ctx{s, &s->vm};
    cide::HostFunctions::RegisterAll(s, &s->vm, &ctx);

        // Execute until first StepEvent
        while (true) {
            auto result = s->vm.Step();
            if (result == cide::CideVM::StepResult::Paused) {
                s->runtime.currentLine = s->vm.GetCurrentLine();
                return 0;
            }
            if (result == cide::CideVM::StepResult::Finished) {
                s->runtime.running = false;
                s->runtime.currentLine = s->vm.GetCurrentLine();
                return -1;
            }
            if (result == cide::CideVM::StepResult::Trap) {
                s->runtime.error = s->vm.GetError();
                s->runtime.running = false;
                s->runtime.currentLine = s->vm.GetCurrentLine();
                return -1;
            }
        }
    }

    // Continue to next step
    s->vm.Resume();
    while (true) {
        auto result = s->vm.Step();
        if (result == cide::CideVM::StepResult::Paused) {
            s->runtime.currentLine = s->vm.GetCurrentLine();
            return 0;
        }
        if (s->vm.WasStepEventHit()) {
            s->vm.Pause();
            s->runtime.currentLine = s->vm.GetCurrentLine();
            return 0;
        }
        if (result == cide::CideVM::StepResult::Finished) {
            s->runtime.running = false;
            s->runtime.currentLine = s->vm.GetCurrentLine();
            return -1;
        }
        if (result == cide::CideVM::StepResult::Trap) {
            s->runtime.error = s->vm.GetError();
            s->runtime.running = false;
            s->runtime.currentLine = s->vm.GetCurrentLine();
            return -1;
        }
    }
}

extern "C" int cide_get_current_line(CideSession* s) {
    if (!s) return 0;
    return s->runtime.currentLine;
}

// ============================================================================
// Breakpoints
// ============================================================================

extern "C" void cide_breakpoint_add(CideSession* s, int line) {
    if (!s || line <= 0) return;
    s->vm.AddBreakpoint(line);
}

extern "C" void cide_breakpoint_remove(CideSession* s, int line) {
    if (!s || line <= 0) return;
    s->vm.RemoveBreakpoint(line);
}

extern "C" void cide_breakpoint_clear(CideSession* s) {
    if (!s) return;
    s->vm.ClearBreakpoints();
}

// ============================================================================
// Call Stack
// ============================================================================

extern "C" int cide_callstack_count(CideSession* s) {
    if (!s) return 0;
    return static_cast<int>(s->vm.GetCallStack().size());
}

extern "C" void cide_callstack_get(
    CideSession* s, int index,
    char* name, int name_size,
    int* line) {
    if (!s || index < 0 || index >= static_cast<int>(s->vm.GetCallStack().size())) {
        if (name && name_size > 0) name[0] = '\0';
        if (line) *line = 0;
        return;
    }

    const auto& frame = s->vm.GetCallStack()[index];
    // Look up source line from returnIP via source map
    int bestLine = 0;
    if (!s->compile.sourceMap.empty() && frame.returnIP > 0) {
        uint32_t retIP = static_cast<uint32_t>(frame.returnIP);
        const auto& map = s->compile.sourceMap;
        auto it = std::upper_bound(map.begin(), map.end(), retIP,
            [](uint32_t val, const auto& pair) { return val < pair.first; });
        if (it != map.begin()) {
            --it;
            bestLine = it->second.line;
        }
    }

    if (name && name_size > 0) {
        size_t copied = frame.funcName.copy(name, name_size - 1);
        name[copied] = '\0';
    }
    if (line) *line = bestLine;
}

extern "C" const char* cide_get_runtime_error(CideSession* s) {
    if (!s || s->runtime.error.empty()) return nullptr;
    return s->runtime.error.c_str();
}

extern "C" int cide_get_runtime_error_buf(CideSession* s, char* buf, int max_len) {
    if (!s || !buf || max_len <= 0) return -1;
    if (s->runtime.error.empty()) {
        buf[0] = '\0';
        return 0;
    }
    int copyLen = static_cast<int>(s->runtime.error.size());
    if (copyLen >= max_len) copyLen = max_len - 1;
    std::memcpy(buf, s->runtime.error.c_str(), copyLen);
    buf[copyLen] = '\0';
    return copyLen;
}

// ============================================================================
// Output
// ============================================================================

extern "C" int cide_get_output_length(CideSession* s) {
    if (!s) return 0;
    int total = 0;
    for (const auto& line : s->runtime.outputLines) {
        total += static_cast<int>(line.size());
    }
    return total;
}

extern "C" void cide_get_output(CideSession* s, char* buf, int max_len) {
    if (!s || !buf || max_len <= 0) return;

    std::string all;
    for (const auto& line : s->runtime.outputLines) {
        all += line;
    }

    int copyLen = static_cast<int>(all.size());
    if (copyLen >= max_len) copyLen = max_len - 1;

    std::memcpy(buf, all.c_str(), copyLen);
    buf[copyLen] = '\0';
}

// ============================================================================
// Memory View
// ============================================================================

extern "C" int cide_memory_region_count(CideSession* s) {
    if (!s) return 0;
    return static_cast<int>(s->memory.regions.size());
}

extern "C" void cide_memory_region_get(
    CideSession* s, int index,
    unsigned int* addr, int* size,
    char* name, int name_size,
    char* type, int type_size,
    int* is_heap, int* is_freed) {

    if (!s || index < 0 || index >= static_cast<int>(s->memory.regions.size())) {
        if (addr) *addr = 0;
        if (size) *size = 0;
        if (is_heap) *is_heap = 0;
        if (is_freed) *is_freed = 0;
        return;
    }

    const auto& r = s->memory.regions[index];
    if (addr) *addr = r.addr;
    if (size) *size = r.size;
    if (name && name_size > 0) {
        size_t copied = r.name.copy(name, name_size - 1);
        name[copied] = '\0';
    }
    if (type && type_size > 0) {
        size_t copied = r.type.copy(type, type_size - 1);
        type[copied] = '\0';
    }
    if (is_heap) *is_heap = r.isHeap ? 1 : 0;
    if (is_freed) *is_freed = r.isFreed ? 1 : 0;
}

extern "C" int cide_memory_get_value(CideSession* s, unsigned int addr, int* out_val) {
    if (!s || !out_val) return -1;
    uint8_t* mem = s->vm.GetMemory();
    uint32_t memSize = s->vm.GetMemorySize();
    if (mem && addr + 4 <= memSize) {
        std::memcpy(out_val, mem + addr, sizeof(int32_t));
        return 0;
    }
    *out_val = 0;
    return -1;
}

extern "C" int cide_memory_get_pointer_target(CideSession* s, unsigned int addr, unsigned int* out_target) {
    if (!s || !out_target) return -1;
    *out_target = 0;
    uint8_t* mem = s->vm.GetMemory();
    uint32_t memSize = s->vm.GetMemorySize();
    if (mem && addr + 4 <= memSize) {
        std::memcpy(out_target, mem + addr, sizeof(uint32_t));
        return 0;
    }
    return -1;
}

// ============================================================================
// Diagnostics
// ============================================================================

extern "C" int cide_diagnostic_count(CideSession* s) {
    if (!s) return 0;
    return static_cast<int>(s->compile.diagnostics.size());
}

extern "C" void cide_diagnostic_get(
    CideSession* s, int index,
    int* line, int* column, int* error_code, int* severity,
    char* message, int msg_size,
    char* fix_suggestion, int fix_size) {

    if (!s || index < 0 || index >= static_cast<int>(s->compile.diagnostics.size())) {
        if (line) *line = 0;
        if (column) *column = 0;
        if (error_code) *error_code = 0;
        if (severity) *severity = 0;
        return;
    }

    const auto& d = s->compile.diagnostics[index];
    if (line) *line = d.line;
    if (column) *column = d.column;
    if (error_code) *error_code = d.errorCode;
    if (severity) *severity = d.severity;
    if (message && msg_size > 0) {
        size_t copied = d.message.copy(message, msg_size - 1);
        message[copied] = '\0';
    }
    if (fix_suggestion && fix_size > 0) {
        size_t copied = d.fixSuggestion.copy(fix_suggestion, fix_size - 1);
        fix_suggestion[copied] = '\0';
    }
}

extern "C" void cide_diagnostic_get_fix(
    CideSession* s, int index,
    int* fix_kind,
    int* start_line, int* start_column,
    int* end_line, int* end_column,
    char* replacement_text, int replacement_size) {

    if (!s || index < 0 || index >= static_cast<int>(s->compile.diagnostics.size())) {
        if (fix_kind) *fix_kind = 0;
        if (start_line) *start_line = 0;
        if (start_column) *start_column = 0;
        if (end_line) *end_line = 0;
        if (end_column) *end_column = 0;
        return;
    }

    const auto& d = s->compile.diagnostics[index];
    if (fix_kind) *fix_kind = static_cast<int>(d.fixKind);
    if (start_line) *start_line = d.replaceStartLine;
    if (start_column) *start_column = d.replaceStartColumn;
    if (end_line) *end_line = d.replaceEndLine;
    if (end_column) *end_column = d.replaceEndColumn;
    if (replacement_text && replacement_size > 0) {
        size_t copied = d.replacementText.copy(replacement_text, replacement_size - 1);
        replacement_text[copied] = '\0';
    }
}

// ============================================================================
// Source Map
// ============================================================================

extern "C" int cide_sourcemap_lookup(
    CideSession* s, unsigned int bytecode_offset,
    int* out_line, int* out_column) {

    if (!s || !out_line || !out_column) return -1;

    const auto& map = s->compile.sourceMap;
    if (map.empty()) return -1;

    // Binary search: find the last entry with ip <= bytecode_offset
    auto it = std::upper_bound(map.begin(), map.end(), bytecode_offset,
        [](uint32_t val, const auto& pair) { return val < pair.first; });

    if (it == map.begin()) return -1;
    --it;

    *out_line = it->second.line;
    *out_column = it->second.column;
    return 0;
}

// ============================================================================
// Execution Trace
// ============================================================================

extern "C" int cide_trace_count(CideSession* s) {
    if (!s) return 0;
    return static_cast<int>(s->runtime.trace.size());
}

extern "C" void cide_trace_get(
    CideSession* s, int index,
    int* line, char* operation, int op_size) {

    if (!s || index < 0 || index >= static_cast<int>(s->runtime.trace.size())) {
        if (line) *line = 0;
        return;
    }

    const auto& t = s->runtime.trace[index];
    if (line) *line = t.line;
    if (operation && op_size > 0) {
        size_t copied = t.operation.copy(operation, op_size - 1);
        operation[copied] = '\0';
    }
}

// ============================================================================
// Input
// ============================================================================

extern "C" void cide_set_input(CideSession* s, const char* input) {
    if (!s || !input) return;
    s->runtime.inputLines.clear();
    s->runtime.inputIndex = 0;
    std::string_view view(input);
    size_t start = 0;
    while (start <= view.size()) {
        size_t end = view.find('\n', start);
        std::string line;
        if (end == std::string_view::npos) {
            line = std::string(view.substr(start));
        } else {
            line = std::string(view.substr(start, end - start));
        }
        if (!line.empty() && line.back() == '\r')
            line.pop_back();
        s->runtime.inputLines.emplace_back(std::move(line));
        if (end == std::string_view::npos) break;
        start = end + 1;
    }
}

extern "C" int cide_input_count(CideSession* s) {
    if (!s) return 0;
    return static_cast<int>(s->runtime.inputLines.size());
}

// ============================================================================
// Variable Panel (Stage 3)
// ============================================================================

extern "C" int cide_variable_count(CideSession* s) {
    if (!s) return 0;
    s->runtime.variableSnapshot = s->vm.GetVariableSnapshot();
    return static_cast<int>(s->runtime.variableSnapshot.size());
}

extern "C" void cide_variable_get(
    CideSession* s, int index,
    char* name, int name_size,
    unsigned int* addr,
    int* is_local, int* is_array, int* array_size,
    int* value) {

    if (!s || index < 0 || index >= static_cast<int>(s->runtime.variableSnapshot.size())) {
        if (name && name_size > 0) name[0] = '\0';
        if (addr) *addr = 0;
        if (is_local) *is_local = 0;
        if (is_array) *is_array = 0;
        if (array_size) *array_size = 0;
        if (value) *value = 0;
        return;
    }

    const auto& v = s->runtime.variableSnapshot[index];
    if (name && name_size > 0) {
        size_t copied = v.name.copy(name, name_size - 1);
        name[copied] = '\0';
    }
    if (addr) *addr = v.addr;
    if (is_local) *is_local = v.isLocal ? 1 : 0;
    if (is_array) *is_array = v.type.isArray() ? 1 : 0;
    if (array_size) *array_size = v.type.isArray() ? v.type.arraySize : 0;
    if (value) *value = v.value;
}

static std::string FormatType(const cide::Type& t) {
    switch (t.kind) {
        case cide::TypeKind::Int: return "int";
        case cide::TypeKind::Char: return "char";
        case cide::TypeKind::Void: return "void";
        case cide::TypeKind::Struct: return "struct " + t.name;
        case cide::TypeKind::Pointer: {
            std::string base = FormatType(cide::Type{t.baseKind, t.name, 0, cide::TypeKind::Void});
            return base + "*";
        }
        case cide::TypeKind::Array: {
            std::string base = FormatType(cide::Type{t.baseKind, t.name, 0, cide::TypeKind::Void});
            return base + "[" + std::to_string(t.arraySize) + "]";
        }
    }
    return "unknown";
}

extern "C" int cide_variable_get_type(
    CideSession* s, int index,
    char* type_buf, int type_buf_size) {

    if (!s || index < 0 || index >= static_cast<int>(s->runtime.variableSnapshot.size())) {
        if (type_buf && type_buf_size > 0) type_buf[0] = '\0';
        return -1;
    }
    const auto& v = s->runtime.variableSnapshot[index];
    std::string typeStr = FormatType(v.type);
    if (type_buf && type_buf_size > 0) {
        size_t copied = typeStr.copy(type_buf, type_buf_size - 1);
        type_buf[copied] = '\0';
    }
    return static_cast<int>(typeStr.size());
}

extern "C" int cide_variable_find_by_addr(
    CideSession* s, unsigned int addr,
    char* name, int name_size,
    int* offset) {

    if (!s || !name || name_size <= 0) return -1;
    auto vars = s->vm.GetVariableSnapshot();
    const cide::CideVM::VMVariableSnapshot* best = nullptr;
    uint32_t bestBase = 0;
    for (const auto& v : vars) {
        uint32_t base = v.addr;
        uint32_t size = v.type.isArray() ? static_cast<uint32_t>(v.type.arraySize) * 4u : 4u;
        if (addr >= base && addr < base + size) {
            best = &v;
            bestBase = base;
            break; // exact match, prefer first
        }
    }
    if (!best) {
        name[0] = '\0';
        if (offset) *offset = 0;
        return -1;
    }
    size_t copied = best->name.copy(name, name_size - 1);
    name[copied] = '\0';
    if (offset) *offset = static_cast<int>(addr - bestBase);
    return 0;
}

extern "C" int cide_variable_get_field(
    CideSession* s, int var_index, int field_index,
    int* offset, char* name, int name_size) {

    if (!s || var_index < 0 || field_index < 0) return -1;
    if (var_index >= static_cast<int>(s->runtime.variableSnapshot.size())) return -1;

    const auto& v = s->runtime.variableSnapshot[var_index];
    std::string structName;
    if (v.type.kind == cide::TypeKind::Struct) {
        structName = v.type.name;
    } else if (v.type.kind == cide::TypeKind::Pointer && v.type.baseKind == cide::TypeKind::Struct) {
        structName = v.type.name;
    } else {
        return -1; // not a struct or pointer-to-struct
    }

    auto it = s->compile.structFields.find(structName);
    if (it == s->compile.structFields.end()) return -1;
    if (field_index >= static_cast<int>(it->second.size())) return -1;

    const auto& field = it->second[field_index];
    if (offset) *offset = field.second;
    if (name && name_size > 0) {
        size_t copied = field.first.copy(name, name_size - 1);
        name[copied] = '\0';
    }
    return 0;
}

// ============================================================================
// Runtime Vis Events (Stage 4)
// ============================================================================

static void RefreshVisEvents(CideSession* s) {
    if (!s) return;
    auto events = s->vm.TakeVisEvents();
    s->runtime.visEventCache.insert(s->runtime.visEventCache.end(), events.begin(), events.end());
}

extern "C" int cide_vis_event_count(CideSession* s) {
    if (!s) return 0;
    RefreshVisEvents(s);
    return static_cast<int>(s->runtime.visEventCache.size());
}

extern "C" void cide_vis_event_get(CideSession* s, int index, int* type, int* line) {
    if (!s) {
        if (type) *type = 0;
        if (line) *line = 0;
        return;
    }
    RefreshVisEvents(s);
    if (index < 0 || index >= static_cast<int>(s->runtime.visEventCache.size())) {
        if (type) *type = 0;
        if (line) *line = 0;
        return;
    }
    const auto& e = s->runtime.visEventCache[index];
    if (type) *type = e.type;
    if (line) *line = e.line;
}

extern "C" void cide_vis_event_get_ex(CideSession* s, int index,
                                          int* type, int* line,
                                          int* extra0, int* extra1, int* extra2) {
    if (!s) {
        if (type) *type = 0;
        if (line) *line = 0;
        if (extra0) *extra0 = 0;
        if (extra1) *extra1 = 0;
        if (extra2) *extra2 = 0;
        return;
    }
    RefreshVisEvents(s);
    if (index < 0 || index >= static_cast<int>(s->runtime.visEventCache.size())) {
        if (type) *type = 0;
        if (line) *line = 0;
        if (extra0) *extra0 = 0;
        if (extra1) *extra1 = 0;
        if (extra2) *extra2 = 0;
        return;
    }
    const auto& e = s->runtime.visEventCache[index];
    if (type) *type = e.type;
    if (line) *line = e.line;
    if (extra0) *extra0 = e.extra[0];
    if (extra1) *extra1 = e.extra[1];
    if (extra2) *extra2 = e.extra[2];
}

extern "C" void cide_vis_event_clear(CideSession* s) {
    if (!s) return;
    (void)s->vm.TakeVisEvents();
    s->runtime.visEventCache.clear();
}

// ============================================================================
// Algorithm Pattern Recognition (Phase 4)
// ============================================================================

extern "C" int cide_algorithm_match_count(CideSession* s) {
    if (!s) return 0;
    return static_cast<int>(s->compile.algorithmMatches.size());
}

extern "C" void cide_algorithm_match_get(
    CideSession* s, int index,
    char* name, int name_size,
    char* display_name, int display_name_size,
    char* func_name, int func_name_size,
    int* confidence,
    char* suggestion, int suggestion_size,
    int* line) {

    if (!s || index < 0 || index >= static_cast<int>(s->compile.algorithmMatches.size())) {
        if (name && name_size > 0) name[0] = '\0';
        if (display_name && display_name_size > 0) display_name[0] = '\0';
        if (func_name && func_name_size > 0) func_name[0] = '\0';
        if (confidence) *confidence = 0;
        if (suggestion && suggestion_size > 0) suggestion[0] = '\0';
        if (line) *line = 0;
        return;
    }

    const auto& m = s->compile.algorithmMatches[index];
    if (name && name_size > 0) {
        size_t copied = m.name.copy(name, name_size - 1);
        name[copied] = '\0';
    }
    if (display_name && display_name_size > 0) {
        size_t copied = m.displayName.copy(display_name, display_name_size - 1);
        display_name[copied] = '\0';
    }
    if (func_name && func_name_size > 0) {
        size_t copied = m.funcName.copy(func_name, func_name_size - 1);
        func_name[copied] = '\0';
    }
    if (confidence) *confidence = m.confidence;
    if (suggestion && suggestion_size > 0) {
        size_t copied = m.suggestion.copy(suggestion, suggestion_size - 1);
        suggestion[copied] = '\0';
    }
    if (line) *line = m.line;
}

extern "C" int cide_algorithm_match_vis_event_count(CideSession* s, int match_index) {
    if (!s || match_index < 0 || match_index >= static_cast<int>(s->compile.algorithmMatches.size()))
        return 0;
    return static_cast<int>(s->compile.algorithmMatches[match_index].visEvents.size());
}

extern "C" void cide_algorithm_match_vis_event_get(
    CideSession* s, int match_index, int event_index,
    int* type, int* line, char* context, int context_size) {
    if (!s || match_index < 0 || match_index >= static_cast<int>(s->compile.algorithmMatches.size())) {
        if (type) *type = 0;
        if (line) *line = 0;
        if (context && context_size > 0) context[0] = '\0';
        return;
    }
    const auto& m = s->compile.algorithmMatches[match_index];
    if (event_index < 0 || event_index >= static_cast<int>(m.visEvents.size())) {
        if (type) *type = 0;
        if (line) *line = 0;
        if (context && context_size > 0) context[0] = '\0';
        return;
    }
    const auto& ev = m.visEvents[event_index];
    if (type) *type = std::get<1>(ev);
    if (line) *line = std::get<0>(ev);
    if (context && context_size > 0) {
        const std::string& ctx = std::get<2>(ev);
        size_t copied = ctx.copy(context, context_size - 1);
        context[copied] = '\0';
    }
}


// ============================================================================
// Session Serialization (R7)
// ============================================================================

#include <fstream>

namespace {

static constexpr const char* kSessionMagic = "CIDESV01";

// --- Write helpers ---
void WriteU8(std::ofstream& f, uint8_t v) { f.put(static_cast<char>(v)); }
void WriteU32(std::ofstream& f, uint32_t v) { f.write(reinterpret_cast<const char*>(&v), 4); }
void WriteI32(std::ofstream& f, int32_t v) { f.write(reinterpret_cast<const char*>(&v), 4); }
void WriteU64(std::ofstream& f, uint64_t v) { f.write(reinterpret_cast<const char*>(&v), 8); }
void WriteStr(std::ofstream& f, const std::string& s) {
    WriteU32(f, static_cast<uint32_t>(s.size()));
    f.write(s.c_str(), static_cast<std::streamsize>(s.size()));
}
void WriteType(std::ofstream& f, const cide::Type& t) {
    WriteU8(f, static_cast<uint8_t>(t.kind));
    WriteStr(f, t.name);
    WriteI32(f, t.arraySize);
    WriteU8(f, static_cast<uint8_t>(t.baseKind));
    WriteU32(f, static_cast<uint32_t>(t.dims.size()));
    for (int d : t.dims) {
        WriteI32(f, d);
    }
}

// --- Read helpers ---
uint8_t ReadU8(std::ifstream& f) { return static_cast<uint8_t>(f.get()); }
uint32_t ReadU32(std::ifstream& f) { uint32_t v; f.read(reinterpret_cast<char*>(&v), 4); return v; }
int32_t ReadI32(std::ifstream& f) { int32_t v; f.read(reinterpret_cast<char*>(&v), 4); return v; }
uint64_t ReadU64(std::ifstream& f) { uint64_t v; f.read(reinterpret_cast<char*>(&v), 8); return v; }
static constexpr uint32_t kMaxDeserializeCount = 10'000'000;
static constexpr uint32_t kMaxDeserializeStrLen = 10 * 1024 * 1024;

std::string ReadStr(std::ifstream& f, uint32_t maxLen = kMaxDeserializeStrLen) {
    uint32_t len = ReadU32(f);
    if (len > maxLen) {
        throw std::runtime_error("DeserializeSession: string length exceeds limit");
    }
    std::string s(len, '\0');
    if (len > 0) {
        f.read(s.data(), static_cast<std::streamsize>(len));
        if (static_cast<uint32_t>(f.gcount()) != len) {
            throw std::runtime_error("DeserializeSession: truncated string data");
        }
    }
    return s;
}
cide::Type ReadType(std::ifstream& f) {
    cide::Type t;
    t.kind = static_cast<cide::TypeKind>(ReadU8(f));
    t.name = ReadStr(f);
    t.arraySize = ReadI32(f);
    t.baseKind = static_cast<cide::TypeKind>(ReadU8(f));
    uint32_t dimCount = ReadU32(f);
    if (dimCount > 0 && dimCount <= 16) {
        t.dims.resize(dimCount);
        for (uint32_t i = 0; i < dimCount; ++i) {
            t.dims[i] = ReadI32(f);
        }
    }
    return t;
}

bool SerializeSession(CideSession* s, const char* filepath) {
    std::ofstream f(filepath, std::ios::binary);
    if (!f) return false;

    f.write(kSessionMagic, 8);

    // --- CompileUnits ---
    WriteU32(f, static_cast<uint32_t>(s->compile.compileUnits.size()));
    for (auto& u : s->compile.compileUnits) {
        WriteStr(f, u.filename);
        WriteStr(f, u.source);
    }

    // --- Bytecode ---
    WriteU32(f, static_cast<uint32_t>(s->compile.bytecode.size()));
    for (auto& inst : s->compile.bytecode) {
        WriteU8(f, static_cast<uint8_t>(inst.op));
        WriteI32(f, inst.operand);
        WriteI32(f, inst.loc.line);
        WriteI32(f, inst.loc.column);
    }

    // --- GlobalsInit ---
    WriteU32(f, static_cast<uint32_t>(s->compile.globalsInit.size()));
    for (auto v : s->compile.globalsInit) {
        WriteI32(f, v);
    }

    // --- FuncTable ---
    WriteU32(f, static_cast<uint32_t>(s->compile.funcTable.size()));
    for (auto& kv : s->compile.funcTable) {
        WriteStr(f, kv.first);
        WriteU64(f, kv.second.ip);
        WriteI32(f, kv.second.argCount);
        WriteI32(f, kv.second.localCount);
    }

    // --- FuncIndex ---
    WriteU32(f, static_cast<uint32_t>(s->compile.funcIndex.size()));
    for (auto& kv : s->compile.funcIndex) {
        WriteStr(f, kv.first);
        WriteI32(f, kv.second);
    }

    // --- StringData ---
    WriteU32(f, static_cast<uint32_t>(s->compile.stringData.size()));
    for (auto& kv : s->compile.stringData) {
        WriteU32(f, kv.first);
        WriteStr(f, kv.second);
    }

    // --- SourceMap ---
    WriteU32(f, static_cast<uint32_t>(s->compile.sourceMap.size()));
    for (auto& kv : s->compile.sourceMap) {
        WriteU32(f, kv.first);
        WriteI32(f, kv.second.line);
        WriteI32(f, kv.second.column);
    }

    // --- Symbols ---
    WriteU32(f, static_cast<uint32_t>(s->compile.symbols.size()));
    for (auto& sym : s->compile.symbols) {
        WriteStr(f, sym.name);
        WriteU32(f, sym.addr);
        WriteU8(f, sym.isLocal ? 1 : 0);
        WriteType(f, sym.type);
        WriteI32(f, sym.scopeDepth);
    }

    // --- AlgorithmMatches ---
    WriteU32(f, static_cast<uint32_t>(s->compile.algorithmMatches.size()));
    for (auto& m : s->compile.algorithmMatches) {
        WriteStr(f, m.name);
        WriteStr(f, m.displayName);
        WriteStr(f, m.funcName);
        WriteI32(f, m.confidence);
        WriteStr(f, m.suggestion);
        WriteI32(f, m.line);
        WriteU32(f, static_cast<uint32_t>(m.visEvents.size()));
        for (auto& ev : m.visEvents) {
            WriteI32(f, std::get<0>(ev));
            WriteI32(f, std::get<1>(ev));
            WriteStr(f, std::get<2>(ev));
        }
    }

    // --- RuntimeState (minimal) ---
    WriteI32(f, s->runtime.currentLine);
    WriteU8(f, s->runtime.stepMode ? 1 : 0);
    WriteI32(f, s->runtime.stepCount);
    WriteU32(f, static_cast<uint32_t>(s->runtime.inputIndex));
    WriteU32(f, static_cast<uint32_t>(s->runtime.inputLines.size()));
    for (auto& line : s->runtime.inputLines) {
        WriteStr(f, line);
    }

    // --- MemoryState ---
    WriteU32(f, s->memory.heapOffset);
    WriteI32(f, s->memory.allocCounter);
    WriteU32(f, static_cast<uint32_t>(s->memory.regions.size()));
    for (auto& r : s->memory.regions) {
        WriteU32(f, r.addr);
        WriteI32(f, r.size);
        WriteStr(f, r.name);
        WriteStr(f, r.type);
        WriteU8(f, r.isHeap ? 1 : 0);
        WriteU8(f, r.isFreed ? 1 : 0);
    }

    return f.good();
}

bool DeserializeSession(CideSession* s, const char* filepath) {
    std::ifstream f(filepath, std::ios::binary);
    if (!f) return false;

    try {
        char magic[8];
        f.read(magic, 8);
        if (!f || std::memcmp(magic, kSessionMagic, 8) != 0) return false;

        // Reset current state
        s->compile = CideCompileState{};
        s->runtime = CideRuntimeState{};
        s->memory = CideMemoryState{};

        // --- CompileUnits ---
        uint32_t unitCount = ReadU32(f);
        if (unitCount > kMaxDeserializeCount) return false;
        for (uint32_t i = 0; i < unitCount; i++) {
            auto filename = ReadStr(f);
            auto source = ReadStr(f);
            s->compile.compileUnits.push_back({filename, source});
        }

        // --- Bytecode ---
        uint32_t bcCount = ReadU32(f);
        if (bcCount > kMaxDeserializeCount) return false;
        s->compile.bytecode.reserve(bcCount);
        for (uint32_t i = 0; i < bcCount; i++) {
            cide::OpCode op = static_cast<cide::OpCode>(ReadU8(f));
            int32_t operand = ReadI32(f);
            cide::SourceLoc loc{ReadI32(f), ReadI32(f)};
            s->compile.bytecode.emplace_back(op, operand, loc);
        }

        // --- GlobalsInit ---
        uint32_t giCount = ReadU32(f);
        if (giCount > kMaxDeserializeCount) return false;
        s->compile.globalsInit.reserve(giCount);
        for (uint32_t i = 0; i < giCount; i++) {
            s->compile.globalsInit.push_back(ReadI32(f));
        }

        // --- FuncTable ---
        uint32_t ftCount = ReadU32(f);
        if (ftCount > kMaxDeserializeCount) return false;
        for (uint32_t i = 0; i < ftCount; i++) {
            auto name = ReadStr(f);
            cide::FuncMeta meta;
            meta.ip = static_cast<size_t>(ReadU64(f));
            meta.argCount = ReadI32(f);
            meta.localCount = ReadI32(f);
            s->compile.funcTable[name] = meta;
        }

        // --- FuncIndex ---
        uint32_t fiCount = ReadU32(f);
        if (fiCount > kMaxDeserializeCount) return false;
        for (uint32_t i = 0; i < fiCount; i++) {
            auto name = ReadStr(f);
            s->compile.funcIndex[name] = ReadI32(f);
        }

        // --- StringData ---
        uint32_t sdCount = ReadU32(f);
        if (sdCount > kMaxDeserializeCount) return false;
        for (uint32_t i = 0; i < sdCount; i++) {
            uint32_t addr = ReadU32(f);
            auto str = ReadStr(f);
            s->compile.stringData.push_back({addr, str});
        }

        // --- SourceMap ---
        uint32_t smCount = ReadU32(f);
        if (smCount > kMaxDeserializeCount) return false;
        for (uint32_t i = 0; i < smCount; i++) {
            uint32_t ip = ReadU32(f);
            cide::SourceLoc loc{ReadI32(f), ReadI32(f)};
            s->compile.sourceMap.push_back({ip, loc});
        }

        // --- Symbols ---
        uint32_t symCount = ReadU32(f);
        if (symCount > kMaxDeserializeCount) return false;
        for (uint32_t i = 0; i < symCount; i++) {
            cide::VMSymbol sym;
            sym.name = ReadStr(f);
            sym.addr = ReadU32(f);
            sym.isLocal = ReadU8(f) != 0;
            sym.type = ReadType(f);
            sym.scopeDepth = ReadI32(f);
            s->compile.symbols.push_back(sym);
        }

        // --- AlgorithmMatches ---
        uint32_t amCount = ReadU32(f);
        if (amCount > kMaxDeserializeCount) return false;
        for (uint32_t i = 0; i < amCount; i++) {
            CideAlgorithmMatch m;
            m.name = ReadStr(f);
            m.displayName = ReadStr(f);
            m.funcName = ReadStr(f);
            m.confidence = ReadI32(f);
            m.suggestion = ReadStr(f);
            m.line = ReadI32(f);
            uint32_t veCount = ReadU32(f);
            if (veCount > kMaxDeserializeCount) return false;
            for (uint32_t j = 0; j < veCount; j++) {
                int type = ReadI32(f);
                int line = ReadI32(f);
                std::string ctx = ReadStr(f);
                m.visEvents.push_back({type, line, ctx});
            }
            s->compile.algorithmMatches.push_back(m);
        }

        // --- RuntimeState ---
        s->runtime.currentLine = ReadI32(f);
        s->runtime.stepMode = ReadU8(f) != 0;
        s->runtime.stepCount = ReadI32(f);
        s->runtime.inputIndex = ReadU32(f);
        uint32_t inputCount = ReadU32(f);
        if (inputCount > kMaxDeserializeCount) return false;
        for (uint32_t i = 0; i < inputCount; i++) {
            s->runtime.inputLines.push_back(ReadStr(f));
        }

        // --- MemoryState ---
        s->memory.heapOffset = ReadU32(f);
        s->memory.allocCounter = ReadI32(f);
        uint32_t regionCount = ReadU32(f);
        if (regionCount > kMaxDeserializeCount) return false;
        for (uint32_t i = 0; i < regionCount; i++) {
            CideMemoryRegion r;
            r.addr = ReadU32(f);
            r.size = ReadI32(f);
            r.name = ReadStr(f);
            r.type = ReadStr(f);
            r.isHeap = ReadU8(f) != 0;
            r.isFreed = ReadU8(f) != 0;
            s->memory.regions.push_back(r);
        }

        s->compile.compiled = true;
        return f.good();
    } catch (const std::exception&) {
        // Reset to clean state on any deserialization error
        s->compile = CideCompileState{};
        s->runtime = CideRuntimeState{};
        s->memory = CideMemoryState{};
        return false;
    }
}

} // anonymous namespace

extern "C" int cide_session_save(CideSession* s, const char* filepath) {
    if (!s || !filepath) return -1;
    return SerializeSession(s, filepath) ? 0 : -1;
}

extern "C" int cide_session_load(CideSession* s, const char* filepath) {
    if (!s || !filepath) return -1;
    return DeserializeSession(s, filepath) ? 0 : -1;
}
