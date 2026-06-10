// native/runtime_libc/cide/list_int.cpp
// Cide 内置容器 list<int> 的 C++ 接口声明
// 当前实现由 cide_list_*.c 提供，未来替换为纯 C++ 实现

#ifndef CIDE_BUILTIN_CONTAINER
#define CIDE_BUILTIN_CONTAINER

template<class T>
class list {
    void* head;
    void* tail;
    int n;
public:
    list();
    void push_back(T x);
    void push_front(T x);
    T pop_back();
    int size();
    T front();
    T back();
    void pop_front();
    T get(int i);
    void clear();
    ~list();
};

// 显式实例化，提取脚本据此推导 cide_list_int
template class list<int>;

#endif
