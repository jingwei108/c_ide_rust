using System.Collections.Generic;
using System.IO;

namespace Cide.Client.Shared.Core;

/// <summary>
/// Platform-specific resource provider for loading knowledge card JSON streams.
/// </summary>
public interface IKnowledgeCardResourceProvider
{
    /// <summary>
    /// Enumerates all JSON resource streams containing knowledge card data.
    /// </summary>
    IEnumerable<Stream> EnumerateCardStreams();
}
