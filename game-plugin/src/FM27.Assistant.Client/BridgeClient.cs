using System.Net;
using System.Net.Sockets;
using System.Runtime.InteropServices;
using System.Text.Json;

namespace FM27.Assistant.Client;

public sealed record Message(long Id, string Role, string Text, string State, string Client);
public sealed record ModelChoice(string Id, string Name, bool IsDefault);
public sealed record Conversation(string Scope, string Club, string Source, string CapturedAt,
    string? Model, Message[] Messages, bool Busy, string Live, long? ActiveTurnId,
    ModelChoice[] Models, string? Error);
public sealed record Descriptor(int Protocol, string Address, int Port, string Token);
internal sealed record Reply(int Protocol, bool Ok, JsonElement Result, string? Error);

/// <summary>Small transport-only client: no analysis, model credentials or local chat history.</summary>
public sealed class BridgeClient
{
    public const int Protocol = 1;
    public const int MaxRequestBytes = 16 * 1024;
    public const int MaxResponseBytes = 3 * 1024 * 1024;
    private static readonly JsonSerializerOptions Json = new() { PropertyNamingPolicy = JsonNamingPolicy.CamelCase, PropertyNameCaseInsensitive = true };
    private readonly string descriptorPath;

    public BridgeClient(string? descriptorPath = null) => this.descriptorPath = descriptorPath ?? DefaultDescriptorPath();

    public static string DefaultDescriptorPath()
    {
        var root = RuntimeInformation.IsOSPlatform(OSPlatform.OSX)
            ? Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), "Library", "Application Support")
            : Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData);
        return Path.Combine(root, "studio.kutra.fm27assistant", "in-game", "bridge.json");
    }

    public Task<Conversation> StateAsync(CancellationToken token = default) => RequestAsync<Conversation>("state", null, token);
    public Task<Conversation> SendAsync(string scope, string prompt, CancellationToken token = default)
    {
        if (string.IsNullOrWhiteSpace(prompt) || System.Text.Encoding.UTF8.GetByteCount(prompt) > 8000)
            throw new ArgumentException("Mesaj boş olamaz ve en fazla 8.000 bayt olabilir.");
        return RequestAsync<Conversation>("send", new { scope, prompt }, token);
    }
    public Task<Conversation> SetModelAsync(string scope, string? model, CancellationToken token = default) => RequestAsync<Conversation>("setModel", new { scope, model }, token);
    public async Task CancelAsync(long turnId, CancellationToken token = default) => await RequestAsync<JsonElement>("cancel", new { turnId }, token).ConfigureAwait(false);

    private async Task<T> RequestAsync<T>(string method, object? parameters, CancellationToken cancellation)
    {
        using var timeout = CancellationTokenSource.CreateLinkedTokenSource(cancellation);
        timeout.CancelAfter(TimeSpan.FromSeconds(5));
        var token = timeout.Token;
        Descriptor descriptor;
        try
        {
            using var file = new FileStream(descriptorPath, FileMode.Open, FileAccess.Read, FileShare.ReadWrite | FileShare.Delete, 4096, true);
            var bytes = await ReadBoundedAsync(file, 4096, false, token).ConfigureAwait(false);
            descriptor = JsonSerializer.Deserialize<Descriptor>(bytes, Json) ?? throw new IOException("Bağlantı bilgisi okunamadı.");
        }
        catch (FileNotFoundException) { throw new IOException("FM Assistant masaüstü uygulamasını aç ve Veri & bağlantı bölümünden oyun içi bağlantıyı etkinleştir."); }
        catch (DirectoryNotFoundException) { throw new IOException("FM Assistant masaüstü uygulamasını aç ve oyun içi bağlantıyı etkinleştir."); }
        if (descriptor.Protocol != Protocol || descriptor.Address != "127.0.0.1" || descriptor.Port is < 1 or > 65535
            || descriptor.Token is null || descriptor.Token.Length != 64 || !descriptor.Token.All(Uri.IsHexDigit))
            throw new IOException("Eklenti ve masaüstü bağlantısı uyumlu değil. İkisini de güncelle.");

        var request = JsonSerializer.SerializeToUtf8Bytes(new { protocol = Protocol, token = descriptor.Token, method, @params = parameters }, Json);
        if (request.Length + 1 > MaxRequestBytes) throw new IOException("Mesaj bağlantı sınırını aşıyor.");
        using var client = new TcpClient(AddressFamily.InterNetwork);
        await client.ConnectAsync(IPAddress.Loopback, descriptor.Port, token).ConfigureAwait(false);
        using var stream = client.GetStream();
        await stream.WriteAsync(request, token).ConfigureAwait(false);
        await stream.WriteAsync(new byte[] { 10 }, token).ConfigureAwait(false);
        var response = await ReadBoundedAsync(stream, MaxResponseBytes, true, token).ConfigureAwait(false);
        var reply = JsonSerializer.Deserialize<Reply>(response, Json) ?? throw new IOException("Asistan boş bir yanıt döndürdü.");
        if (reply.Protocol != Protocol) throw new IOException("Eklenti ve masaüstü sürümleri uyumsuz.");
        if (!reply.Ok) throw new IOException(reply.Error ?? "Asistan isteği tamamlanamadı.");
        if (typeof(T) == typeof(JsonElement)) return (T)(object)reply.Result;
        return reply.Result.Deserialize<T>(Json) ?? throw new IOException("Asistan durumu okunamadı.");
    }

    private static async Task<byte[]> ReadBoundedAsync(Stream stream, int maximum, bool line, CancellationToken token)
    {
        using var output = new MemoryStream();
        var buffer = new byte[4096];
        while (true)
        {
            var read = await stream.ReadAsync(buffer, token).ConfigureAwait(false);
            if (read == 0)
            {
                if (line) throw new IOException("Asistan bağlantısı yanıt tamamlanmadan kapandı.");
                return output.ToArray();
            }
            var newline = line ? Array.IndexOf(buffer, (byte)10, 0, read) : -1;
            var length = newline < 0 ? read : newline;
            if (output.Length + length > maximum) throw new IOException("Asistan yanıtı izin verilen boyutu aşıyor.");
            output.Write(buffer, 0, length);
            if (newline >= 0) return output.ToArray();
        }
    }
}
