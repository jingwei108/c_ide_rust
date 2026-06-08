#include <stdio.h>
template<class T>
struct Pair { T first; T second; };
int main() {
    Pair<int> p;
    p.first = 1;
    printf("%d\n", p.first);
    return 0;
}
