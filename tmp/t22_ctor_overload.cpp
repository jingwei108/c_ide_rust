#include <stdio.h>
class Box {
public:
    int x;
    Box() { x = 0; }
    Box(int v) { x = v; }
};
int main() {
    Box a;
    Box b(42);
    printf("%d %d\n", a.x, b.x);
    return 0;
}
