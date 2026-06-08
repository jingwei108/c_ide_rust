#include <stdio.h>
int main() {
    auto p = new int;
    *p = 42;
    printf("%d\n", *p);
    delete p;
    return 0;
}
