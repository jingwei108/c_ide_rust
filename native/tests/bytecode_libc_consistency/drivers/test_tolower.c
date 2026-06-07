#include <stdio.h>

int tolower(int c);
int toupper(int c);

int main() {
    printf("%c\n", tolower('A'));
    printf("%c\n", tolower('Z'));
    printf("%c\n", tolower('a'));
    printf("%c\n", tolower('5'));
    printf("%c\n", toupper('a'));
    printf("%c\n", toupper('z'));
    printf("%c\n", toupper('Z'));
    printf("%c\n", toupper('5'));
    return 0;
}
