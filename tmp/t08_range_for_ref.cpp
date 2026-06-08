#include <stdio.h>
int main() {
    int arr[] = {1, 2, 3};
    for (auto& x : arr) { x = x * 2; }
    for (const auto& x : arr) { printf("%d\n", x); }
    return 0;
}
