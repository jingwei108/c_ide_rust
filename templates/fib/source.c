#include <stdio.h>

int fibonacci(int n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

int main() {
    int result = fibonacci(/*__PARAM_n__*/ 7);
    printf("%d\n", result);
    return 0;
}
