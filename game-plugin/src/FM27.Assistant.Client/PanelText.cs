using System.Text;
namespace FM27.Assistant.Client;

public static class PanelText
{
    public static string Context(Conversation state) => $"{state.Club} · {(state.Source == "demo" ? "KURGUSAL ÖRNEK KADRO" : "MANUEL KADRO")}\nMasaüstündeki kadro kullanılıyor. Canlı oyun verisi henüz okunmuyor.";

    public static string Transcript(Conversation state)
    {
        var text = new StringBuilder();
        foreach (var message in state.Messages.TakeLast(12))
        {
            var author = message.Role == "user" ? $"Sen · {(message.Client == "game" ? "oyun" : "masaüstü")}" : "FM Assistant";
            text.AppendLine(author);
            text.AppendLine(message.State == "pending" ? (state.Live.Length > 0 ? state.Live : "Yanıt hazırlanıyor…") : message.Text);
            if (message.State is "failed" or "interrupted") text.AppendLine("[Yanıt tamamlanmadı]");
            text.AppendLine();
        }
        if (text.Length == 0) text.Append("Aynı asistanla burada da konuşabilirsin. Girişini masaüstü uygulamasında tamamla.");
        // Keep Unity text layout work bounded even after a long conversation.
        if (text.Length > 20000)
        {
            var start = text.Length - 20000;
            if (char.IsLowSurrogate(text[start])) start++;
            return "…\n" + text.ToString(start, text.Length - start);
        }
        return text.ToString();
    }
}
