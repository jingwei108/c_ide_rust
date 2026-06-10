typedef struct {
    int n;
    int m;
    float *a;
} cide_vec_float;

void cide_vec_init_float(cide_vec_float *v) {
    v->n = 0;
    v->m = 0;
    v->a = (float *)0;
}

void cide_vec_push_float(cide_vec_float *v, float x) {
    if (v->n == v->m) {
        v->m = v->m ? v->m << 1 : 2;
        v->a = (float *)realloc(v->a, sizeof(float) * v->m);
    }
    v->a[v->n++] = x;
}

float cide_vec_pop_float(cide_vec_float *v) {
    if (v->n == 0) return 0.0;
    return v->a[--v->n];
}

int cide_vec_size_float(cide_vec_float *v) {
    return v->n;
}

int cide_vec_capacity_float(cide_vec_float *v) {
    return v->m;
}

float cide_vec_front_float(cide_vec_float *v) {
    if (v->n == 0) return 0.0f;
    return v->a[0];
}

float cide_vec_back_float(cide_vec_float *v) {
    if (v->n == 0) return 0.0f;
    return v->a[v->n - 1];
}

void cide_vec_pop_front_float(cide_vec_float *v) {
    if (v->n == 0) return;
    int i;
    for (i = 0; i < v->n - 1; i++) {
        v->a[i] = v->a[i + 1];
    }
    v->n--;
}

float cide_vec_get_float(cide_vec_float *v, int i) {
    return v->a[i];
}

void cide_vec_clear_float(cide_vec_float *v) {
    v->n = 0;
}

void cide_vec_destroy_float(cide_vec_float *v) {
    free(v->a);
    v->a = (float *)0;
    v->n = 0;
    v->m = 0;
}
