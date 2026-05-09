namespace Cide.Client.ViewModels;

public record CodeTemplate(string Key, string DisplayName, string Category, string Code)
    : Cide.Client.Shared.ViewModels.CodeTemplate(Key, DisplayName, Category, Code)
{
    public CodeTemplate(Cide.Client.Shared.ViewModels.CodeTemplate t)
        : this(t.Key, t.DisplayName, t.Category, t.Code) { }
}
