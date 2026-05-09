#include <iostream>
#include <cstring>
#include "../include/cide_capi.h"

bool testCase(const char* name, const char* source, int expected) {
    CideSession* s = cide_session_create();
    int result = cide_compile(s, source);
    if (result != 0) {
        const char* err = cide_get_compile_errors(s);
        std::cout << "[FAIL] " << name << " - Compile error: " << (err ? err : "unknown") << std::endl;
        cide_session_destroy(s);
        return false;
    }
    result = cide_run(s);
    if (result != 0) {
        const char* err = cide_get_runtime_error(s);
        std::cout << "[FAIL] " << name << " - Runtime error: " << (err ? err : "unknown") << std::endl;
        cide_session_destroy(s);
        return false;
    }
    int len = cide_get_output_length(s);
    char* buf = new char[len + 1];
    cide_get_output(s, buf, len + 1);
    std::string output(buf);
    delete[] buf;

    // Parse return value from output: "...返回值：X"
    int actual = -999;
    size_t pos = output.find("return value:");
    if (pos == std::string::npos) {
        // Try Chinese
        pos = output.find("\xe8\xbf\x94\xe5\x9b\x9e\xe5\x80\xbc\xef\xbc\x9a"); // UTF-8 bytes for "返回值："
    }
    if (pos != std::string::npos) {
        actual = std::atoi(output.c_str() + pos + 12); // skip prefix
    }

    cide_session_destroy(s);
    if (actual == expected) {
        std::cout << "[OK]   " << name << " -> " << actual << std::endl;
        return true;
    } else {
        std::cout << "[FAIL] " << name << " -> expected " << expected << ", got " << actual << " (output: " << output << ")" << std::endl;
        return false;
    }
}

int main() {
    std::cout << "=== Phase 3 Batch 1 Test ===" << std::endl;
    int passed = 0, total = 0;

    // 1. Array basic
    total++;
    if (testCase("Array basic",
        "int main() { int arr[5]; arr[0] = 10; return arr[0]; }", 10)) passed++;

    // 2. Array with index variable
    total++;
    if (testCase("Array index var",
        "int main() { int arr[5]; int i = 2; arr[i] = 7; return arr[i]; }", 7)) passed++;

    // 3. Array in loop (bubble sort prep)
    total++;
    if (testCase("Array loop",
        "int main() { int arr[3]; arr[0] = 1; arr[1] = 2; arr[2] = 3; int sum = 0; int i; for (i = 0; i < 3; i = i + 1) { sum = sum + arr[i]; } return sum; }", 6)) passed++;

    // 4. Pointer via malloc
    total++;
    if (testCase("Pointer malloc",
        "int main() { int* p = malloc(4); *p = 42; return *p; }", 42)) passed++;

    // 5. Struct basic
    total++;
    if (testCase("Struct basic",
        "struct Node { int val; }; int main() { struct Node n; n.val = 7; return n.val; }", 7)) passed++;

    // 6. Struct pointer
    total++;
    if (testCase("Struct pointer",
        "struct Node { int val; }; int main() { struct Node* p = malloc(4); p->val = 9; return p->val; }", 9)) passed++;

    // 7. Pointer to array element (pointer arithmetic)
    total++;
    if (testCase("Pointer to array elem",
        "int main() { int arr[5]; arr[2] = 15; int* p = arr; return *(p + 2); }", 15)) passed++;

    // 8. Array assignment to pointer
    total++;
    if (testCase("Array to pointer",
        "int main() { int arr[3]; arr[0] = 5; int* p = arr; return *p; }", 5)) passed++;

    // 9. Bubble sort (no pointer arithmetic, uses indexing)
    total++;
    if (testCase("Bubble sort",
        "void bubbleSort(int arr[], int n) {"
        "    int i; int j;"
        "    for (i = 0; i < n - 1; i = i + 1) {"
        "        for (j = 0; j < n - i - 1; j = j + 1) {"
        "            if (arr[j] > arr[j + 1]) {"
        "                int tmp = arr[j];"
        "                arr[j] = arr[j + 1];"
        "                arr[j + 1] = tmp;"
        "            }"
        "        }"
        "    }"
        "}"
        "int main() {"
        "    int arr[5];"
        "    arr[0] = 5; arr[1] = 3; arr[2] = 8; arr[3] = 1; arr[4] = 2;"
        "    bubbleSort(arr, 5);"
        "    return arr[0];"
        "}", 1)) passed++;

    std::cout << "\nResults: " << passed << "/" << total << " passed" << std::endl;
    return (passed == total) ? 0 : 1;
}
