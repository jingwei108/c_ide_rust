#include <stdio.h>

template<class T>
class unique_ptr {
    T* p;
public:
    unique_ptr() : p((T*)0) {}
    unique_ptr(T* ptr) : p(ptr) {}
    T* get() { return p; }
    ~unique_ptr() { delete p; }
};

int main() {
    unique_ptr<int> p(new int(42));
    printf("%d\n", *p.get());
    return 0;
}
