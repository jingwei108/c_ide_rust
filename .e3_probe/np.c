#include <stdio.h>
int main(){ int *p = nullptr; printf("%d %d", p == (int*)0, p != 0); return 0; }
