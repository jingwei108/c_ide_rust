#include <stdio.h>
class Box {
public:
    int x;
    explicit Box(int v) { x = v; }
};
int main() {
    Box b(42);
    printf("%d\n", b.x);
    return 0;
}
