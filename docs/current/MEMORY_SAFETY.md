# Cide 内存安全规范

> **目标：** 通过工程规范 + 静态分析 + 代码审查 + 自动化测试，系统性防止内存泄漏、资源泄漏、UAF（Use-After-Free）和悬空指针问题。
> **适用范围：** `CideFlutter/`（Flutter/Dart 前端）、`native/src/`（Rust 后端）

---

## 一、核心原则（不可违背）

### 原则 1：谁分配，谁释放（RAII）
- **Rust**：所有权系统自动管理内存；`Box<T>` / `Vec<T>` / `String` 在作用域结束时自动释放；禁止裸指针算术
- **Dart/Flutter**：Native 资源通过 `flutter_rust_bridge` 自动管理；`finalizer` 或 `dispose()` 模式释放大型资源

### 原则 2：事件订阅必须对称释放（Flutter）
- `Riverpod` provider 在 `dispose()` 中取消订阅和清理资源
- `AnimationController`、`ScrollController`、`TextEditingController` 必须 `dispose()`
- 避免在 `StatefulWidget` 中创建长期存活的无名监听器而不移除

### 原则 3：跨语言边界必须拷贝，不得传引用
- **Rust C API**：返回字符串禁止返回内部 `String` 的裸指针；必须拷贝到调用者提供的缓冲区，或明确文档化生命周期（"仅在下次编译前有效"）
- **Flutter 获取 Native 字符串**：通过 `flutter_rust_bridge` 自动处理 UTF-8 拷贝，禁止在 Dart 侧长期持有裸指针

### 原则 4：异步任务必须可取消
- **Dart**：`Future` 和 `Stream` 使用 `cancel()` 或 `dispose()` 终止
- **Rust**：`tokio::select!` 或自定义 `CancellationToken` 模式（当前 VM 为同步单步，无异步风险）

### 原则 5：并发释放必须线程安全
- **Rust**：利用所有权和 `Mutex` / `RwLock` 保证并发安全；`Drop` 实现避免阻塞操作
- **Dart/Flutter**：UI 操作必须在主线程；`dispose()` 中禁止执行异步阻塞操作

---

## 二、Flutter/Dart 前端规范

### 2.1 StatefulWidget 生命周期契约

每个自定义 `StatefulWidget` 必须正确管理控制器和监听器：

```dart
class MyWidget extends StatefulWidget {
  @override
  _MyWidgetState createState() => _MyWidgetState();
}

class _MyWidgetState extends State<MyWidget> {
  late final AnimationController _controller;
  final _scrollController = ScrollController();

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      duration: const Duration(milliseconds: 500),
      vsync: this,
    );
    _scrollController.addListener(_onScroll);
  }

  @override
  void dispose() {
    // 1. 按"反向订阅顺序"释放
    _scrollController.removeListener(_onScroll);
    _scrollController.dispose();
    _controller.dispose();

    // 2. 释放 Riverpod provider 监听器（如有）
    // ref.read(myProvider.notifier).dispose();

    super.dispose();
  }

  void _onScroll() { /* ... */ }
}
```

### 2.2 应用生命周期契约（Desktop 端）

**关键认知：** 应用退出时不会自动触发所有 Widget 的 `dispose()`。

```dart
// main.dart
void main() {
  WidgetsFlutterBinding.ensureInitialized();
  
  // 监听应用生命周期
  SystemChannels.lifecycle.setMessageHandler((msg) async {
    if (msg == AppLifecycleState.detached.toString()) {
      // 全局清理：释放 VM Session、保存学习进度
      await GlobalCleanup.disposeAll();
    }
    return msg;
  });

  runApp(const ProviderScope(child: MyApp()));
}
```

### 2.3 VM Session 释放模板

```dart
class VMSessionNotifier extends StateNotifier<VMState> {
  VMSession? _session;
  bool _disposed = false;

  void dispose() {
    if (_disposed) return;
    _disposed = true;
    
    // 通过 FRB 调用 Rust 释放 Session
    rust.apiDispose(sessionId: _session?.id ?? 0);
    _session = null;
    
    super.dispose();
  }

  void ensureNotDisposed() {
    if (_disposed) throw StateError('VMSessionNotifier already disposed');
  }
}
```

### 2.4 异步任务规范

```dart
class RunCodeNotifier extends StateNotifier<RunState> {
  CancelableOperation? _currentOperation;

  Future<void> runAsync() async {
    // 取消上一个任务
    await _currentOperation?.cancel();
    
    _currentOperation = CancelableOperation.fromFuture(
      _doRun(),
      onCancel: () {
        rust.apiStop(sessionId: state.sessionId);
      },
    );
    
    await _currentOperation!.value;
  }

  @override
  void dispose() {
    _currentOperation?.cancel();
    super.dispose();
  }
}
```

### 2.5 Flutter 资源缓存

在 `CustomPainter` 中禁止在 `paint()` 中创建 `Paint`、`Path`、`TextSpan`：

```dart
// ❌ 错误
class BadPainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()..color = Colors.red;  // 每帧创建
    canvas.drawCircle(Offset.zero, 10, paint);
  }
}

// ✅ 正确
class GoodPainter extends CustomPainter {
  final _paint = Paint()..color = Colors.red;
  
  @override
  void paint(Canvas canvas, Size size) {
    canvas.drawCircle(Offset.zero, 10, _paint);
  }
  
  @override
  bool shouldRepaint(covariant CustomPainter old) => false;
}
```

---

## 三、Rust 后端规范

### 3.1 C API 设计规范

| 规则 | 示例 |
|------|------|
| 返回指针必须声明生命周期 | `/// 仅在下次 compile 前有效` |
| 所有输出字符串参数必须提供缓冲区+长度 | `char* buf, int buf_size` |
| 输入数组必须提供长度+上限校验 | `if (count > MAX_COUNT) return -1;` |
| 版本号+校验和（序列化） | `kSessionMagic = "CIDESV01"` |

```rust
// ❌ 错误：返回悬空指针
#[no_mangle]
pub extern "C" fn cide_get_errors(s: *mut Session) -> *const c_char {
    let session = unsafe { &*s };
    session.errors.as_ptr()  // 重新分配后指针失效
}

// ✅ 正确：拷贝到调用者缓冲区
#[no_mangle]
pub extern "C" fn cide_get_errors_buf(
    s: *mut Session, buf: *mut c_char, max_len: i32
) -> i32 {
    if s.is_null() || buf.is_null() || max_len <= 0 { return -1; }
    let session = unsafe { &*s };
    let c_str = CString::new(session.errors.clone()).unwrap_or_default();
    let bytes = c_str.as_bytes_with_nul();
    let copy_len = std::cmp::min(bytes.len(), max_len as usize - 1);
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf as *mut u8, copy_len);
        *buf.add(copy_len) = 0;
    }
    copy_len as i32
}
```

### 3.2 VM 内存安全

```rust
// 所有 Host Function 内存访问必须通过封装接口
impl CideVM {
    fn load_i32(&self, addr: u32) -> Result<i32, Trap> {
        let end = addr.checked_add(4)
            .filter(|&end| end <= self.memory.len() as u32 && addr >= k_null_trap_size)
            .ok_or_else(|| Trap::BoundsError(addr))?;
        Ok(read_i32_le(&self.memory[addr as usize..end as usize]))
    }
}

// 禁止裸指针算术
// ❌ memory[p1] = val;  // p1 为负时缓冲区下溢
// ✅ self.store_i32(addr, val)?;
```

### 3.3 reset() 必须清理全部运行时状态

```rust
impl CideVM {
    fn reset(&mut self) {
        self.code.clear();
        self.stack.clear();
        self.call_stack.clear();
        self.func_table.clear();        // 不要忘记！
        self.func_names.clear();        // 不要忘记！
        self.host_funcs.clear();
        self.symbols.clear();
        self.vis_event_lines.clear();   // 不要忘记！
        self.vis_event_queue.clear();   // 不要忘记！
        self.breakpoints.clear();       // 不要忘记！
        self.snapshot_vars.clear();
        self.memory.fill(0);
        self.step_count = 0;
        self.heap_offset = k_heap_start;
        // ... 其他状态归零
    }
}
```

---

## 四、代码审查 Checklist

### 4.1 PR 审查时必须逐项确认

**Flutter/Dart 前端：**
- [ ] **Controller 释放：** 所有 `AnimationController`、`ScrollController`、`TextEditingController` 是否在 `dispose()` 中释放
- [ ] **Riverpod Provider：** 自定义 `StateNotifier` 是否重写了 `dispose()` 并清理资源
- [ ] **Listener 移除：** 所有 `addListener` 都有对应的 `removeListener`
- [ ] **异步任务取消：** `CancelableOperation` / `Timer` 是否在 dispose 时取消

**Rust 后端：**
- [ ] **所有权检查：** 是否存在裸指针解引用未包裹 `unsafe`？`unsafe` 块是否最小化？
- [ ] **C API 生命周期：** 返回的裸指针是否有明确的生命周期文档注释
- [ ] **Reset/clear：** 新添加的容器字段是否在 `reset()`/`clear()` 中处理
- [ ] **溢出检查：** `addr + size`、`new_offset` 等是否使用 `checked_add`
- [ ] **跨语言边界：** C API 是否返回内部指针/引用？

### 4.2 高风险变更额外审查

以下变更类型必须**双人审查**：
1. 新增 C API 函数
2. 修改 Flutter `dispose()` / 全局清理逻辑
3. 新增 Riverpod Provider 或事件订阅
4. 修改 `CideVM::reset()` 或内存管理逻辑
5. 新增 `unsafe` 块或裸指针操作

---

## 五、静态分析与工具链

### 5.1 Dart/Flutter 静态分析

```yaml
# analysis_options.yaml
analyzer:
  language:
    strict-casts: true
    strict-raw-types: true
  errors:
    missing_required_param: error
    missing_return: error
    invalid_assignment: warning
```

关键规则：
| 规则 | 说明 | 级别 |
|------|------|------|
| `missing_required_param` | 缺少必需参数 | Error |
| `missing_return` | 函数缺少 return | Error |
| `unused_import` | 未使用的 import | Warning |
| `avoid_print` | 避免使用 print（使用日志框架）| Warning |

### 5.2 Rust 分析器（Clippy）

```bash
# 运行 clippy（已在 CI 中配置）
cd native && cargo clippy --all-targets --all-features -- -D warnings
```

关键规则：
| 规则 | 说明 |
|------|------|
| `clippy::unwrap_used` | 禁止在库代码中使用 `.unwrap()` |
| `clippy::expect_used` | 限制 `.expect()` 的使用 |
| `clippy::missing_safety_doc` | `unsafe` 函数必须有安全文档 |
| `clippy::cast_lossless` | 避免有损类型转换 |

### 5.3 预提交检查脚本

```powershell
# scripts/check-memory-safety.ps1
# 在 git commit 前自动运行
```

运行方式：
```powershell
# 手动运行
.\scripts\check-memory-safety.ps1

# 或配置为 git hook
copy scripts\check-memory-safety.ps1 .git\hooks\pre-commit.ps1
```

---

## 六、测试策略

### 6.1 单元测试：Dispose 路径覆盖

```csharp
[Test]
public void CompilerService_Dispose_IsIdempotent()
{
    var svc = new CompilerService();
    svc.Dispose();
    svc.Dispose();  // 不应抛异常
    Assert.IsTrue(svc.IsDisposed);
}

[Test]
public void MainViewModel_Dispose_CancelsPendingFlash()
{
    var vm = new MainViewModel();
    vm.RunCodeAsync();  // 启动某些异步任务
    vm.Dispose();       // 应取消所有任务
    // 验证无异常
}
```

### 6.2 Stress Test：重复编译-运行-Dispose

```cpp
TEST(MemoryStress, CompileRunDispose_1000Times) {
    for (int i = 0; i < 1000; ++i) {
        auto* s = cide_session_create();
        cide_compile(s, "int main() { return 0; }");
        cide_run(s);
        cide_session_destroy(s);
    }
    // 进程 RSS 不应持续增长
}
```

### 6.3 监控指标

在 `Debug` 配置下输出以下指标到日志：

```csharp
// MainViewModel.Dispose()
Debug.WriteLine($"[CIDE_MEM] MainViewModel disposed. Compiler disposed: {_compiler?.IsDisposed}");

// CompilerService.Dispose()
Debug.WriteLine($"[CIDE_MEM] CompilerService disposed. Session: {_session}");

// App 退出时
Debug.WriteLine($"[CIDE_MEM] Process exit. PeakWorkingSet: {Process.GetCurrentProcess().PeakWorkingSet64 / 1024}KB");
```

---

## 七、相关文档索引

| 文档 | 说明 |
|------|------|
| `current/DESIGN.md` | 总体架构设计 |
| `current/BUILD.md` | 构建指南 |
| `archive/INCIDENT_2026_04_26_MEMORY_LEAK.md` | 历史泄漏事件记录与修复详情 |

---

*本文档由 Kimi Code CLI 维护。每次发现新的内存安全问题时，必须同步更新本文档的"代码审查 Checklist"和"相关文档索引"。*
