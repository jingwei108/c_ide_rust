#include <stdio.h>

int main() {
    int arr[] = {1, 2, 3, 4, 5};
    for (auto x : arr) {
        printf("%d\n", x);
    }
    return 0;
}
