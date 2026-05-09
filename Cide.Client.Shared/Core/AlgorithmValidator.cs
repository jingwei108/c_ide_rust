using System.Text;
using System.Text.RegularExpressions;

namespace Cide.Client.Shared.Core;

/// <summary>
/// A test case for algorithm validation.
/// </summary>
public readonly record struct AlgorithmTestCase(
    string Description,
    int[] InputArray,
    int? SearchTarget = null);

/// <summary>
/// Result of validating a student algorithm against test cases.
/// </summary>
public readonly record struct AlgorithmValidationResult(
    bool Passed,
    string Message,
    AlgorithmTestCase? FailedCase = null,
    string? ActualOutput = null,
    string? ExpectedOutput = null);

/// <summary>
/// Validates student algorithms using property-based testing.
/// Generates test harnesses, runs them through the compiler, and verifies output properties.
/// </summary>
public static class AlgorithmValidator
{
    /// <summary>
    /// Validates the detected algorithm in the student source code.
    /// </summary>
    public static AlgorithmValidationResult Validate(string sourceCode, AlgorithmMatch match)
    {
        if (string.IsNullOrEmpty(match.FuncName))
            return new AlgorithmValidationResult(false, "无法获取函数名，无法验证算法。");

        var testCases = GenerateTestCases(match.Name);
        if (testCases.Count == 0)
            return new AlgorithmValidationResult(false, $"暂不支持验证算法: {match.DisplayName}");

        foreach (var tc in testCases)
        {
            var result = RunSingleTest(sourceCode, match.FuncName, match.Name, tc);
            if (!result.Passed)
                return result;
        }

        return new AlgorithmValidationResult(true,
            $"✅ {match.DisplayName} 通过了 {testCases.Count} 组测试用例！");
    }

    private static List<AlgorithmTestCase> GenerateTestCases(string algorithmName)
    {
        return algorithmName switch
        {
            // Sorting algorithms
            "bubble_sort" or "selection_sort" or "insertion_sort"
                or "quick_sort" or "merge_sort" => new List<AlgorithmTestCase>
            {
                new("随机数组", new[] { 5, 3, 8, 1, 2 }),
                new("已有序", new[] { 1, 2, 3, 4, 5 }),
                new("逆序", new[] { 5, 4, 3, 2, 1 }),
                new("单元素", new[] { 42 }),
                new("全部相同", new[] { 2, 2, 2, 2 }),
                new("空数组", Array.Empty<int>()),
                new("包含负数", new[] { -3, 5, -1, 0, 2 }),
            },
            // Search algorithms
            "binary_search" => new List<AlgorithmTestCase>
            {
                new("找到目标", new[] { 1, 3, 5, 7, 9 }, 5),
                new("找到首个", new[] { 1, 3, 5, 7, 9 }, 1),
                new("找到末尾", new[] { 1, 3, 5, 7, 9 }, 9),
                new("未找到（偏小）", new[] { 1, 3, 5, 7, 9 }, 0),
                new("未找到（偏大）", new[] { 1, 3, 5, 7, 9 }, 10),
                new("单元素找到", new[] { 5 }, 5),
                new("单元素未找到", new[] { 5 }, 3),
                new("空数组", Array.Empty<int>(), 1),
            },
            _ => new List<AlgorithmTestCase>()
        };
    }

    private static AlgorithmValidationResult RunSingleTest(
        string sourceCode, string funcName, string algorithmName, AlgorithmTestCase tc)
    {
        string harness = BuildHarness(sourceCode, funcName, algorithmName, tc);
        if (string.IsNullOrEmpty(harness))
            return new AlgorithmValidationResult(false, "生成测试代码失败。");

        using var compiler = new CompilerService();
        if (!compiler.Compile(harness))
        {
            string? errors = compiler.GetCompileErrors();
            return new AlgorithmValidationResult(false,
                $"测试用例「{tc.Description}」编译失败: {errors}");
        }

        bool ok = compiler.Run();
        string output = compiler.GetOutput();
        string? runtimeError = ok ? null : compiler.GetRuntimeError();

        if (!ok)
        {
            return new AlgorithmValidationResult(false,
                $"测试用例「{tc.Description}」运行时错误: {runtimeError}",
                tc, output);
        }

        return VerifyOutput(algorithmName, tc, output.Trim());
    }

    private static string BuildHarness(string sourceCode, string funcName, string algorithmName, AlgorithmTestCase tc)
    {
        // Replace student's main() so we can inject our own
        string modifiedSource = Regex.Replace(sourceCode, @"(?<!\w)int\s+main\s*\(", "int __cide_original_main(");

        var sb = new StringBuilder();
        sb.AppendLine(modifiedSource);
        sb.AppendLine();
        sb.AppendLine("int main() {");

        if (tc.InputArray.Length == 0)
        {
            // Empty array: use a dummy pointer and size 0
            sb.AppendLine("    int* arr = 0;");
            sb.AppendLine("    int n = 0;");
        }
        else
        {
            sb.Append("    int arr[] = {");
            for (int i = 0; i < tc.InputArray.Length; i++)
            {
                if (i > 0) sb.Append(", ");
                sb.Append(tc.InputArray[i]);
            }
            sb.AppendLine("};");
            sb.AppendLine($"    int n = {tc.InputArray.Length};");
        }

        if (algorithmName is "bubble_sort" or "selection_sort" or "insertion_sort"
            or "quick_sort" or "merge_sort")
        {
            sb.AppendLine($"    {funcName}(arr, n);");
            sb.AppendLine("    for (int i = 0; i < n; i = i + 1) {");
            sb.AppendLine("        printf(\"%d \", arr[i]);");
            sb.AppendLine("    }");
        }
        else if (algorithmName == "binary_search")
        {
            if (tc.SearchTarget.HasValue)
            {
                sb.AppendLine($"    int result = {funcName}(arr, n, {tc.SearchTarget.Value});");
                sb.AppendLine("    printf(\"%d\", result);");
            }
            else
            {
                return "";
            }
        }
        else
        {
            return "";
        }

        sb.AppendLine("    return 0;");
        sb.AppendLine("}");

        return sb.ToString();
    }

    private static AlgorithmValidationResult VerifyOutput(string algorithmName, AlgorithmTestCase tc, string output)
    {
        if (algorithmName is "bubble_sort" or "selection_sort" or "insertion_sort"
            or "quick_sort" or "merge_sort")
        {
            return VerifySorted(tc, output);
        }

        if (algorithmName == "binary_search")
        {
            return VerifyBinarySearch(tc, output);
        }

        return new AlgorithmValidationResult(false, $"未知算法类型: {algorithmName}", tc, output);
    }

    private static AlgorithmValidationResult VerifySorted(AlgorithmTestCase tc, string output)
    {
        var parts = output.Split(' ', StringSplitOptions.RemoveEmptyEntries);
        var actual = new List<int>();
        foreach (var p in parts)
        {
            if (int.TryParse(p, out int v))
                actual.Add(v);
        }

        // Property 1: length must match
        if (actual.Count != tc.InputArray.Length)
        {
            return new AlgorithmValidationResult(false,
                $"输出长度不匹配。期望 {tc.InputArray.Length} 个元素，实际得到 {actual.Count} 个。",
                tc, output, string.Join(" ", tc.InputArray.OrderBy(x => x)));
        }

        // Property 2: must be non-decreasing
        for (int i = 1; i < actual.Count; i++)
        {
            if (actual[i] < actual[i - 1])
            {
                return new AlgorithmValidationResult(false,
                    $"排序结果不是非递减的。arr[{i - 1}] = {actual[i - 1]}，arr[{i}] = {actual[i]}。",
                    tc, output);
            }
        }

        // Property 3: must be a permutation (element conservation)
        var expectedSorted = tc.InputArray.OrderBy(x => x).ToList();
        for (int i = 0; i < actual.Count; i++)
        {
            if (actual[i] != expectedSorted[i])
            {
                return new AlgorithmValidationResult(false,
                    $"元素守恒被破坏。排序后 arr[{i}] = {actual[i]}，但期望 {expectedSorted[i]}。",
                    tc, output);
            }
        }

        return new AlgorithmValidationResult(true, "");
    }

    private static AlgorithmValidationResult VerifyBinarySearch(AlgorithmTestCase tc, string output)
    {
        if (!int.TryParse(output, out int actualIndex))
        {
            return new AlgorithmValidationResult(false,
                $"输出无法解析为整数: '{output}'", tc, output);
        }

        var sorted = tc.InputArray.OrderBy(x => x).ToArray();

        // Find expected index
        int expectedIndex = -1;
        for (int i = 0; i < sorted.Length; i++)
        {
            if (sorted[i] == tc.SearchTarget)
            {
                expectedIndex = i;
                break;
            }
        }

        if (actualIndex != expectedIndex)
        {
            if (expectedIndex == -1)
            {
                return new AlgorithmValidationResult(false,
                    $"目标 {tc.SearchTarget} 不在数组中，应返回 -1，但返回了 {actualIndex}。",
                    tc, output);
            }
            else
            {
                return new AlgorithmValidationResult(false,
                    $"目标 {tc.SearchTarget} 应在索引 {expectedIndex} 处，但返回了 {actualIndex}。",
                    tc, output);
            }
        }

        return new AlgorithmValidationResult(true, "");
    }
}
