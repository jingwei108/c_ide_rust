#include <stdio.h>

struct Node {
    int data;
};

int main() {
    printf("%d\n", sizeof(struct Node));
    struct Node* p = new struct Node;
    printf("%d\n", p == 0 ? 0 : 1);
    return 0;
}
