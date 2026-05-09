#include <iostream>
#include <cstring>
#include "../include/cide_capi.h"

bool testRuntimeError(const char* name, const char* source, const char* expectedSubstr) {
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

    if (runtimeErr && std::string(runtimeErr).find(expectedSubstr) != std::string::npos) {
        cide_session_destroy(s);
        std::cout << "[OK]   " << name << std::endl;
        return true;
    }
    std::cout << "[FAIL] " << name << " - missing '" << expectedSubstr << "' in runtime error: "
              << (runtimeErr ? runtimeErr : "none") << std::endl;
    cide_session_destroy(s);
    return false;
}

int main() {
    std::cout << "=== Stage 2 Diagnostic Test ===" << std::endl;
    int p = 0, t = 0;

    // 1. Global array bounds error with precise diagnostic
    t++; if (testRuntimeError("global array bounds",
        "int arr[5];\n"
        "int main() {\n"
        "    int i = 10;\n"
        "    arr[i] = 1;\n"
        "    return 0;\n"
        "}", "arr[10]")) p++;

    // 2. Local array bounds error with precise diagnostic
    t++; if (testRuntimeError("local array bounds",
        "int main() {\n"
        "    int arr[5];\n"
        "    int i = 7;\n"
        "    arr[i] = 1;\n"
        "    return 0;\n"
        "}", "arr[7]")) p++;

    // 3. Division by zero with variable hint
    t++; if (testRuntimeError("div zero variable hint",
        "int main() {\n"
        "    int a = 10;\n"
        "    int b = 0;\n"
        "    return a / b;\n"
        "}", "b")) p++;

    // 4. NULL pointer dereference
    t++; if (testRuntimeError("null pointer",
        "int main() {\n"
        "    int* p = 0;\n"
        "    *p = 10;\n"
        "    return 0;\n"
        "}", "NULL")) p++;

    // 5. Infinite loop step limit with variable state
    t++; if (testRuntimeError("infinite loop",
        "int main() {\n"
        "    int i = 0;\n"
        "    while (1) {\n"
        "        i = i + 0;\n"
        "    }\n"
        "    return 0;\n"
        "}", "无限循环")) p++;

    std::cout << "\nResults: " << p << "/" << t << " passed" << std::endl;
    return (p == t) ? 0 : 1;
}
