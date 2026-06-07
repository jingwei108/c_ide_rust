#include <stdio.h>

char *strcat(char *dest, char *src);

int main() {
    char buf[32] = "Hello";
    printf("%s\n", strcat(buf, ", World!"));
    printf("%s\n", strcat(buf, ""));
    return 0;
}
