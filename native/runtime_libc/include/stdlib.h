typedef unsigned int size_t;

void* malloc(int size);
void* calloc(int nmemb, int size);
void free(void* ptr);
void* realloc(void* ptr, int size);
int atoi(const char* s);
int abs(int n);
int rand(void);
void srand(int seed);
void qsort(void* base, int nmemb, int size, int (*compar)(const void*, const void*));
void* bsearch(const void* key, const void* base, int nmemb, int size, int (*compar)(const void*, const void*));
void exit(int status);
