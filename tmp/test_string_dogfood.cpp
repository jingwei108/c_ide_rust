#include <stdio.h>

class string {
    char* data;
    int size_;
    int capacity_;
public:
    string() : data(0), size_(0), capacity_(0) {}
    void push_back(char c) {
        if (size_ >= capacity_) {
            int new_cap = capacity_ == 0 ? 4 : capacity_ * 2;
            char* new_data = new char[new_cap];
            for (int i = 0; i < size_; i++) new_data[i] = data[i];
            delete[] data;
            data = new_data;
            capacity_ = new_cap;
        }
        data[size_++] = c;
    }
    char get(int i) { return data[i]; }
    int size() { return size_; }
    ~string() { delete[] data; }
};

int main() {
    string s;
    s.push_back('h');
    s.push_back('i');
    printf("%d\n", s.size());
    printf("%c\n", s.get(0));
    printf("%c\n", s.get(1));
    return 0;
}
