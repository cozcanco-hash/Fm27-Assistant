import { useCallback, useEffect, useRef, useState } from 'react';
import { Activity, ArrowRight, ArrowUpRight, CalendarDays, CheckCircle2, ChevronDown, CircleHelp, Database, FlaskConical, FolderInput, LayoutDashboard, LoaderCircle, Shield, Sparkles, Target, Users, Waypoints, X } from 'lucide-react';
import { api, native } from './api';
import { FORMATIONS, scoreText } from './types';
import type { Dashboard, Formation, PlayerDetail, PlayerPage, Position } from './types';
import Pitch from './components/Pitch';
import PlayerTable from './components/PlayerTable';
import PlayerDrawer from './components/PlayerDrawer';
import ChatPanel from './components/ChatPanel';
import DataPanel from './components/DataPanel';

type View = 'overview' | 'squad' | 'tactics' | 'assistant' | 'data';
const navigation = [
  { id: 'overview' as const, label: 'Genel bakış', icon: LayoutDashboard },
  { id: 'squad' as const, label: 'Kadro', icon: Users },
  { id: 'tactics' as const, label: 'Taktik laboratuvarı', icon: Waypoints },
  { id: 'assistant' as const, label: 'AI asistan', icon: Sparkles },
];
const headings: Record<View, { title: string; subtitle: string }> = {
  overview: { title: 'Bir sonraki hamleni planla.', subtitle: 'Kadronu tanı. Seçeneklerini gör. Kararını güçlendir.' },
  squad: { title: 'Kadrondaki potansiyeli keşfet.', subtitle: 'Özellikler, form ve rol uyumu tek çalışma alanında.' },
  tactics: { title: 'Fikirlerini sahaya yerleştir.', subtitle: 'Üç dizilişi dene, rol eşleşmelerini ve eksikleri karşılaştır.' },
  assistant: { title: 'Yanında bir taktik ortağı.', subtitle: 'Yerel kadro analizini ChatGPT ile birlikte değerlendir.' },
  data: { title: 'Kontrol sende.', subtitle: 'Veri kaynağını, çalışma alanını ve bağlantılarını yönet.' },
};

export default function App() {
  const [view, setView] = useState<View>('overview');
  const [dashboard, setDashboard] = useState<Dashboard | null>(null);
  const [page, setPage] = useState<PlayerPage | null>(null);
  const [search, setSearch] = useState('');
  const [position, setPosition] = useState('');
  const [offset, setOffset] = useState(0);
  const [revision, setRevision] = useState(0);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [chatBusy, setChatBusy] = useState(false);
  const [detail, setDetail] = useState<PlayerDetail | null>(null);
  const [toast, setToast] = useState('');
  const [fatal, setFatal] = useState('');
  const fileInput = useRef<HTMLInputElement>(null);
  const readySent = useRef(false);
  const playerRequest = useRef(0);
  const closePlayer = useCallback(() => setDetail(null), []);

  useEffect(() => { let active = true; void api.dashboard().then(d => { if (active) setDashboard(d); }).catch(e => { if (active) setFatal(String(e)); }); return () => { active = false; }; }, []);
  useEffect(() => {
    let active = true;
    setLoading(true);
    const timer = setTimeout(() => {
      void api.players({ search, position: (position || null) as Position | null, offset, limit: 30 }).then(p => { if (active) setPage(p); }).catch(e => { if (active) setToast(String(e)); }).finally(() => { if (active) setLoading(false); });
    }, search ? 180 : 0);
    return () => { active = false; clearTimeout(timer); };
  }, [search, position, offset, revision]);
  useEffect(() => {
    if (dashboard && page && !readySent.current) {
      readySent.current = true;
      requestAnimationFrame(() => requestAnimationFrame(() => { void api.ready().catch(() => {}); }));
    }
  }, [dashboard, page]);
  useEffect(() => { if (!toast) return; const timer = setTimeout(() => setToast(''), 7000); return () => clearTimeout(timer); }, [toast]);

  async function openPlayer(id: number) {
    const request = ++playerRequest.current;
    try { const player = await api.player(id); if (request === playerRequest.current) setDetail(player); }
    catch (e) { setToast(String(e)); }
  }
  async function changeFormation(formation: Formation) {
    setBusy(true);
    try { setDashboard(await api.formation(formation)); }
    catch (e) { setToast(String(e)); }
    finally { setBusy(false); }
  }
  async function importFile(file: File | undefined) {
    if (!file) return;
    if (file.size > 4 * 1024 * 1024) { setToast('Kadro dosyası en fazla 4 MB olabilir.'); return; }
    if (!file.name.toLowerCase().endsWith('.json')) { setToast('Uygulamanın kadro şemasında bir JSON dosyası seç.'); return; }
    setBusy(true);
    try {
      const next = await api.importRoster(await file.text());
      setDashboard(next); setRevision(v => v + 1); setSearch(''); setPosition(''); setOffset(0); setDetail(null);
      setToast('Kadro içe aktarıldı ve yerel çalışma alanına kaydedildi.');
    } catch (e) { setToast(`İçe aktarılamadı: ${String(e)}`); }
    finally { setBusy(false); }
  }
  async function loadDemo() {
    setBusy(true);
    try { setDashboard(await api.demo()); setRevision(v => v + 1); setSearch(''); setPosition(''); setOffset(0); setDetail(null); setToast('Kurgusal örnek kadro yüklendi.'); }
    catch (e) { setToast(String(e)); }
    finally { setBusy(false); }
  }

  if (fatal) return <main className="startup-state"><Shield size={35} /><h1>Çalışma alanı açılamadı.</h1><p>{fatal}</p><p>Kaydedilmiş veri otomatik olarak silinmedi.</p></main>;
  if (!dashboard) return <main className="startup-state"><LoaderCircle size={32} className="spin" /><h1>Çalışma alanın hazırlanıyor…</h1></main>;

  const table = <PlayerTable page={page} search={search} position={position} onSearch={s => { setSearch(s); setOffset(0); }} onPosition={p => { setPosition(p); setOffset(0); }} onPlayer={id => { void openPlayer(id); }} onPage={setOffset} loading={loading} compact={view === 'overview'} />;
  return <div className="app-shell">
    <div className="window-drag" data-tauri-drag-region><span>FM27 ASSISTANT</span><span>{native ? 'Kişisel çalışma alanı' : 'Tarayıcı önizlemesi'}</span></div>
    <aside className="sidebar"><button className="brand" onClick={() => setView('overview')} aria-label="Genel bakışa dön"><span className="brand-mark">F<span>27</span></span><span className="brand-word">FM27<span>ASSISTANT</span></span></button>
      <button className="club-switch" onClick={() => setView('data')}><span className="club-crest"><Shield size={23} strokeWidth={1.4} /><b>K</b></span><span><strong>{dashboard.club}</strong><small>{dashboard.season} sezonu</small></span><ChevronDown size={14} /></button>
      <span className="nav-heading">ÇALIŞMA ALANI</span><nav aria-label="Ana menü">{navigation.map(item => <button key={item.id} aria-label={item.label} className={view === item.id ? 'active' : ''} onClick={() => setView(item.id)} aria-current={view === item.id ? 'page' : undefined}><item.icon size={18} /><span>{item.label}</span>{item.id === 'assistant' ? <span className="nav-ai">AI</span> : null}</button>)}</nav>
      <div className="sidebar-note"><span className="note-orbit"><FlaskConical size={19} /></span><strong>Oyun öncesi, bir adım önde.</strong><p>Kurgusal kadroyla fikirlerini şimdiden dene.</p><button onClick={() => setView('tactics')}>Taktiğini keşfet <ArrowUpRight size={14} /></button></div>
      <div className="sidebar-bottom"><button className={view === 'data' ? 'active' : ''} onClick={() => setView('data')}><Database size={17} />Veri & bağlantı</button><button onClick={() => { setView('data'); }}><CircleHelp size={17} />Nasıl çalışır?</button><div className="local-status"><span className="status-dot" /><span>Yerel çalışma alanı<small>Geliştirme önizlemesi · 0.1.0</small></span></div></div>
    </aside>
    <main className="main-area"><header className="topbar"><div className="breadcrumb">Çalışma alanı<span>/</span><strong>{view === 'data' ? 'Veri & bağlantı' : navigation.find(n => n.id === view)?.label}</strong></div><div className="topbar-right"><span className="season-label"><CalendarDays size={14} />{dashboard.season}</span><span className="user-avatar">C</span></div></header>
      <div className="page-content"><div className="page-heading"><div><span className="eyebrow">{dashboard.club} <span className="eyebrow-slash">/</span> TEKNİK EKİP</span><h1>{headings[view].title}</h1><p>{headings[view].subtitle}</p></div><button className="button primary" onClick={() => fileInput.current?.click()} disabled={busy || chatBusy || !native}><FolderInput size={16} />Kadro içe aktar</button></div>
        <div className={`source-banner ${dashboard.source === 'demo' ? '' : 'manual-banner'}`}><span className="source-banner-icon">{dashboard.source === 'demo' ? <FlaskConical size={19} /> : <Database size={19} />}</span><div><strong>{dashboard.source === 'demo' ? 'Örnek kadroyla özgürce keşfet.' : 'Manuel kadroyla çalışıyorsun.'}</strong><span>{dashboard.source === 'demo' ? 'Oyun gerektirmez. Oyuncular ve özellikleri kurgusaldır.' : 'Veriler dosyadan alındı. Oyuna canlı bağlantı bulunmuyor.'}</span></div><span className="source-badge">{dashboard.source === 'demo' ? 'ÖRNEK VERİ' : 'MANUEL VERİ'}</span></div>
        {view === 'overview' ? <>
          <div className="stat-grid"><Stat icon={<Users size={18} />} label="Kadro büyüklüğü" value={String(dashboard.playerCount)} detail="Oyuncu çalışma alanında" /><Stat icon={<Activity size={18} />} label="Maça hazır" value={String(dashboard.analysis.available)} detail="Sakat değil · kondisyon ≥ %80" /><Stat icon={<Target size={18} />} label="İlk 11 rol uyumu" value={scoreText(dashboard.analysis.score)} detail="Deneysel özellik puanı / 100" accent /><Stat icon={<Waypoints size={18} />} label="Denenebilir diziliş" value="3" detail="Kadrona göre karşılaştır" /></div>
          <div className="overview-grid"><div className="overview-left">{table}<section className="insight-section"><div className="section-heading"><div><Sparkles size={16} /><h2>Kadrodan notlar</h2><span className="small muted">Yerel analiz</span></div><button className="text-button" onClick={() => setView('assistant')}>Asistana sor <ArrowRight size={14} /></button></div><div className="insight-cards">{dashboard.analysis.insights.slice(1, 3).map((insight, i) => <article className={`insight-card ${insight.kind}`} key={i}><span className="insight-icon">{insight.kind === 'warning' ? <Activity size={18} /> : <CheckCircle2 size={18} />}</span><h3>{insight.title}</h3><p>{insight.detail}</p></article>)}</div></section></div><Pitch analysis={dashboard.analysis} onFormation={f => { void changeFormation(f); }} onPlayer={id => { void openPlayer(id); }} busy={busy || chatBusy} /></div>
        </> : null}
        {view === 'squad' ? table : null}
        {view === 'tactics' ? <div className="tactic-layout"><Pitch large analysis={dashboard.analysis} onFormation={f => { void changeFormation(f); }} onPlayer={id => { void openPlayer(id); }} busy={busy || chatBusy} /><div className="tactic-side"><section className="panel tactic-options"><span className="eyebrow">DİZİLİŞ SEÇENEKLERİ</span><h2>Kadronun dilini bul.</h2>{FORMATIONS.map(f => <button key={f.id} className={dashboard.analysis.formation === f.id ? 'selected' : ''} onClick={() => { void changeFormation(f.id); }} disabled={busy}><span className="formation-mini"><Waypoints size={24} /></span><span><strong>{f.name}</strong><small>{f.detail}</small></span>{dashboard.analysis.formation === f.id ? <CheckCircle2 size={19} /> : <ArrowUpRight size={17} />}</button>)}</section><section className="panel tactic-notes"><h2>Bu dizilişte öne çıkanlar</h2>{dashboard.analysis.insights.map((note, i) => <div key={i}><span className={`note-dot ${note.kind}`} /><div><h3>{note.title}</h3><p>{note.detail}</p></div></div>)}<button className="button secondary" onClick={() => setView('assistant')}><Sparkles size={15} />Asistanla değerlendir</button></section></div></div> : null}
        <div hidden={view !== 'assistant'}><ChatPanel source={dashboard.source} revision={revision} onBusyChange={setChatBusy} /></div>
        {view === 'data' ? <DataPanel dashboard={dashboard} onImport={() => fileInput.current?.click()} onDemo={() => { void loadDemo(); }} busy={busy || chatBusy} /> : null}
        <footer className="workspace-footer"><span><span className="status-dot" />{dashboard.source === 'demo' ? 'Kurgusal örnek veri' : 'Manuel içe aktarma'}<span className="footer-divider">·</span>Canlı oyun bağlantısı yok</span><span>Senin takımın. Senin kararın.</span></footer>
      </div>
    </main>
    <input ref={fileInput} className="sr-only" type="file" accept=".json,application/json" aria-label="Kadro JSON dosyası seç" onChange={e => { void importFile(e.target.files?.[0]); e.target.value = ''; }} />
    <PlayerDrawer detail={detail} onClose={closePlayer} />
    {toast ? <div className="toast" role="status"><span>{toast}</span><button className="icon-button" aria-label="Bildirimi kapat" onClick={() => setToast('')}><X size={16} /></button></div> : null}
  </div>;
}

function Stat({ icon, label, value, detail, accent }: { icon: React.ReactNode; label: string; value: string; detail: string; accent?: boolean }) {
  return <article className={`stat-card ${accent ? 'accent' : ''}`}><div className="stat-top"><span>{label}</span><span className="stat-icon">{icon}</span></div><strong>{value}</strong><span className="stat-detail">{detail}</span></article>;
}
