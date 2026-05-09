using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Avalonia.Platform;
using Cide.Client.Shared.Core;

namespace Cide.Client.Core;

public class KnowledgeCardResourceProvider : IKnowledgeCardResourceProvider
{
    public IEnumerable<Stream> EnumerateCardStreams()
    {
        var assembly = typeof(KnowledgeCardResourceProvider).Assembly;
        var baseUri = new Uri($"avares://{assembly.GetName().Name}/Assets/KnowledgeCards");

        var assets = AssetLoader.GetAssets(baseUri, baseUri);
        foreach (var uri in assets)
        {
            if (!uri.ToString().EndsWith(".json", StringComparison.OrdinalIgnoreCase))
                continue;

            yield return AssetLoader.Open(uri);
        }
    }
}
