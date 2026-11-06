import { CheckCircle2, Database, FileJson, FolderInput, FlaskConical, LockKeyhole, Monitor, RefreshCw } from 'lucide-react';
import { native } from '../api';
import type { Dashboard } from '../types';

export default function DataPanel({ dashboard, onImport, onDemo, busy }: { dashboard: Dashboard; onImport: () => void; onDemo: () => void; busy: boolean }) {
  return <div className="data-grid"><section className="panel data-panel"><div className="panel-heading"><div><span className="eyebrow">ÇALIŞMA ALANIN</span><h2>Verinin kaynağını bil.</h2></div><Database size={21} className="lime" /></div>
    <div className="source-card"><span className="source-icon"><FlaskConical size={27} /></span><div><strong>{dashboard.source === 'demo' ? 'Kurgusal örnek kadro' : 'Manuel içe aktarılan kadro'}</strong><p>{dashboard.club} · {dashboard.playerCount} oyuncu · {dashboard.season}</p><span className="source-badge">{dashboard.source === 'demo' ? 'ÖRNEK VERİ' : 'MANUEL VERİ'}</span></div></div>
    <dl className="data-details"><div><dt>Kaynak tarihi</dt><dd>{new Date(dashboard.capturedAt).toLocaleString('tr-TR')}</dd></div><div><dt>Oyuna canlı bağlantı</dt><dd>Henüz eklenmedi</dd></div><div><dt>Veri saklama</dt><dd>{native ? 'Mac’inde, yerel SQLite' : 'Tarayıcı önizlemesi'}</dd></div><div><dt>Özellik ölçeği</dt><dd>1–20 · Bilinmeyen değerler boş</dd></div></dl>
    <div className="data-actions"><button className="button primary" onClick={onImport} disabled={busy || !native}><FolderInput size={16} />Kadro içe aktar</button><button className="button secondary" onClick={onDemo} disabled={busy}><RefreshCw size={16} />Örnek kadroyu yükle</button></div>
  </section><section className="panel data-panel"><div className="panel-heading"><div><span className="eyebrow">OYUN ÖNCESİ HAZIRLIK</span><h2>Şimdiden başlayabilirsin.</h2></div><Monitor size={21} className="muted" /></div>
    <div className="feature-item"><CheckCircle2 /><div><h3>Kadro ve rol analizi</h3><p>Üç dizilişi karşılaştır, oyuncu özelliklerini ve rol eşleşmelerini incele.</p></div></div>
    <div className="feature-item"><FileJson /><div><h3>Kendi kadro dosyan</h3><p>Uygulamanın JSON şemasında hazırlanmış bir kadroyu içe aktarabilirsin. En fazla 4 MB / 5.000 oyuncu. Bu dosya FM kayıt dosyası değildir.</p></div></div>
    <div className="feature-item"><LockKeyhole /><div><h3>Hesap bilgilerin CLI’da kalır</h3><p>ChatGPT girişini resmi Codex CLI yönetir. Uygulama API anahtarı istemez ve oyun dosyalarını değiştirmez.</p></div></div>
    <div className="method-card"><strong>Puanları nasıl okumalıyım?</strong><p>Rol uyumu, bilinen özelliklerin ağırlıklı ortalamasıdır. Gerekli verinin en az %70’i yoksa puan üretilmez. İlk 11 seçiminde mevki, sakatlık ve kondisyon ayrıca değerlendirilir. FM maç motorunun resmi formülü kullanılmaz.</p></div>
  </section></div>;
}
