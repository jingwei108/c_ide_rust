typedef char* va_list;

void __cide_va_start(char** ap, void* last, int last_size);
void* __cide_va_arg(char** ap, int size);
void __cide_va_end(char** ap);

#define va_start(ap, last) __cide_va_start(&(ap), &(last), sizeof(last))
#define va_arg(ap, type) (*(type*)__cide_va_arg(&(ap), sizeof(type)))
#define va_end(ap) __cide_va_end(&(ap))
