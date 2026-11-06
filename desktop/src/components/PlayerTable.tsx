import { ArrowUpRight, Bookmark, ChevronLeft, ChevronRight, Search, SlidersHorizontal } from 'lucide-react';
import { native } from '../api';
import { initials, POSITION_LABELS, scoreText } from '../types';
import type { PlayerPage, Position } from '../types';

interface Props { page: PlayerPage | null; search: string; position: string; onSearch: (s: string) => void; onPosition: (s: string) => void; onPlayer: (id: number) => void; onPage: (offset: number) => void; loading: boolean; compact?: boolean; compared: number[]; onCompare: (id: number) => void; shortlisted: number[]; onShortlist: (id: number) => void; listBusy: boolean }

export default function PlayerTable({ page, search, position, onSearch, onPosition, onPlayer, onPage, loading, compact, compared, onCompare, shortlisted, onShortlist, listBusy }: Props) {
  return <section className="panel squad-panel" aria-label="Oyuncu listesi">
    <div className="panel-heading"><div><span className="eyebrow">KADRO RAPORU</span><h2>Her oyuncunun bir rolü var.</h2></div><span className="count-chip">{page?.total ?? '—'} oyuncu</span></div>
    <div className="table-controls"><label className="search-field"><Search size={16} /><input aria-label="Oyuncu ara" value={search} placeholder="Oyuncu ara…" onChange={e => onSearch(e.target.value)} /></label>
      <label className="position-filter"><SlidersHorizontal size={14} /><select aria-label="Mevkiye göre filtrele" value={position} onChange={e => onPosition(e.target.value)}><option value="">Tüm mevkiler</option>{Object.entries(POSITION_LABELS).map(([id, name]) => <option key={id} value={id}>{name}</option>)}</select></label>
    </div>
    <div className="table-scroll" aria-busy={loading}>
      <table><thead><tr>{!compact ? <th>SEÇ</th> : null}<th>OYUNCU</th><th>ROL / MEVKİ</th><th className="numeric">KOND.</th><th className="numeric">FORM</th><th className="numeric">UYUM</th><th>TAKİP</th><th><span className="sr-only">Ayrıntı</span></th></tr></thead>
        <tbody>{page?.players.slice(0, compact ? 7 : 30).map(p => <tr key={p.id} className={p.injured ? 'injured-row' : ''}>
          {!compact ? <td><input type="checkbox" aria-label={`${p.name} karşılaştır`} checked={compared.includes(p.id)} disabled={!compared.includes(p.id) && compared.length >= 3} onChange={() => onCompare(p.id)} /></td> : null}
          <td><button className="player-name-button" onClick={() => onPlayer(p.id)}><span className={`avatar avatar-${p.id % 5}`}>{initials(p.name)}</span><span><strong>{p.name}</strong><small>{p.age} yaş {p.injured ? '· Sakat' : ''}</small></span></button></td>
          <td><span className="role-name">{p.role?.roleName ?? 'Veri eksik'}</span><small className="muted position-list">{p.positions.map((pos: Position) => POSITION_LABELS[pos]).join(' / ')}</small></td>
          <td className="numeric"><span className={`condition-value ${p.condition < 80 ? 'amber' : ''}`}>{p.condition}%</span><span className="condition-track"><span style={{ width: `${p.condition}%` }} /></span></td>
          <td className="numeric"><span className={`form-pill ${p.form != null && p.form >= 7.2 ? 'strong-form' : ''}`}>{p.form?.toFixed(2) ?? '—'}</span></td>
          <td className="numeric"><strong className={`fit-score ${(p.role?.score ?? 0) >= 80 ? 'lime' : ''}`}>{scoreText(p.role?.score)}</strong></td>
          <td><button className={`icon-button ${shortlisted.includes(p.id) ? 'is-saved' : ''}`} aria-label={`${p.name} ${shortlisted.includes(p.id) ? 'listeden çıkar' : 'listeye ekle'}`} aria-pressed={shortlisted.includes(p.id)} disabled={!native || listBusy} onClick={() => onShortlist(p.id)}><Bookmark size={15} fill={shortlisted.includes(p.id) ? 'currentColor' : 'none'} /></button></td>
          <td><button className="icon-button row-open" aria-label={`${p.name} ayrıntıları`} onClick={() => onPlayer(p.id)}><ArrowUpRight size={15} /></button></td>
        </tr>)}</tbody>
      </table>
      {page?.total === 0 ? <div className="empty-state"><Search size={23} /><strong>Bu filtrede oyuncu yok.</strong><span>Başka bir isim veya mevki deneyebilirsin.</span></div> : null}
    </div>
    <div className="table-footer"><span className="small muted">{compact ? 'Kadrodan seçilen ilk oyuncular' : `${(page?.offset ?? 0) + (page?.total ? 1 : 0)}–${Math.min((page?.offset ?? 0) + 30, page?.total ?? 0)} / ${page?.total ?? 0}`}</span>
      {!compact && page ? <div className="pagination"><button className="icon-button" aria-label="Önceki sayfa" disabled={page.offset === 0} onClick={() => onPage(Math.max(0, page.offset - 30))}><ChevronLeft size={16} /></button><button className="icon-button" aria-label="Sonraki sayfa" disabled={page.offset + 30 >= page.total} onClick={() => onPage(page.offset + 30)}><ChevronRight size={16} /></button></div> : <span className="small muted">Puanlar 100 üzerinden</span>}
    </div>
  </section>;
}
