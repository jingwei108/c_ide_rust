using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using Avalonia.Collections;
using Avalonia.Input;
using Avalonia.Media;
using AvaloniaEdit;
using AvaloniaEdit.TextMate;
using System;
using System.Collections.Generic;
using System.Linq;
using TextMateSharp.Grammars;
using Cide.Client.ViewModels;

namespace Cide.Client.Views;

/// <summary>
/// Represents a single line number item with optional error/warning state.
/// </summary>
public class LineNumberItem
{
    public int LineNumber { get; set; }
    public bool IsError { get; set; }
    public bool IsWarning { get; set; }
    public bool IsBreakpoint { get; set; }
    public string DisplayText => LineNumber.ToString();

    private static bool IsDark => Application.Current?.ActualThemeVariant == Avalonia.Styling.ThemeVariant.Dark;

    public string ForegroundColor => IsError
        ? (IsDark ? "#F48771" : "#D32F2F")
        : IsWarning
            ? (IsDark ? "#CCA700" : "#F57F17")
            : (IsDark ? "#858585" : "#666666");

    public string BackgroundColor => IsError
        ? (IsDark ? "#5C1E1E" : "#FFEBEE")
        : IsWarning
            ? (IsDark ? "#4A3A00" : "#FFF8E1")
            : IsBreakpoint
                ? (IsDark ? "#C75450" : "#FFCDD2")
                : "Transparent";

    public string CircleColor => IsBreakpoint
        ? (IsDark ? "#FF6B6B" : "#E53935")
        : "Transparent";
}

/// <summary>
/// A code editor with line numbers, syntax highlighting and dark theme styling.
/// </summary>
public partial class CodeEditor : UserControl
{
    public static readonly StyledProperty<string> CodeTextProperty =
        AvaloniaProperty.Register<CodeEditor, string>(nameof(CodeText), defaultValue: "");

    public string CodeText
    {
        get => GetValue(CodeTextProperty);
        set => SetValue(CodeTextProperty, value);
    }

    public static readonly StyledProperty<IList<int>> ErrorLinesProperty =
        AvaloniaProperty.Register<CodeEditor, IList<int>>(nameof(ErrorLines), new List<int>());

    public IList<int> ErrorLines
    {
        get => GetValue(ErrorLinesProperty);
        set => SetValue(ErrorLinesProperty, value);
    }

    public static readonly StyledProperty<IList<int>> WarningLinesProperty =
        AvaloniaProperty.Register<CodeEditor, IList<int>>(nameof(WarningLines), new List<int>());

    public IList<int> WarningLines
    {
        get => GetValue(WarningLinesProperty);
        set => SetValue(WarningLinesProperty, value);
    }

    public static readonly StyledProperty<IList<int>> BreakpointLinesProperty =
        AvaloniaProperty.Register<CodeEditor, IList<int>>(nameof(BreakpointLines), new List<int>());

    public IList<int> BreakpointLines
    {
        get => GetValue(BreakpointLinesProperty);
        set => SetValue(BreakpointLinesProperty, value);
    }

    public static readonly StyledProperty<IList<CodeTemplate>> TemplatesProperty =
        AvaloniaProperty.Register<CodeEditor, IList<CodeTemplate>>(nameof(Templates), new List<CodeTemplate>());

    public IList<CodeTemplate> Templates
    {
        get => GetValue(TemplatesProperty);
        set => SetValue(TemplatesProperty, value);
    }

    private AvaloniaList<LineNumberItem> _lineNumbers = new();
    private bool _suppressDocumentEvent;
    private TextMate.Installation? _textMateInstallation;
    private RegistryOptions? _registryOptions;
    private ThemeName _currentThemeName = ThemeName.DarkPlus;

    public CodeEditor()
    {
        Console.WriteLine("[CIDE_EDITOR] Constructor START");
        try
        {
            InitializeComponent();
            Console.WriteLine("[CIDE_EDITOR] InitializeComponent done");
        }
        catch (Exception ex)
        {
            Console.WriteLine($"[CIDE_EDITOR] InitializeComponent FAILED: {ex}");
            throw;
        }

        LineNumbers.ItemsSource = _lineNumbers;
        Console.WriteLine("[CIDE_EDITOR] LineNumbers.ItemsSource set");

        try
        {
            Editor.Document.TextChanged += OnDocumentTextChanged;
            Editor.TextArea.KeyDown += OnEditorKeyDown;
            Editor.TextArea.TextView.ScrollOffsetChanged += OnEditorScrollOffsetChanged;
            LineNumbers.PointerPressed += OnLineNumbersPointerPressed;
            TemplateSuggestionList.SelectionChanged += OnTemplateSuggestionSelected;
            UpdateLineNumbers();
            Console.WriteLine("[CIDE_EDITOR] Constructor END");
        }
        catch (Exception ex)
        {
            Console.WriteLine($"[CIDE_EDITOR] Event hook FAILED: {ex}");
            throw;
        }

        // Focus and caret setup
        Editor.Focusable = true;
        Editor.TextArea.Focusable = true;

        // Caret visibility: bright white for high contrast on dark background
        Editor.TextArea.Caret.CaretBrush = Brushes.White;

        // Ensure virtual keyboard shows on mobile when tapping editor area
        Editor.PointerPressed += OnEditorPointerPressed;
        Editor.GotFocus += OnEditorGotFocus;
        Editor.TextArea.GotFocus += OnTextAreaGotFocus;

        // Listen for app theme changes to switch TextMate theme
        if (Application.Current != null)
        {
            Application.Current.PropertyChanged += OnAppPropertyChanged;
        }
    }

    protected override void OnAttachedToVisualTree(VisualTreeAttachmentEventArgs e)
    {
        base.OnAttachedToVisualTree(e);
        // Delay TextMate setup until TextArea/TextView is fully materialized
        SetupTextMate();
    }

    protected override void OnDetachedFromVisualTree(VisualTreeAttachmentEventArgs e)
    {
        base.OnDetachedFromVisualTree(e);

        Editor.Document.TextChanged -= OnDocumentTextChanged;
        Editor.TextArea.KeyDown -= OnEditorKeyDown;
        Editor.TextArea.TextView.ScrollOffsetChanged -= OnEditorScrollOffsetChanged;
        LineNumbers.PointerPressed -= OnLineNumbersPointerPressed;
        TemplateSuggestionList.SelectionChanged -= OnTemplateSuggestionSelected;
        Editor.PointerPressed -= OnEditorPointerPressed;
        Editor.GotFocus -= OnEditorGotFocus;
        Editor.TextArea.GotFocus -= OnTextAreaGotFocus;

        if (Application.Current != null)
        {
            Application.Current.PropertyChanged -= OnAppPropertyChanged;
        }

        _textMateInstallation?.Dispose();
        _textMateInstallation = null;
        _registryOptions = null;
    }

    private void SetupTextMate()
    {
        // Skip on Android — TextMateSharp relies on file-system grammar loading
        // which doesn't work inside APK assets
        if (OperatingSystem.IsAndroid())
        {
            Console.WriteLine("[CIDE_EDITOR] Skipping TextMate on Android");
            return;
        }

        try
        {
            var themeName = Application.Current?.ActualThemeVariant == Avalonia.Styling.ThemeVariant.Dark
                ? ThemeName.DarkPlus
                : ThemeName.LightPlus;

            // If theme hasn't changed, avoid re-initializing
            if (_registryOptions != null && _currentThemeName == themeName)
            {
                Console.WriteLine($"[CIDE_EDITOR] TextMate theme already set ({themeName}), skipping");
                return;
            }
            _currentThemeName = themeName;

            _registryOptions = new RegistryOptions(themeName);
            _textMateInstallation = Editor.InstallTextMate(_registryOptions);

            var language = _registryOptions.GetLanguageByExtension(".c");
            if (language != null)
            {
                var scopeName = _registryOptions.GetScopeByLanguageId(language.Id);
                if (!string.IsNullOrEmpty(scopeName))
                {
                    _textMateInstallation.SetGrammar(scopeName);
                    Console.WriteLine($"[CIDE_EDITOR] TextMate grammar set: {scopeName}");
                }
                else
                {
                    Console.WriteLine($"[CIDE_EDITOR] WARNING: scopeName is null for language {language.Id}");
                }
            }
            else
            {
                Console.WriteLine("[CIDE_EDITOR] WARNING: No language found for .c extension, trying fallback source.c");
                _textMateInstallation.SetGrammar("source.c");
            }
            Console.WriteLine($"[CIDE_EDITOR] TextMate init done ({themeName})");

            // Force redraw to apply highlighting to existing text
            Editor.TextArea.TextView.Redraw();
        }
        catch (Exception ex)
        {
            Console.WriteLine($"[CIDE_EDITOR] TextMate init failed: {ex}");
        }
    }

    private void OnAppPropertyChanged(object? sender, AvaloniaPropertyChangedEventArgs e)
    {
        if (e.Property.Name == "ActualThemeVariant")
        {
            Console.WriteLine("[CIDE_EDITOR] ActualThemeVariant changed, re-setting up TextMate");
            SetupTextMate();
        }
    }

    private void OnEditorPointerPressed(object? sender, PointerPressedEventArgs e)
    {
        Editor.Focus();
        Editor.TextArea.Focus();
    }

    private void OnEditorGotFocus(object? sender, GotFocusEventArgs e)
    {
        // Focus ensures caret visibility
    }

    private void OnTextAreaGotFocus(object? sender, GotFocusEventArgs e)
    {
        // Focus ensures caret visibility
    }

    protected override void OnPropertyChanged(AvaloniaPropertyChangedEventArgs change)
    {
        base.OnPropertyChanged(change);
        if (change.Property == CodeTextProperty)
        {
            if (!_suppressDocumentEvent && Editor.Document.Text != CodeText)
            {
                _suppressDocumentEvent = true;
                Editor.Document.Text = CodeText;
                _suppressDocumentEvent = false;
            }
            UpdateLineNumbers();
        }
        else if (change.Property == ErrorLinesProperty || change.Property == WarningLinesProperty || change.Property == BreakpointLinesProperty)
        {
            UpdateLineNumbers();
        }
    }

    private void OnDocumentTextChanged(object? sender, EventArgs e)
    {
        if (_suppressDocumentEvent) return;
        _suppressDocumentEvent = true;
        CodeText = Editor.Document.Text;
        _suppressDocumentEvent = false;
        UpdateLineNumbers();
        UpdateTemplateSuggestions();
    }

    private void OnEditorScrollOffsetChanged(object? sender, EventArgs e)
    {
        LineNumberScrollViewer.Offset = new Vector(0, Editor.TextArea.TextView.ScrollOffset.Y);
    }

    private void OnEditorKeyDown(object? sender, KeyEventArgs e)
    {
        if (e.Key == Key.Tab)
        {
            if (TryExpandTemplate())
            {
                e.Handled = true;
            }
        }
        else if (e.Key == Key.Escape)
        {
            if (TemplatePopup != null && TemplatePopup.IsOpen)
            {
                TemplatePopup.IsOpen = false;
                e.Handled = true;
            }
        }
    }

    /// <summary>
    /// Show template suggestion popup when typing on the first line.
    /// Matches prefix against template Key (English) and DisplayName (Chinese).
    /// </summary>
    private void UpdateTemplateSuggestions()
    {
        var templates = Templates;
        if (templates == null || templates.Count == 0 || TemplatePopup == null)
        {
            TemplatePopup?.SetValue(Popup.IsOpenProperty, false);
            return;
        }

        string text = Editor.Document.Text ?? "";
        int caretOffset = Editor.CaretOffset;

        // Only show suggestions on the first line
        int firstLineEnd = text.IndexOf('\n');
        if (firstLineEnd < 0) firstLineEnd = text.Length;
        if (caretOffset > firstLineEnd)
        {
            TemplatePopup.IsOpen = false;
            return;
        }

        // Extract word before caret
        int wordStart = caretOffset;
        while (wordStart > 0 && !char.IsWhiteSpace(text[wordStart - 1]))
        {
            wordStart--;
        }

        if (wordStart >= caretOffset)
        {
            TemplatePopup.IsOpen = false;
            return;
        }

        string word = text.Substring(wordStart, caretOffset - wordStart);
        if (string.IsNullOrEmpty(word))
        {
            TemplatePopup.IsOpen = false;
            return;
        }

        // Prefix match: English key or Chinese display name
        var matches = templates.Where(t =>
            t.Key.StartsWith(word, StringComparison.OrdinalIgnoreCase) ||
            t.DisplayName.StartsWith(word, StringComparison.OrdinalIgnoreCase)
        ).ToList();

        if (matches.Count == 0)
        {
            TemplatePopup.IsOpen = false;
            return;
        }

        TemplateSuggestionList.ItemsSource = matches;

        // Position popup at caret location
        var textView = Editor.TextArea.TextView;
        var pos = textView.GetVisualPosition(
            Editor.TextArea.Caret.Position,
            AvaloniaEdit.Rendering.VisualYPosition.TextBottom);

        TemplatePopup.PlacementTarget = Editor;
        TemplatePopup.Placement = PlacementMode.BottomEdgeAlignedLeft;
        TemplatePopup.HorizontalOffset = Editor.Padding.Left + pos.X;
        TemplatePopup.VerticalOffset = Editor.Padding.Top + pos.Y;
        TemplatePopup.IsOpen = true;
    }

    private void OnTemplateSuggestionSelected(object? sender, SelectionChangedEventArgs e)
    {
        if (TemplateSuggestionList.SelectedItem is not CodeTemplate template) return;

        string text = Editor.Document.Text ?? "";
        int caretOffset = Editor.CaretOffset;

        // Find word start to replace
        int wordStart = caretOffset;
        while (wordStart > 0 && !char.IsWhiteSpace(text[wordStart - 1]))
        {
            wordStart--;
        }

        string before = text.Substring(0, wordStart);
        string after = text.Substring(caretOffset);
        string newText = before + template.Code + after;
        Editor.Document.Text = newText;
        Editor.CaretOffset = wordStart + template.Code.Length;
        CodeText = newText;

        TemplatePopup.IsOpen = false;
        TemplateSuggestionList.SelectedIndex = -1;
    }

    /// <summary>
    /// Try to expand the word before the caret into a template.
    /// Returns true if a template was expanded.
    /// </summary>
    private bool TryExpandTemplate()
    {
        var templates = Templates;
        if (templates == null || templates.Count == 0) return false;

        string text = Editor.Document.Text ?? "";
        int caretOffset = Editor.CaretOffset;

        // Find the start of the current word (before caret)
        int wordStart = caretOffset;
        while (wordStart > 0 && !char.IsWhiteSpace(text[wordStart - 1]))
        {
            wordStart--;
        }

        if (wordStart >= caretOffset) return false;

        string word = text.Substring(wordStart, caretOffset - wordStart);
        var template = templates.FirstOrDefault(t => t.Key == word || t.DisplayName == word);
        if (template == null) return false;

        // Replace the word with the template code
        string before = text.Substring(0, wordStart);
        string after = text.Substring(caretOffset);
        string newText = before + template.Code + after;
        Editor.Document.Text = newText;
        Editor.CaretOffset = wordStart + template.Code.Length;
        CodeText = newText;
        return true;
    }

    /// <summary>
    /// Insert a template by its key. Used by mobile template picker.
    /// </summary>
    public void InsertTemplate(string key)
    {
        var templates = Templates;
        if (templates == null) return;

        var template = templates.FirstOrDefault(t => t.Key == key);
        if (template == null) return;

        string text = Editor.Document.Text ?? "";
        int caretOffset = Editor.CaretOffset;
        string newText = text.Substring(0, caretOffset) + template.Code + text.Substring(caretOffset);
        Editor.Document.Text = newText;
        Editor.CaretOffset = caretOffset + template.Code.Length;
        CodeText = newText;
    }

    private void OnLineNumbersPointerPressed(object? sender, PointerPressedEventArgs e)
    {
        if (e.GetCurrentPoint(LineNumbers).Properties.IsLeftButtonPressed)
        {
            var pos = e.GetPosition(LineNumbers);
            double offsetY = LineNumberScrollViewer.Offset.Y;
            // Estimate line number from vertical position (FontSize 13 + Padding 2+2 = ~17px per line)
            int line = (int)((pos.Y + offsetY) / 17.0) + 1;
            if (line >= 1)
            {
                ToggleBreakpoint(line);
            }
        }
    }

    private void ToggleBreakpoint(int line)
    {
        var bpSet = new HashSet<int>(BreakpointLines ?? new List<int>());
        if (bpSet.Contains(line))
            bpSet.Remove(line);
        else
            bpSet.Add(line);
        BreakpointLines = bpSet.ToList();
    }

    private void UpdateLineNumbers()
    {
        var lines = Editor.Document.LineCount;
        if (lines < 1) lines = 1;

        var errorSet = new HashSet<int>(ErrorLines ?? new List<int>());
        var warningSet = new HashSet<int>(WarningLines ?? new List<int>());
        var bpSet = new HashSet<int>(BreakpointLines ?? new List<int>());

        _lineNumbers.Clear();
        for (int i = 1; i <= lines; i++)
        {
            _lineNumbers.Add(new LineNumberItem
            {
                LineNumber = i,
                IsError = errorSet.Contains(i),
                IsWarning = warningSet.Contains(i),
                IsBreakpoint = bpSet.Contains(i)
            });
        }
    }
}
