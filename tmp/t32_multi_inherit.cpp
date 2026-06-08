#include <stdio.h>
class A { public: int a; };
class B { public: int b; };
class C : public A, public B { public: int c; };
int main() {
    C obj;
    printf("ok\n");
    return 0;
}
