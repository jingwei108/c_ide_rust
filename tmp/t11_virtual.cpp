#include <stdio.h>
class Base {
public:
    virtual int get() { return 1; }
};
class Derived : public Base {
public:
    int get() { return 2; }
};
int main() {
    Base* b = new Derived;
    printf("%d\n", b->get());
    return 0;
}
