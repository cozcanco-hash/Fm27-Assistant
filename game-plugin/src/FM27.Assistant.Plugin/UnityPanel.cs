using System.Collections;
using System.Reflection;
using FM27.Assistant.Client;

namespace FM27.Assistant.Plugin;

/// <summary>
/// Late-binds Unity's generated interop wrappers. No FM assets or game assemblies
/// are redistributed. This adapter must still be accepted on the exact FM build.
/// </summary>
internal sealed class UnityPanel : IDisposable
{
    private static readonly System.Collections.Concurrent.ConcurrentDictionary<string, Type> Types = new();
    private const BindingFlags Public = BindingFlags.Public | BindingFlags.Instance | BindingFlags.Static;
    private readonly Action tick;
    private readonly Action<string> send;
    private readonly Action cancel;
    private readonly Action nextModel;
    private readonly Action<string> warn;
    private readonly List<Subscription> controls = new();
    private Subscription? render;
    private object? root;
    private object? content;
    private object? context;
    private object? transcript;
    private object? input;
    private object? status;
    private object? modelButton;
    private object? sendButton;
    private object? stopButton;
    private string lastTranscript = "";
    private int lastKeyFrame = -1;
    private bool hotkeyAvailable = true;
    public bool Visible { get; private set; }

    internal UnityPanel(Action tick, Action<string> send, Action cancel, Action nextModel, Action<string> warn)
    { this.tick = tick; this.send = send; this.cancel = cancel; this.nextModel = nextModel; this.warn = warn; }

    public void Start() => render = Subscribe(FindType("UnityEngine.Application", "UnityEngine.CoreModule"), null, "onBeforeRender", tick);

    public void EnsureAttached()
    {
        if (root is not null && Get(root, "panel") is not null && Get(root, "parent") is not null) return;
        Detach();
        var documentType = FindType("UnityEngine.UIElements.UIDocument", "UnityEngine.UIElementsModule");
        var resources = FindType("UnityEngine.Resources", "UnityEngine.CoreModule");
        var find = resources.GetMethods(Public).Single(m => m.Name == "FindObjectsOfTypeAll" && m.IsGenericMethodDefinition && m.GetParameters().Length == 0);
        var documents = (IEnumerable?)find.MakeGenericMethod(documentType).Invoke(null, null);
        if (documents is null) return;
        object? parent = null;
        foreach (var document in documents)
        {
            if (document is null || Get(document, "enabled") is false) continue;
            var candidate = Get(document, "rootVisualElement");
            if (candidate is not null && Get(candidate, "panel") is not null) { parent = candidate; break; }
        }
        if (parent is null) return;
        root = New("VisualElement");
        Set(root, "name", "fm27-assistant-panel");
        Style(root, "position", "Absolute"); Style(root, "right", 16f); Style(root, "top", 56f);
        Style(root, "width", Visible ? 420f : 160f);
        Add(root, Button("FM Assistant · F8", Toggle));
        content = New("VisualElement");
        Style(content, "backgroundColor", Color(0.063f, 0.082f, 0.106f, 1));
        Style(content, "paddingLeft", 12f); Style(content, "paddingRight", 12f);
        Style(content, "paddingTop", 12f); Style(content, "paddingBottom", 12f);
        Style(content, "height", 530f);
        context = Label("Masaüstü bağlantısı bekleniyor…"); Add(content, context);
        modelButton = Button("Model: Codex varsayılanı", nextModel); Add(content, modelButton);
        transcript = New("TextField"); Set(transcript, "multiline", true); Set(transcript, "isReadOnly", true);
        Style(transcript, "flexGrow", 1f); Add(content, transcript);
        status = Label("Girişini masaüstü uygulamasında tamamla."); Add(content, status);
        input = New("TextField"); Set(input, "multiline", true); Set(input, "maxLength", 1800);
        Set(input, "label", "Asistana mesaj"); Style(input, "height", 72f); Add(content, input);
        sendButton = Button("Gönder", () => send((string?)Get(input, "value") ?? "")); Add(content, sendButton);
        stopButton = Button("Yanıtı durdur", cancel); Add(content, stopButton);
        Add(root, content); Add(parent, root); ApplyVisibility();
        lastTranscript = "";
    }

    public void CheckHotkey()
    {
        if (!hotkeyAvailable) return;
        try
        {
            var frame = (int?)FindType("UnityEngine.Time", "UnityEngine.CoreModule").GetProperty("frameCount", Public)?.GetValue(null) ?? -1;
            if (frame == lastKeyFrame) return;
            lastKeyFrame = frame;
            var inputType = FindType("UnityEngine.Input", "UnityEngine.InputLegacyModule");
            var keyType = FindType("UnityEngine.KeyCode", "UnityEngine.CoreModule");
            var method = inputType.GetMethod("GetKeyDown", Public, null, new[] { keyType }, null);
            if (method?.Invoke(null, new[] { Enum.Parse(keyType, "F8") }) is true) Toggle();
        }
        catch { hotkeyAvailable = false; warn("F8 is unavailable with this input backend. Use the FM Assistant button."); }
    }

    private void Toggle() { Visible = !Visible; ApplyVisibility(); }
    private void ApplyVisibility()
    {
        if (root is null || content is null) return;
        Style(root, "width", Visible ? 420f : 160f);
        Style(content, "display", Visible ? "Flex" : "None");
        if (Visible && input is not null) Call(input, "Focus");
        else if (input is not null) Call(input, "Blur");
    }

    public void Render(Conversation? state, string? error, bool pending, string? acceptedDraft)
    {
        if (root is null || content is null || !Visible) return;
        var connected = state is not null && error is null;
        Set(context!, "text", state is null ? "Masaüstündeki oyun içi bağlantıyı aç." : PanelText.Context(state));
        Set(status!, "text", error ?? state?.Error ?? (state?.Busy == true ? "Yanıt hazırlanıyor…" : "Aynı konuşma masaüstünde de açık."));
        var body = state is null ? "FM Assistant masaüstü uygulamasını arka planda açık tut." : PanelText.Transcript(state);
        if (body != lastTranscript) { Set(transcript!, "value", body); lastTranscript = body; }
        Set(modelButton!, "text", "Model: " + (state?.Model ?? "Codex varsayılanı"));
        Call(sendButton!, "SetEnabled", connected && !pending && state?.Busy == false);
        Call(modelButton!, "SetEnabled", connected && !pending && state?.Busy == false);
        Call(stopButton!, "SetEnabled", connected && !pending && state?.Busy == true);
        Call(input!, "SetEnabled", !pending && state?.Busy != true);
        if (acceptedDraft is not null && (string?)Get(input!, "value") == acceptedDraft) Set(input!, "value", "");
    }

    private static object New(string name) => Activator.CreateInstance(FindType("UnityEngine.UIElements." + name, "UnityEngine.UIElementsModule")) ?? throw new MissingMethodException(name);
    private static object Label(string text) { var label = New("Label"); Set(label, "text", text); Set(label, "enableRichText", false); Style(label, "color", Color(0.88f, 0.91f, 0.94f, 1)); Style(label, "whiteSpace", "Normal"); return label; }
    private object Button(string text, Action action)
    {
        var button = New("Button"); Set(button, "text", text);
        controls.Add(Subscribe(button.GetType(), button, "clicked", action));
        return button;
    }
    private static object Color(float r, float g, float b, float a) => Activator.CreateInstance(FindType("UnityEngine.Color", "UnityEngine.CoreModule"), r, g, b, a)!;
    private static void Add(object parent, object child) => Call(parent, "Add", child);
    private static object? Get(object owner, string name) => owner.GetType().GetProperty(name, Public)?.GetValue(owner);
    private static void Set(object owner, string name, object value)
    {
        var property = owner.GetType().GetProperty(name, Public) ?? throw new MissingMemberException(owner.GetType().FullName, name);
        property.SetValue(owner, value);
    }
    private static void Call(object owner, string name, params object[] args)
    {
        var method = owner.GetType().GetMethods(Public).FirstOrDefault(m => m.Name == name && !m.ContainsGenericParameters
            && m.GetParameters().Length == args.Length && m.GetParameters().Zip(args).All(p => p.First.ParameterType.IsInstanceOfType(p.Second)));
        (method ?? throw new MissingMethodException(owner.GetType().FullName, name)).Invoke(owner, args);
    }
    private static void Style(object element, string name, object value)
    {
        var style = Get(element, "style") ?? throw new MissingMemberException("VisualElement.style");
        var property = style.GetType().GetProperty(name, Public) ?? throw new MissingMemberException("IStyle." + name);
        object converted = value;
        if (!property.PropertyType.IsInstanceOfType(value))
        {
            var implicitOperators = property.PropertyType.GetMethods(Public).Where(m => m.Name == "op_Implicit" && m.GetParameters().Length == 1);
            var conversion = implicitOperators.FirstOrDefault(m => m.GetParameters()[0].ParameterType.IsInstanceOfType(value));
            if (conversion is null && value is string text)
            {
                conversion = implicitOperators.FirstOrDefault(m => m.GetParameters()[0].ParameterType.IsEnum && Enum.IsDefined(m.GetParameters()[0].ParameterType, text));
                if (conversion is not null) converted = Enum.Parse(conversion.GetParameters()[0].ParameterType, text);
            }
            converted = (conversion ?? throw new MissingMethodException(property.PropertyType.FullName, "op_Implicit")).Invoke(null, new[] { converted })!;
        }
        property.SetValue(style, converted);
    }
    private static Type FindType(string name, string assembly)
    {
        return Types.GetOrAdd(name + "|" + assembly, _ => {
            var found = AppDomain.CurrentDomain.GetAssemblies().Select(a => a.GetType(name, false)).FirstOrDefault(t => t is not null);
            return found ?? Assembly.Load(new AssemblyName(assembly)).GetType(name, true)!;
        });
    }
    private static Subscription Subscribe(Type ownerType, object? owner, string name, Action callback)
    {
        var add = ownerType.GetEvent(name, Public)?.GetAddMethod() ?? ownerType.GetMethod("add_" + name, Public) ?? throw new MissingMethodException(ownerType.FullName, "add_" + name);
        var remove = ownerType.GetEvent(name, Public)?.GetRemoveMethod() ?? ownerType.GetMethod("remove_" + name, Public) ?? throw new MissingMethodException(ownerType.FullName, "remove_" + name);
        var type = add.GetParameters()[0].ParameterType;
        object handler = callback;
        if (!type.IsInstanceOfType(callback))
        {
            var support = FindType("Il2CppInterop.Runtime.DelegateSupport", "Il2CppInterop.Runtime");
            var convert = support.GetMethods(Public).Single(m => m.Name == "ConvertDelegate" && m.IsGenericMethodDefinition && m.GetParameters().Length == 1);
            handler = convert.MakeGenericMethod(type).Invoke(null, new object[] { callback })!;
        }
        add.Invoke(owner, new[] { handler });
        return new Subscription(owner, remove, handler);
    }
    private sealed class Subscription(object? owner, MethodInfo remove, object handler) : IDisposable
    {
        // Keep the interop delegate rooted for the whole subscription lifetime.
        public void Dispose() { try { remove.Invoke(owner, new[] { handler }); } catch { /* The game may already have destroyed the document. */ } }
    }
    private void Detach()
    {
        foreach (var handler in controls) handler.Dispose(); controls.Clear();
        if (root is not null) { try { Call(root, "RemoveFromHierarchy"); } catch { } }
        root = content = context = transcript = input = status = modelButton = sendButton = stopButton = null;
    }
    public void Dispose() { render?.Dispose(); render = null; Detach(); }
}
