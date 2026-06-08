#include <stdio.h>
int main() {
    cide_string s;
    cide_string_init(&s);
    cide_string_push_back(&s, 'h');
    cide_string_push_back(&s, 'i');
    printf("%d\n", cide_string_size(&s));
    printf("%c\n", cide_string_get(&s, 0));
    printf("%c\n", cide_string_get(&s, 1));
    cide_string_destroy(&s);
    return 0;
}
