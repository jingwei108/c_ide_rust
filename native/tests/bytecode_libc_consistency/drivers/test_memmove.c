#include <stdio.h>

void *memmove(void *dest, void *src, int n);

int main() {
    char buf[] = "ABCDEF";
    memmove(buf + 2, buf, 4);
    buf[6] = '\0';
    printf("%s\n", buf);
    return 0;
}
