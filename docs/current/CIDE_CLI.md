# Cide CLI 使用手册

`cide_cli` 是 Cide 项目 Rust 后端的命令行调试工具，无需启动 Flutter 前端即可直接编译、运行和单步调试 C 代码。

## 构建

```bash
cd native
cargo build --release --bin cide_cli
```

构建产物位于 `native/target/release/cide_cli`（Linux/macOS）或 `native/target/release/cide_cli.exe`（Windows）。

## 基本用法

```bash
cide_cli <command> <file> [options]
```

## 命令

| 命令 | 说明 |
|------|------|
| `compile <file>` | 编译 C 文件并显示诊断信息（错误/警告/建议） |
| `run <file>` | 编译并全速运行程序 |
| `step <file>` | 交互式单步调试 |
| `unified <file>` | 统一模式（时间旅行引擎）批量执行并输出摘要 |

## 选项

| 选项 | 说明 |
|------|------|
| `-i <file>` | 从指定文件读取标准输入（多行输入，供 `scanf`/`fgets` 等使用） |

## 特殊文件名

使用 `-` 作为文件名时，CLI 从**标准输入**读取源代码，便于快速测试代码片段：

```bash
# 管道方式
echo '#include <stdio.h>
int main() { printf("hello\n"); return 0; }' | cide_cli run -

# here-document 方式
cide_cli compile - <<'EOF'
#include <stdio.h>
int main() {
    int a = 10, b = 20;
    printf("%d\n", a + b);
    return 0;
}
EOF
```

## 使用示例

### 1. 编译并检查诊断

```bash
cide_cli compile hello.c
```

输出示例：
```
编译成功。
检测到算法:
  • 数组遍历 (置信度: 95%)
```

若存在错误：
```
=== 诊断信息 ===
[错误] 4:5  类型不匹配：无法将 'char[6]' 赋值给 'int' (E3004)
    建议: 赋值或传参时，左右两边的类型不一致...

编译失败。
```

### 2. 全速运行

```bash
cide_cli run hello.c
```

输出示例：
```
编译成功。

=== 运行输出 ===
Hello, Cide CLI!

程序运行完成，返回值：0
```

### 3. 带输入运行

```bash
# input.txt 内容：
# 5 7

cide_cli run sum.c -i input.txt
```

### 4. 交互式单步调试

```bash
cide_cli step hello.c
```

进入调试交互后，支持的命令：

| 调试命令 | 说明 |
|----------|------|
| `Enter`（空输入） | 执行下一步 |
| `p` / `print` | 打印当前局部变量 |
| `o` / `output` | 打印当前程序输出 |
| `r` / `run` | 全速运行到结束 |
| `q` / `quit` | 退出调试 |

输出示例：
```
=== 交互式单步调试 ===
命令: [Enter]=下一步, p=打印变量, o=打印输出, q=退出, r=运行到结束

步    0 | 行   0:   >
步    1 | 行   3: int main() {  > p
  a: Int = 10
  b: Int = 20
步    2 | 行   4: int a = 10;  >
```

### 5. 统一模式（时间旅行引擎）

```bash
cide_cli unified hello.c
```

输出示例：
```
=== 统一模式执行（时间旅行引擎）===
  共执行 117 步

=== 执行摘要 ===
总步数: 117
状态: 正常结束

=== 最终输出 ===
sum=15
```

统一模式会完整记录每一步的 VM 状态，支持检查点保存和回溯，与前端"时间旅行"功能使用同一引擎。

## 快速测试片段

无需创建临时文件，直接通过标准输入快速验证代码：

```bash
# 测试 printf
echo '#include <stdio.h>
int main() { printf("ok\n"); return 0; }' | cide_cli run -

# 测试循环
cide_cli unified - <<'EOF'
#include <stdio.h>
int main() {
    int s = 0;
    for (int i = 1; i <= 100; i++) s += i;
    printf("%d\n", s);
    return 0;
}
EOF

# 测试 scanf + 输入
cat <<'EOF' | cide_cli run - -i /dev/stdin
#include <stdio.h>
int main() { int a,b; scanf("%d%d",&a,&b); printf("%d\n",a+b); return 0; }
EOF
# 然后输入两个数字并按 Ctrl+D（Unix）或 Ctrl+Z（Windows）
```

> **注意**：当使用 `-` 从 stdin 读取源代码时，不能再通过 `-i -` 从同一 stdin 读取输入数据，建议将输入数据写入文件后使用 `-i data.txt`。
