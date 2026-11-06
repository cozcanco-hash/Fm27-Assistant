using System.Diagnostics;
using System.Net;
using System.Net.Sockets;
using System.Text;
using System.Text.Json;
using FM27.Assistant.Client;

if (args.Length != 2) throw new ArgumentException("Pass the Rust bridge_host executable and the fake-shared-codex.py fixture.");
var binary = Path.GetFullPath(args[0]);
var fake = Path.GetFullPath(args[1]);
var directory = Path.Combine(Path.GetTempPath(), "fm27-csharp-check-" + Guid.NewGuid().ToString("N"));
Directory.CreateDirectory(directory);
Process? process = null;
var descriptor = Path.Combine(directory, "in-game", "bridge.json");
try
{
    process = StartHost();
    await Until(async () => { if (!File.Exists(descriptor)) return false; try { await new BridgeClient(descriptor).StateAsync(); return true; } catch (IOException) { return false; } });
    var first = new BridgeClient(descriptor);
    var second = new BridgeClient(descriptor);
    var state = await first.StateAsync();
    Check(state.Source == "demo" && state.Messages.Length == 0, "Source is truthful and the new conversation is empty");
    state = await first.SetModelAsync(state.Scope, "protocol-test");
    Check((await second.StateAsync()).Model == "protocol-test", "Both clients see one model choice");
    state = await first.SendAsync(state.Scope, "C# istemcisinden Türkçe mesaj: şığöüç");
    Check(state.Busy, "Send returns promptly while the Rust host performs the turn");
    await Fails(async () => await second.SendAsync(state.Scope, "Concurrent send"), "Concurrent turns are rejected");
    await Until(async () => !(await second.StateAsync()).Busy);
    state = await second.StateAsync();
    Check(state.Messages.Length == 2 && state.Messages[0].Client == "game" && state.Messages[1].Text == "TEST: first answer", "The actual C# client reads the Rust host's authoritative messages");
    await second.SendAsync(state.Scope, "Önceki konuşmadan devam");
    await Until(async () => !(await first.StateAsync()).Busy);
    state = await first.StateAsync();
    Check(state.Messages.Last().Text == "TEST: shared conversation received", "The host supplies shared history to the protocol peer");
    state = await first.SendAsync(state.Scope, "İptal edilecek yanıt");
    await second.CancelAsync(state.ActiveTurnId!.Value);
    await Until(async () => !(await first.StateAsync()).Busy);
    state = await first.StateAsync();
    Check(state.Messages.Last().State == "interrupted", "Cancellation is visible on the other client");
    Check(PanelText.Context(state).Contains("KURGUSAL") && PanelText.Transcript(state).Contains("tamamlanmadı"), "The game panel preserves provenance and interruption status");
    await StopHost(process); process = null;
    Check(!File.Exists(descriptor), "Shutdown removes discovery credentials");
    process = StartHost();
    await Until(() => Task.FromResult(File.Exists(descriptor)));
    var restored = await first.StateAsync();
    Check(restored.Messages.Length == 6 && restored.Model == "protocol-test", "History and model survive host restart");
    await Fails(async () => await first.SendAsync("stale-career", "Stale request"), "Old career context is rejected");
    try { await first.SendAsync(restored.Scope, new string('ş', 4001)); throw new Exception("UTF-8 length was not bounded"); } catch (ArgumentException) { }
    var bad = Path.Combine(directory, "bad.json");
    await File.WriteAllTextAsync(bad, JsonSerializer.Serialize(new Descriptor(1, "192.0.2.1", 80, new string('a', 64))));
    await Fails(async () => await new BridgeClient(bad).StateAsync(), "Descriptor cannot redirect to a remote host");
    await File.WriteAllTextAsync(bad, new string('x', 4097));
    await Fails(async () => await new BridgeClient(bad).StateAsync(), "Discovery reads are bounded");
    await OversizedResponse(bad);
    Console.WriteLine("PASS: C# ↔ Rust integration, persistence, cancellation and bounds. No game and no real AI account were used.");
}
finally
{
    if (process is not null) await StopHost(process);
    Directory.Delete(directory, true);
}

Process StartHost()
{
    var info = new ProcessStartInfo(binary) { RedirectStandardInput = true, RedirectStandardOutput = true, RedirectStandardError = true, UseShellExecute = false };
    info.ArgumentList.Add(directory); info.Environment["FM27_CODEX_PATH"] = fake;
    var child = Process.Start(info) ?? throw new Exception("Rust host could not start");
    child.BeginOutputReadLine(); child.BeginErrorReadLine();
    return child;
}
static async Task StopHost(Process process)
{
    if (!process.HasExited)
    {
        await process.StandardInput.WriteLineAsync();
        try { using var timeout = new CancellationTokenSource(TimeSpan.FromSeconds(8)); await process.WaitForExitAsync(timeout.Token); }
        catch (OperationCanceledException) { process.Kill(true); await process.WaitForExitAsync(); }
    }
    process.Dispose();
}
static void Check(bool value, string message) { if (!value) throw new Exception(message); Console.WriteLine("PASS: " + message); }
static async Task Until(Func<Task<bool>> condition)
{
    var timer = Stopwatch.StartNew();
    while (!await condition()) { if (timer.Elapsed > TimeSpan.FromSeconds(8)) throw new TimeoutException("Acceptance check timed out"); await Task.Delay(40); }
}
static async Task Fails(Func<Task> action, string message)
{
    try { await action(); } catch (IOException) { Console.WriteLine("PASS: " + message); return; }
    throw new Exception(message);
}
static async Task OversizedResponse(string path)
{
    var listener = new TcpListener(IPAddress.Loopback, 0); listener.Start();
    var port = ((IPEndPoint)listener.LocalEndpoint).Port;
    await File.WriteAllTextAsync(path, JsonSerializer.Serialize(new Descriptor(1, "127.0.0.1", port, new string('a', 64))));
    var serving = Task.Run(async () => {
        using var socket = await listener.AcceptTcpClientAsync();
        try { await socket.GetStream().WriteAsync(Encoding.UTF8.GetBytes(new string('x', BridgeClient.MaxResponseBytes + 1) + "\n")); }
        catch (IOException) { }
    });
    try { await Fails(async () => await new BridgeClient(path).StateAsync(), "Network responses are bounded before full allocation"); }
    finally { listener.Stop(); await serving; }
}
