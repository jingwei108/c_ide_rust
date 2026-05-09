using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using Cide.Client.Shared.ViewModels;

namespace Cide.Client.Shared.Core;

/// <summary>
/// Loads knowledge cards from JSON resource files.
/// Must be initialized with a platform-specific <see cref="IKnowledgeCardResourceProvider"/>.
/// </summary>
public static class KnowledgeCardLoader
{
    private static readonly List<KnowledgeCardData> _cards = new();
    private static volatile bool _loaded;
    private static readonly object _loadLock = new();
    private static IKnowledgeCardResourceProvider? _provider;

    private record KnowledgeCardData(
        int ErrorCode,
        string MessageContains,
        string Emoji,
        string Title,
        string PlainExplanation,
        string WrongCode,
        string CorrectCode,
        string MemoryAnimationDescription,
        string Exercise,
        int Difficulty);

    /// <summary>
    /// Initializes the loader with a platform-specific resource provider.
    /// </summary>
    public static void Initialize(IKnowledgeCardResourceProvider provider)
    {
        _provider = provider;
        _loaded = false;
        _cards.Clear();
    }

    public static void EnsureLoaded()
    {
        if (_loaded) return;
        lock (_loadLock)
        {
            if (_loaded) return;
            if (_provider == null)
                throw new InvalidOperationException("KnowledgeCardLoader has not been initialized with a resource provider.");

        foreach (var stream in _provider.EnumerateCardStreams())
        {
            try
            {
                using var reader = new StreamReader(stream);
                var json = reader.ReadToEnd();
                var doc = JsonDocument.Parse(json);
                var root = doc.RootElement;

                var card = new KnowledgeCardData(
                    ErrorCode: root.GetProperty("errorCode").GetInt32(),
                    MessageContains: root.TryGetProperty("messageContains", out var mc) ? mc.GetString() ?? "" : "",
                    Emoji: root.GetProperty("emoji").GetString() ?? "💡",
                    Title: root.GetProperty("title").GetString() ?? "",
                    PlainExplanation: root.GetProperty("plainExplanation").GetString() ?? "",
                    WrongCode: root.TryGetProperty("wrongCode", out var wc) ? wc.GetString() ?? "" : "",
                    CorrectCode: root.TryGetProperty("correctCode", out var cc) ? cc.GetString() ?? "" : "",
                    MemoryAnimationDescription: root.TryGetProperty("memoryAnimationDescription", out var mad) ? mad.GetString() ?? "" : "",
                    Exercise: root.TryGetProperty("exercise", out var ex) ? ex.GetString() ?? "" : "",
                    Difficulty: root.TryGetProperty("difficulty", out var diff) ? diff.GetInt32() : 1
                );
                _cards.Add(card);
            }
            catch (Exception)
            {
                // Skip malformed JSON files
            }
        }

        _loaded = true;
        }
    }

    public static KnowledgeCardViewModel? FindCard(int errorCode, string message, string codeSnippet)
    {
        EnsureLoaded();

        foreach (var card in _cards)
        {
            if (card.ErrorCode != errorCode)
                continue;

            if (!string.IsNullOrEmpty(card.MessageContains) && !message.Contains(card.MessageContains))
                continue;

            return new KnowledgeCardViewModel
            {
                Title = card.Title,
                Emoji = card.Emoji,
                CodeSnippet = codeSnippet,
                PlainExplanation = card.PlainExplanation,
                WrongCode = card.WrongCode,
                CorrectCode = card.CorrectCode,
                MemoryAnimationDescription = card.MemoryAnimationDescription,
                Exercise = card.Exercise,
                IsVisible = true
            };
        }

        // Fallback generic card
        if (message.Contains("返回值") || message.Contains("编译"))
            return null;

        return new KnowledgeCardViewModel
        {
            Title = "诊断提示",
            Emoji = "💡",
            CodeSnippet = codeSnippet,
            PlainExplanation = message,
            WrongCode = codeSnippet,
            CorrectCode = "请根据上方提示修改代码",
            IsVisible = true
        };
    }
}
