#include <iostream>
#include <cstring>
#include "../include/cide_capi.h"

void testAlgorithmMatch();

void test(const char* name, const char* source, int expected) {
    CideSession* s = cide_session_create();
    int result = cide_compile(s, source);
    if (result != 0) {
        std::cout << name << " COMPILE FAILED" << std::endl;
        const char* errs = cide_get_compile_errors(s);
        if (errs) std::cout << "  " << errs << std::endl;
        cide_session_destroy(s);
        return;
    }
    // Print warnings
    int n = cide_diagnostic_count(s);
    for (int i = 0; i < n; i++) {
        int line, col, code, sev;
        char msg[256];
        cide_diagnostic_get(s, i, &line, &col, &code, &sev, msg, 256, nullptr, 0);
        if (sev == 1) std::cout << name << " Warning: " << msg << std::endl;
    }
    result = cide_run(s);
    const char* err = cide_get_runtime_error(s);
    int len = cide_get_output_length(s);
    char* buf = new char[len + 1];
    cide_get_output(s, buf, len + 1);
    int retVal = 0;
    // parse return value from output
    const char* p = strstr(buf, "返回值：");
    if (p) retVal = atoi(p + 12); // 跳过 "返回值："
    if (err) {
        std::cout << name << " RUNTIME ERROR: " << err << std::endl;
    } else if (retVal == expected) {
        std::cout << name << " PASS (ret=" << retVal << ")" << std::endl;
    } else {
        std::cout << name << " FAIL expected=" << expected << " got=" << retVal << " output=[" << buf << "]" << std::endl;
    }
    delete[] buf;
    cide_session_destroy(s);
}

void testDiagnosticFix() {
    std::cout << "--- Diagnostic Fix Tests ---" << std::endl;

    // Test 1: Assignment in condition warning
    {
        const char* source = R"(
int main() {
    int a = 5;
    int b = 10;
    if (a = b) {
        return 1;
    }
    return 0;
}
)";
        CideSession* s = cide_session_create();
        int result = cide_compile(s, source);
        if (result != 0) {
            std::cout << "assign_in_condition COMPILE FAILED" << std::endl;
            cide_session_destroy(s);
        } else {
            int diagCount = cide_diagnostic_count(s);
            bool foundWarning = false;
            bool hasFix = false;
            for (int i = 0; i < diagCount; i++) {
                int line, column, code, severity;
                char message[512] = {0};
                char fix[512] = {0};
                cide_diagnostic_get(s, i, &line, &column, &code, &severity,
                                    message, sizeof(message), fix, sizeof(fix));
                if (severity == 1 && strstr(message, "赋值运算符") != nullptr) {
                    foundWarning = true;
                    hasFix = (strlen(fix) > 0);
                    std::cout << "  assign_in_condition line=" << line << " msg=" << message << std::endl;
                    std::cout << "  fix=" << fix << std::endl;
                }
            }
            if (foundWarning && hasFix) {
                std::cout << "assign_in_condition PASS" << std::endl;
            } else {
                std::cout << "assign_in_condition FAIL (found=" << foundWarning << ", fix=" << hasFix << ", diags=" << diagCount << ")" << std::endl;
            }
            cide_session_destroy(s);
        }
    }

    // Test 2: <= in for loop warning
    {
        const char* source = R"(
int main() {
    int arr[5];
    int i;
    for (i = 0; i <= 5; i = i + 1) {
        arr[i] = i;
    }
    return 0;
}
)";
        CideSession* s = cide_session_create();
        int result = cide_compile(s, source);
        if (result != 0) {
            std::cout << "le_in_loop COMPILE FAILED" << std::endl;
            cide_session_destroy(s);
        } else {
            int diagCount = cide_diagnostic_count(s);
            bool foundWarning = false;
            bool hasFix = false;
            for (int i = 0; i < diagCount; i++) {
                int line, column, code, severity;
                char message[512] = {0};
                char fix[512] = {0};
                cide_diagnostic_get(s, i, &line, &column, &code, &severity,
                                    message, sizeof(message), fix, sizeof(fix));
                if (severity == 1 && (strstr(message, "off-by-one") != nullptr || strstr(message, "'<='") != nullptr)) {
                    foundWarning = true;
                    hasFix = (strlen(fix) > 0);
                    std::cout << "  le_in_loop line=" << line << " msg=" << message << std::endl;
                    std::cout << "  fix=" << fix << std::endl;
                }
            }
            if (foundWarning && hasFix) {
                std::cout << "le_in_loop PASS" << std::endl;
            } else {
                std::cout << "le_in_loop FAIL (found=" << foundWarning << ", fix=" << hasFix << ", diags=" << diagCount << ")" << std::endl;
            }
            cide_session_destroy(s);
        }
    }

    // Test 3: Missing semicolon — structured fix
    {
        const char* source = R"(
int main() {
    int x = 5
    return 0;
}
)";
        CideSession* s = cide_session_create();
        int result = cide_compile(s, source);
        if (result == 0) {
            std::cout << "missing_semicolon COMPILE UNEXPECTEDLY PASSED" << std::endl;
            cide_session_destroy(s);
        } else {
            int diagCount = cide_diagnostic_count(s);
            bool found = false;
            bool structuredOk = false;
            for (int i = 0; i < diagCount; i++) {
                int line, column, code, severity;
                char message[512] = {0};
                char fix[512] = {0};
                cide_diagnostic_get(s, i, &line, &column, &code, &severity,
                                    message, sizeof(message), fix, sizeof(fix));
                if (code == 2005) {
                    found = true;
                    int fixKind, startLine, startCol, endLine, endCol;
                    char replacement[512] = {0};
                    cide_diagnostic_get_fix(s, i, &fixKind, &startLine, &startCol,
                                            &endLine, &endCol, replacement, sizeof(replacement));
                    std::cout << "  missing_semicolon line=" << line << " col=" << column << std::endl;
                    std::cout << "  fixKind=" << fixKind << " startLine=" << startLine << " startCol=" << startCol
                              << " endLine=" << endLine << " endCol=" << endCol
                              << " text=[" << replacement << "]" << std::endl;
                    structuredOk = (fixKind == 2 && startLine == 3 && startCol == 13
                                    && endLine == 3 && endCol == 13
                                    && strcmp(replacement, ";") == 0);
                }
            }
            if (found && structuredOk) {
                std::cout << "missing_semicolon PASS" << std::endl;
            } else {
                std::cout << "missing_semicolon FAIL (found=" << found << ", structured=" << structuredOk << ")" << std::endl;
            }
            cide_session_destroy(s);
        }
    }

    // Test 4: Unsupported operator '|' — structured fix
    {
        const char* source = R"(
int main() {
    int a = 5;
    int b = 3;
    if (a | b) {
        return 1;
    }
    return 0;
}
)";
        CideSession* s = cide_session_create();
        int result = cide_compile(s, source);
        if (result == 0) {
            std::cout << "unsupported_op COMPILE UNEXPECTEDLY PASSED" << std::endl;
            cide_session_destroy(s);
        } else {
            int diagCount = cide_diagnostic_count(s);
            bool found = false;
            bool structuredOk = false;
            for (int i = 0; i < diagCount; i++) {
                int line, column, code, severity;
                char message[512] = {0};
                char fix[512] = {0};
                cide_diagnostic_get(s, i, &line, &column, &code, &severity,
                                    message, sizeof(message), fix, sizeof(fix));
                if (code == 1004) {
                    found = true;
                    int fixKind, startLine, startCol, endLine, endCol;
                    char replacement[512] = {0};
                    cide_diagnostic_get_fix(s, i, &fixKind, &startLine, &startCol,
                                            &endLine, &endCol, replacement, sizeof(replacement));
                    std::cout << "  unsupported_op line=" << line << " col=" << column << std::endl;
                    std::cout << "  fixKind=" << fixKind << " startLine=" << startLine << " startCol=" << startCol
                              << " endLine=" << endLine << " endCol=" << endCol
                              << " text=[" << replacement << "]" << std::endl;
                    structuredOk = (fixKind == 1 && startLine == 5 && startCol == 10
                                    && endLine == 5 && endCol == 11
                                    && strcmp(replacement, "||") == 0);
                }
            }
            if (found && structuredOk) {
                std::cout << "unsupported_op PASS" << std::endl;
            } else {
                std::cout << "unsupported_op FAIL (found=" << found << ", structured=" << structuredOk << ")" << std::endl;
            }
            cide_session_destroy(s);
        }
    }

    // Test 5: Missing closing brace — structured fix
    {
        const char* source = R"(
int main() {
    int x = 5;
)";
        std::cout << "[SOURCE_LEN] strlen=" << strlen(source) << std::endl;
        for (size_t i = 0; i < strlen(source); i++) {
            if (source[i] == '\n') std::cout << "  [" << i << "]=\\n" << std::endl;
            else std::cout << "  [" << i << "]=" << source[i] << std::endl;
        }
        CideSession* s = cide_session_create();
        int result = cide_compile(s, source);
        if (result == 0) {
            std::cout << "missing_brace COMPILE UNEXPECTEDLY PASSED" << std::endl;
            cide_session_destroy(s);
        } else {
            int diagCount = cide_diagnostic_count(s);
            bool found = false;
            bool structuredOk = false;
            for (int i = 0; i < diagCount; i++) {
                int line, column, code, severity;
                char message[512] = {0};
                char fix[512] = {0};
                cide_diagnostic_get(s, i, &line, &column, &code, &severity,
                                    message, sizeof(message), fix, sizeof(fix));
                if (code == 2006) {
                    found = true;
                    int fixKind, startLine, startCol, endLine, endCol;
                    char replacement[512] = {0};
                    cide_diagnostic_get_fix(s, i, &fixKind, &startLine, &startCol,
                                            &endLine, &endCol, replacement, sizeof(replacement));
                    std::cout << "  missing_brace line=" << line << " col=" << column << std::endl;
                    std::cout << "  fixKind=" << fixKind << " startLine=" << startLine << " startCol=" << startCol
                              << " endLine=" << endLine << " endCol=" << endCol
                              << " text=[" << replacement << "]" << std::endl;
                    structuredOk = (fixKind == 2 && startLine == 3 && startCol == 14
                                    && endLine == 3 && endCol == 14
                                    && strcmp(replacement, "}") == 0);
                }
            }
            if (found && structuredOk) {
                std::cout << "missing_brace PASS" << std::endl;
            } else {
                std::cout << "missing_brace FAIL (found=" << found << ", structured=" << structuredOk << ")" << std::endl;
            }
            cide_session_destroy(s);
        }
    }

    // Test 6: Missing closing paren — structured fix
    {
        const char* source = R"(
int main() {
    int x = 5;
    if (x == 5 {
        return 1;
    }
    return 0;
}
)";
        CideSession* s = cide_session_create();
        int result = cide_compile(s, source);
        if (result == 0) {
            std::cout << "missing_paren COMPILE UNEXPECTEDLY PASSED" << std::endl;
            cide_session_destroy(s);
        } else {
            int diagCount = cide_diagnostic_count(s);
            bool found = false;
            bool structuredOk = false;
            for (int i = 0; i < diagCount; i++) {
                int line, column, code, severity;
                char message[512] = {0};
                char fix[512] = {0};
                cide_diagnostic_get(s, i, &line, &column, &code, &severity,
                                    message, sizeof(message), fix, sizeof(fix));
                if (code == 2007) {
                    found = true;
                    int fixKind, startLine, startCol, endLine, endCol;
                    char replacement[512] = {0};
                    cide_diagnostic_get_fix(s, i, &fixKind, &startLine, &startCol,
                                            &endLine, &endCol, replacement, sizeof(replacement));
                    std::cout << "  missing_paren line=" << line << " col=" << column << std::endl;
                    std::cout << "  fixKind=" << fixKind << " startLine=" << startLine << " startCol=" << startCol
                              << " endLine=" << endLine << " endCol=" << endCol
                              << " text=[" << replacement << "]" << std::endl;
                    structuredOk = (fixKind == 2 && startLine == 4 && startCol == 16
                                    && endLine == 4 && endCol == 16
                                    && strcmp(replacement, ")") == 0);
                }
            }
            if (found && structuredOk) {
                std::cout << "missing_paren PASS" << std::endl;
            } else {
                std::cout << "missing_paren FAIL (found=" << found << ", structured=" << structuredOk << ")" << std::endl;
            }
            cide_session_destroy(s);
        }
    }

    // Test 7: Linked list traversal detection
    {
        const char* source = R"(
struct Node { int val; struct Node* next; };

void printList(struct Node* head) {
    struct Node* p = head;
    while (p) {
        p = p->next;
    }
}

int main() {
    return 0;
}
)";
        CideSession* s = cide_session_create();
        int result = cide_compile(s, source);
        if (result != 0) {
            std::cout << "linked_list_traversal COMPILE FAILED" << std::endl;
            cide_session_destroy(s);
        } else {
            int count = cide_algorithm_match_count(s);
            bool found = false;
            for (int i = 0; i < count; i++) {
                char name[64] = {0};
                char displayName[64] = {0};
                int confidence = 0;
                char suggestion[512] = {0};
                int line = 0;
                char funcName[64] = {0};
                cide_algorithm_match_get(s, i, name, sizeof(name), displayName, sizeof(displayName), funcName, sizeof(funcName),
                                         &confidence, suggestion, sizeof(suggestion), &line);
                if (strcmp(name, "linked_list_traversal") == 0) {
                    found = true;
                    std::cout << "linked_list_traversal PASS (confidence=" << confidence << ", line=" << line << ")" << std::endl;
                }
            }
            if (!found) {
                std::cout << "linked_list_traversal FAIL (not detected, matches=" << count << ")" << std::endl;
            }
            cide_session_destroy(s);
        }
    }

    // Test 4: Linked list reverse detection
    {
        const char* source = R"(
struct Node { int val; struct Node* next; };

struct Node* reverse(struct Node* head) {
    struct Node* prev = 0;
    struct Node* curr = head;
    while (curr) {
        struct Node* next = curr->next;
        curr->next = prev;
        prev = curr;
        curr = next;
    }
    return prev;
}

int main() {
    return 0;
}
)";
        CideSession* s = cide_session_create();
        int result = cide_compile(s, source);
        if (result != 0) {
            std::cout << "linked_list_reverse COMPILE FAILED" << std::endl;
            cide_session_destroy(s);
        } else {
            int count = cide_algorithm_match_count(s);
            bool found = false;
            for (int i = 0; i < count; i++) {
                char name[64] = {0};
                char displayName[64] = {0};
                int confidence = 0;
                char suggestion[512] = {0};
                int line = 0;
                char funcName[64] = {0};
                cide_algorithm_match_get(s, i, name, sizeof(name), displayName, sizeof(displayName), funcName, sizeof(funcName),
                                         &confidence, suggestion, sizeof(suggestion), &line);
                if (strcmp(name, "linked_list_reverse") == 0) {
                    found = true;
                    std::cout << "linked_list_reverse PASS (confidence=" << confidence << ", line=" << line << ")" << std::endl;
                }
            }
            if (!found) {
                std::cout << "linked_list_reverse FAIL (not detected, matches=" << count << ")" << std::endl;
            }
            cide_session_destroy(s);
        }
    }
}

int main() {
    // char type
    test("char_basic", "int main() { char c = 65; return c; }", 65);
    test("char_int_conv", "int main() { int a = 65; char c = a; return c; }", 65);

    // do...while
    test("dowhile_basic", "int main() { int i = 0; int s = 0; do { s = s + i; i = i + 1; } while (i < 5); return s; }", 10);

    // break
    test("break_basic", "int main() { int i = 0; while (1) { i = i + 1; if (i > 5) break; } return i; }", 6);

    // continue
    test("continue_basic", "int main() { int s = 0; int i = 0; while (i < 5) { i = i + 1; if (i == 3) continue; s = s + i; } return s; }", 12);

    // break in for
    test("break_for", "int main() { int s = 0; for (int i = 0; i < 10; i = i + 1) { s = s + i; if (i == 5) break; } return s; }", 15);

    // do...while with break
    test("dowhile_break", "int main() { int i = 0; do { i = i + 1; if (i > 3) break; } while (i < 10); return i; }", 4);

    // sizeof
    test("sizeof_int", "int main() { return sizeof(int); }", 4);
    test("sizeof_char", "int main() { return sizeof(char); }", 4);
    test("sizeof_var", "int main() { int a; return sizeof(a); }", 4);
    test("sizeof_ptr", "int main() { int* p; return sizeof(p); }", 4);

    // switch/case
    test("switch_basic", "int main() { int x = 2; int r = 0; switch (x) { case 1: r = 10; break; case 2: r = 20; break; case 3: r = 30; break; default: r = 99; } return r; }", 20);
    test("switch_default", "int main() { int x = 5; int r = 0; switch (x) { case 1: r = 10; break; default: r = 99; } return r; }", 99);
    test("switch_fallthrough", "int main() { int x = 1; int r = 0; switch (x) { case 1: r = r + 1; case 2: r = r + 2; break; default: r = 99; } return r; }", 3);
    test("switch_no_default", "int main() { int x = 5; int r = 10; switch (x) { case 1: r = 1; break; } return r; }", 10);

    // typedef
    test("typedef_basic", "typedef int MyInt; int main() { MyInt a = 5; return a; }", 5);
    test("typedef_ptr", "typedef int* IntPtr; int main() { int a = 5; IntPtr p = &a; return *p; }", 5);

    // enum
    test("enum_basic", "enum Color { Red, Green, Blue }; int main() { return Green; }", 1);
    test("enum_with_value", "enum Status { OK = 0, Error = 1, Warning = 2 }; int main() { return Warning; }", 2);
    test("enum_var", "enum Color { Red, Green, Blue }; int main() { Color c = Blue; return c; }", 2);

    // unsigned
    test("unsigned_basic", "int main() { unsigned a = 5; return a; }", 5);
    test("unsigned_int", "int main() { unsigned int a = 10; return a; }", 10);

    // array initializer
    test("array_init_basic", "int main() { int a[3] = {1, 2, 3}; return a[0] + a[1] + a[2]; }", 6);
    test("array_init_infer", "int main() { int a[] = {1, 2, 3, 4}; return a[3]; }", 4);
    test("array_init_global", "int a[3] = {1, 2, 3}; int main() { return a[0] + a[1] + a[2]; }", 6);
    test("char_array_string", "int main() { char s[] = \"hello\"; return s[4]; }", 111);
    test("char_array_string_global", "char s[] = \"hello\"; int main() { return s[0] + s[4]; }", 215);

    testAlgorithmMatch();
    testDiagnosticFix();

    return 0;
}

void testAlgorithmMatch() {
    std::cout << "--- Algorithm Match Tests ---" << std::endl;

    // Binary search detection
    {
        const char* source = R"(
int binarySearch(int arr[], int n, int target) {
    int left = 0;
    int right = n - 1;
    while (left <= right) {
        int mid = (left + right) / 2;
        if (arr[mid] == target) {
            return mid;
        } else if (arr[mid] < target) {
            left = mid + 1;
        } else {
            right = mid - 1;
        }
    }
    return -1;
}

int main() {
    int arr[5];
    arr[0] = 1; arr[1] = 3; arr[2] = 5; arr[3] = 7; arr[4] = 9;
    return binarySearch(arr, 5, 5);
}
)";
        CideSession* s = cide_session_create();
        int result = cide_compile(s, source);
        if (result != 0) {
            std::cout << "binary_search_detection COMPILE FAILED" << std::endl;
            cide_session_destroy(s);
        } else {
            int count = cide_algorithm_match_count(s);
            bool found = false;
            for (int i = 0; i < count; i++) {
                char name[64] = {0};
                char displayName[64] = {0};
                int confidence = 0;
                char suggestion[512] = {0};
                int line = 0;
                char funcName[64] = {0};
                cide_algorithm_match_get(s, i, name, sizeof(name), displayName, sizeof(displayName), funcName, sizeof(funcName),
                                         &confidence, suggestion, sizeof(suggestion), &line);
                if (strcmp(name, "binary_search") == 0) {
                    found = true;
                    std::cout << "binary_search_detection PASS (confidence=" << confidence << ", line=" << line << ")" << std::endl;
                }
            }
            if (!found) {
                std::cout << "binary_search_detection FAIL (not detected, matches=" << count << ")" << std::endl;
            }
            cide_session_destroy(s);
        }
    }

    // Bubble sort detection (regression)
    {
        const char* source = R"(
void bubbleSort(int arr[], int n) {
    int i;
    for (i = 0; i < n - 1; i = i + 1) {
        int j;
        for (j = 0; j < n - i - 1; j = j + 1) {
            if (arr[j] > arr[j + 1]) {
                int temp = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = temp;
            }
        }
    }
}

int main() {
    int arr[3];
    arr[0] = 3; arr[1] = 1; arr[2] = 2;
    bubbleSort(arr, 3);
    return arr[0];
}
)";
        CideSession* s = cide_session_create();
        int result = cide_compile(s, source);
        if (result != 0) {
            std::cout << "bubble_sort_detection COMPILE FAILED" << std::endl;
            cide_session_destroy(s);
        } else {
            int count = cide_algorithm_match_count(s);
            bool found = false;
            for (int i = 0; i < count; i++) {
                char name[64] = {0};
                char displayName[64] = {0};
                int confidence = 0;
                char suggestion[512] = {0};
                int line = 0;
                char funcName[64] = {0};
                cide_algorithm_match_get(s, i, name, sizeof(name), displayName, sizeof(displayName), funcName, sizeof(funcName),
                                         &confidence, suggestion, sizeof(suggestion), &line);
                if (strcmp(name, "bubble_sort") == 0) {
                    found = true;
                    std::cout << "bubble_sort_detection PASS (confidence=" << confidence << ")" << std::endl;
                }
            }
            if (!found) {
                std::cout << "bubble_sort_detection FAIL (not detected)" << std::endl;
            }
            cide_session_destroy(s);
        }
    }
}


