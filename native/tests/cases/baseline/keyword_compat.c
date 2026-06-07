// @category: baseline
inline int add(int a, int b) { return a + b; }

int main() {
    register int x = 5;
    auto int y = 10;
    _Bool b1 = 1;
    bool b2 = 0;
    int *restrict p = &x;
    printf("%d %d %d %d %d", add(x, y), b1, b2, *p, x + y);
    return 0;
}
