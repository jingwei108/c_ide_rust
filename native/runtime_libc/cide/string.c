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
    if (str->n == str->m) {
        str->m = str->m ? str->m << 1 : 2;
        str->s = (char *)realloc(str->s, str->m);
    }
    str->s[str->n++] = c;
}

char cide_string_pop_back(cide_string *str) {
    return str->s[--str->n];
}

int cide_string_size(cide_string *str) {
    return str->n;
}

char cide_string_get(cide_string *str, int i) {
    return str->s[i];
}

void cide_string_clear(cide_string *str) {
    str->n = 0;
}

void cide_string_destroy(cide_string *str) {
    free(str->s);
    str->s = (char *)0;
    str->n = 0;
    str->m = 0;
}
