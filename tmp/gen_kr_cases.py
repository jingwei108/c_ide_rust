import subprocess
import tempfile
from pathlib import Path

BASE = Path("native/tests/cases/knr")
GOLDEN = Path("native/tests/cases_golden/knr")
CLANG = "clang"

BASE.mkdir(parents=True, exist_ok=True)
GOLDEN.mkdir(parents=True, exist_ok=True)

cases = []

def add(name, code, input_data=None):
    cases.append((name, code, input_data))

# Helper to add EOF define if code contains EOF but no define
EOF_DEFINE = "#define EOF -1\n"

def ensure_eof_define(code):
    if "EOF" in code and "#define EOF" not in code:
        # Insert after #include lines
        lines = code.splitlines(keepends=True)
        insert_idx = 0
        for i, line in enumerate(lines):
            if line.strip().startswith("#include"):
                insert_idx = i + 1
        lines.insert(insert_idx, EOF_DEFINE)
        return "".join(lines)
    return code

add("kr_1_3", '#include <stdio.h>\nint main() {\n    float fahr, celsius;\n    int lower, upper, step;\n    lower = 0; upper = 300; step = 20;\n    fahr = lower;\n    while (fahr <= upper) {\n        celsius = (5.0/9.0) * (fahr - 32.0);\n        printf("%3.0f %6.1f\\n", fahr, celsius);\n        fahr = fahr + step;\n    }\n    return 0;\n}\n')

add("kr_1_4", '#include <stdio.h>\nint main() {\n    float fahr, celsius;\n    int lower, upper, step;\n    lower = 0; upper = 300; step = 20;\n    celsius = lower;\n    while (celsius <= upper) {\n        fahr = (9.0/5.0) * celsius + 32.0;\n        printf("%3.0f %6.1f\\n", celsius, fahr);\n        celsius = celsius + step;\n    }\n    return 0;\n}\n')

add("kr_1_5", '#include <stdio.h>\nint main() {\n    int fahr;\n    for (fahr = 300; fahr >= 0; fahr = fahr - 20)\n        printf("%3d %6.1f\\n", fahr, (5.0/9.0)*(fahr-32));\n    return 0;\n}\n')

add("kr_1_6", '#include <stdio.h>\nint main() {\n    int c;\n    c = getchar() != EOF;\n    printf("%d\\n", c);\n    return 0;\n}\n', 'A\n')

add("kr_1_7", '#include <stdio.h>\nint main() {\n    printf("%d\\n", EOF);\n    return 0;\n}\n')

add("kr_1_8", '#include <stdio.h>\nint main() {\n    int c, nl, nt, nb;\n    nl = nt = nb = 0;\n    while ((c = getchar()) != EOF) {\n        if (c == \' \') ++nb;\n        if (c == \'\\t\') ++nt;\n        if (c == \'\\n\') ++nl;\n    }\n    printf("%d %d %d\\n", nb, nt, nl);\n    return 0;\n}\n', 'a b\tc\nd e\n')

add("kr_1_9", '#include <stdio.h>\nint main() {\n    int c, lastc;\n    lastc = 0;\n    while ((c = getchar()) != EOF) {\n        if (c != \' \' || lastc != \' \') {\n            putchar(c);\n        }\n        lastc = c;\n    }\n    return 0;\n}\n', 'a  b   c\n')

add("kr_1_10", '#include <stdio.h>\nint main() {\n    int c;\n    while ((c = getchar()) != EOF) {\n        switch (c) {\n        case \'\\t\': printf("\\\\t"); break;\n        case \'\\b\': printf("\\\\b"); break;\n        case \'\\\\\': printf("\\\\\\\\"); break;\n        default: putchar(c);\n        }\n    }\n    return 0;\n}\n', 'a\tb\\c\n')

add("kr_1_11", '#include <stdio.h>\n#define IN 1\n#define OUT 0\nint main() {\n    int c, nl, nw, nc, state;\n    state = OUT;\n    nl = nw = nc = 0;\n    while ((c = getchar()) != EOF) {\n        ++nc;\n        if (c == \'\\n\')\n            ++nl;\n        if (c == \' \' || c == \'\\n\' || c == \'\\t\')\n            state = OUT;\n        else if (state == OUT) {\n            state = IN;\n            ++nw;\n        }\n    }\n    printf("%d %d %d\\n", nl, nw, nc);\n    return 0;\n}\n', 'hello world\n')

add("kr_1_12", '#include <stdio.h>\n#define IN 1\n#define OUT 0\nint main() {\n    int c, state;\n    state = OUT;\n    while ((c = getchar()) != EOF) {\n        if (c == \' \' || c == \'\\n\' || c == \'\\t\') {\n            if (state == IN) {\n                putchar(\'\\n\');\n                state = OUT;\n            }\n        } else {\n            state = IN;\n            putchar(c);\n        }\n    }\n    return 0;\n}\n', 'hello world\n')

add("kr_1_13", '#include <stdio.h>\n#define IN 1\n#define OUT 0\n#define MAXLEN 10\nint main() {\n    int c, state, len;\n    int lengths[MAXLEN];\n    for (int i = 0; i < MAXLEN; i++) lengths[i] = 0;\n    state = OUT;\n    len = 0;\n    while ((c = getchar()) != EOF) {\n        if (c == \' \' || c == \'\\n\' || c == \'\\t\') {\n            if (state == IN) {\n                if (len < MAXLEN) lengths[len]++;\n                len = 0;\n            }\n            state = OUT;\n        } else {\n            state = IN;\n            ++len;\n        }\n    }\n    for (int i = 1; i < MAXLEN; i++) {\n        printf("%2d: ", i);\n        for (int j = 0; j < lengths[i]; j++) putchar(\'*\');\n        putchar(\'\\n\');\n    }\n    return 0;\n}\n', 'hello world a bb\n')

add("kr_1_14", '#include <stdio.h>\nint main() {\n    int c;\n    int freq[26];\n    for (int i = 0; i < 26; i++) freq[i] = 0;\n    while ((c = getchar()) != EOF) {\n        if (c >= \'a\' && c <= \'z\') freq[c - \'a\']++;\n    }\n    for (int i = 0; i < 26; i++) {\n        putchar(\'a\' + i);\n        putchar(\':\');\n        for (int j = 0; j < freq[i]; j++) putchar(\'*\');\n        putchar(\'\\n\');\n    }\n    return 0;\n}\n', 'aabbc\n')

add("kr_1_15", '#include <stdio.h>\nfloat fahr_to_celsius(float fahr) {\n    return (5.0/9.0) * (fahr - 32.0);\n}\nint main() {\n    for (float fahr = 0; fahr <= 300; fahr += 20)\n        printf("%3.0f %6.1f\\n", fahr, fahr_to_celsius(fahr));\n    return 0;\n}\n')

add("kr_1_16", '#include <stdio.h>\n#define MAXLINE 100\nint getline(char s[], int lim) {\n    int c, i;\n    for (i = 0; i < lim - 1 && (c = getchar()) != EOF && c != \'\\n\'; ++i)\n        s[i] = c;\n    if (c == \'\\n\') {\n        s[i] = c;\n        ++i;\n    }\n    s[i] = \'\\0\';\n    return i;\n}\nvoid copy(char to[], char from[]) {\n    int i = 0;\n    while ((to[i] = from[i]) != \'\\0\') ++i;\n}\nint main() {\n    int len, max;\n    char line[MAXLINE], longest[MAXLINE];\n    max = 0;\n    while ((len = getline(line, MAXLINE)) > 0)\n        if (len > max) {\n            max = len;\n            copy(longest, line);\n        }\n    if (max > 0)\n        printf("%s", longest);\n    return 0;\n}\n', 'short\nthis is the longest line\nok\n')

add("kr_1_17", '#include <stdio.h>\n#define MAXLINE 100\nint getline(char s[], int lim) {\n    int c, i;\n    for (i = 0; i < lim - 1 && (c = getchar()) != EOF && c != \'\\n\'; ++i)\n        s[i] = c;\n    if (c == \'\\n\') {\n        s[i] = c;\n        ++i;\n    }\n    s[i] = \'\\0\';\n    return i;\n}\nint main() {\n    int len;\n    char line[MAXLINE];\n    while ((len = getline(line, MAXLINE)) > 0)\n        if (len > 10)\n            printf("%s", line);\n    return 0;\n}\n', 'short\nthis is long enough\nok\n')

add("kr_1_18", '#include <stdio.h>\n#define MAXLINE 100\nint getline(char s[], int lim) {\n    int c, i;\n    for (i = 0; i < lim - 1 && (c = getchar()) != EOF && c != \'\\n\'; ++i)\n        s[i] = c;\n    if (c == \'\\n\') {\n        s[i] = c;\n        ++i;\n    }\n    s[i] = \'\\0\';\n    return i;\n}\nint main() {\n    int len;\n    char line[MAXLINE];\n    while ((len = getline(line, MAXLINE)) > 0) {\n        int end = len - 1;\n        while (end >= 0 && (line[end] == \' \' || line[end] == \'\\t\' || line[end] == \'\\n\'))\n            --end;\n        if (end >= 0) {\n            line[end + 1] = \'\\n\';\n            line[end + 2] = \'\\0\';\n            printf("%s", line);\n        }\n    }\n    return 0;\n}\n', 'hello   \nworld\t\n\na\n')

add("kr_1_19", '#include <stdio.h>\n#define MAXLINE 100\nint getline(char s[], int lim) {\n    int c, i;\n    for (i = 0; i < lim - 1 && (c = getchar()) != EOF && c != \'\\n\'; ++i)\n        s[i] = c;\n    if (c == \'\\n\') {\n        s[i] = c;\n        ++i;\n    }\n    s[i] = \'\\0\';\n    return i;\n}\nvoid reverse(char s[]) {\n    int i, j;\n    char temp;\n    for (i = 0; s[i] != \'\\0\'; ++i);\n    --i;\n    if (s[i] == \'\\n\') --i;\n    for (j = 0; j < i; j++, i--) {\n        temp = s[j];\n        s[j] = s[i];\n        s[i] = temp;\n    }\n}\nint main() {\n    char line[MAXLINE];\n    while (getline(line, MAXLINE) > 0) {\n        reverse(line);\n        printf("%s", line);\n    }\n    return 0;\n}\n', 'hello\nworld\n')

add("kr_2_3", '#include <stdio.h>\nint htoi(char s[]) {\n    int i, n;\n    n = 0;\n    for (i = 0; s[i] != \'\\0\'; i++) {\n        int c = s[i];\n        if (c >= \'0\' && c <= \'9\')\n            n = 16 * n + (c - \'0\');\n        else if (c >= \'a\' && c <= \'f\')\n            n = 16 * n + (c - \'a\' + 10);\n        else if (c >= \'A\' && c <= \'F\')\n            n = 16 * n + (c - \'A\' + 10);\n    }\n    return n;\n}\nint main() {\n    printf("%d\\n", htoi("0x1A"));\n    printf("%d\\n", htoi("FF"));\n    printf("%d\\n", htoi("0"));\n    return 0;\n}\n')

add("kr_2_4", '#include <stdio.h>\nvoid squeeze(char s[], char s2[]) {\n    int i, j, k;\n    for (k = 0; s2[k] != \'\\0\'; k++) {\n        for (i = j = 0; s[i] != \'\\0\'; i++)\n            if (s[i] != s2[k])\n                s[j++] = s[i];\n        s[j] = \'\\0\';\n    }\n}\nint main() {\n    char s[] = "hello world";\n    squeeze(s, "lo");\n    printf("%s\\n", s);\n    return 0;\n}\n')

add("kr_2_5", '#include <stdio.h>\nint any(char s1[], char s2[]) {\n    int i, j;\n    for (i = 0; s1[i] != \'\\0\'; i++)\n        for (j = 0; s2[j] != \'\\0\'; j++)\n            if (s1[i] == s2[j])\n                return i;\n    return -1;\n}\nint main() {\n    printf("%d\\n", any("hello", "aeiou"));\n    printf("%d\\n", any("xyz", "abc"));\n    return 0;\n}\n')

add("kr_2_6", '#include <stdio.h>\nunsigned setbits(unsigned x, int p, int n, unsigned y) {\n    return (x & ~(~(~0 << n) << (p - n + 1))) |\n           ((y & ~(~0 << n)) << (p - n + 1));\n}\nint main() {\n    printf("%u\\n", setbits(170, 4, 3, 5));\n    return 0;\n}\n')

add("kr_2_7", '#include <stdio.h>\nunsigned invert(unsigned x, int p, int n) {\n    return x ^ (~(~0 << n) << (p - n + 1));\n}\nint main() {\n    printf("%u\\n", invert(170, 4, 3));\n    return 0;\n}\n')

add("kr_2_8", '#include <stdio.h>\nunsigned rightrot(unsigned x, int n) {\n    int wordlength(void);\n    int rbit;\n    while (n-- > 0) {\n        rbit = (x & 1) << (wordlength() - 1);\n        x = x >> 1;\n        x = x | rbit;\n    }\n    return x;\n}\nint wordlength(void) {\n    int i;\n    unsigned v = (unsigned)~0;\n    for (i = 1; (v = v >> 1) > 0; i++);\n    return i;\n}\nint main() {\n    printf("%u\\n", rightrot(170, 2));\n    return 0;\n}\n')

add("kr_2_9", '#include <stdio.h>\nint bitcount(unsigned x) {\n    int b;\n    for (b = 0; x != 0; x &= x - 1)\n        ++b;\n    return b;\n}\nint main() {\n    printf("%d\\n", bitcount(170));\n    printf("%d\\n", bitcount(0xFFFFFFFF));\n    return 0;\n}\n')

add("kr_2_10", '#include <stdio.h>\nint lower(int c) {\n    return (c >= \'A\' && c <= \'Z\') ? c + \'a\' - \'A\' : c;\n}\nint main() {\n    printf("%c\\n", lower(\'A\'));\n    printf("%c\\n", lower(\'z\'));\n    return 0;\n}\n')

for name, code, input_data in cases:
    code = ensure_eof_define(code)
    c_path = BASE / f"{name}.c"
    c_path.write_text(code, encoding="utf-8")
    if input_data is not None:
        in_path = BASE / f"{name}.in"
        in_path.write_text(input_data, encoding="utf-8")
    else:
        in_path = BASE / f"{name}.in"
        if in_path.exists():
            in_path.unlink()

    with tempfile.TemporaryDirectory() as tmpdir:
        cfile = Path(tmpdir) / "test.c"
        cfile.write_text(code, encoding="utf-8")
        exe = Path(tmpdir) / "test.exe"
        subprocess.run([CLANG, str(cfile), "-o", str(exe)], check=True, capture_output=True)
        if input_data is not None:
            result = subprocess.run([str(exe)], input=input_data, capture_output=True, text=True)
        else:
            result = subprocess.run([str(exe)], capture_output=True, text=True)
        out_path = GOLDEN / f"{name}.out"
        out_path.write_text(result.stdout, encoding="utf-8")
        print(f"Generated {name}: golden lines={len(result.stdout.splitlines())}")

print(f"Done. Total cases: {len(cases)}")
