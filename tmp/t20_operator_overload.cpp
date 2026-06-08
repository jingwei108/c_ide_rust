#include <stdio.h>
class A {
public:
    int x;
    A operator+(const A& o) { A r; r.x = x + o.x; return r; }
};
int main() {
    A a, b;
    A c = a + b;
    printf("%d\n", c.x);
    return 0;
}
