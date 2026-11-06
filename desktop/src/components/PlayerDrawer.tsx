import { useEffect, useRef } from 'react';
import { Activity, ShieldCheck, X } from 'lucide-react';
import { ATTRIBUTE_LABELS, initials, POSITION_LABELS, scoreText } from '../types';
import type { PlayerDetail } from '../types';

interface Props { detail: PlayerDetail | null; onClose: () => void }
const groups = [
  { name: 'Teknik', keys: ['passing', 'firstTouch', 'technique', 'dribbling', 'finishing', 'crossing', 'tackling', 'marking', 'heading'] },
  { name: 'Zihinsel', keys: ['positioning', 'anticipation', 'decisions', 'vision', 'offTheBall', 'teamwork', 'workRate', 'composure', 'concentration'] },
  { name: 'Fiziksel', keys: ['acceleration', 'pace', 'stamina', 'strength', 'agility', 'jumpingReach'] },
];

export default function PlayerDrawer({ detail, onClose }: Props) {
  const panel = useRef<HTMLElement>(null);
  const close = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    if (!detail) return;
    const previous = document.activeElement as HTMLElement | null;
    close.current?.focus();
    const key = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
      if (e.key === 'Tab') {
        const elements = panel.current?.querySelectorAll<HTMLElement>('button, [href], input, select, [tabindex="0"]');
        if (!elements?.length) return;
        const first = elements[0], last = elements[elements.length - 1];
        if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last.focus(); }
        if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first.focus(); }
      }
    };
    document.addEventListener('keydown', key);
    return () => { document.removeEventListener('keydown', key); previous?.focus(); };
  }, [detail, onClose]);
  if (!detail) return null;
  const { player, roles } = detail;
  const actualGroups = player.positions.includes('GK') ? [...groups, { name: 'Kalecilik', keys: ['reflexes', 'handling', 'aerialReach', 'oneOnOnes', 'kicking'] }] : groups;
  return <div className="drawer-backdrop" onClick={onClose}><aside className="player-drawer" ref={panel} role="dialog" aria-modal="true" aria-labelledby="player-heading" onClick={e => e.stopPropagation()}>
    <div className="drawer-top"><span className="eyebrow">OYUNCU PROFİLİ</span><button ref={close} className="icon-button" onClick={onClose} aria-label="Profili kapat"><X size={20} /></button></div>
    <div className="profile-identity"><span className={`avatar avatar-large avatar-${player.id % 5}`}>{initials(player.name)}</span><div><h2 id="player-heading">{player.name}</h2><p>{player.age} yaş <span>·</span> {player.positions.map(p => POSITION_LABELS[p]).join(' / ')} <span>·</span> {player.foot === 'left' ? 'Sol' : player.foot === 'right' ? 'Sağ' : 'İki'} ayak</p></div></div>
    <div className="profile-stats"><div><Activity size={15} /><span>Kondisyon</span><strong>{player.condition}%</strong></div><div><ShieldCheck size={15} /><span>Form</span><strong>{player.form?.toFixed(2) ?? 'Bilinmiyor'}</strong></div><div><span>Moral</span><strong>{player.morale}/20</strong></div><div><span>Süre</span><strong>{player.minutes} dk</strong></div></div>
    <h3>Rol uyumu</h3><div className="role-fits">{roles.length ? roles.map(role => <div key={role.roleId}><div className="role-fit-label"><span>{role.roleName}</span><strong>{scoreText(role.score)}<small>/100</small></strong></div><div className="fit-track"><span style={{ width: `${role.score ?? 0}%` }} /></div><small className="muted">Veri kapsamı %{role.coverage}{role.missing.length ? ` · Eksik: ${role.missing.map(k => ATTRIBUTE_LABELS[k]).join(', ')}` : ' · Tüm gerekli özellikler mevcut'}</small></div>) : <p className="muted">Bu mevkiler için rol tanımı bulunmuyor.</p>}</div>
    <div className="attribute-groups">{actualGroups.map(group => <section key={group.name}><h3>{group.name}</h3>{group.keys.map(key => <div className="attribute" key={key}><span>{ATTRIBUTE_LABELS[key]}</span><div><span style={{ width: `${(player.attributes[key] ?? 0) * 5}%` }} /></div><strong>{player.attributes[key] ?? '—'}</strong></div>)}</section>)}</div>
    <p className="drawer-disclaimer">Rol puanı, bu uygulamanın açıklanabilir özellik ağırlıklarından hesaplanır. Mevki uygunluğu kontrol edilir; form ve kondisyon ayrı gösterilir. Eksik özellikler sıfır kabul edilmez.</p>
  </aside></div>;
}
