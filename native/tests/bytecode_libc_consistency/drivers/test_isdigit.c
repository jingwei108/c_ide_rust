#include <stdio.h>

int isdigit(int c);

int main() {
    // C 标准只保证 isdigit 返回"非零值"表示真，不保证具体数值。
    // 为避免 CRT 实现差异，统一转换为 0/1 输出。
    printf("%d\n", isdigit('0') ? 1 : 0);
    printf("%d\n", isdigit('9') ? 1 : 0);
    printf("%d\n", isdigit('a') ? 1 : 0);
    printf("%d\n", isdigit(' ') ? 1 : 0);
    printf("%d\n", isdigit('/') ? 1 : 0);
    printf("%d\n", isdigit(':') ? 1 : 0);
    return 0;
}
