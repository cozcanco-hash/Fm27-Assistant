import { useEffect, useRef, useState } from 'react';
import { ArrowUpRight, Check, CircleStop, LoaderCircle, MessageSquare, Plug, Send, Sparkles } from 'lucide-react';
import { api, native } from '../api';
import type { Conversation, CodexStatus, Source } from '../types';

const suggestions = ['Bu kadro için hangi dizilişi önerirsin?', 'İlk 11’de hangi rol eşleşmelerini geliştirebiliriz?', 'Rotasyonda kimi değerlendirmeliyim?'];

export default function ChatPanel({ source, revision, onBusyChange }: { source: Source; revision: number; onBusyChange: (value: boolean) => void }) {
  const [status, setStatus] = useState<CodexStatus | null>(null);
  const [connecting, setConnecting] = useState(false);
  const [loginPending, setLoginPending] = useState(false);
  const [shared, setShared] = useState<Conversation | null>(null);
  const [sending, setSending] = useState(false);
  const scopeRef = useRef('');
  const [input, setInput] = useState('');
  const messages = shared?.messages ?? [];
  const busy = Boolean(shared?.busy) || sending;
  const live = shared?.live ?? '';
  const model = shared?.model ?? '';
  const [error, setError] = useState('');
  const bottom = useRef<HTMLDivElement>(null);
  const textarea = useRef<HTMLTextAreaElement>(null);
  const ready = status?.state === 'ready';

  useEffect(() => {
    let active = true;
    let timer: ReturnType<typeof setTimeout>;
    async function refresh() {
      try {
        const next = await api.conversation();
        if (!active) return;
        if (scopeRef.current && scopeRef.current !== next.scope) { setInput(''); setError(''); }
        scopeRef.current = next.scope;
        setShared(next); onBusyChange(next.busy);
      } catch (e) { if (active) setError(String(e)); }
      if (active && native) timer = setTimeout(refresh, 600);
    }
    void refresh();
    return () => { active = false; clearTimeout(timer); };
  }, [revision, onBusyChange]);
  useEffect(() => { bottom.current?.scrollIntoView({ block: 'nearest' }); }, [messages, live]);
  useEffect(() => {
    if (!loginPending) return;
    let cancelled = false, attempts = 0;
    let timer: ReturnType<typeof setTimeout>;
    const check = async () => {
      try {
        const next = await api.codexStatus();
        if (cancelled) return;
        if (next.state === 'ready') { setStatus(next); setLoginPending(false); return; }
        if (++attempts >= 150) { setLoginPending(false); setError('Giriş süresi doldu. Bağlantıyı yeniden kontrol edebilirsin.'); return; }
        timer = setTimeout(check, 2000);
      } catch { if (!cancelled) { setLoginPending(false); setError('Giriş durumu kontrol edilemedi.'); } }
    };
    timer = setTimeout(check, 2000);
    return () => { cancelled = true; clearTimeout(timer); };
  }, [loginPending]);

  async function connect() {
    setConnecting(true); setError('');
    try { const next = await api.codexStatus(); setStatus(next); }
    catch (e) { setError(String(e)); }
    finally { setConnecting(false); }
  }
  async function login() {
    setConnecting(true); setError('');
    try { await api.login(); setLoginPending(true); }
    catch (e) { setError(String(e)); }
    finally { setConnecting(false); }
  }
  async function send() {
    const prompt = input.trim();
    if (!prompt || !shared || !ready || busy) return;
    setError(''); setSending(true); onBusyChange(true);
    try { const next = await api.chat(shared.scope, prompt); setShared(next); setInput(''); }
    catch (e) { setError(String(e)); }
    finally { setSending(false); textarea.current?.focus(); }
  }
  async function selectModel(value: string) {
    if (!shared) return;
    try { setShared(await api.setChatModel(shared.scope, value || null)); }
    catch (e) { setError(String(e)); }
  }

  return <section className="panel chat-panel">
    <div className="chat-heading"><div className="assistant-icon"><Sparkles size={22} /></div><div><span className="eyebrow">TAKTİK ORTAĞIN</span><h2>Birlikte düşünelim.</h2><p>Kadronu ve yerel analizini konuşmanın merkezine al.</p></div><span className={`connection-pill ${ready ? 'connected' : ''}`}>{ready ? <Check size={12} /> : <span className="status-dot" />}{ready ? 'ChatGPT bağlı' : 'Bağlantı bekleniyor'}</span></div>
    <div className="connection-strip"><div><Plug size={17} /><span>{status?.message ?? 'ChatGPT aboneliğinle kullandığın yerel Codex bağlantısını aç.'}</span></div>
      {ready ? <select aria-label="AI modeli" value={model} onChange={e => { void selectModel(e.target.value); }} disabled={busy}><option value="">Codex varsayılanı</option>{status.models.map(m => <option key={m.id} value={m.id}>{m.name}</option>)}</select> : <button className="button secondary" onClick={connect} disabled={connecting || !native}>{connecting ? <LoaderCircle size={15} className="spin" /> : <ArrowUpRight size={15} />}Bağlantıyı kontrol et</button>}
    </div>
    {status && !ready && native ? <div className="login-notice"><p>Resmi Codex CLI Mac’inde kurulu olmalı. Giriş ve abonelik sınırlarını Codex yönetir.</p><button className="button primary" onClick={login} disabled={connecting || loginPending}>{loginPending ? <LoaderCircle className="spin" size={16} /> : <Plug size={16} />}{loginPending ? 'Tarayıcıdaki giriş bekleniyor…' : 'ChatGPT ile giriş yap'}</button></div> : null}
    <div className="chat-messages" aria-label="Konuşma">
      {messages.length === 0 && !busy ? <div className="chat-welcome"><div className="welcome-orbit"><MessageSquare size={32} /></div><h3>İyi bir taktik, iyi bir soruyla başlar.</h3><p>{source === 'demo' ? 'Kurgusal kadronun güçlü yanlarını keşfet. Oyun sahibi olman gerekmiyor.' : 'İçe aktardığın kadroyu, rol uyumunu ve diziliş seçeneklerini birlikte değerlendir.'}</p><div className="prompt-suggestions">{suggestions.map(s => <button key={s} onClick={() => { setInput(s); textarea.current?.focus(); }}>{s}<ArrowUpRight size={16} /></button>)}</div></div> : null}
      {messages.map(m => <article key={m.id} className={`chat-message ${m.role}`}><span className="message-avatar">{m.role === 'assistant' ? <Sparkles size={15} /> : 'C'}</span><div><span className="message-author">{m.role === 'assistant' ? 'Taktik asistanı' : `Sen · ${m.client === 'game' ? 'oyun' : 'masaüstü'}`}</span><p>{m.state === 'pending' ? live || <span className="thinking"><LoaderCircle className="spin" size={15} />Kadron değerlendiriliyor…</span> : m.text}</p>{m.state === 'failed' || m.state === 'interrupted' ? <small className="inline-error">Yanıt tamamlanmadı.</small> : null}</div></article>)}
      <div ref={bottom} />
    </div>
    {error || shared?.error ? <p className="inline-error" role="alert">{error || shared?.error}</p> : null}
    <form className="chat-composer" onSubmit={e => { e.preventDefault(); void send(); }}><label className="sr-only" htmlFor="chat-input">Asistana mesaj</label><textarea id="chat-input" ref={textarea} rows={2} maxLength={1800} value={input} onChange={e => setInput(e.target.value)} placeholder={ready ? 'Kadron, roller veya taktik seçenekleri hakkında sor…' : 'Önce ChatGPT bağlantısını aç…'} disabled={busy}
      onKeyDown={e => { if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); void send(); } }} />
      {busy ? <button type="button" className="send-button" aria-label="Yanıtı durdur" onClick={() => { if (shared?.activeTurnId != null) void api.stopChat(shared.activeTurnId).catch(e => setError(String(e))); }}><CircleStop size={20} /></button> : <button className="send-button" type="submit" disabled={!ready || !input.trim()} aria-label="Mesajı gönder"><Send size={19} /></button>}
    </form>
    <p className="chat-privacy">Mesaj gönderdiğinde kadro özeti ve son konuşma mesajları ChatGPT’ye iletilir. {source === 'demo' ? 'Örnek veriler kurgusaldır. ' : ''}Sohbet geçmişi bu kariyer için Mac’inde saklanır ve oyun içi panelle paylaşılır.</p>
  </section>;
}
