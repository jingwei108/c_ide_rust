typedef struct cide_list_node_int {
    int data;
    struct cide_list_node_int *next;
} cide_list_node_int;

typedef struct {
    cide_list_node_int *head;
    cide_list_node_int *tail;
    int n;
} cide_list_int;

void cide_list_init_int(cide_list_int *l) {
    l->head = (cide_list_node_int *)0;
    l->tail = (cide_list_node_int *)0;
    l->n = 0;
}

void cide_list_push_back_int(cide_list_int *l, int x) {
    cide_list_node_int *node = (cide_list_node_int *)malloc(sizeof(cide_list_node_int));
    node->data = x;
    node->next = (cide_list_node_int *)0;
    if (l->tail) {
        l->tail->next = node;
    } else {
        l->head = node;
    }
    l->tail = node;
    l->n++;
}

void cide_list_push_front_int(cide_list_int *l, int x) {
    cide_list_node_int *node = (cide_list_node_int *)malloc(sizeof(cide_list_node_int));
    node->data = x;
    node->next = l->head;
    l->head = node;
    if (l->tail == (cide_list_node_int *)0) {
        l->tail = node;
    }
    l->n++;
}

int cide_list_pop_back_int(cide_list_int *l) {
    if (!l->head) return 0;
    if (l->head == l->tail) {
        int val = l->head->data;
        free(l->head);
        l->head = (cide_list_node_int *)0;
        l->tail = (cide_list_node_int *)0;
        l->n = 0;
        return val;
    }
    cide_list_node_int *p = l->head;
    while (p->next != l->tail) p = p->next;
    int val = l->tail->data;
    free(l->tail);
    l->tail = p;
    p->next = (cide_list_node_int *)0;
    l->n--;
    return val;
}

int cide_list_size_int(cide_list_int *l) {
    return l->n;
}

int cide_list_front_int(cide_list_int *l) {
    if (!l->head) return 0;
    return l->head->data;
}

int cide_list_back_int(cide_list_int *l) {
    if (!l->tail) return 0;
    return l->tail->data;
}

void cide_list_pop_front_int(cide_list_int *l) {
    if (!l->head) return;
    cide_list_node_int *node = l->head;
    l->head = node->next;
    if (!l->head) l->tail = (cide_list_node_int *)0;
    free(node);
    l->n--;
}

int cide_list_get_int(cide_list_int *l, int i) {
    cide_list_node_int *p = l->head;
    while (i-- > 0 && p != (cide_list_node_int *)0) p = p->next;
    return p != (cide_list_node_int *)0 ? p->data : 0;
}

void cide_list_destroy_int(cide_list_int *l) {
    cide_list_node_int *p = l->head;
    while (p != (cide_list_node_int *)0) {
        cide_list_node_int *next = p->next;
        free(p);
        p = next;
    }
    l->head = (cide_list_node_int *)0;
    l->tail = (cide_list_node_int *)0;
    l->n = 0;
}
