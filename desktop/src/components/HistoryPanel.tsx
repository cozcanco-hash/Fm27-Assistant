import { useEffect, useState } from 'react';
import { api } from '../api';
import { ATTRIBUTE_LABELS, gameDateText } from '../types';
import type { PlayerHistory } from '../types';

export default function HistoryPanel({ id, showHidden }: { id: number; showHidden: boolean }) {
  const [history, setHistory] = useState<PlayerHistory | null>(null);
  const [error, setError] = useState('');
  const [attribute, setAttribute] = useState('passing');
  useEffect(() => {
    let active = true; setHistory(null); setError('');
    void api.history(id).then(result => { if (active) setHistory(result); }).catch(e => { if (active) setError(String(e)); });
    return () => { active = false; };
  }, [id]);
  const points = history?.points ?? [];
  const first = points[0]?.attributes[attribute], last = points.at(-1)?.attributes[attribute];
  const delta = points.length >= 2 && first != null && last != null ? last - first : null;
  return <section className="history-panel" aria-label="Oyuncu gelişimi"><h3>Gelişimini izle</h3>{error ? <p role="alert" className="scout-error">{error}</p> : !history ? <p className="muted">Geçmiş hazırlanıyor…</p> : <><p className="small muted">{history.message}</p>{points.length ? <><div className="history-controls"><label>İzlenen özellik<select aria-label="Gelişimde izlenen özellik" value={attribute} onChange={e => setAttribute(e.target.value)}>{Object.entries(ATTRIBUTE_LABELS).map(([key, label]) => <option key={key} value={key}>{label}</option>)}</select></label>{delta != null ? <span className="history-change">İlk kayıttan bu yana <strong className={delta > 0 ? 'lime' : delta < 0 ? 'amber' : ''}>{delta > 0 ? '+' : ''}{delta}</strong></span> : null}</div><div className="table-scroll"><table><thead><tr><th>OYUN TARİHİ</th><th>{ATTRIBUTE_LABELS[attribute]}</th>{showHidden ? <><th>CA</th><th>PA</th></> : null}</tr></thead><tbody>{points.map(point => <tr key={point.gameDate}><td>{gameDateText(point.gameDate)}</td><td>{point.attributes[attribute] ?? '—'}</td>{showHidden ? <><td>{point.currentAbility ?? '—'}</td><td>{point.potentialAbility ?? '—'}</td></> : null}</tr>)}</tbody></table></div></> : null}</>}</section>;
}
