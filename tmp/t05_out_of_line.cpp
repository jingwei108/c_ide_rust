#include <stdio.h>
class Bar {
public:
    int x;
    void set(int v);
};
void Bar::set(int v) { x = v; }
int main() {
    Bar b;
    b.set(42);
    printf("%d\n", b.x);
    return 0;
}
