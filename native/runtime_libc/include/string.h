typedef unsigned int size_t;

int strlen(const char* s);
char* strcpy(char* dest, const char* src);
char* strncpy(char* dest, const char* src, int n);
int strcmp(const char* s1, const char* s2);
char* strcat(char* dest, const char* src);
void* memcpy(void* dest, const void* src, int n);
void* memmove(void* dest, const void* src, int n);
void* memset(void* ptr, int value, int n);
char* strdup(const char* s);
