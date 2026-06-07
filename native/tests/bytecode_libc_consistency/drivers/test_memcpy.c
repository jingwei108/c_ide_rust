#include <stdio.h>

void *memcpy(void *dest, void *src, int n);

int main() {
    char src[] = "ABCDEF";
    char dest[16];
    memcpy(dest, src, 6);
    dest[6] = '\0';
    printf("%s\n", dest);
    return 0;
}
