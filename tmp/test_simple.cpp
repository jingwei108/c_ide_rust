#include <stdio.h>

struct Node {
    int data;
    Node* next;
};

class Foo {
    Node* head;
public:
    Foo() : head((Node*)0) {}
};

int main() {
    Foo f;
    printf("ok\n");
    return 0;
}
