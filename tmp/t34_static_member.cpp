#include <stdio.h>
class A {
public:
    static int count;
    A() { count++; }
};
int A::count = 0;
int main() {
    A a;
    A b;
    printf("%d\n", A::count);
    return 0;
}
