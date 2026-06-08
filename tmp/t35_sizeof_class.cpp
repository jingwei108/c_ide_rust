#include <stdio.h>
class A { int x; };
class B : public A { int y; };
int main() {
    printf("%d\n", sizeof(A));
    printf("%d\n", sizeof(B));
    return 0;
}
