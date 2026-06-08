#include <stdio.h>

template<class T>
class Foo {
    T* head;
public:
    Foo() : head((T*)0) {}
};

int main() {
    Foo<int> f;
    printf("ok\n");
    return 0;
}
