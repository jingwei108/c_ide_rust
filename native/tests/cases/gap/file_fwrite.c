// @category: file_io
int main() { FILE* f = fopen("out.txt", "w"); fwrite("hello", 1, 5, f); fclose(f); printf("ok"); return 0; }
