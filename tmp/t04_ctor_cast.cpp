#include <stdio.h>
struct Foo {
    int* p;
    Foo() : p((int*)0) {}
};
int main() {
    Foo f;
    printf("ok\n");
    return 0;
}
