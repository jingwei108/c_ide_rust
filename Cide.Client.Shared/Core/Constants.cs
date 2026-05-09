namespace Cide.Client.Shared.Core;

/// <summary>
/// Shared constants for the C IDE frontend, aligned with the native VM memory layout.
/// </summary>
public static class Constants
{
    /// <summary>End of NULL trap zone and start of global data segment (0x1000 = 4KB).</summary>
    public const uint NullTrapEnd = 0x1000;

    /// <summary>Total linear memory size (0x40000 = 256KB).</summary>
    public const uint LinearMemorySize = 0x40000;

    /// <summary>Size of a 32-bit integer in bytes.</summary>
    public const int IntSize = 4;

    /// <summary>Default offset for the 'next' pointer inside a linked-list node struct.</summary>
    public const int DefaultStructNextOffset = 4;

    /// <summary>Default offset for the 'data' field inside a linked-list node struct.</summary>
    public const int DefaultStructDataOffset = 0;
}
