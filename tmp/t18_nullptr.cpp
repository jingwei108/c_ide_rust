#include <stdio.h>
int main() {
    int* p = nullptr;
    printf("%d\n", p == 0 ? 1 : 0);
    return 0;
}
