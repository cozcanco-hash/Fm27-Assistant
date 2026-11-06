import { useEffect, useState } from 'react';
import { X } from 'lucide-react';
import { api } from '../api';
import { ATTRIBUTE_LABELS, HIDDEN_LABELS, POSITION_LABELS, scoreText } from '../types';
import type { PlayerDetail, RoleOption } from '../types';

export default function ComparisonPanel({ ids, roles, onRemove, onClear }: { ids: number[]; roles: RoleOption[]; onRemove: (id: number) => void; onClear: () => void }) {
  const [players, setPlayers] = useState<PlayerDetail[]>([]);
  const [roleId, setRoleId] = useState('');
  const [hidden, setHidden] = useState(false);
  const [error, setError] = useState('');
  useEffect(() => {
    let active = true;
    setPlayers([]); setError('');
    void Promise.all(ids.map(id => api.player(id))).then(result => { if (active) setPlayers(result); }).catch(e => { if (active) setError(String(e)); });
    return () => { active = false; };
  }, [ids]);
  if (!ids.length) return null;
  const selectedRole = roles.find(r => r.id === roleId);
  return <section className="panel comparison-panel" aria-label="Oyuncu karşılaştırma"><div className="panel-heading"><div><span className="eyebrow">YAN YANA DEĞERLENDİR</span><h2>Seçeneklerini aynı yerde gör.</h2></div><button className="text-button" onClick={onClear}>Karşılaştırmayı temizle</button></div>
    <div className="comparison-controls"><label>Aynı rolü karşılaştır<select aria-label="Karşılaştırılacak rol" value={roleId} onChange={e => setRoleId(e.target.value)}><option value="">Rol seç</option>{roles.map(r => <option key={r.id} value={r.id}>{r.name}</option>)}</select></label><label className="check-filter"><input type="checkbox" checked={hidden} onChange={e => setHidden(e.target.checked)} />Gizli özellikleri göster</label></div>
    {error ? <p className="scout-error" role="alert">{error}</p> : !players.length ? <p className="scout-note">Oyuncular hazırlanıyor…</p> : <div className="table-scroll"><table className="comparison-table"><thead><tr><th>DEĞERLENDİRME</th>{players.map(d => <th key={d.player.id}>{d.player.name}<button className="icon-button" aria-label={`${d.player.name} karşılaştırmadan çıkar`} onClick={() => onRemove(d.player.id)}><X size={14} /></button></th>)}</tr></thead><tbody>
      <tr><th>Yaş / mevki</th>{players.map(d => <td key={d.player.id}>{d.player.age} · {d.player.positions.map(p => POSITION_LABELS[p]).join(' / ')}</td>)}</tr>
      <tr><th>Kondisyon</th>{players.map(d => <td key={d.player.id}>{d.player.condition}%{d.player.injured ? ' · Sakat' : ''}</td>)}</tr>
      <tr><th>Form</th>{players.map(d => <td key={d.player.id}>{d.player.form?.toFixed(2) ?? 'Bilinmiyor'}</td>)}</tr>
      <tr><th>{selectedRole?.name ?? 'Rol uyumu'}</th>{players.map(d => { const fit = d.roles.find(r => r.roleId === roleId); return <td key={d.player.id}>{!roleId ? 'Rol seç' : !fit ? 'Mevki uygun değil' : fit.score == null ? 'Veri eksik' : `${scoreText(fit.score)} / 100`} {fit ? <small className="muted">· %{fit.coverage} veri</small> : null}</td>; })}</tr>
      {hidden ? <><tr><th>Mevcut yetenek (CA)</th>{players.map(d => <td key={d.player.id}>{d.player.currentAbility ?? 'Bilinmiyor'}</td>)}</tr><tr><th>Potansiyel yetenek (PA)</th>{players.map(d => <td key={d.player.id}>{d.player.potentialAbility ?? 'Bilinmiyor'}</td>)}</tr>{Object.entries(HIDDEN_LABELS).map(([key, label]) => <tr key={key}><th>{label}</th>{players.map(d => <td key={d.player.id}>{d.player.hiddenAttributes?.[key] ?? '—'}</td>)}</tr>)}</> : null}
      {Object.entries(ATTRIBUTE_LABELS).map(([key, label]) => <tr key={key}><th>{label}</th>{players.map(d => <td key={d.player.id}>{d.player.attributes[key] ?? '—'}</td>)}</tr>)}
    </tbody></table></div>}
    <p className="scout-note">{ids.length < 2 ? 'Karşılaştırmak için bir oyuncu daha seç. ' : ''}En fazla üç oyuncu. Rol puanları mevcut hesaplamadan gelir; eksik değerler tamamlanmaz.</p>
  </section>;
}
