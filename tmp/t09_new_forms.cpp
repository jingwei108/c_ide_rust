#include <stdio.h>
int main() {
    int* p1 = new int;
    int* p2 = new int(42);
    int* p3 = new int[5];
    printf("ok\n");
    return 0;
}
