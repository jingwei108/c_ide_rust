#include <stdio.h>

int strlen(char *s);

int main() {
    printf("%d\n", strlen("hello"));
    printf("%d\n", strlen(""));
    printf("%d\n", strlen("a"));
    printf("%d\n", strlen("1234567890"));
    return 0;
}
