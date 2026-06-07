#include <stdio.h>

int isspace(int c);
int isalnum(int c);
int isprint(int c);
int iscntrl(int c);
int isxdigit(int c);

int main() {
    // C 标准只保证 ctype 函数返回"非零值"表示真，不保证具体数值。
    // 为避免 CRT 实现差异，统一转换为 0/1 输出。
    printf("%d\n", isspace(' ') ? 1 : 0);
    printf("%d\n", isspace('A') ? 1 : 0);
    printf("%d\n", isalnum('a') ? 1 : 0);
    printf("%d\n", isalnum('5') ? 1 : 0);
    printf("%d\n", isalnum(' ') ? 1 : 0);
    printf("%d\n", isprint('A') ? 1 : 0);
    printf("%d\n", isprint('\n') ? 1 : 0);
    printf("%d\n", iscntrl('\n') ? 1 : 0);
    printf("%d\n", iscntrl('A') ? 1 : 0);
    printf("%d\n", isxdigit('a') ? 1 : 0);
    printf("%d\n", isxdigit('F') ? 1 : 0);
    printf("%d\n", isxdigit('g') ? 1 : 0);
    return 0;
}
