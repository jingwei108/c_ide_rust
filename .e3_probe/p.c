#include <stdio.h>
#include <stddef.h>
int main(){ int x = 0; if (x) { printf("never"); } else { unreachable(); } return 0; }