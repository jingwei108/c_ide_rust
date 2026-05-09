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
}
