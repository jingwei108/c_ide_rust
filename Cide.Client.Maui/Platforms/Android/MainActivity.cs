using Android.App;
using Android.Content.PM;
using Android.OS;
using Android.Views;
using AndroidX.Core.View;

namespace Cide.Client.Maui;

[Activity(
    Theme = "@style/Maui.SplashTheme",
    MainLauncher = true,
    WindowSoftInputMode = SoftInput.AdjustResize,
    ConfigurationChanges = ConfigChanges.ScreenSize | ConfigChanges.Orientation | ConfigChanges.UiMode | ConfigChanges.ScreenLayout | ConfigChanges.SmallestScreenSize | ConfigChanges.Density)]
public class MainActivity : MauiAppCompatActivity
{
    protected override void OnCreate(Bundle? savedInstanceState)
    {
        base.OnCreate(savedInstanceState);

        // Ensure the WebView can show the soft keyboard when CodeMirror editor is focused.
        // On Android 11+ (API 30), use WindowInsetsController to handle system bars gracefully.
        // On older devices, AdjustResize alone is sufficient.
        if (OperatingSystem.IsAndroidVersionAtLeast(30) && Window != null)
        {
            WindowCompat.SetDecorFitsSystemWindows(Window!, false);
        }
    }
}
