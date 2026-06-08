#include <stdio.h>
int main() {
    int x = 42;
    auto& r = x;
    const auto& cr = x;
    printf("%d %d\n", r, cr);
    return 0;
}
