export type Position = 'GK' | 'DC' | 'DL' | 'DR' | 'DM' | 'MC' | 'AML' | 'AMR' | 'AMC' | 'ST';
export type Formation = '433dm' | '4231' | '442';
export type Source = 'demo' | 'manual';
export interface RoleFit { roleId: string; roleName: string; score: number | null; coverage: number; missing: string[]; strengths: string[] }
export interface PlayerRow { id: number; name: string; age: number; positions: Position[]; condition: number; form: number | null; injured: boolean; role: RoleFit | null; currentAbility?: number | null; potentialAbility?: number | null }
export interface Player extends Omit<PlayerRow, 'role'> { foot: 'left' | 'right' | 'both'; morale: number; minutes: number; attributes: Record<string, number>; hiddenAttributes?: Record<string, number> }
export interface PlayerDetail { player: Player; roles: RoleFit[] }
export interface Slot { key: string; label: string; x: number; y: number; roleId: string; player: PlayerRow | null; fit: RoleFit | null }
export interface Insight { kind: 'info' | 'positive' | 'warning'; title: string; detail: string }
export interface Analysis { formation: Formation; slots: Slot[]; score: number | null; filled: number; available: number; insights: Insight[] }
export interface RoleOption { id: string; name: string; position: Position }
export interface Dashboard { club: string; season: string; source: Source; capturedAt: string; playerCount: number; analysis: Analysis; gameDate: string | null; roles: RoleOption[] }
export interface PlayerPage { players: PlayerRow[]; total: number; offset: number }
export interface AttributeFilter { key: string; min?: number; max?: number; hidden: boolean }
export interface PlayerQuery { search?: string; position?: Position | null; offset?: number; limit?: number; minAge?: number; maxAge?: number; minCondition?: number; minForm?: number; minCa?: number; minPa?: number; foot?: Player['foot']; availableOnly?: boolean; roleId?: string; minRoleScore?: number; attributes?: AttributeFilter[]; sort?: 'name' | 'age' | 'ability' | 'potential' | 'condition' | 'form' | 'role'; descending?: boolean; listId?: number }
export interface ShortlistEntry { playerId: number; name: string; available: boolean }
export interface Shortlist { id: number; name: string; entries: ShortlistEntry[] }
export interface HistoryPoint { gameDate: string; capturedAt: string; currentAbility: number | null; potentialAbility: number | null; attributes: Record<string, number> }
export interface PlayerHistory { points: HistoryPoint[]; message: string }
export interface ModelChoice { id: string; name: string; isDefault: boolean }
export interface CodexStatus { state: string; message: string; plan: string | null; models: ModelChoice[] }
export interface ChatMessage { role: 'user' | 'assistant'; text: string }
export type ChatEvent = { type: 'delta'; text: string } | { type: 'complete'; text: string } | { type: 'interrupted' };

export const FORMATIONS: { id: Formation; name: string; detail: string }[] = [
  { id: '433dm', name: '4–3–3 DM', detail: 'Savunma önünde güvence, genişlik ve iki yönlü orta saha.' },
  { id: '4231', name: '4–2–3–1', detail: 'Merkezde yaratıcılık, tek forvetin arkasında destek.' },
  { id: '442', name: '4–4–2', detail: 'Çift forvet, iki merkez oyuncusu ve kanat genişliği.' },
];
export const POSITION_LABELS: Record<Position, string> = { GK: 'KL', DC: 'STP', DL: 'SLB', DR: 'SĞB', DM: 'DM', MC: 'MO', AML: 'SLK', AMR: 'SĞK', AMC: 'OMO', ST: 'FV' };
export const ATTRIBUTE_LABELS: Record<string, string> = {
  acceleration: 'Hızlanma', pace: 'Hız', stamina: 'Dayanıklılık', strength: 'Güç', agility: 'Çeviklik', jumpingReach: 'Zıplama',
  passing: 'Pas', firstTouch: 'İlk kontrol', technique: 'Teknik', dribbling: 'Top sürme', finishing: 'Bitiricilik', crossing: 'Orta',
  tackling: 'Top kapma', marking: 'Markaj', heading: 'Kafa', positioning: 'Pozisyon alma', anticipation: 'Önsezi', decisions: 'Karar',
  vision: 'Vizyon', offTheBall: 'Topsuz alan', teamwork: 'Takım oyunu', workRate: 'Çalışkanlık', composure: 'Soğukkanlılık', concentration: 'Konsantrasyon',
  reflexes: 'Refleks', handling: 'Elle kontrol', aerialReach: 'Hava hakimiyeti', oneOnOnes: 'Bire bir', kicking: 'Ayakla dağıtım',
};
export const initials = (name: string) => name.split(' ').map(v => v[0]).slice(0, 2).join('');
export const scoreText = (score: number | null | undefined) => score == null ? '—' : Math.round(score).toString();
export const HIDDEN_LABELS: Record<string, string> = { consistency: 'İstikrar', importantMatches: 'Önemli maçlar', injuryProneness: 'Sakatlık yatkınlığı', versatility: 'Çok yönlülük', professionalism: 'Profesyonellik', ambition: 'Hırs', pressure: 'Baskıyla başa çıkma', temperament: 'Mizaç', loyalty: 'Sadakat', adaptability: 'Uyum sağlama', sportsmanship: 'Sportmenlik', controversy: 'Tartışma eğilimi', dirtiness: 'Sertlik' };
export const gameDateText = (date: string) => new Intl.DateTimeFormat('tr-TR', { timeZone: 'UTC' }).format(new Date(`${date}T00:00:00Z`));

export interface SharedMessage extends ChatMessage { id: number; state: 'complete' | 'pending' | 'failed' | 'interrupted'; client: 'desktop' | 'game' }
export interface Conversation { scope: string; club: string; source: Source; capturedAt: string; model: string | null; messages: SharedMessage[]; busy: boolean; live: string; activeTurnId: number | null; models: ModelChoice[]; error: string | null }
export interface GameLinkStatus { enabled: boolean; available: boolean; message: string }
