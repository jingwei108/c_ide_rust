using System.Collections.Generic;
using System.Linq;
using Cide.Client.Shared.ViewModels;

namespace Cide.Client.Shared.Core;

/// <summary>
/// Pure logic for building array / visualization data from raw compiler outputs.
/// </summary>
public static class VisualizationService
{
    public static ArrayElementVisual[] BuildArrayElements(int[] values, int[]? flashIndices = null, int[]? compareIndices = null)
    {
        if (values.Length == 0) return System.Array.Empty<ArrayElementVisual>();
        flashIndices ??= System.Array.Empty<int>();
        compareIndices ??= System.Array.Empty<int>();
        var flashSet = new HashSet<int>(flashIndices);
        var compareSet = new HashSet<int>(compareIndices);
        int maxVal = 1;
        foreach (var v in values) maxVal = System.Math.Max(maxVal, System.Math.Abs(v));
        var elements = new ArrayElementVisual[values.Length];
        for (int i = 0; i < values.Length; i++)
        {
            double pct = (System.Math.Abs(values[i]) * 100.0) / maxVal;
            bool isFlashing = flashSet.Contains(i);
            bool isCompare = compareSet.Contains(i);
            bool isInversion = (i < values.Length - 1 && values[i] > values[i + 1]);
            string bg = isFlashing ? "#FFFFFF"
                      : isCompare ? "#FFE066"
                      : isInversion ? "#FF6B6B"
                      : "#4ECDC4";
            string border = isFlashing ? "#FFD700"
                          : isCompare ? "#F59E0B"
                          : isInversion ? "#FF4757"
                          : "#45B7AA";
            elements[i] = new ArrayElementVisual(values[i], pct, bg, border, isFlashing, isCompare);
        }
        return elements;
    }

    public static int[] DetectSwapIndices(int[] oldValues, int[] newValues)
    {
        if (oldValues.Length != newValues.Length) return System.Array.Empty<int>();
        var swapped = new List<int>();
        for (int i = 0; i < oldValues.Length; i++)
        {
            if (oldValues[i] != newValues[i])
            {
                swapped.Add(i);
            }
        }
        // Only report if exactly 2 elements swapped positions
        if (swapped.Count == 2)
        {
            int a = swapped[0], b = swapped[1];
            if (oldValues[a] == newValues[b] && oldValues[b] == newValues[a])
            {
                return swapped.ToArray();
            }
        }
        return System.Array.Empty<int>();
    }
}
