#include <stdio.h>
struct Node { int x; };
int main() {
    auto p = new Node;
    p->x = 42;
    printf("%d\n", p->x);
    delete p;
    return 0;
}
