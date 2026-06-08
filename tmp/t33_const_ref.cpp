#include <stdio.h>
int main() {
    int x = 5;
    const int& r = x;
    printf("%d\n", r);
    return 0;
}
