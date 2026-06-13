#include <stdio.h>

class Point {
public:
    int x;
    int y;
    int sum() { return x + y; }
};

int main() {
    Point p;
    p.x = 3;
    p.y = 4;
    printf("%d\n", p.sum());
    return 0;
}
