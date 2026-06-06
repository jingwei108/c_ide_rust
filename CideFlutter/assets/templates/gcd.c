#include <stdio.h>

int gcd(int a, int b) {
    while (b != 0) {
        int temp = b;
        b = a % b;
        a = temp;
    }
    return a;
}

int main() {
    int a = /*__PARAM_a__*/ 48, b = /*__PARAM_b__*/ 18;
    printf("%d\n", gcd(a, b));
    return 0;
}
