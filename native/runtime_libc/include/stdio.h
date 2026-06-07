typedef unsigned int size_t;
typedef void* FILE;

int printf(const char* fmt);
int scanf(const char* fmt);
int fprintf(FILE* stream, const char* fmt);
int getchar(void);
int putchar(int c);
FILE* fopen(const char* path, const char* mode);
int fclose(FILE* stream);
int fread(void* ptr, int size, int nmemb, FILE* stream);
int fwrite(void* ptr, int size, int nmemb, FILE* stream);
int feof(FILE* stream);
char* fgets(char* s, int n, FILE* stream);
int fputs(const char* s, FILE* stream);
