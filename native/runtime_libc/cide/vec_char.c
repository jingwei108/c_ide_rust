typedef struct {
    int n;
    int m;
    char *a;
} cide_vec_char;

void cide_vec_init_char(cide_vec_char *v) {
    v->n = 0;
    v->m = 0;
    v->a = (char *)0;
}

void cide_vec_push_char(cide_vec_char *v, char x) {
    if (v->n == v->m) {
        v->m = v->m ? v->m << 1 : 2;
        v->a = (char *)realloc(v->a, sizeof(char) * v->m);
    }
    v->a[v->n++] = x;
}

char cide_vec_pop_char(cide_vec_char *v) {
    return v->a[--v->n];
}

int cide_vec_size_char(cide_vec_char *v) {
    return v->n;
}

char cide_vec_get_char(cide_vec_char *v, int i) {
    return v->a[i];
}

void cide_vec_clear_char(cide_vec_char *v) {
    v->n = 0;
}

void cide_vec_destroy_char(cide_vec_char *v) {
    free(v->a);
    v->a = (char *)0;
    v->n = 0;
    v->m = 0;
}
