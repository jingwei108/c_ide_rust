// native/runtime_libc/cide/sort_int.cpp
// Cide 内置 int 数组排序的 C++ 实现

template<class T>
static void cide_sort_int_swap(T* a, T* b) {
    T t = *a;
    *a = *b;
    *b = t;
}

template<class T>
static void cide_sort_int_qsort(T* a, int left, int right) {
    if (left >= right) {
        return;
    }
    T pivot = a[(left + right) / 2];
    int i = left;
    int j = right;
    while (i <= j) {
        while (a[i] < pivot) {
            i++;
        }
        while (a[j] > pivot) {
            j--;
        }
        if (i <= j) {
            cide_sort_int_swap(&a[i], &a[j]);
            i++;
            j--;
        }
    }
    if (left < j) {
        cide_sort_int_qsort(a, left, j);
    }
    if (i < right) {
        cide_sort_int_qsort(a, i, right);
    }
}

template<class T>
void cide_sort_int(T* a, int n) {
    if (n > 1) {
        cide_sort_int_qsort(a, 0, n - 1);
    }
}

void __cide_force_instantiate_cide_sort_int() {
    int a[5] = {3, 1, 4, 1, 5};
    cide_sort_int(a, 5);
}
