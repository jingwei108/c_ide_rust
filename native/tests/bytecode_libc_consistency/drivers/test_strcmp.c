#include <stdio.h>

int strcmp(char *s1, char *s2);

int main() {
    int r;
    r = strcmp("abc", "abc");
    printf("%d\n", r == 0 ? 0 : (r < 0 ? -1 : 1));
    r = strcmp("abc", "abd");
    printf("%d\n", r == 0 ? 0 : (r < 0 ? -1 : 1));
    r = strcmp("abd", "abc");
    printf("%d\n", r == 0 ? 0 : (r < 0 ? -1 : 1));
    r = strcmp("", "");
    printf("%d\n", r == 0 ? 0 : (r < 0 ? -1 : 1));
    r = strcmp("a", "ab");
    printf("%d\n", r == 0 ? 0 : (r < 0 ? -1 : 1));
    r = strcmp("ab", "a");
    printf("%d\n", r == 0 ? 0 : (r < 0 ? -1 : 1));
    return 0;
}
