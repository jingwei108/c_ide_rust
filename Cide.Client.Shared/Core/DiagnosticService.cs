using Cide.Client.Shared.ViewModels;

namespace Cide.Client.Shared.Core;

/// <summary>
/// Result of loading diagnostics from the compiler.
/// </summary>
public readonly record struct DiagnosticLoadResult(
    List<Diagnostic> Diagnostics,
    List<int> ErrorLines,
    List<int> WarningLines,
    KnowledgeCardViewModel? FirstCard);

/// <summary>
/// Loads compiler diagnostics, enriches them with code snippets, and matches knowledge cards.
/// </summary>
public static class DiagnosticService
{
    public static DiagnosticLoadResult LoadDiagnostics(CompilerService compiler, string sourceCode)
    {
        var diagnostics = new List<Diagnostic>();
        var errorLines = new HashSet<int>();
        var warningLines = new HashSet<int>();
        KnowledgeCardViewModel? firstCard = null;

        var sourceLines = sourceCode.Replace("\r\n", "\n").Split('\n');
        int count = compiler.GetDiagnosticCount();

        for (int i = 0; i < count; i++)
        {
            var diag = compiler.GetDiagnostic(i);
            string snippet = "";
            if (diag.Line > 0 && diag.Line <= sourceLines.Length)
            {
                snippet = sourceLines[diag.Line - 1].TrimEnd();
            }

            diagnostics.Add(diag with { CodeSnippet = snippet });

            if (diag.Severity == 0 && diag.Line > 0)
            {
                errorLines.Add(diag.Line);
            }
            else if (diag.Severity == 1 && diag.Line > 0)
            {
                warningLines.Add(diag.Line);
            }

            if (firstCard == null)
            {
                firstCard = KnowledgeCardViewModel.FromErrorCode(diag.ErrorCode, diag.Message, snippet);
            }
        }

        return new DiagnosticLoadResult(diagnostics, errorLines.ToList(), warningLines.ToList(), firstCard);
    }

    /// <summary>
    /// Summarizes hints and warnings for display in the console output.
    /// </summary>
    public static string GetConversionHintsSummary(List<Diagnostic> diagnostics)
    {
        int hintCount = diagnostics.Count(d => d.Severity == 2);
        int warnCount = diagnostics.Count(d => d.Severity == 1);
        if (hintCount == 0 && warnCount == 0) return string.Empty;

        var sb = new System.Text.StringBuilder();
        sb.AppendLine("--- 编译提示 ---");
        if (warnCount > 0)
        {
            sb.AppendLine($"发现 {warnCount} 个警告，请注意可能的类型安全问题。");
        }
        if (hintCount > 0)
        {
            sb.AppendLine($"发现 {hintCount} 处隐式类型转换（已自动处理）。如需更明确，可添加显式强制转换。");
        }
        sb.AppendLine();
        return sb.ToString();
    }
}
