#include <stdio.h>
int main() {
    int a = 1, b = 2;
    auto f1 = [a](int x) { return x + a; };
    auto f2 = [&a](int x) { return x + a; };
    auto f3 = [a, &b](int x) { return x + a + b; };
    printf("%d\n", f1(10));
    printf("%d\n", f2(10));
    printf("%d\n", f3(10));
    return 0;
}
