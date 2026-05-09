#include <iostream>
#include <cstring>
#include "../include/cide_capi.h"

bool testCase(const char* name, const char* source, int expected) {
    CideSession* s = cide_session_create();
    int result = cide_compile(s, source);
    if (result != 0) {
        const char* err = cide_get_compile_errors(s);
        std::cout << "[FAIL] " << name << " - Compile: " << (err ? err : "?") << std::endl;
        cide_session_destroy(s);
        return false;
    }
    result = cide_run(s);
    if (result != 0) {
        const char* err = cide_get_runtime_error(s);
        std::cout << "[FAIL] " << name << " - Runtime: " << (err ? err : "?") << std::endl;
        cide_session_destroy(s);
        return false;
    }
    int len = cide_get_output_length(s);
    char* buf = new char[len + 1];
    cide_get_output(s, buf, len + 1);
    std::string out(buf);
    delete[] buf;
    int actual = -999;
    size_t pos = out.find("return value:");
    if (pos == std::string::npos) pos = out.find("\xe8\xbf\x94\xe5\x9b\x9e\xe5\x80\xbc\xef\xbc\x9a");
    if (pos != std::string::npos) actual = std::atoi(out.c_str() + pos + 12);
    cide_session_destroy(s);
    if (actual == expected) {
        std::cout << "[OK]   " << name << std::endl;
        return true;
    }
    std::cout << "[FAIL] " << name << " -> exp " << expected << ", got " << actual << std::endl;
    return false;
}

int main() {
    std::cout << "=== Phase 2 Regression Test ===" << std::endl;
    int p = 0, t = 0;

    t++; if (testCase("Simple Add", "int main() { int a = 3; int b = 5; return a + b; }", 8)) p++;
    t++; if (testCase("Loop Sum", "int main() { int sum = 0; int i; for (i = 1; i <= 5; i = i + 1) { sum = sum + i; } return sum; }", 15)) p++;
    t++; if (testCase("If/Else", "int max(int a, int b) { if (a > b) { return a; } else { return b; } } int main() { return max(3, 7); }", 7)) p++;
    t++; if (testCase("While", "int main() { int n = 0; int i = 0; while (i < 5) { n = n + 1; i = i + 1; } return n; }", 5)) p++;
    t++; if (testCase("Recursive Factorial", "int factorial(int n) { if (n <= 1) { return 1; } return n * factorial(n - 1); } int main() { return factorial(5); }", 120)) p++;

    std::cout << "\nResults: " << p << "/" << t << " passed" << std::endl;
    return (p == t) ? 0 : 1;
}
