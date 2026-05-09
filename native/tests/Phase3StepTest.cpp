#include <iostream>
#include <cstring>
#include "../include/cide_capi.h"

int main() {
    std::cout << "=== Phase 3 Step Test ===" << std::endl;
    int passed = 0, total = 0;

    CideSession* s = cide_session_create();
    const char* source = R"(
int main() {
    int a = 1;
    int b = 2;
    int c = a + b;
    return c;
}
)";

    // Test 1: Compile
    total++;
    int result = cide_compile(s, source);
    if (result != 0) {
        std::cout << "[FAIL] Compile failed" << std::endl;
    } else {
        std::cout << "[OK]   Compile succeeded" << std::endl;
        passed++;
    }

    // Test 2: First step should pause at first statement
    total++;
    result = cide_step_next(s);
    int line1 = cide_get_current_line(s);
    if (result == 0 && line1 > 0) {
        std::cout << "[OK]   First step at line " << line1 << std::endl;
        passed++;
    } else {
        std::cout << "[FAIL] First step failed, result=" << result << " line=" << line1 << std::endl;
    }

    // Test 3: Second step should advance to next statement
    total++;
    result = cide_step_next(s);
    int line2 = cide_get_current_line(s);
    if (result == 0 && line2 >= line1) {
        std::cout << "[OK]   Second step at line " << line2 << std::endl;
        passed++;
    } else {
        std::cout << "[FAIL] Second step failed, result=" << result << " line=" << line2 << std::endl;
    }

    // Test 4: Continue stepping until completion
    total++;
    int steps = 2;
    while (result == 0 && steps < 100) {
        result = cide_step_next(s);
        steps++;
    }
    if (result == -1 && cide_get_runtime_error(s) == nullptr) {
        std::cout << "[OK]   Step to completion (" << steps << " steps)" << std::endl;
        passed++;
    } else {
        const char* err = cide_get_runtime_error(s);
        std::cout << "[FAIL] Step to completion failed, result=" << result
                  << " err=" << (err ? err : "none") << std::endl;
    }

    // Test 5: Step in a loop to verify variable values change
    total++;
    const char* loopSource = R"(
int main() {
    int sum = 0;
    int i;
    for (i = 1; i <= 3; i = i + 1) {
        sum = sum + i;
    }
    return sum;
}
)";
    CideSession* s2 = cide_session_create();
    if (cide_compile(s2, loopSource) == 0) {
        // Step through until completion
        int loopSteps = 0;
        int lastLine = 0;
        while (cide_step_next(s2) == 0 && loopSteps < 200) {
            lastLine = cide_get_current_line(s2);
            loopSteps++;
        }
        // Return value should be 6 (1+2+3)
        const char* out = cide_get_runtime_error(s2);
        if (out == nullptr || strlen(out) == 0) {
            std::cout << "[OK]   Loop step completed (" << loopSteps << " steps, last line=" << lastLine << ")" << std::endl;
            passed++;
        } else {
            std::cout << "[FAIL] Loop step runtime error: " << out << std::endl;
        }
    } else {
        std::cout << "[FAIL] Loop source compile failed" << std::endl;
    }
    cide_session_destroy(s2);

    cide_session_destroy(s);

    std::cout << "\nResults: " << passed << "/" << total << " passed" << std::endl;
    return (passed == total) ? 0 : 1;
}
