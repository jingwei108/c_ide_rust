#include <stdio.h>
class Box {
public:
    int x;
    Box() { x = 42; }
    ~Box() { printf("dtor\n"); }
};
int main() {
    Box* arr = new Box[3];
    printf("%d\n", arr[0].x);
    delete[] arr;
    return 0;
}
