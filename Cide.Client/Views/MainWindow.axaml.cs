using Avalonia.Controls;
using Cide.Client.ViewModels;

namespace Cide.Client.Views;

public partial class MainWindow : Window
{
    public MainWindow()
    {
        InitializeComponent();

        this.SizeChanged += OnWindowSizeChanged;
        this.Closing += OnWindowClosing;

    }

    private void OnWindowSizeChanged(object? sender, SizeChangedEventArgs e)
    {
        if (DataContext is MainViewModel vm)
        {
            vm.UpdateLayout(e.NewSize.Width, e.NewSize.Height);
        }
    }

    private void OnWindowClosing(object? sender, WindowClosingEventArgs e)
    {
        this.SizeChanged -= OnWindowSizeChanged;
        this.Closing -= OnWindowClosing;
        if (DataContext is IDisposable disposable)
        {
            disposable.Dispose();
        }
    }
}
