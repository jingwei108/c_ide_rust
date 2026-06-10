// native/runtime_libc/cide/string.cpp
// Cide 内置容器 string 的 C++ 接口声明
// 当前实现由 cide_string.c 提供，未来替换为纯 C++ 实现

#ifndef CIDE_BUILTIN_CONTAINER
#define CIDE_BUILTIN_CONTAINER

class string {
    int n;
    int m;
    char* s;
public:
    string();
    void push_back(char c);
    char pop_back();
    int size();
    int capacity();
    char front();
    char back();
    char get(int i);
    char* c_str();
    void pop_front();
    void clear();
    ~string();
};

#endif
