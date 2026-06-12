#include <stdio.h>
class unique_ptr_int {
    int* p;
public:
    unique_ptr_int(int* x) : p(x) {}
    void move_from(unique_ptr_int* o) { p = o->p; o->p = (int*)0; }
    int* get() { return p; }
    ~unique_ptr_int() { if (p) delete p; }
};
int main() {
    unique_ptr_int a(new int(7));
    unique_ptr_int b((int*)0);
    b.move_from(&a);
    printf("%d\n", *b.get());
    printf("%d\n", a.get() ? 1 : 0);
    return 0;
}
