#include <iostream>
#include <cstring>
#include "../include/cide_capi.h"

int main() {
    std::cout << "=== Phase 3 Batch 3 Test (Memory View) ===" << std::endl;

    CideSession* s = cide_session_create();
    const char* source = R"(
int main() {
    int* p = malloc(12);
    *p = 100;
    *(p + 1) = 200;
    free(p);
    return 0;
}
)";

    int result = cide_compile(s, source);
    if (result != 0) {
        std::cout << "[FAIL] Compile: " << (cide_get_compile_errors(s) ? cide_get_compile_errors(s) : "?") << std::endl;
        cide_session_destroy(s);
        return 1;
    }

    result = cide_run(s);
    if (result != 0) {
        std::cout << "[FAIL] Runtime: " << (cide_get_runtime_error(s) ? cide_get_runtime_error(s) : "?") << std::endl;
        cide_session_destroy(s);
        return 1;
    }

    int memCount = cide_memory_region_count(s);
    std::cout << "Memory regions: " << memCount << std::endl;

    bool foundHeap = false;
    for (int i = 0; i < memCount; i++) {
        unsigned int addr;
        int size;
        char name[64] = {0};
        char type[32] = {0};
        int isHeap, isFreed;
        cide_memory_region_get(s, i, &addr, &size, name, sizeof(name), type, sizeof(type), &isHeap, &isFreed);
        std::cout << "  Region " << i << ": addr=0x" << std::hex << addr << std::dec
                  << " size=" << size << " name=" << name
                  << " type=" << type << " heap=" << isHeap << " freed=" << isFreed << std::endl;
        if (isHeap) foundHeap = true;
    }

    // Read memory value
    if (memCount > 0) {
        unsigned int addr;
        int size;
        char name[64] = {0};
        char type[32] = {0};
        int isHeap, isFreed;
        cide_memory_region_get(s, 0, &addr, &size, name, sizeof(name), type, sizeof(type), &isHeap, &isFreed);
        int val;
        cide_memory_get_value(s, addr, &val);
        std::cout << "Value at region 0: " << val << std::endl;
    }

    cide_session_destroy(s);

    if (foundHeap && memCount >= 1) {
        std::cout << "[OK] Memory view test passed" << std::endl;
        return 0;
    }
    std::cout << "[FAIL] Expected heap region not found" << std::endl;
    return 1;
}
