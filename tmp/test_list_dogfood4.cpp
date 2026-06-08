#include <stdio.h>

struct list_node_int {
    int data;
    struct list_node_int* next;
};

class list_int {
    struct list_node_int* head;
    struct list_node_int* tail;
    int n_;
public:
    list_int() : head(0), tail(0), n_(0) {}
    void push_back(int x) {
        struct list_node_int* node = new struct list_node_int;
        node->data = x;
        node->next = 0;
        if (tail) {
            tail->next = node;
        } else {
            head = node;
        }
        tail = node;
        n_++;
    }
    int size() { return n_; }
    int get(int i) {
        struct list_node_int* p = head;
        while (i-- > 0 && p != 0) p = p->next;
        return p != 0 ? p->data : 0;
    }
    ~list_int() {
        struct list_node_int* p = head;
        while (p != 0) {
            struct list_node_int* next = p->next;
            delete p;
            p = next;
        }
    }
};

int main() {
    list_int l;
    l.push_back(1);
    l.push_back(2);
    printf("%d\n", l.size());
    printf("%d\n", l.get(0));
    printf("%d\n", l.get(1));
    return 0;
}
