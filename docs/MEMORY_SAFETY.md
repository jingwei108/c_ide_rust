# Cide 内存安全规范

> **目标：** 通过工程规范 + 静态分析 + 代码审查 + 自动化测试，系统性防止内存泄漏、资源泄漏、UAF（Use-After-Free）和悬空指针问题。
> **适用范围：** `Cide.Client/`（C# Avalonia 前端）、`native/src/`（C++ 后端）

---

## 一、核心原则（不可违背）

### 原则 1：谁分配，谁释放（RAII）
- C++：所有 `new`/`malloc` 必须有对应的 `delete`/`free`；优先使用 `std::unique_ptr`/`std::vector`
- C#：所有 `IntPtr` / Native 资源必须由包装类实现 `IDisposable`

### 原则 2：事件订阅必须对称释放
- 所有 `+=` 必须有对应的 `-=`
- 匿名 lambda 禁止订阅生命周期长于自身的对象的事件
- 应用级静态事件（`Application.Current`、`TaskScheduler`）仅在 `App` 单例中订阅

### 原则 3：跨语言边界必须拷贝，不得传引用
- C API 返回字符串：禁止返回 `std::string::c_str()`，必须拷贝到调用者提供的缓冲区
- C# 获取 Native 字符串：禁止持有 `IntPtr` 超过单个语句，立即 `Marshal.PtrToStringUTF8` 或拷贝到 `byte[]`

### 原则 4：异步任务必须有 CancellationToken
- 所有 `Task.Delay`、`await` 循环、`Fire-and-Forget` 必须接受 `CancellationToken`
- `Dispose()` / `Stop()` 必须传播 `Cancel()` 到所有活跃 Token

### 原则 5：并发 Dispose 必须线程安全
- `Dispose()` 必须使用 `Interlocked.Exchange` 或 `lock` 保护
- 禁止在 `Dispose()` 中执行阻塞操作或重新进入可 Dispose 对象

---

## 二、C# 前端（Avalonia）规范

### 2.1 UserControl 生命周期契约

每个自定义 `UserControl` 必须实现以下模板：

```csharp
public partial class MyView : UserControl
{
    // 1. 所有事件订阅的字段必须可空，用于释放时判断
    private Button? _someButton;
    private CancellationTokenSource? _cts;

    public MyView()
    {
        InitializeComponent();
        this.Loaded += OnLoaded;
    }

    private void OnLoaded(object? sender, RoutedEventArgs e)
    {
        _someButton = this.FindControl<Button>("SomeButton");
        if (_someButton != null)
            _someButton.Click += OnButtonClick;
    }

    protected override void OnDetachedFromVisualTree(VisualTreeAttachmentEventArgs e)
    {
        base.OnDetachedFromVisualTree(e);

        // 2. 按"反向订阅顺序"释放
        if (_someButton != null)
        {
            _someButton.Click -= OnButtonClick;
            _someButton = null;
        }

        // 3. 取消所有异步任务
        _cts?.Cancel();
        _cts?.Dispose();
        _cts = null;

        // 4. 释放 DataContext（如果是 IDisposable）
        if (DataContext is IDisposable disposable)
            disposable.Dispose();

        // 5. 解除自身事件
        this.Loaded -= OnLoaded;
    }
}
```

### 2.2 Window 生命周期契约（Desktop 端）

**关键认知：** Window 关闭时**不会**触发子控件的 `OnDetachedFromVisualTree`。

```csharp
public partial class MainWindow : Window
{
    public MainWindow()
    {
        InitializeComponent();
        this.Closing += OnWindowClosing;  // 必须！
    }

    private void OnWindowClosing(object? sender, WindowClosingEventArgs e)
    {
        this.Closing -= OnWindowClosing;
        if (DataContext is IDisposable disposable)
            disposable.Dispose();
    }
}
```

### 2.3 IDisposable 实现模板

```csharp
public class NativeWrapper : IDisposable
{
    private IntPtr _handle;
    private int _disposed;  // int 支持 Interlocked

    public void Dispose()
    {
        if (Interlocked.Exchange(ref _disposed, 1) == 0)
        {
            NativeMethods.destroy(_handle);
            _handle = IntPtr.Zero;
            GC.SuppressFinalize(this);
        }
    }

    // 每个公共方法必须检查状态
    private void EnsureNotDisposed()
    {
        if (_disposed != 0)
            throw new ObjectDisposedException(nameof(NativeWrapper));
    }
}
```

### 2.4 异步任务规范

```csharp
public partial class MyViewModel : ObservableObject, IDisposable
{
    private CancellationTokenSource? _cts;

    [RelayCommand]
    private async Task RunAsync()
    {
        _cts?.Cancel();
        _cts = new CancellationTokenSource();
        var ct = _cts.Token;

        try
        {
            while (IsRunning)
            {
                ct.ThrowIfCancellationRequested();
                DoWork();
                await Task.Delay(100, ct);  // 必须传 Token！
            }
        }
        catch (OperationCanceledException)
        {
            // 正常取消，无需处理
        }
    }

    public void Dispose()
    {
        _cts?.Cancel();
        _cts?.Dispose();
    }
}
```

### 2.5 Avalonia 资源缓存

禁止在热路径（每帧调用、每次 UI 更新）中 `new SolidColorBrush`、`new Pen`、`new Geometry`：

```csharp
// ❌ 错误
void DrawEdge() => new SolidColorBrush(Color.Parse("#808080"));

// ✅ 正确
private static readonly SolidColorBrush EdgeBrush = new(Color.Parse("#808080"));
void DrawEdge() => EdgeBrush;
```

---

## 三、C++ 后端规范

### 3.1 C API 设计规范

| 规则 | 示例 |
|------|------|
| 返回指针必须声明生命周期 | `// 仅在下次 compile 前有效` → 应改为 copy-out |
| 所有输出字符串参数必须提供缓冲区+长度 | `char* buf, int buf_size` |
| 输入数组必须提供长度+上限校验 | `if (count > MAX_COUNT) return -1;` |
| 版本号+校验和（序列化） | `kSessionMagic = "CIDESV01"` |

```cpp
// ❌ 错误：返回悬空指针
extern "C" const char* cide_get_errors(CideSession* s) {
    return s->errors.c_str();  // s->errors 重新分配后指针失效
}

// ✅ 正确：拷贝到调用者缓冲区
extern "C" int cide_get_errors_buf(CideSession* s, char* buf, int max_len) {
    if (!s || !buf || max_len <= 0) return -1;
    size_t copied = s->errors.copy(buf, max_len - 1);
    buf[copied] = '\0';
    return static_cast<int>(copied);
}
```

### 3.2 VM 内存安全

```cpp
// 所有 Host Function 内存访问必须通过封装接口
int32_t CideVM::LoadI32(uint32_t addr) {
    if (addr + 4 > memory_.size() || addr < kNullTrapSize)
        Trap("内存越界"); return 0;
    return ReadI32LE(memory_.data() + addr);
}

// 禁止裸指针算术
// ❌ mem[p1] = val;  // p1 为负时缓冲区下溢
// ✅ StoreI32(static_cast<uint32_t>(addr), val);
```

### 3.3 Reset() 必须清理全部运行时状态

```cpp
void CideVM::Reset() {
    code_.clear();
    stack_.clear();
    callStack_.clear();
    funcTable_.clear();       // 不要忘记！
    funcNames_.clear();       // 不要忘记！
    hostFuncs_.clear();
    symbols_.clear();
    visEventLines_.clear();   // 不要忘记！
    visEventQueue_.clear();   // 不要忘记！
    breakpoints_.clear();     // 不要忘记！
    snapshotVars_.clear();
    std::fill(memory_.begin(), memory_.end(), 0);
    // ... 其他状态归零
}
```

---

## 四、代码审查 Checklist

### 4.1 PR 审查时必须逐项确认

- [ ] **C# 事件：** 所有 `+=` 都有对应的 `-=` 在 `OnDetachedFromVisualTree` / `Unloaded` / `Dispose` 中
- [ ] **C# Window：** 如果有 `Window`，是否有 `Closing` 事件释放资源
- [ ] **C# IDisposable：** 类是否持有 `IntPtr`/Native 资源/Brush/Timer？如果是，必须实现 `IDisposable`
- [ ] **C# Dispose 线程安全：** 是否使用 `Interlocked.Exchange` 或 `lock`
- [ ] **C# async：** 所有 `Task.Delay`、`await` 循环是否接受 `CancellationToken`
- [ ] **C++ new/malloc：** 是否有对应的 `delete`/`free`
- [ ] **C++ c_str()：** 返回值是否跨函数调用使用？如果是，改为 copy-out
- [ ] **C++ Reset/clear：** 新添加的容器字段是否在 `Reset()`/`clear()` 中处理
- [ ] **跨语言边界：** C API 是否返回内部指针/引用？

### 4.2 高风险变更额外审查

以下变更类型必须**双人审查**：
1. 新增 C API 函数
2. 修改 `Dispose()` / `OnDetachedFromVisualTree` / `Closing`
3. 新增事件订阅
4. 修改 `CideVM::Reset()` 或内存管理逻辑
5. 新增 `async void` 或 Fire-and-Forget 任务

---

## 五、静态分析与工具链

### 5.1 C# 分析器（已配置）

在 `.csproj` 中启用：

```xml
<PropertyGroup>
  <EnableNETAnalyzers>true</EnableNETAnalyzers>
  <AnalysisLevel>latest-recommended</AnalysisLevel>
  <TreatWarningsAsErrors>false</TreatWarningsAsErrors>
</PropertyGroup>

<ItemGroup>
  <PackageReference Include="Microsoft.CodeAnalysis.NetAnalyzers" Version="9.0.0">
    <PrivateAssets>all</PrivateAssets>
    <IncludeAssets>runtime; build; native; contentfiles; analyzers</IncludeAssets>
  </PackageReference>
</ItemGroup>
```

关键规则：
| 规则 ID | 说明 | 级别 |
|---------|------|------|
| CA1001 | 具有可释放字段的类型应实现 IDisposable | Warning |
| CA1063 | 正确实现 IDisposable | Warning |
| CA1816 | 调用 GC.SuppressFinalize | Info |
| CA2000 | 在范围丢失前释放对象 | Warning |
| CA2213 | 可释放字段应由 Dispose 释放 | Warning |

### 5.2 C++ 分析器（建议配置）

```bash
# Clang-Tidy 配置（.clang-tidy）
checks: >
  cppcoreguidelines-owning-memory,
  cppcoreguidelines-no-malloc,
  cppcoreguidelines-pro-type-const-cast,
  cppcoreguidelines-pro-type-reinterpret-cast,
  clang-analyzer-unix.Malloc,
  clang-analyzer-cplusplus.NewDelete,
  memory-unsafe-fnpointer
```

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
| `CODE_REVIEW_REPORT_20260429.md` | 历史泄漏事件记录与修复详情 |
| `CLANG_MIGRATION.md` | C++ 编译器迁移记录（含 sanitizer 配置） |
| `ARCHIVE_ANDROID_LAUNCH_CRASH_FIX.md` | Android 生命周期相关修复 |

---

*本文档由 Kimi Code CLI 维护。每次发现新的内存安全问题时，必须同步更新本文档的"代码审查 Checklist"和"相关文档索引"。*
