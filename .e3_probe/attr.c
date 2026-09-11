#include <stdio.h>
[[maybe_unused]] int u = 5;
int main(){ [[maybe_unused]] int v = 6; printf("%d %d", u, v); return 0; }
