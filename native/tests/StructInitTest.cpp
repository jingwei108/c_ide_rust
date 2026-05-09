#include <iostream>
#include <cstring>
#include "../include/cide_capi.h"

static int GetReturnValue(CideSession* s) {
    int len = cide_get_output_length(s);
    char* buf = new char[len + 1];
    cide_get_output(s, buf, len + 1);
    std::string out(buf);
    delete[] buf;
    int actual = -999;
    size_t pos = out.find("return value:");
    if (pos == std::string::npos) pos = out.find("\xe8\xbf\x94\xe5\x9b\x9e\xe5\x80\xbc\xef\xbc\x9a");
    if (pos != std::string::npos) actual = std::atoi(out.c_str() + pos + 12);
    return actual;
}

static void PrintOutput(CideSession* s) {
    int len = cide_get_output_length(s);
    if (len > 0) {
        char* buf = new char[len + 1];
        cide_get_output(s, buf, len + 1);
        std::cout << "  Output: " << buf << std::endl;
        delete[] buf;
    }
}

bool testLocalStructInit() {
    const char* source = R"(
struct Node { int val; struct Node* next; };

int main() {
    struct Node n = {10, 0};
    return n.val;
}
)";

    CideSession* s = cide_session_create();
    int result = cide_compile(s, source);
    if (result != 0) {
        const char* err = cide_get_compile_errors(s);
        std::cout << "[FAIL] Compile error: " << (err ? err : "unknown") << std::endl;
        cide_session_destroy(s);
        return false;
    }

    int runResult = cide_run(s);
    if (runResult != 0) {
        const char* err = cide_get_runtime_error(s);
        std::cout << "[FAIL] Runtime error: " << (err ? err : "unknown") << std::endl;
        cide_session_destroy(s);
        return false;
    }

    int ret = GetReturnValue(s);
    cide_session_destroy(s);
    if (ret == 10) {
        std::cout << "[OK]   Local struct init: n.val = " << ret << std::endl;
        return true;
    }
    std::cout << "[FAIL] Expected n.val = 10, got " << ret << std::endl;
    return false;
}

bool testPartialStructInit() {
    const char* source = R"(
struct Node { int val; struct Node* next; };

int main() {
    struct Node n = {5};
    return n.val;
}
)";

    CideSession* s = cide_session_create();
    int result = cide_compile(s, source);
    if (result != 0) {
        const char* err = cide_get_compile_errors(s);
        std::cout << "[FAIL] Compile error: " << (err ? err : "unknown") << std::endl;
        cide_session_destroy(s);
        return false;
    }

    int runResult = cide_run(s);
    if (runResult != 0) {
        const char* err = cide_get_runtime_error(s);
        std::cout << "[FAIL] Runtime error: " << (err ? err : "unknown") << std::endl;
        cide_session_destroy(s);
        return false;
    }

    int ret = GetReturnValue(s);
    cide_session_destroy(s);
    if (ret == 5) {
        std::cout << "[OK]   Partial struct init: n.val = " << ret << std::endl;
        return true;
    }
    std::cout << "[FAIL] Expected 5, got " << ret << std::endl;
    return false;
}

bool testStructInitWithPointer() {
    const char* source = R"(
struct Node { int val; struct Node* next; };

int main() {
    struct Node a = {1, 0};
    struct Node b = {2, &a};
    return b.next->val;
}
)";

    CideSession* s = cide_session_create();
    int result = cide_compile(s, source);
    if (result != 0) {
        const char* err = cide_get_compile_errors(s);
        std::cout << "[FAIL] Compile error: " << (err ? err : "unknown") << std::endl;
        cide_session_destroy(s);
        return false;
    }

    int runResult = cide_run(s);
    if (runResult != 0) {
        const char* err = cide_get_runtime_error(s);
        std::cout << "[FAIL] Runtime error: " << (err ? err : "unknown") << std::endl;
        cide_session_destroy(s);
        return false;
    }

    int ret = GetReturnValue(s);
    cide_session_destroy(s);
    if (ret == 1) {
        std::cout << "[OK]   Struct init with pointer: b.next->val = " << ret << std::endl;
        return true;
    }
    std::cout << "[FAIL] Expected 1, got " << ret << std::endl;
    return false;
}

bool testNestedStructInit() {
    const char* source = R"(
struct Point { int x; int y; };

int main() {
    struct Point p = {7, 8};
    return p.x + p.y;
}
)";

    CideSession* s = cide_session_create();
    int result = cide_compile(s, source);
    if (result != 0) {
        const char* err = cide_get_compile_errors(s);
        std::cout << "[FAIL] Compile error: " << (err ? err : "unknown") << std::endl;
        cide_session_destroy(s);
        return false;
    }

    int runResult = cide_run(s);
    if (runResult != 0) {
        const char* err = cide_get_runtime_error(s);
        std::cout << "[FAIL] Runtime error: " << (err ? err : "unknown") << std::endl;
        cide_session_destroy(s);
        return false;
    }

    int ret = GetReturnValue(s);
    cide_session_destroy(s);
    if (ret == 15) {
        std::cout << "[OK]   Nested struct init: p.x + p.y = " << ret << std::endl;
        return true;
    }
    std::cout << "[FAIL] Expected 15, got " << ret << std::endl;
    return false;
}

int main() {
    std::cout << "=== Struct Init Test ===" << std::endl;
    int passed = 0;
    if (testLocalStructInit()) passed++;
    if (testPartialStructInit()) passed++;
    if (testStructInitWithPointer()) passed++;
    if (testNestedStructInit()) passed++;
    std::cout << "=== Passed " << passed << "/4 ===" << std::endl;
    return (passed == 4) ? 0 : 1;
}
