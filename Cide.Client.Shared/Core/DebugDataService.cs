using Cide.Client.Shared.ViewModels;

namespace Cide.Client.Shared.Core;

/// <summary>
/// Result of loading variables from the compiler, including derived visualizations.
/// </summary>
public readonly record struct VariableLoadResult(
    List<VariableSnapshot> Variables,
    List<PointerViewModel> Pointers,
    List<ArrayVisualization> Arrays,
    Dictionary<string, int[]> UpdatedArrayValues,
    List<(string ArrayName, int[] FlashIndices)> FlashRequests,
    List<(string ArrayName, int[] CompareIndices)> CompareRequests);

/// <summary>
/// Encapsulates all debug-data loading logic: variables, call stack, memory regions,
/// algorithm matches, watch expressions, and linked-list graph visualization.
/// </summary>
public class DebugDataService
{
    private readonly CompilerService _compiler;

    public DebugDataService(CompilerService compiler)
    {
        _compiler = compiler;
    }

    public List<CallStackFrame> LoadCallStack()
    {
        var frames = new List<CallStackFrame>();
        int count = _compiler.GetCallStackCount();
        for (int i = 0; i < count; i++)
        {
            var (name, line) = _compiler.GetCallStackFrame(i);
            frames.Add(new CallStackFrame(name, line, i == count - 1));
        }
        return frames;
    }

    public List<MemoryRegion> LoadMemoryRegions()
    {
        var regions = new List<MemoryRegion>();
        int count = _compiler.GetMemoryRegionCount();
        for (int i = 0; i < count; i++)
        {
            regions.Add(_compiler.GetMemoryRegion(i));
        }
        return regions;
    }

    public List<AlgorithmMatch> LoadAlgorithmMatches()
    {
        var matches = new List<AlgorithmMatch>();
        int count = _compiler.GetAlgorithmMatchCount();
        for (int i = 0; i < count; i++)
        {
            matches.Add(_compiler.GetAlgorithmMatch(i));
        }
        return matches;
    }

    public VariableLoadResult LoadVariables(
        Dictionary<string, int[]> lastArrayValues,
        List<VisEventEx>? visEvents = null,
        List<AlgorithmMatch>? algorithmMatches = null)
    {
        var variables = new List<VariableSnapshot>();
        var pointers = new List<PointerViewModel>();
        var arrays = new List<ArrayVisualization>();
        var updatedArrayValues = new Dictionary<string, int[]>();
        var flashRequests = new List<(string, int[])>();
        var compareRequests = new List<(string, int[])>();
        var currentArrayNames = new HashSet<string>();

        int count = _compiler.GetVariableCount();
        for (int i = 0; i < count; i++)
        {
            var v = _compiler.GetVariable(i);
            variables.Add(v);

            // Array visualization
            if (v.IsArray && v.ArraySize > 0)
            {
                currentArrayNames.Add(v.Name);
                int[] values = _compiler.ReadArray(v.Address, v.ArraySize);
                int[] flashIndices = Array.Empty<int>();
                if (lastArrayValues.TryGetValue(v.Name, out var oldValues))
                {
                    flashIndices = VisualizationService.DetectSwapIndices(oldValues, values);
                }
                updatedArrayValues[v.Name] = values.ToArray();

                // Compute compare highlights from vis events
                int[] compareIndices = ComputeCompareIndices(v.Name, visEvents, algorithmMatches, variables);
                if (compareIndices.Length > 0)
                {
                    compareRequests.Add((v.Name, compareIndices));
                }

                var elements = VisualizationService.BuildArrayElements(values, flashIndices, compareIndices);
                arrays.Add(new ArrayVisualization(v.Name, v.Address, elements));
                if (flashIndices.Length > 0)
                {
                    flashRequests.Add((v.Name, flashIndices));
                }
            }

            // Pointer tracking
            if (v.Value > Constants.NullTrapEnd && v.Value < Constants.LinearMemorySize && !v.IsArray)
            {
                uint targetAddr = (uint)v.Value;
                string? targetName = _compiler.FindVariableByAddr(targetAddr);
                if (targetName != null)
                {
                    pointers.Add(new PointerViewModel(v.Name, v.Address, targetAddr, targetName));
                }
            }
        }

        // Prune stale entries
        foreach (var key in lastArrayValues.Keys.Where(k => !currentArrayNames.Contains(k)).ToList())
        {
            updatedArrayValues.Remove(key);
        }
        // Ensure all surviving keys from lastArrayValues that are still current are preserved
        foreach (var key in currentArrayNames)
        {
            if (!updatedArrayValues.ContainsKey(key) && lastArrayValues.ContainsKey(key))
            {
                updatedArrayValues[key] = lastArrayValues[key];
            }
        }

        return new VariableLoadResult(variables, pointers, arrays, updatedArrayValues, flashRequests, compareRequests);
    }

    private int[] ComputeCompareIndices(
        string arrayName,
        List<VisEventEx>? visEvents,
        List<AlgorithmMatch>? algorithmMatches,
        List<VariableSnapshot> variables)
    {
        if (visEvents == null || visEvents.Count == 0 || algorithmMatches == null)
            return Array.Empty<int>();

        var indices = new List<int>();
        foreach (var ev in visEvents)
        {
            if (ev.Type != 1) continue; // Only Compare events

            // Find algorithm match that covers this line
            foreach (var match in algorithmMatches)
            {
                foreach (var ve in match.VisEvents)
                {
                    if (ve.Line == ev.Line && ve.Type == 1 && !string.IsNullOrEmpty(ve.Context))
                    {
                        // Context is index expressions separated by ':'
                        var parts = ve.Context.Split(':');
                        foreach (var part in parts)
                        {
                            string expr = part.Trim();
                            if (string.IsNullOrEmpty(expr)) continue;
                            string result = EvaluateWatchExpression(expr, variables);
                            if (int.TryParse(result, out int idx) && idx >= 0)
                            {
                                indices.Add(idx);
                            }
                        }
                    }
                }
            }
        }
        return indices.Distinct().ToArray();
    }

    public List<GraphNodeViewModel> LoadLinkedListGraph(List<VariableSnapshot> variables,
        List<VisEventEx>? visEvents = null)
    {
        var graphNodes = new List<GraphNodeViewModel>();

        // Collect flash colors from graph vis events (last event wins per address)
        var flashColors = new Dictionary<uint, string>();
        if (visEvents != null)
        {
            foreach (var ev in visEvents)
            {
                uint addr = (uint)ev.Extra0;
                switch (ev.Type)
                {
                    case 4: // NodeCreate
                        flashColors[addr] = "#32D74B"; // green
                        break;
                    case 5: // EdgeConnect
                        // Edge flash handled separately; no node color change
                        break;
                    case 6: // NodeAccess
                        flashColors[addr] = "#0A84FF"; // blue
                        break;
                    case 7: // NodeDelete
                        flashColors[addr] = "#FF453A"; // red
                        break;
                }
            }
        }

        var headVars = variables.Select((v, i) => (v, i))
                                .Where(x => x.v.TypeName.StartsWith("struct ") && x.v.TypeName.EndsWith("*"))
                                .ToList();
        if (headVars.Count == 0) return graphNodes;

        var visited = new HashSet<uint>();
        int x = 20, y = 20;

        foreach (var (headVar, varIndex) in headVars)
        {
            uint currentAddr = (uint)headVar.Value;
            if (currentAddr == 0 || currentAddr < Constants.NullTrapEnd) continue;

            int nextOffset = -1;
            int dataOffset = -1;
            for (int f = 0; ; f++)
            {
                var field = _compiler.GetVariableField(varIndex, f);
                if (field == null) break;
                var (name, offset) = field.Value;
                if (name.Equals("next", StringComparison.OrdinalIgnoreCase))
                    nextOffset = offset;
                else if (name.Equals("data", StringComparison.OrdinalIgnoreCase) ||
                         name.Equals("val", StringComparison.OrdinalIgnoreCase) ||
                         name.Equals("value", StringComparison.OrdinalIgnoreCase))
                    dataOffset = offset;
            }

            if (nextOffset < 0) nextOffset = Constants.DefaultStructNextOffset;
            if (dataOffset < 0) dataOffset = Constants.DefaultStructDataOffset;

            while (currentAddr != 0 && currentAddr >= Constants.NullTrapEnd && !visited.Contains(currentAddr))
            {
                visited.Add(currentAddr);

                int dataValue = 0;
                int nextValue = 0;
                _compiler.TryReadMemory(currentAddr + (uint)dataOffset, out dataValue);
                _compiler.TryReadMemory(currentAddr + (uint)nextOffset, out nextValue);

                var node = new GraphNodeViewModel
                {
                    Address = currentAddr,
                    Label = dataValue.ToString(),
                    X = x,
                    Y = y,
                    NextAddr = nextValue != 0 ? (uint?)nextValue : null,
                    IsHighlighted = false,
                    FlashColor = flashColors.TryGetValue(currentAddr, out var fc) ? fc : string.Empty
                };
                graphNodes.Add(node);

                x += 100;
                if (x > 500) { x = 20; y += 80; }

                currentAddr = (uint)nextValue;
            }
        }

        return graphNodes;
    }

    public string EvaluateWatchExpression(string expr, List<VariableSnapshot> variables)
    {
        expr = expr.Trim();
        if (string.IsNullOrEmpty(expr)) return "空表达式";

        int varCount = variables.Count;

        // 1. Direct variable name
        for (int i = 0; i < varCount; i++)
        {
            var v = variables[i];
            if (v.Name == expr)
            {
                if (v.IsArray)
                    return $"数组 (地址 0x{v.Address:X4})";
                return v.Value.ToString();
            }
        }

        // 2. Simple arithmetic: j+1, j-1, i+1, etc.
        foreach (var v in variables)
        {
            if (v.IsArray) continue;
            string name = v.Name;
            if (expr == name + "+1" || expr == name + " + 1")
                return (v.Value + 1).ToString();
            if (expr == name + "-1" || expr == name + " - 1")
                return (v.Value - 1).ToString();
        }

        // 3. Array index: arr[i]
        if (expr.Contains('[') && expr.Contains(']'))
        {
            int idxStart = expr.IndexOf('[');
            int idxEnd = expr.IndexOf(']');
            if (idxStart > 0 && idxEnd > idxStart)
            {
                string name = expr.Substring(0, idxStart).Trim();
                string idxStr = expr.Substring(idxStart + 1, idxEnd - idxStart - 1).Trim();
                if (int.TryParse(idxStr, out int idx) && idx >= 0)
                {
                    for (int i = 0; i < varCount; i++)
                    {
                        var v = variables[i];
                        if (v.Name == name && v.IsArray)
                        {
                            if (idx >= v.ArraySize)
                                return "错误: 数组越界";
                            uint addr = v.Address + (uint)(idx * Constants.IntSize);
                            int val = _compiler.ReadMemoryValue(addr);
                            return val.ToString();
                        }
                    }
                }
            }
        }

        // 4. Pointer dereference: *p
        if (expr.StartsWith("*"))
        {
            string name = expr.Substring(1).Trim();
            for (int i = 0; i < varCount; i++)
            {
                var v = variables[i];
                if (v.Name == name)
                {
                    uint addr = (uint)v.Value;
                    if (addr < Constants.NullTrapEnd || addr >= Constants.LinearMemorySize)
                        return "错误: 无效地址";
                    int val = _compiler.ReadMemoryValue(addr);
                    return val.ToString();
                }
            }
        }

        // 5. Address-of: &var
        if (expr.StartsWith("&"))
        {
            string name = expr.Substring(1).Trim();
            for (int i = 0; i < varCount; i++)
            {
                var v = variables[i];
                if (v.Name == name)
                {
                    return $"0x{v.Address:X4}";
                }
            }
        }

        // 6. String literal
        if (expr.StartsWith("\"") && expr.EndsWith("\""))
        {
            return "字符串字面量";
        }

        return "未知表达式";
    }
}
