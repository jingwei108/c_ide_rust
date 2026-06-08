#include <stdio.h>
class A { public: int x; };
int main() {
    A a;
    // a.operator+(1);  // should error
    printf("ok\n");
    return 0;
}
