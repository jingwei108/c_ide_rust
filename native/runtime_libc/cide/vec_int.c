typedef struct {
    int n;
    int m;
    int *a;
} cide_vec_int;

void cide_vec_init_int(cide_vec_int *v) {
    v->n = 0;
    v->m = 0;
    v->a = (int *)0;
}

void cide_vec_push_int(cide_vec_int *v, int x) {
    if (v->n == v->m) {
        v->m = v->m ? v->m << 1 : 2;
        v->a = (int *)realloc(v->a, sizeof(int) * v->m);
    }
    v->a[v->n++] = x;
}

int cide_vec_pop_int(cide_vec_int *v) {
    if (v->n == 0) return 0;
    return v->a[--v->n];
}

int cide_vec_size_int(cide_vec_int *v) {
    return v->n;
}

int cide_vec_capacity_int(cide_vec_int *v) {
    return v->m;
}

int cide_vec_front_int(cide_vec_int *v) {
    if (v->n == 0) return 0;
    return v->a[0];
}

int cide_vec_back_int(cide_vec_int *v) {
    if (v->n == 0) return 0;
    return v->a[v->n - 1];
}

void cide_vec_pop_front_int(cide_vec_int *v) {
    if (v->n == 0) return;
    int i;
    for (i = 0; i < v->n - 1; i++) {
        v->a[i] = v->a[i + 1];
    }
    v->n--;
}

int cide_vec_get_int(cide_vec_int *v, int i) {
    return v->a[i];
}

void cide_vec_clear_int(cide_vec_int *v) {
    v->n = 0;
}

void cide_vec_destroy_int(cide_vec_int *v) {
    free(v->a);
    v->a = (int *)0;
    v->n = 0;
    v->m = 0;
}
