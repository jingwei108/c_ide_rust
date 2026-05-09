using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using Cide.Client.Shared.Core;

namespace Cide.Client.Maui.Core;

public class KnowledgeCardResourceProvider : IKnowledgeCardResourceProvider
{
    public IEnumerable<Stream> EnumerateCardStreams()
    {
        var assembly = typeof(KnowledgeCardResourceProvider).Assembly;
        var resourceNames = assembly.GetManifestResourceNames()
            .Where(r => r.EndsWith(".json", System.StringComparison.OrdinalIgnoreCase));

        foreach (var name in resourceNames)
        {
            var stream = assembly.GetManifestResourceStream(name);
            if (stream != null)
                yield return stream;
        }
    }
}
