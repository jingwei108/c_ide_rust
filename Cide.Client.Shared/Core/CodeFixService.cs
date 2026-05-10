using System.Text;

namespace Cide.Client.Shared.Core;

/// <summary>
/// Result of an attempted code fix application.
/// </summary>
public readonly record struct CodeFixResult(bool Applied, string? NewSourceCode, string Message);

/// <summary>
/// Applies automatic fixes based on compiler diagnostic suggestions.
/// First tries structured fix data (FixKind/ReplaceRange/ReplacementText),
/// then falls back to Chinese string substring matching for legacy diagnostics.
/// </summary>
public static class CodeFixService
{
    public static CodeFixResult TryApplyFix(string sourceCode, Diagnostic diagnostic)
    {
        if (diagnostic.Line <= 0)
            return new CodeFixResult(false, null, string.Empty);

        // 1. Try structured fix first (when backend provides it)
        if (diagnostic.FixKind == FixKind.ReplaceText && diagnostic.ReplaceStartLine > 0)
        {
            var structuredResult = ApplyStructuredReplace(sourceCode, diagnostic);
            if (structuredResult.Applied)
                return structuredResult;
        }
        else if (diagnostic.FixKind == FixKind.ManualHint)
        {
            return new CodeFixResult(false, null,
                $"💡 修复提示（第{diagnostic.Line}行）：{diagnostic.FixSuggestion}\n请手动修改代码。");
        }

        // 2. Fallback to legacy string-based matching
        return ApplyLegacyFix(sourceCode, diagnostic);
    }

    private static CodeFixResult ApplyStructuredReplace(string sourceCode, Diagnostic diagnostic)
    {
        var lines = sourceCode.Replace("\r\n", "\n").Split('\n');
        int startLine = diagnostic.ReplaceStartLine - 1;
        int endLine = diagnostic.ReplaceEndLine - 1;

        if (startLine < 0 || startLine >= lines.Length || endLine < 0 || endLine >= lines.Length)
            return new CodeFixResult(false, null, string.Empty);

        if (startLine == endLine)
        {
            string line = lines[startLine];
            int startCol = diagnostic.ReplaceStartColumn;
            int endCol = diagnostic.ReplaceEndColumn;
            // startCol == endCol means insert; startCol > endCol is invalid
            if (startCol < 0 || endCol > line.Length || startCol > endCol)
                return new CodeFixResult(false, null, string.Empty);

            string before = line.Substring(0, startCol);
            string after = line.Substring(endCol);
            lines[startLine] = before + diagnostic.ReplacementText + after;

            string newSource = string.Join("\n", lines);
            return new CodeFixResult(true, newSource,
                $"✅ 已应用修复（第{diagnostic.ReplaceStartLine}行）");
        }

        // Multi-line replace not yet supported
        return new CodeFixResult(false, null, string.Empty);
    }

    private static CodeFixResult ApplyLegacyFix(string sourceCode, Diagnostic diagnostic)
    {
        if (string.IsNullOrEmpty(diagnostic.FixSuggestion))
            return new CodeFixResult(false, null, string.Empty);

        var lines = sourceCode.Replace("\r\n", "\n").Split('\n');
        int lineIndex = diagnostic.Line - 1;
        if (lineIndex < 0 || lineIndex >= lines.Length)
            return new CodeFixResult(false, null, string.Empty);

        bool applied = false;
        string fix = diagnostic.FixSuggestion;

        if (fix.Contains("分号") || fix.Contains("';'"))
        {
            string trimmed = lines[lineIndex].TrimEnd();
            if (!trimmed.EndsWith(";") && !trimmed.EndsWith("{") && !trimmed.EndsWith("}"))
            {
                lines[lineIndex] = trimmed + ";";
                applied = true;
            }
        }
        else if (fix.Contains("声明变量"))
        {
            return new CodeFixResult(false, null,
                $"💡 修复提示（第{diagnostic.Line}行）：{fix}\n请手动修改代码。");
        }
        else if (fix.Contains("检查函数名"))
        {
            return new CodeFixResult(false, null,
                $"💡 修复提示（第{diagnostic.Line}行）：{fix}\n请手动修改代码。");
        }
        else if (fix.Contains("类型一致"))
        {
            return new CodeFixResult(false, null,
                $"💡 修复提示（第{diagnostic.Line}行）：{fix}\n请手动修改代码。");
        }
        else if (fix.Contains("=' 改为 '=='"))
        {
            string line = lines[lineIndex];
            int parenStart = line.IndexOf('(');
            int parenEnd = line.LastIndexOf(')');
            if (parenStart >= 0 && parenEnd > parenStart)
            {
                string before = line.Substring(0, parenStart + 1);
                string cond = line.Substring(parenStart + 1, parenEnd - parenStart - 1);
                string after = line.Substring(parenEnd);
                var sb = new StringBuilder();
                for (int i = 0; i < cond.Length; i++)
                {
                    if (cond[i] == '=')
                    {
                        bool precededByOp = (i > 0 && (cond[i - 1] == '=' || cond[i - 1] == '!' || cond[i - 1] == '<' || cond[i - 1] == '>'));
                        bool followedByEq = (i + 1 < cond.Length && cond[i + 1] == '=');
                        if (!precededByOp && !followedByEq)
                        {
                            sb.Append("==");
                            sb.Append(cond.Substring(i + 1));
                            applied = true;
                            break;
                        }
                    }
                    sb.Append(cond[i]);
                }
                if (applied)
                {
                    lines[lineIndex] = before + sb.ToString() + after;
                }
            }
        }
        else if (fix.Contains("'<=' 改为 '<'"))
        {
            string trimmed = lines[lineIndex];
            int idx = trimmed.IndexOf("<=");
            if (idx >= 0)
            {
                lines[lineIndex] = trimmed.Substring(0, idx) + "<" + trimmed.Substring(idx + 2);
                applied = true;
            }
        }

        if (applied)
        {
            string newSource = string.Join("\n", lines);
            return new CodeFixResult(true, newSource,
                $"✅ 已应用修复（第{diagnostic.Line}行）：{fix}");
        }

        return new CodeFixResult(false, null,
            $"💡 修复提示（第{diagnostic.Line}行）：{fix}\n该修复需要手动操作。");
    }
}
