#include <stdio.h>
struct Node { int x; };
int main() {
    Node n;
    n.x = 1;
    printf("%d\n", n.x);
    return 0;
}
