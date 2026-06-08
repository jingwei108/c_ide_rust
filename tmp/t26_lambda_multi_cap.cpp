#include <stdio.h>
int main() {
    int a = 1, b = 2;
    auto f = [a, &b](int x) { return x + a + b; };
    printf("%d\n", f(10));
    return 0;
}
