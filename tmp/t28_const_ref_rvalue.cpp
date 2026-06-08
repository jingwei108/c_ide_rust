#include <stdio.h>
int main() {
    const int& r = 5;
    printf("%d\n", r);
    return 0;
}
