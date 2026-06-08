#include <stdio.h>
int main() {
    cide_list_int l;
    cide_list_init_int(&l);
    cide_list_push_back_int(&l, 1);
    cide_list_push_back_int(&l, 2);
    printf("%d\n", cide_list_size_int(&l));
    printf("%d\n", cide_list_get_int(&l, 0));
    printf("%d\n", cide_list_get_int(&l, 1));
    cide_list_destroy_int(&l);
    return 0;
}
