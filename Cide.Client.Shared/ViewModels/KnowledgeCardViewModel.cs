using CommunityToolkit.Mvvm.ComponentModel;
using Cide.Client.Shared.Core;

namespace Cide.Client.Shared.ViewModels;

/// <summary>
/// A knowledge card that explains the concept behind a common error.
/// L3 diagnostic: principle-level understanding with memory animation description + exercise.
/// </summary>
public partial class KnowledgeCardViewModel : ObservableObject
{
    [ObservableProperty]
    private string _title = "";

    [ObservableProperty]
    private string _emoji = "💡";

    [ObservableProperty]
    private string _codeSnippet = "";

    [ObservableProperty]
    private string _plainExplanation = "";

    [ObservableProperty]
    private string _wrongCode = "";

    [ObservableProperty]
    private string _correctCode = "";

    [ObservableProperty]
    private string _memoryAnimationDescription = "";

    [ObservableProperty]
    private string _exercise = "";

    [ObservableProperty]
    private bool _isVisible = false;

    public static KnowledgeCardViewModel? FromErrorCode(int errorCode, string message, string codeSnippet)
    {
        return KnowledgeCardLoader.FindCard(errorCode, message, codeSnippet);
    }
}
