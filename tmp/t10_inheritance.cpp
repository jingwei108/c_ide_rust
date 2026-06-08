#include <stdio.h>
class Base { public: int x; };
class Derived : public Base { public: int y; };
int main() {
    Derived d;
    d.x = 1;
    d.y = 2;
    printf("%d %d\n", d.x, d.y);
    return 0;
}
