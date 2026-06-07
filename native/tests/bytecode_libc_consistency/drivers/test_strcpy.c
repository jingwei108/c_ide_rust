#include <stdio.h>

char *strcpy(char *dest, char *src);

int main() {
    char buf[16];
    printf("%s\n", strcpy(buf, "hello"));
    printf("%s\n", strcpy(buf, ""));
    printf("%s\n", strcpy(buf, "1234567890"));
    return 0;
}
