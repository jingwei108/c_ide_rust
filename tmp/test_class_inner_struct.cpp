#include <stdio.h>

template<class T>
class Foo {
    struct Node {
        T data;
        Node* next;
    };
    Node* head;
public:
    Foo() : head(0) {}
};

int main() {
    Foo<int> f;
    printf("ok\n");
    return 0;
}
