using Avalonia;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Markup.Xaml;
using Cide.Client.Core;
using Cide.Client.Shared.Core;
using Cide.Client.ViewModels;
using Cide.Client.Views;

namespace Cide.Client;

public partial class App : Application
{
    public override void Initialize()
    {
        AvaloniaXamlLoader.Load(this);
    }

    public override void OnFrameworkInitializationCompleted()
    {
        Console.WriteLine("[CIDE_APP] OnFrameworkInitializationCompleted START");

        // Initialize shared knowledge card loader with platform-specific provider
        KnowledgeCardLoader.Initialize(new KnowledgeCardResourceProvider());

        AppDomain.CurrentDomain.UnhandledException += (s, e) =>
        {
            var msg = e.ExceptionObject?.ToString() ?? "Unknown exception";
            Console.WriteLine($"[CIDE_UNHANDLED] {msg}");
        };

        TaskScheduler.UnobservedTaskException += (s, e) =>
        {
            Console.WriteLine($"[CIDE_TASK_EX] {e.Exception}");
            e.SetObserved();
        };

        try
        {
            if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
            {
                Console.WriteLine("[CIDE_APP] Desktop lifetime detected");
                desktop.MainWindow = new MainWindow
                {
                    DataContext = new MainViewModel()
                };
                Console.WriteLine("[CIDE_APP] MainWindow created");
            }
            else if (ApplicationLifetime is ISingleViewApplicationLifetime singleViewPlatform)
            {
                Console.WriteLine("[CIDE_APP] SingleView lifetime detected (Android)");

                singleViewPlatform.MainView = new MainView
                {
                    DataContext = new MainViewModel()
                };
                Console.WriteLine("[CIDE_APP] MainView assigned");
            }
            else
            {
                Console.WriteLine($"[CIDE_APP] Unknown lifetime: {ApplicationLifetime?.GetType().Name}");
            }
        }
        catch (Exception ex)
        {
            Console.WriteLine($"[CIDE_APP] EXCEPTION during view creation: {ex}");
            throw;
        }

        Console.WriteLine("[CIDE_APP] Calling base.OnFrameworkInitializationCompleted");
        base.OnFrameworkInitializationCompleted();
        Console.WriteLine("[CIDE_APP] OnFrameworkInitializationCompleted END");
    }
}
