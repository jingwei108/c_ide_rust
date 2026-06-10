// native/runtime_libc/cide/vector_float.cpp
// Cide 内置容器 vector<float> 的 C++ 接口声明
// 当前实现由 cide_vec_*.c 提供，未来替换为纯 C++ 实现

#ifndef CIDE_BUILTIN_CONTAINER
#define CIDE_BUILTIN_CONTAINER

template<class T>
class vector {
    int n;
    int m;
    T* a;
public:
    vector();
    void push_back(T x);
    T pop_back();
    int size();
    int capacity();
    T front();
    T back();
    void pop_front();
    T get(int i);
    void clear();
    ~vector();
};

// 显式实例化，提取脚本据此推导 cide_vec_float
template class vector<float>;

#endif
