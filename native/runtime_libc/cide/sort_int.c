static void cide_sort_int_swap(int *a, int *b) {
    int t = *a;
    *a = *b;
    *b = t;
}

static void cide_sort_int_qsort(int *a, int left, int right) {
    if (left >= right) return;
    int pivot = a[(left + right) / 2];
    int i = left;
    int j = right;
    while (i <= j) {
        while (a[i] < pivot) i++;
        while (a[j] > pivot) j--;
        if (i <= j) {
            cide_sort_int_swap(&a[i], &a[j]);
            i++;
            j--;
        }
    }
    if (left < j) cide_sort_int_qsort(a, left, j);
    if (i < right) cide_sort_int_qsort(a, i, right);
}

void cide_sort_int(int *a, int n) {
    if (n > 1) {
        cide_sort_int_qsort(a, 0, n - 1);
    }
}
