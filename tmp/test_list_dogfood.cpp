#include <stdio.h>

template<class T>
class list {
    struct Node {
        T data;
        Node* next;
    };
    Node* head;
    Node* tail;
    int n_;
public:
    list() : head((Node*)0), tail((Node*)0), n_(0) {}
    void push_back(T x) {
        Node* node = new Node;
        node->data = x;
        node->next = (Node*)0;
        if (tail) {
            tail->next = node;
        } else {
            head = node;
        }
        tail = node;
        n_++;
    }
    int size() { return n_; }
    T get(int i) {
        Node* p = head;
        while (i-- > 0 && p != (Node*)0) p = p->next;
        return p != (Node*)0 ? p->data : (T)0;
    }
    ~list() {
        Node* p = head;
        while (p != (Node*)0) {
            Node* next = p->next;
            delete p;
            p = next;
        }
    }
};

int main() {
    list<int> l;
    l.push_back(1);
    l.push_back(2);
    printf("%d\n", l.size());
    printf("%d\n", l.get(0));
    printf("%d\n", l.get(1));
    return 0;
}
