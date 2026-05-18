# 递归类型系统重构方案

## 背景与动机

当前 `Type` 枚举采用 `base_kind: TypeKind + name: String` 的扁平设计来避免递归分配。这在项目早期（仅支持基础类型、简单指针和数组）运行良好，但随着函数指针、多维数组、结构体指针等特性加入，这套设计已到达扩展极限。

教育 IDE 的类型系统必须**语义清晰、无歧义**。学生在学习 `int *arr[3]` 和 `int (*fp[2])(int)` 时，编译器给出的类型诊断必须和教科书一致，而不能用 `Array(base_kind=Pointer, name="int")` 这种丢失精确信息的 hack。

---

## 当前系统的根本缺陷

```rust
// 当前设计：Pointer / Array 只能存 TypeKind + name
Pointer { base_kind: TypeKind, name: String, ... }
Array  { base_kind: TypeKind, name: String, ... }
```

| 问题场景 | 现状 | 后果 |
|---------|------|------|
| `int *arr[3]` | `Array(base_kind=Pointer, name="int")` | 无法区分"指针数组"和"指向数组的指针" |
| `int (*fp)(int)` | 独立变体 `FunctionPointer { ... }` | 与 `Pointer` 无关联，`sizeof` 和 `&` 需特殊处理 |
| `int (*fp[2])(int)` | **完全无法表示** | Parser 被迫拒绝或给出错误类型 |
| `int **f()` | `FunctionPointer(return=Pointer(...))` | 语义混乱，学生看到诊断信息会困惑 |

---

## 新设计：完全递归类型系统

核心原则：**`Pointer` 和 `Array` 存储完整的 `Box<Type>`，而非 `TypeKind + name`**。

```rust
pub enum Type {
    Void { is_const: bool },
    Int { is_unsigned: bool, is_const: bool },
    Char { is_unsigned: bool, is_const: bool },
    Float { is_const: bool },
    Double { is_const: bool },
    LongLong { is_unsigned: bool, is_const: bool },
    Pointer {
        pointee: Box<Type>,      // ← 完整的子类型
        is_const: bool,
    },
    Array {
        element: Box<Type>,      // ← 完整的子类型
        array_size: i32,
        dims: Vec<i32>,
        is_const: bool,
    },
    Struct { name: String, is_const: bool },
    Union { name: String, is_const: bool },
    Function {
        return_type: Box<Type>,
        param_types: Vec<Type>,
        is_const: bool,
    },
}
```

### 关键语义变化

| C 类型 | 旧表示 | 新表示 |
|--------|--------|--------|
| `int *p` | `Pointer(base_kind=Int, name="int")` | `Pointer(Int)` |
| `int *arr[3]` | `Array(base_kind=Pointer, name="int")` ❌ | `Array(Pointer(Int), 3)` |
| `int (*fp)(int)` | `FunctionPointer(return=Int, params=[Int])` | `Pointer(Function(Int, [Int]))` |
| `int (*fp[2])(int)` | **无法表示** ❌ | `Array(Pointer(Function(Int, [Int])), 2)` |
| `int **f()` | `FunctionPointer(return=Pointer(base_kind=Int))` 混乱 | `Function(Pointer(Pointer(Int)))` |
| `sizeof(fp)` | 特殊 case `FunctionPointer = 4` | `Pointer(_)` 自然等于 4 |

### 类型判断方法（TypeChecker）

```rust
impl Type {
    pub fn is_pointer(&self) -> bool {
        matches!(self, Type::Pointer { .. })
    }
    pub fn is_function_pointer(&self) -> bool {
        matches!(self, Type::Pointer { pointee, .. } if matches!(pointee.as_ref(), Type::Function { .. }))
    }
    pub fn is_array(&self) -> bool {
        matches!(self, Type::Array { .. })
    }
    pub fn is_scalar(&self) -> bool {
        matches!(self.kind(), TypeKind::Int | TypeKind::Char | TypeKind::Float | TypeKind::Double | TypeKind::LongLong)
    }
}
```

### 类型大小计算（纯递归）

```rust
fn type_size(ty: &Type) -> i32 {
    match ty {
        Type::Void { .. } => 0,
        Type::Int { .. } | Type::Float { .. } => 4,
        Type::Char { .. } => 1,
        Type::Double { .. } | Type::LongLong { .. } => 8,
        Type::Pointer { .. } => 4,
        Type::Array { element, array_size, .. } => type_size(element) * array_size,
        Type::Struct { name, .. } => struct_size(name),
        Type::Union { name, .. } => union_size(name),
        Type::Function { .. } => 4, // 函数名退化后的函数指针大小
    }
}
```

---

## Parser 的简化

引入 `DeclaratorNode` 树后，`interpret_declarator_node` 可以删除所有特殊分支，成为**纯递归解释器**：

```rust
fn interpret_declarator_node(node: &DeclaratorNode, base_type: &Type) -> Type {
    match node {
        DeclaratorNode::Base => base_type.clone(),
        DeclaratorNode::Pointer(inner) => {
            Type::Pointer {
                pointee: Box::new(interpret_declarator_node(inner, base_type)),
                is_const: false,
            }
        }
        DeclaratorNode::Array(inner, size) => {
            Type::Array {
                element: Box::new(interpret_declarator_node(inner, base_type)),
                array_size: *size,
                dims: vec![*size],
                is_const: false,
            }
        }
        DeclaratorNode::Function(inner, params) => {
            Type::Function {
                return_type: Box::new(interpret_declarator_node(inner, base_type)),
                param_types: params.iter().map(|p| p.ty.clone()).collect(),
                is_const: false,
            }
        }
    }
}
```

**为什么不再需要特殊分支？**

- `*f()` → `Pointer(Function(Base))` → `Function` 先解释得 `Function(int)`，`Pointer` 再包裹得 `Pointer(Function(int))` → 函数返回指针 ✅
- `(*fp)(int)` → `Function(Pointer(Base), [int])` → `Pointer` 先解释得 `Pointer(int)`，`Function` 再包裹得 `Function(Pointer(int), [int])`？不对...

等等，这里有个关键问题。对于 `(*fp)(int)`，我们希望最终类型是 `Pointer(Function(Int, [Int]))`，但树是 `Function(Pointer(Base), [int])`，从内到外解释得到 `Function(Pointer(Int), [int])`。

这意味着：**仅用递归解释还不够，需要额外的归一化步骤**。

实际上，`Function(Pointer(Base), params)` 在 C 语义中等价于 `Pointer(Function(Base, params))`。因为 `(*fp)(params)` 表示 "fp 是指针，指向返回 base_type 的函数"。

因此可以在 `interpret_declarator_node` 之后加一个 **normalize** 步骤：

```rust
fn normalize_type(ty: Type) -> Type {
    match ty {
        // Function(Pointer(X), params) → Pointer(Function(X, params))
        Type::Function { return_type, param_types, is_const }
            if matches!(return_type.as_ref(), Type::Pointer { .. }) =>
        {
            if let Type::Pointer { pointee, .. } = *return_type {
                Type::Pointer {
                    pointee: Box::new(Type::Function {
                        return_type: pointee,
                        param_types,
                        is_const,
                    }),
                    is_const: false,
                }
            } else { unreachable!() }
        }
        // Array(Pointer(X), n) 已经是正确的（指针数组），无需转换
        // Pointer(Array(X, n)) → Array(Pointer(X), n)（需要讨论）
        _ => ty,
    }
}
```

但这又引入了特殊规则。

### 更干净的方案：改变树构建顺序

与其在解释阶段做 normalize，不如在 **Parser 构建树阶段** 就确保树的结构和最终类型一致。

对于 `(*fp)(int)`：
- 括号内 `*fp` → `Pointer(Base)`
- 括号外 `()` 应该让 `Pointer(Base)` 变成 `Pointer(Function(Base))`
- 也就是说，`()` 后缀在应用到 `Pointer` 节点时，应该修改 `Pointer` 的 `pointee`，而不是包裹 `Pointer`

但这要求 `parse_declarator_node` 在构建树时就知道节点的上下文。

**最终结论**：采用 **DeclaratorNode 树 + normalize 归一化** 的组合方案。这是 clang/gcc 等编译器的标准做法：先构建声明符树，再做类型归一化。

---

## Serde 兼容层

### 新序列化格式（嵌套 JSON）

```json
{
  "kind": "Pointer",
  "pointee": {
    "kind": "Function",
    "return_type": { "kind": "Int", "is_unsigned": false, "is_const": false },
    "param_types": [
      { "kind": "Int", "is_unsigned": false, "is_const": false }
    ],
    "is_const": false
  },
  "is_const": false
}
```

### 兼容策略

1. `serialize` 总是输出新格式
2. `deserialize` 优先尝试新格式（递归 `Type`）
3. 失败时回退到旧格式（扁平 `TypeHelper`），将旧格式转换为新格式：
   - `Pointer { base_kind, name, ... }` → 根据 `base_kind` 和 `name` 重建 `pointee`
   - `Array { base_kind, name, ... }` → 根据 `base_kind` 和 `name` 重建 `element`
   - `FunctionPointer { ... }` → `Pointer { pointee: Function { ... } }`

旧 session snapshot 在加载后会丢失部分精确类型信息，但重新编译即可完全恢复。

---

## 影响范围与工作量

| 模块 | 改动内容 | 复杂度 |
|------|---------|--------|
| **AST (`ast.rs`)** | `Type` 枚举重构为完全递归；`TypeKind` 降级为分类标签；serde 重写 | ★★★☆☆ |
| **Parser (`parser.rs`)** | `interpret_declarator_node` 删除所有特殊分支，改为纯递归 + normalize | ★★☆☆☆ |
| **TypeChecker** | `is_pointer`、`is_array`、`type_size`、`check_assignable` 等改为递归；新增 `is_function_pointer` | ★★☆☆☆ |
| **BytecodeGen** | `type_size`、`elem_type_size` 递归化；`CallPtr` 已有，无需改动 | ★★☆☆☆ |
| **VM / Opcode** | 无需改动，`CallPtr` 已经支持动态调用 | ☆☆☆☆☆ |
| **CompilePipeline** | `type_size` 辅助函数递归化 | ★☆☆☆☆ |
| **算法检测器** | 少量 match 分支补全 | ★☆☆☆☆ |
| **测试** | 验证 `sizeof`、数组初始化、指针运算、函数指针数组等行为 | ★★☆☆☆ |

**总工作量**：中等（约 1~2 天）。核心是 `ast.rs` 的 `Type` 重构，一旦编译通过，其余模块的错误由编译器逐个指出，属于机械性修改。

---

## 推进步骤

1. **冻结当前分支** (`refactor/recursive-type-system`)，保留函数指针基础实现作为 fallback
2. **在新的 commit 中重构 `ast.rs`**：
   - 重写 `Type` 为完全递归
   - 重写 `TypeKind` 为纯分类标签
   - 重写 serde（嵌套 JSON + 旧格式兼容）
3. **编译驱动修复**：按编译错误逐个修复 Parser、TypeChecker、BytecodeGen、CompilePipeline
4. **回归测试**：确保全部 140+ E2E 测试通过
5. **补全高级测试**：
   - 函数指针数组 `int (*fp[2])(int) = {f1, f2};`
   - 指向函数指针的指针 `int (**fp)();`
   - `sizeof` 对各类组合类型的验证
6. **文档更新**：更新 `AGENTS.md` 中的类型系统描述

---

## 与补丁方案的对比

| 维度 | 补丁兼容方案 | 递归重构方案 |
|------|-----------|-----------|
| 函数指针数组 | ❌ 不支持 | ✅ `Array(Pointer(Function(...)), n)` |
| 指向函数指针的指针 | ❌ 不支持 | ✅ `Pointer(Pointer(Function(...)))` |
| `int *f()` vs `int (*fp)()` | 😵 依赖特殊分支和顺序判断 | ✅ normalize 后树结构即语义 |
| 代码可维护性 | 😵 越来越复杂，补丁摞补丁 | ✅ 统一规则，无特殊分支 |
| 后续拓展成本 | 高（每加类型都要打补丁） | 低（递归即自然） |
| 当前工作量 | 低（已完成） | 中等（1~2 天） |
| 教育价值 | 低（类型诊断含糊） | 高（类型诊断精确，和教科书一致） |

---

## 结论

当前 `base_kind + name` 的设计本质上是为了避免 `Box<Type>` 的轻微分配开销，却牺牲了类型系统的**语义完整性**。对于教育 IDE，让学生看到 `int (*fp[2])(int)` 被正确识别为 "函数指针数组"，比节省几个 Box 分配重要得多。

这次重构和之前 VM 的 `double` → `long long` 拓展是同一类决策：短期是"不必要的折腾"，长期是**解除架构锁死**。如果不在此刻做，半年后支持 `typedef` 函数指针数组、`const` 函数指针、复杂指针运算时，还会回到这里重新讨论。

**建议：按本方案推进。**
