#include <stdio.h>

template<class T>
struct list_node {
    T data;
    list_node* next;
};

template<class T>
class list {
    list_node<T>* head;
    list_node<T>* tail;
    int n_;
public:
    list() : head((list_node<T>*)0), tail((list_node<T>*)0), n_(0) {}
    void push_back(T x) {
        list_node<T>* node = new list_node<T>;
        node->data = x;
        node->next = (list_node<T>*)0;
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
        list_node<T>* p = head;
        while (i-- > 0 && p != (list_node<T>*)0) p = p->next;
        return p != (list_node<T>*)0 ? p->data : (T)0;
    }
    ~list() {
        list_node<T>* p = head;
        while (p != (list_node<T>*)0) {
            list_node<T>* next = p->next;
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
