# Cide CLI User Manual

> [中文版](CIDE_CLI.md)

`cide_cli` is the Rust backend command-line debugging tool of the Cide project. It can compile, run, and step-debug C code directly without launching the Flutter frontend.

## Build

```bash
cd native
cargo build --release --bin cide_cli
```

Output:
- Linux/macOS: `native/target/release/cide_cli`
- Windows: `native/target/release/cide_cli.exe`

## Basic Usage

```bash
cide_cli <command> <file> [options]
```

## Commands

| Command | Description |
|:--------|:------------|
| `compile <file>` | Compile C file and display diagnostics (errors/warnings/suggestions) |
| `run <file>` | Compile and run program at full speed |
| `step <file>` | Interactive step debugging |
| `unified <file>` | Unified mode (time-travel engine) batch execution and summary output |

## Options

| Option | Description |
|:-------|:------------|
| `-i <file>` | Read standard input from specified file (multi-line input for `scanf`/`fgets`, etc.) |

## Special Filenames

Using `-` as the filename tells the CLI to read source code from **standard input**, convenient for quick code snippet testing:

```bash
# Pipe mode
echo '#include <stdio.h>
int main() { printf("hello\n"); return 0; }' | cide_cli run -

# here-document mode
cide_cli compile - <<'EOF'
#include <stdio.h>
int main() {
    int a = 10, b = 20;
    printf("%d\n", a + b);
    return 0;
}
EOF
```

## Examples

### 1. Compile and Check Diagnostics

```bash
cide_cli compile hello.c
```

Sample output on success:
```
Compilation successful.
Detected algorithms:
  • Array traversal (confidence: 95%)
```

On error:
```
=== Diagnostics ===
[Error] 4:5  Type mismatch: cannot assign 'char[6]' to 'int' (E3004)
    Suggestion: The left and right sides of assignment or argument passing have inconsistent types...

Compilation failed.
```

### 2. Run at Full Speed

```bash
cide_cli run hello.c
```

Sample output:
```
Compilation successful.

=== Run Output ===
Hello, Cide CLI!

Program finished, return value: 0
```

### 3. Run with Input

```bash
# input.txt content:
# 5 7

cide_cli run sum.c -i input.txt
```

### 4. Interactive Step Debugging

```bash
cide_cli step hello.c
```

Interactive debug commands:

| Debug Command | Description |
|:--------------|:------------|
| `Enter` (empty input) | Execute next step |
| `p` / `print` | Print current local variables |
| `o` / `output` | Print current program output |
| `r` / `run` | Run to completion |
| `q` / `quit` | Quit debugging |

Sample output:
```
=== Interactive Step Debugging ===
Commands: [Enter]=next step, p=print variables, o=print output, q=quit, r=run to end

Step    0 | Line   0:   >
Step    1 | Line   3: int main() {  > p
  a: Int = 10
  b: Int = 20
Step    2 | Line   4: int a = 10;  >
```

### 5. Unified Mode (Time-Travel Engine)

```bash
cide_cli unified hello.c
```

Sample output:
```
=== Unified Mode Execution (Time-Travel Engine) ===
  Executed 117 steps

=== Execution Summary ===
Total steps: 117
Status: Normal termination

=== Final Output ===
sum=15
```

Unified mode records every VM state step, supports checkpoint save and rollback, and uses the same engine as the frontend "time-travel" feature.

## Quick Test Snippets

No temporary files needed; quickly verify code via standard input:

```bash
# Test printf
echo '#include <stdio.h>
int main() { printf("ok\n"); return 0; }' | cide_cli run -

# Test loop
cide_cli unified - <<'EOF'
#include <stdio.h>
int main() {
    int s = 0;
    for (int i = 1; i <= 100; i++) s += i;
    printf("%d\n", s);
    return 0;
}
EOF

# Test scanf + input
cat <<'EOF' | cide_cli run - -i /dev/stdin
#include <stdio.h>
int main() { int a,b; scanf("%d%d",&a,&b); printf("%d\n",a+b); return 0; }
EOF
# Then type two numbers and press Ctrl+D (Unix) or Ctrl+Z (Windows)
```

> **Note**: When reading source code from stdin using `-`, you cannot use `-i -` to read input data from the same stdin. Write input data to a file and use `-i data.txt` instead.
