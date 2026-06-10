typedef struct {
    int n;
    int m;
    char *s;
} cide_string;

void cide_string_init(cide_string *str) {
    str->n = 0;
    str->m = 0;
    str->s = (char *)0;
}

void cide_string_push_back(cide_string *str, char c) {
    if (str->n + 1 >= str->m) {
        str->m = str->m ? str->m << 1 : 2;
        str->s = (char *)realloc(str->s, sizeof(char) * str->m);
    }
    str->s[str->n++] = c;
    str->s[str->n] = (char)'\0';
}

char cide_string_pop_back(cide_string *str) {
    if (str->n == 0) return '\0';
    char c = str->s[--str->n];
    str->s[str->n] = (char)'\0';
    return c;
}

int cide_string_size(cide_string *str) {
    return str->n;
}

char cide_string_get(cide_string *str, int i) {
    return str->s[i];
}

char *cide_string_c_str(cide_string *str) {
    return str->s;
}

int cide_string_capacity(cide_string *str) {
    return str->m;
}

char cide_string_front(cide_string *str) {
    if (str->n == 0) return (char)'\0';
    return str->s[0];
}

char cide_string_back(cide_string *str) {
    if (str->n == 0) return (char)'\0';
    return str->s[str->n - 1];
}

void cide_string_pop_front(cide_string *str) {
    if (str->n == 0) return;
    int i;
    for (i = 0; i < str->n - 1; i++) {
        str->s[i] = str->s[i + 1];
    }
    str->n--;
    str->s[str->n] = (char)'\0';
}

void cide_string_clear(cide_string *str) {
    str->n = 0;
    if (str->s) str->s[0] = (char)'\0';
}

void cide_string_destroy(cide_string *str) {
    free(str->s);
    str->s = (char *)0;
    str->n = 0;
    str->m = 0;
}
