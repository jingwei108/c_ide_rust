#include <stdio.h>
class A {
public:
    int x;
    int get() const { return x; }
};
int main() {
    A a;
    a.x = 42;
    printf("%d\n", a.get());
    return 0;
}
