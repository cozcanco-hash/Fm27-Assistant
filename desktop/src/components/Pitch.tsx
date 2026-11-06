import { ArrowUpRight, Info } from 'lucide-react';
import { FORMATIONS, initials, scoreText } from '../types';
import type { Analysis, Formation } from '../types';

interface Props { analysis: Analysis; onFormation: (f: Formation) => void; onPlayer: (id: number) => void; busy: boolean; large?: boolean }

export default function Pitch({ analysis, onFormation, onPlayer, busy, large }: Props) {
  return <section className={`panel pitch-panel ${large ? 'pitch-large' : ''}`} aria-label="İlk 11 önerisi">
    <div className="panel-heading"><div><span className="eyebrow">TAKTİK TAHTASI</span><h2>Birlikte daha güçlü.</h2></div><span className="status-dot" title="Yerel analiz" /></div>
    <div className="pitch-toolbar"><label className="sr-only" htmlFor="formation">Diziliş</label><select id="formation" value={analysis.formation} onChange={e => onFormation(e.target.value as Formation)} disabled={busy}>
      {FORMATIONS.map(f => <option key={f.id} value={f.id}>{f.name}</option>)}
    </select><span className="small muted">{analysis.filled}/11 pozisyon dolu</span></div>
    <div className="pitch" aria-label="Seçilen diziliş ve oyuncular">
      <div className="field-lines"><div className="halfway" /><div className="centre-circle" /><div className="penalty-area top" /><div className="penalty-area bottom" /><div className="goal-area top" /><div className="goal-area bottom" /><div className="centre-dot" /></div>
      {analysis.slots.map(slot => <button key={slot.key} className={`pitch-player ${slot.player ? '' : 'empty'}`} style={{ left: `${slot.x}%`, top: `${slot.y}%` }} onClick={() => slot.player && onPlayer(slot.player.id)} disabled={!slot.player}
        aria-label={slot.player ? `${slot.player.name}, ${slot.fit?.roleName}, uyum ${scoreText(slot.fit?.score)}` : `${slot.label}: uygun oyuncu yok`}>
        <span className="shirt"><span>{slot.player ? initials(slot.player.name) : '?'}</span><i>{scoreText(slot.fit?.score)}</i></span>
        <span className="pitch-player-name">{slot.player ? slot.player.name.split(' ').slice(-1)[0] : 'Eksik'}</span><span className="pitch-position">{slot.label}</span>
      </button>)}
    </div>
    <div className="pitch-summary"><div><span className="small muted">İlk 11 rol uyumu</span><strong>{scoreText(analysis.score)}<small>/100</small></strong></div><ArrowUpRight size={24} className="lime" /></div>
    <p className="method-note"><Info size={13} /> Özelliklere dayalı deneysel analiz. Maç sonucu tahmini değildir.</p>
  </section>;
}
