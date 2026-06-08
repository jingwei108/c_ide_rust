#include <stdio.h>
class Outer {
    struct Inner { int val; };
    Inner* p;
public:
    Outer() : p(0) {}
};
int main() {
    Outer o;
    printf("ok\n");
    return 0;
}
