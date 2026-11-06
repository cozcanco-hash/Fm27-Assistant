import { useState } from 'react';
import { Plus, Trash2 } from 'lucide-react';
import { native } from '../api';
import type { Shortlist } from '../types';

export default function ShortlistPanel({ lists, activeId, onSelect, onCreate, onDelete, onRemove, busy }: { lists: Shortlist[]; activeId: number | null; onSelect: (id: number) => void; onCreate: (name: string) => void; onDelete: (id: number) => void; onRemove: (id: number) => void; busy: boolean }) {
  const [name, setName] = useState('');
  const list = lists.find(l => l.id === activeId);
  return <section className="panel shortlist-panel" aria-label="Takip listelerini yönet"><div className="panel-heading"><div><span className="eyebrow">KISA LİSTELER</span><h2>Aklındaki oyuncuları kaybetme.</h2></div><span className="count-chip">{list?.entries.length ?? 0} oyuncu</span></div>
    <div className="shortlist-controls"><label>Takip listesi<select aria-label="Takip listesi seç" value={activeId ?? ''} onChange={e => onSelect(Number(e.target.value))}>{lists.map(l => <option key={l.id} value={l.id}>{l.name} ({l.entries.length})</option>)}</select></label><form onSubmit={e => { e.preventDefault(); if (name.trim()) { onCreate(name); setName(''); } }}><label className="sr-only" htmlFor="list-name">Yeni liste adı</label><input id="list-name" value={name} placeholder="Yeni liste adı" maxLength={60} onChange={e => setName(e.target.value)} disabled={!native || busy} /><button className="button secondary" disabled={!native || busy || !name.trim() || lists.length >= 20}><Plus size={15} />Liste oluştur</button></form>{list && lists.length > 1 && list.entries.length === 0 ? <button className="text-button" disabled={!native || busy} onClick={() => onDelete(list.id)}><Trash2 size={14} />Boş listeyi sil</button> : null}</div>
    {!native ? <p className="scout-note">Örnek listeyi inceliyorsun. Listeler Mac uygulamasında düzenlenir ve saklanır.</p> : <p className="scout-note">Oyuncuları kadrodan veya profillerinden seçili listeye ekleyebilirsin. Listeler Mac’inde saklanır.</p>}
    {list?.entries.filter(e => !e.available).map(e => <div className="unavailable-entry" key={e.playerId}><span>{e.name}<small>Bu kadroda bulunmuyor · Liste kaydı korunuyor</small></span><button className="text-button" disabled={!native || busy} onClick={() => onRemove(e.playerId)}>Listeden çıkar</button></div>)}
  </section>;
}
