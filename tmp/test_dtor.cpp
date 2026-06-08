#include <stdio.h>

class Foo {
    int x;
public:
    Foo() : x(0) {}
    ~Foo() {
        printf("dtor\n");
    }
};

int main() {
    Foo f;
    printf("ok\n");
    return 0;
}
