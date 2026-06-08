#include <stdio.h>
class Multi {
    int a;
public:
    int b;
private:
    int c;
public:
    int d;
};
int main() {
    Multi m;
    m.b = 1;
    m.d = 2;
    printf("ok\n");
    return 0;
}
