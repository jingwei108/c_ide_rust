#include <cide_capi.h>
#include <iostream>

int main() {
    const char* code = "int main() { int a = 3; int b = 5; return a + b; }";

    CideSession* s = cide_session_create();
    std::cout << "Session created" << std::endl;
    int ok = cide_compile(s, code);
    std::cout << "Compile result: " << ok << std::endl;
    if (ok != 0) {
        char buf[4096];
        int len = cide_get_compile_errors_buf(s, buf, sizeof(buf));
        std::cout << "COMPILE ERROR: " << std::string(buf, len) << std::endl;
        cide_session_destroy(s);
        return 1;
    }

    std::cout << "Running..." << std::endl;
    int runOk = cide_run(s);
    std::cout << "Run finished" << std::endl;
    
    char outBuf[4096];
    int outLen = cide_get_output_length(s);
    cide_get_output(s, outBuf, sizeof(outBuf));
    outBuf[outLen] = '\0';
    
    std::cout << "Output: [" << outBuf << "]" << std::endl;
    std::cout << "Run OK: " << runOk << std::endl;
    
    cide_session_destroy(s);
    return 0;
}
