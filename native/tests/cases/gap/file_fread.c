// @category: file_io
int main() { FILE* f = fopen("test.txt", "r"); char buf[20]; fread(buf, 1, 5, f); buf[5] = 0; printf("%s", buf); fclose(f); return 0; }
