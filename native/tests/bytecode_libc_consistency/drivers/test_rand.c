#include <stdio.h>

extern void srand(unsigned int s);
extern int rand(void);

int main() {
    srand(1);
    printf("%d\n", rand());
    printf("%d\n", rand());
    printf("%d\n", rand());
    srand(42);
    printf("%d\n", rand());
    printf("%d\n", rand());
    return 0;
}
