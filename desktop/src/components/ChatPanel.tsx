import { useEffect, useRef, useState } from 'react';
import { ArrowUpRight, Check, CircleStop, LoaderCircle, MessageSquare, Plug, Send, Sparkles } from 'lucide-react';
import { api, native } from '../api';
import type { ChatMessage, CodexStatus, Source } from '../types';

const suggestions = ['Bu kadro için hangi dizilişi önerirsin?', 'İlk 11’de hangi rol eşleşmelerini geliştirebiliriz?', 'Rotasyonda kimi değerlendirmeliyim?'];

export default function ChatPanel({ source, revision, onBusyChange }: { source: Source; revision: number; onBusyChange: (value: boolean) => void }) {
  const [status, setStatus] = useState<CodexStatus | null>(null);
  const [connecting, setConnecting] = useState(false);
  const [loginPending, setLoginPending] = useState(false);
  const [model, setModel] = useState('');
  const [input, setInput] = useState('');
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [busy, setBusy] = useState(false);
  const [live, setLive] = useState('');
  const [error, setError] = useState('');
  const bottom = useRef<HTMLDivElement>(null);
  const textarea = useRef<HTMLTextAreaElement>(null);
  const previousRevision = useRef(revision);
  const ready = status?.state === 'ready';

  useEffect(() => {
    if (previousRevision.current !== revision) {
      setMessages([]); setLive(''); setError('');
      previousRevision.current = revision;
    }
  }, [revision]);
  useEffect(() => { bottom.current?.scrollIntoView({ block: 'nearest' }); }, [messages, live]);
  useEffect(() => {
    if (!loginPending) return;
    let cancelled = false, attempts = 0;
    let timer: ReturnType<typeof setTimeout>;
    const check = async () => {
      try {
        const next = await api.codexStatus();
        if (cancelled) return;
        if (next.state === 'ready') { setStatus(next); setModel(next.models.find(m => m.isDefault)?.id ?? ''); setLoginPending(false); return; }
        if (++attempts >= 150) { setLoginPending(false); setError('Giriş süresi doldu. Bağlantıyı yeniden kontrol edebilirsin.'); return; }
        timer = setTimeout(check, 2000);
      } catch { if (!cancelled) { setLoginPending(false); setError('Giriş durumu kontrol edilemedi.'); } }
    };
    timer = setTimeout(check, 2000);
    return () => { cancelled = true; clearTimeout(timer); };
  }, [loginPending]);

  async function connect() {
    setConnecting(true); setError('');
    try { const next = await api.codexStatus(); setStatus(next); setModel(next.models.find(m => m.isDefault)?.id ?? ''); }
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
    if (!prompt || !ready || busy) return;
    const history = messages.slice(-18).map(m => ({ ...m, text: m.text.slice(0, 3500) }));
    setInput(''); setError(''); setLive(''); setBusy(true); onBusyChange(true);
    setMessages(prev => [...prev.slice(-29), { role: 'user', text: prompt }]);
    try {
      const answer = await api.chat(prompt, history, model || null, event => {
        if (event.type === 'delta') setLive(prev => prev + event.text);
        if (event.type === 'complete') setLive(event.text);
      });
      setMessages(prev => [...prev, { role: 'assistant', text: answer }]);
    } catch (e) { setError(String(e)); }
    finally { setLive(''); setBusy(false); onBusyChange(false); textarea.current?.focus(); }
  }

  return <section className="panel chat-panel">
    <div className="chat-heading"><div className="assistant-icon"><Sparkles size={22} /></div><div><span className="eyebrow">TAKTİK ORTAĞIN</span><h2>Birlikte düşünelim.</h2><p>Kadronu ve yerel analizini konuşmanın merkezine al.</p></div><span className={`connection-pill ${ready ? 'connected' : ''}`}>{ready ? <Check size={12} /> : <span className="status-dot" />}{ready ? 'ChatGPT bağlı' : 'Bağlantı bekleniyor'}</span></div>
    <div className="connection-strip"><div><Plug size={17} /><span>{status?.message ?? 'ChatGPT aboneliğinle kullandığın yerel Codex bağlantısını aç.'}</span></div>
      {ready ? <select aria-label="AI modeli" value={model} onChange={e => setModel(e.target.value)} disabled={busy}><option value="">Codex varsayılanı</option>{status.models.map(m => <option key={m.id} value={m.id}>{m.name}</option>)}</select> : <button className="button secondary" onClick={connect} disabled={connecting || !native}>{connecting ? <LoaderCircle size={15} className="spin" /> : <ArrowUpRight size={15} />}Bağlantıyı kontrol et</button>}
    </div>
    {status && !ready && native ? <div className="login-notice"><p>Resmi Codex CLI Mac’inde kurulu olmalı. Giriş ve abonelik sınırlarını Codex yönetir.</p><button className="button primary" onClick={login} disabled={connecting || loginPending}>{loginPending ? <LoaderCircle className="spin" size={16} /> : <Plug size={16} />}{loginPending ? 'Tarayıcıdaki giriş bekleniyor…' : 'ChatGPT ile giriş yap'}</button></div> : null}
    <div className="chat-messages" aria-label="Konuşma">
      {messages.length === 0 && !busy ? <div className="chat-welcome"><div className="welcome-orbit"><MessageSquare size={32} /></div><h3>İyi bir taktik, iyi bir soruyla başlar.</h3><p>{source === 'demo' ? 'Kurgusal kadronun güçlü yanlarını keşfet. Oyun sahibi olman gerekmiyor.' : 'İçe aktardığın kadroyu, rol uyumunu ve diziliş seçeneklerini birlikte değerlendir.'}</p><div className="prompt-suggestions">{suggestions.map(s => <button key={s} onClick={() => { setInput(s); textarea.current?.focus(); }}>{s}<ArrowUpRight size={16} /></button>)}</div></div> : null}
      {messages.map((m, i) => <article key={i} className={`chat-message ${m.role}`}><span className="message-avatar">{m.role === 'assistant' ? <Sparkles size={15} /> : 'C'}</span><div><span className="message-author">{m.role === 'assistant' ? 'Taktik asistanı' : 'Sen'}</span><p>{m.text}</p></div></article>)}
      {busy ? <article className="chat-message assistant"><span className="message-avatar"><Sparkles size={15} /></span><div><span className="message-author">Taktik asistanı</span><p>{live || <span className="thinking"><LoaderCircle className="spin" size={15} />Kadron değerlendiriliyor…</span>}</p></div></article> : null}
      <div ref={bottom} />
    </div>
    {error ? <p className="inline-error" role="alert">{error}</p> : null}
    <form className="chat-composer" onSubmit={e => { e.preventDefault(); void send(); }}><label className="sr-only" htmlFor="chat-input">Asistana mesaj</label><textarea id="chat-input" ref={textarea} rows={2} maxLength={1800} value={input} onChange={e => setInput(e.target.value)} placeholder={ready ? 'Kadron, roller veya taktik seçenekleri hakkında sor…' : 'Önce ChatGPT bağlantısını aç…'} disabled={busy}
      onKeyDown={e => { if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); void send(); } }} />
      {busy ? <button type="button" className="send-button" aria-label="Yanıtı durdur" onClick={() => { void api.stopChat(); }}><CircleStop size={20} /></button> : <button className="send-button" type="submit" disabled={!ready || !input.trim()} aria-label="Mesajı gönder"><Send size={19} /></button>}
    </form>
    <p className="chat-privacy">Mesaj gönderdiğinde kadro özeti ve son konuşma mesajları ChatGPT’ye iletilir. {source === 'demo' ? 'Örnek veriler kurgusaldır. ' : ''}Sohbet geçmişi bu uygulama oturumu boyunca tutulur.</p>
  </section>;
}
