// @category: arch_diff_bug
int f(int a[5]) { return sizeof(a); } int main() { int arr[5]; printf("%d", f(arr)); return 0; }
