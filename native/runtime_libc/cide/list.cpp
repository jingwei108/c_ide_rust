// native/runtime_libc/cide/list.cpp
// Cide 内置容器 list<T> 的 C++ 模板实现

template <class T>
class cide_list_node {
public:
    T data;
    cide_list_node<T>* next;
};

template <class T>
class cide_list {
    cide_list_node<T>* head;
    cide_list_node<T>* tail;
    int n;

public:
    cide_list() {
        head = (cide_list_node<T>*)0;
        tail = (cide_list_node<T>*)0;
        n = 0;
    }

    void push_back(T x) {
        cide_list_node<T>* node = new cide_list_node<T>;
        node->data = x;
        node->next = (cide_list_node<T>*)0;
        if (tail) {
            tail->next = node;
        } else {
            head = node;
        }
        tail = node;
        n++;
    }

    void push_front(T x) {
        cide_list_node<T>* node = new cide_list_node<T>;
        node->data = x;
        node->next = head;
        head = node;
        if (!tail) {
            tail = node;
        }
        n++;
    }

    T pop_back() {
        if (!head) {
            return (T)0;
        }
        if (head == tail) {
            T val = head->data;
            delete head;
            head = (cide_list_node<T>*)0;
            tail = (cide_list_node<T>*)0;
            n = 0;
            return val;
        }
        cide_list_node<T>* p = head;
        while (p->next != tail) {
            p = p->next;
        }
        T val = tail->data;
        delete tail;
        tail = p;
        p->next = (cide_list_node<T>*)0;
        n--;
        return val;
    }

    int size() {
        return n;
    }

    T front() {
        if (!head) {
            return (T)0;
        }
        return head->data;
    }

    T back() {
        if (!tail) {
            return (T)0;
        }
        return tail->data;
    }

    void pop_front() {
        if (!head) {
            return;
        }
        cide_list_node<T>* node = head;
        head = node->next;
        if (!head) {
            tail = (cide_list_node<T>*)0;
        }
        delete node;
        n--;
    }

    T get(int i) {
        cide_list_node<T>* p = head;
        while (i-- > 0 && p != (cide_list_node<T>*)0) {
            p = p->next;
        }
        if (p == (cide_list_node<T>*)0) {
            return (T)0;
        }
        return p->data;
    }

    void clear() {
        cide_list_node<T>* p = head;
        while (p != (cide_list_node<T>*)0) {
            cide_list_node<T>* next = p->next;
            delete p;
            p = next;
        }
        head = (cide_list_node<T>*)0;
        tail = (cide_list_node<T>*)0;
        n = 0;
    }

    ~cide_list() {
        clear();
    }
};

template class cide_list_node<int>;
template class cide_list<int>;
