#include <stdio.h>

struct Node {
    int data;
};

class Foo {
    Node* head;
public:
    Foo() : head(0) {}
    void bar() {
        head = new Node;
    }
    ~Foo() {
        delete head;
    }
};

int main() {
    Foo f;
    f.bar();
    printf("ok\n");
    return 0;
}
