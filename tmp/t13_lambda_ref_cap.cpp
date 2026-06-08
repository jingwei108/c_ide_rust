#include <stdio.h>
int main() {
    int a = 1;
    auto f = [&a](int x) { return x + a; };
    printf("%d\n", f(10));
    return 0;
}
