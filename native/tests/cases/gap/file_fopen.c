// @category: file_io
int main() { FILE* f = fopen("test.txt", "r"); if (f) printf("ok"); fclose(f); return 0; }
