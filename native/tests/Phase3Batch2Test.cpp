#include <iostream>
#include <cstring>
#include "../include/cide_capi.h"

bool testOutput(const char* name, const char* source, const char* expectedSubstr, bool expectRuntimeError = false) {
    CideSession* s = cide_session_create();
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

    if (expectRuntimeError) {
        if (runtimeErr && std::string(runtimeErr).find(expectedSubstr) != std::string::npos) {
            cide_session_destroy(s);
            std::cout << "[OK]   " << name << std::endl;
            return true;
        }
        std::cout << "[FAIL] " << name << " - missing '" << expectedSubstr << "' in runtime error: " << (runtimeErr ? runtimeErr : "none") << std::endl;
        cide_session_destroy(s);
        return false;
    }

    if (result != 0) {
        std::cout << "[FAIL] " << name << " - Unexpected runtime error: " << (runtimeErr ? runtimeErr : "?") << std::endl;
        cide_session_destroy(s);
        return false;
    }
    if (out.find(expectedSubstr) != std::string::npos) {
        cide_session_destroy(s);
        std::cout << "[OK]   " << name << std::endl;
        return true;
    }
    std::cout << "[FAIL] " << name << " - missing '" << expectedSubstr << "' in: " << out << std::endl;
    cide_session_destroy(s);
    return false;
}

int main() {
    std::cout << "=== Phase 3 Batch 2 Test ===" << std::endl;
    int p = 0, t = 0;

    t++; if (testOutput("print_int basic",
        "int main() { print_int(42); return 0; }", "42")) p++;

    t++; if (testOutput("print_int in loop",
        "int main() { int i; for (i = 0; i < 3; i = i + 1) { print_int(i); } return 0; }", "0")) p++;

    t++; if (testOutput("Division by zero error",
        "int main() { int a = 10; int b = 0; return a / b; }", "\xe9\x99\xa4\xe9\x9b\xb6\xe9\x94\x99\xe8\xaf\xaf", true)) p++;

    std::cout << "\nResults: " << p << "/" << t << " passed" << std::endl;
    return (p == t) ? 0 : 1;
}
