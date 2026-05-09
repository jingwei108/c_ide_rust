#include <iostream>
#include <cstring>
#include "../include/cide_capi.h"

bool testOutput(const char* name, const char* source, const char* input,
                const char* expectedSubstr, bool expectRuntimeError = false) {
    CideSession* s = cide_session_create();
    if (input && input[0]) {
        cide_set_input(s, input);
    }
    int result = cide_compile(s, source);
    if (result != 0) {
        const char* err = cide_get_compile_errors(s);
        std::cout << "[FAIL] " << name << " - Compile: " << (err ? err : "?") << std::endl;
        cide_session_destroy(s);
        return false;
    }
    result = cide_run(s);
    const char* runtimeErr = cide_get_runtime_error(s);
    int len = cide_get_output_length(s);
    char* buf = new char[len + 1];
    cide_get_output(s, buf, len + 1);
    std::string out(buf);
    delete[] buf;
    cide_session_destroy(s);

    if (expectRuntimeError) {
        if (runtimeErr && std::string(runtimeErr).find(expectedSubstr) != std::string::npos) {
            std::cout << "[OK]   " << name << std::endl;
            return true;
        }
        std::cout << "[FAIL] " << name << " - missing '" << expectedSubstr << "' in runtime error: " << (runtimeErr ? runtimeErr : "none") << std::endl;
        return false;
    }

    if (result != 0) {
        std::cout << "[FAIL] " << name << " - Unexpected runtime error: " << (runtimeErr ? runtimeErr : "?") << std::endl;
        // Print raw bytes of error for debugging encoding issues
        if (runtimeErr) {
            std::cout << "  Raw bytes: ";
            for (const char* p = runtimeErr; *p; p++) {
                std::cout << std::hex << (unsigned)(unsigned char)*p << " ";
            }
            std::cout << std::dec << std::endl;
        }
        return false;
    }
    if (out.find(expectedSubstr) != std::string::npos) {
        std::cout << "[OK]   " << name << std::endl;
        return true;
    }
    std::cout << "[FAIL] " << name << " - missing '" << expectedSubstr << "' in: " << out << std::endl;
    return false;
}

int main() {
    std::cout << "=== Phase 3 Batch 4 Test (printf/scanf) ===" << std::endl;
    int p = 0, t = 0;

    t++; if (testOutput("printf string only",
        "int main() { printf(\"hello\"); return 0; }", "", "hello")) p++;

    t++; if (testOutput("printf with newline",
        "int main() { printf(\"hello\\n\"); return 0; }", "", "hello\n")) p++;

    t++; if (testOutput("printf with %d",
        "int main() { int x = 42; printf(\"value=%d\", x); return 0; }", "", "value=42")) p++;

    t++; if (testOutput("printf two args",
        "int main() { int a = 1; int b = 2; printf(\"a=%d b=%d\", a, b); return 0; }", "", "a=1 b=2")) p++;

    t++; if (testOutput("scanf basic",
        R"(
int main() {
    int x;
    scanf("%d", &x);
    printf("got=%d", x);
    return 0;
}
)", "99", "got=99")) p++;

    t++; if (testOutput("printf mixed text and numbers",
        "int main() { int n = 7; printf(\"The number is %d!\", n); return 0; }", "", "The number is 7!")) p++;

    std::cout << "\nResults: " << p << "/" << t << " passed" << std::endl;
    return (p == t) ? 0 : 1;
}
