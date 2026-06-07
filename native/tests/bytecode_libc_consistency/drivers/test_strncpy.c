#include <stdio.h>

char *strncpy(char *dest, char *src, int n);

int main() {
    char buf[16];
    strncpy(buf, "hello world", 5);
    buf[5] = '\0';
    printf("%s\n", buf);
    strncpy(buf, "hi", 8);
    printf("%s\n", buf);
    return 0;
}
