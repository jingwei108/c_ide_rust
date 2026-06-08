#include <stdio.h>
int main() {
    int x = 42;
    int&& r = std::move(x);
    printf("%d\n", r);
    return 0;
}
