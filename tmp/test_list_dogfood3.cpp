#include <stdio.h>

struct list_node_int {
    int data;
    list_node_int* next;
};

class list_int {
    list_node_int* head;
    list_node_int* tail;
    int n_;
public:
    list_int() : head((list_node_int*)0), tail((list_node_int*)0), n_(0) {}
    void push_back(int x) {
        list_node_int* node = new list_node_int;
        node->data = x;
        node->next = (list_node_int*)0;
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
        list_node_int* p = head;
        while (i-- > 0 && p != (list_node_int*)0) p = p->next;
        return p != (list_node_int*)0 ? p->data : 0;
    }
    ~list_int() {
        list_node_int* p = head;
        while (p != (list_node_int*)0) {
            list_node_int* next = p->next;
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
