use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn compile_and_run(source: &str) {
    let _ = cide_native::flutter_bridge::compile_and_run(black_box(source.to_string()));
}

fn benchmark_bubble_sort(c: &mut Criterion) {
    let source = r#"
int main() {
    int arr[100];
    for (int i = 0; i < 100; i++) arr[i] = 100 - i;
    for (int i = 0; i < 99; i++) {
        for (int j = 0; j < 99 - i; j++) {
            if (arr[j] > arr[j + 1]) {
                int t = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = t;
            }
        }
    }
    return 0;
}
"#;
    c.bench_function("bubble_sort_100", |b| {
        b.iter(|| compile_and_run(source))
    });
}

fn benchmark_factorial(c: &mut Criterion) {
    let source = r#"
int fact(int n) {
    if (n <= 1) return 1;
    return n * fact(n - 1);
}
int main() {
    return fact(20);
}
"#;
    c.bench_function("factorial_recursive_20", |b| {
        b.iter(|| compile_and_run(source))
    });
}

criterion_group!(benches, benchmark_bubble_sort, benchmark_factorial);
criterion_main!(benches);
