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
    if (v->n == 0) return (char)'\0';
    return v->a[--v->n];
}

int cide_vec_size_char(cide_vec_char *v) {
    return v->n;
}

int cide_vec_capacity_char(cide_vec_char *v) {
    return v->m;
}

char cide_vec_front_char(cide_vec_char *v) {
    if (v->n == 0) return (char)'\0';
    return v->a[0];
}

char cide_vec_back_char(cide_vec_char *v) {
    if (v->n == 0) return (char)'\0';
    return v->a[v->n - 1];
}

void cide_vec_pop_front_char(cide_vec_char *v) {
    if (v->n == 0) return;
    int i;
    for (i = 0; i < v->n - 1; i++) {
        v->a[i] = v->a[i + 1];
    }
    v->n--;
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
