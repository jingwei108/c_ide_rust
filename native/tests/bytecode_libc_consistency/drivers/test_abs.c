#include <stdio.h>

int abs(int n);

int main() {
    printf("%d\n", abs(5));
    printf("%d\n", abs(-5));
    printf("%d\n", abs(0));
    printf("%d\n", abs(-123));
    printf("%d\n", abs(2147483647));
    return 0;
}
