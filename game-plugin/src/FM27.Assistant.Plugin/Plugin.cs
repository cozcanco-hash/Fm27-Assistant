using BepInEx;
using BepInEx.Unity.IL2CPP;
using FM27.Assistant.Client;

namespace FM27.Assistant.Plugin;

[BepInPlugin("studio.kutra.fm27assistant", "FM27 Assistant", "0.1.0")]
[BepInProcess("fm")]
[BepInProcess("fm.exe")]
public sealed class Plugin : BasePlugin
{
    private UnityPanel? panel;
    private CancellationTokenSource? lifetime;
    private BridgeClient? client;
    private readonly object stateLock = new();
    private Conversation? state;
    private string? error;
    private string? acceptedDraft;
    private bool requestPending;
    private bool pollPending;
    private DateTime nextPoll;
    private DateTime nextAttach;
    private int failures;
    private DateTime nextUi;
    private readonly SemaphoreSlim operations = new(1);
    private bool disabled;

    public override void Load()
    {
        var path = Config.Bind("Connection", "DescriptorPath", BridgeClient.DefaultDescriptorPath(),
            "Local FM Assistant connection file. Keep the desktop app running. No model credentials belong here.");
        client = new BridgeClient(path.Value);
        lifetime = new CancellationTokenSource();
        panel = new UnityPanel(Tick, Send, Cancel, NextModel, message => Log.LogWarning(message));
        try
        {
            panel.Start();
            Log.LogInfo("FM27 Assistant plugin loaded. Experimental UI Toolkit adapter; no live game data is read. Open the desktop app and enable the in-game connection.");
        }
        catch (Exception e)
        {
            disabled = true;
            Log.LogError($"FM27 Assistant UI could not attach: {e.GetBaseException().Message}. The desktop assistant remains available.");
        }
    }

    // Unity calls this on its main thread. Network and JSON work stay on worker tasks.
    private void Tick()
    {
        if (disabled || panel is null || lifetime is null) return;
        try
        {
            if (DateTime.UtcNow >= nextAttach)
            {
                nextAttach = DateTime.UtcNow.AddSeconds(2);
                panel.EnsureAttached();
            }
            panel.CheckHotkey();
            if (!panel.Visible || DateTime.UtcNow < nextUi) return;
            nextUi = DateTime.UtcNow.AddMilliseconds(100);
            Conversation? snapshot; string? issue; string? clear;
            lock (stateLock)
            {
                snapshot = state; issue = error; clear = acceptedDraft; acceptedDraft = null;
                if (!pollPending && !requestPending && DateTime.UtcNow >= nextPoll)
                {
                    pollPending = true;
                    var token = lifetime.Token;
                    _ = Task.Run(() => PollAsync(token));
                }
            }
            panel.Render(snapshot, issue, requestPending, clear);
        }
        catch (Exception e)
        {
            // Stop the optional adapter instead of throwing out into the game's render loop.
            if (++failures >= 3) { disabled = true; panel.Dispose(); }
            Log.LogWarning($"FM Assistant panel issue: {e.GetBaseException().Message}");
        }
    }

    private async Task PollAsync(CancellationToken token)
    {
        var acquired = false;
        try { await operations.WaitAsync(token).ConfigureAwait(false); acquired = true; var value = await client!.StateAsync(token).ConfigureAwait(false); lock (stateLock) { state = value; error = null; } }
        catch (OperationCanceledException) { if (!token.IsCancellationRequested) lock (stateLock) error = "Asistan bağlantısı zaman aşımına uğradı."; }
        catch (Exception e) { lock (stateLock) error = e.Message; }
        finally { if (acquired) operations.Release(); lock (stateLock) { pollPending = false; nextPoll = DateTime.UtcNow.AddMilliseconds(750); } }
    }

    private void Send(string text)
    {
        lock (stateLock)
        {
            if (requestPending || state is null || state.Busy || lifetime is null) return;
            requestPending = true;
            var scope = state.Scope;
            var cancellation = lifetime.Token;
            _ = Task.Run(() => ChangeAsync(async token => {
                var value = await client!.SendAsync(scope, text, token).ConfigureAwait(false);
                lock (stateLock) acceptedDraft = text;
                return value;
            }, cancellation));
        }
    }

    private void Cancel()
    {
        lock (stateLock)
        {
            if (requestPending || state?.ActiveTurnId is not long id || lifetime is null) return;
            requestPending = true;
            var cancellation = lifetime.Token;
            _ = Task.Run(() => ChangeAsync(async token => {
                await client!.CancelAsync(id, token).ConfigureAwait(false);
                return await client.StateAsync(token).ConfigureAwait(false);
            }, cancellation));
        }
    }

    private void NextModel()
    {
        lock (stateLock)
        {
            if (requestPending || state is null || state.Busy || lifetime is null) return;
            var choices = new string?[] { null }.Concat(state.Models.Select(m => (string?)m.Id)).ToArray();
            var index = Array.IndexOf(choices, state.Model);
            var model = choices[(index + 1) % choices.Length];
            var scope = state.Scope;
            requestPending = true;
            var cancellation = lifetime.Token;
            _ = Task.Run(() => ChangeAsync(token => client!.SetModelAsync(scope, model, token), cancellation));
        }
    }

    private async Task ChangeAsync(Func<CancellationToken, Task<Conversation>> action, CancellationToken token)
    {
        var acquired = false;
        try { await operations.WaitAsync(token).ConfigureAwait(false); acquired = true; var value = await action(token).ConfigureAwait(false); lock (stateLock) { state = value; error = null; } }
        catch (OperationCanceledException) { if (!token.IsCancellationRequested) lock (stateLock) error = "Asistan bağlantısı zaman aşımına uğradı. Mesaj otomatik olarak tekrar gönderilmedi."; }
        catch (Exception e) { lock (stateLock) error = e.Message; }
        finally { if (acquired) operations.Release(); lock (stateLock) { requestPending = false; nextPoll = DateTime.UtcNow; } }
    }

    public override bool Unload()
    {
        disabled = true;
        lifetime?.Cancel();
        panel?.Dispose();
        lifetime?.Dispose();
        return true;
    }
}
