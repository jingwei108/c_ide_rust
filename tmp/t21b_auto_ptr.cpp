#include <stdio.h>
struct Node { int x; };
int main() {
    struct Node* p = new struct Node;
    p->x = 42;
    printf("%d\n", p->x);
    delete p;
    return 0;
}
