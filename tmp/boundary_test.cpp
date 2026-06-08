// Boundary test suite for C++ compiler
#include <stdio.h>

// Test 1: struct tag alias in C++ mode
struct Node { int x; };
void test1() {
    Node n;  // Should work in standard C++
    n.x = 1;
}

// Test 2: nested struct inside class template
template<class T>
class Outer {
    struct Inner { T val; };
    Inner* p;
public:
    Outer() : p(0) {}
};

// Test 3: template struct
template<class T>
struct Pair {
    T first;
    T second;
};

// Test 4: ctor init list with explicit cast
struct Foo {
    int* p;
    Foo() : p((int*)0) {}
};

// Test 5: out-of-line member definition
class Bar {
public:
    int x;
    void set(int v);
};
void Bar::set(int v) { x = v; }

// Test 6: multiple access specifiers
class Multi {
    int a;
public:
    int b;
private:
    int c;
public:
    int d;
};

// Test 7: lambda capture modes
void test7() {
    int a = 1, b = 2;
    auto f1 = [a](int x) { return x + a; };
    auto f2 = [&a](int x) { return x + a; };
    auto f3 = [a, &b](int x) { return x + a + b; };
    // auto f4 = [&]  // capture-all by reference
    // auto f5 = [=]  // capture-all by value
}

// Test 8: range-for with reference
void test8() {
    int arr[] = {1, 2, 3};
    for (auto& x : arr) { x = x * 2; }
    for (const auto& x : arr) { /* read only */ }
}

// Test 9: new forms
void test9() {
    int* p1 = new int;
    int* p2 = new int(42);
    int* p3 = new int[5];
}

// Test 10: inheritance
class Base { public: int x; };
class Derived : public Base { public: int y; };

int main() {
    printf("boundary test compiled\n");
    return 0;
}
