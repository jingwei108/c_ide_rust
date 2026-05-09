using Microsoft.Extensions.Logging;
using Cide.Client.Maui.Core;
using Cide.Client.Maui.ViewModels;
using Cide.Client.Shared.Core;

namespace Cide.Client.Maui;

public static class MauiProgram
{
    public static MauiApp CreateMauiApp()
    {
        // Initialize shared knowledge card loader with platform-specific provider
        KnowledgeCardLoader.Initialize(new KnowledgeCardResourceProvider());

        var builder = MauiApp.CreateBuilder();
        builder
            .UseMauiApp<App>()
            .ConfigureFonts(fonts =>
            {
                fonts.AddFont("OpenSans-Regular.ttf", "OpenSansRegular");
            });

        builder.Services.AddMauiBlazorWebView();
        builder.Services.AddSingleton<MainViewModel>();

#if DEBUG
        builder.Services.AddBlazorWebViewDeveloperTools();
        builder.Logging.AddDebug();
#endif

        return builder.Build();
    }
}
