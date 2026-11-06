export type Position = 'GK' | 'DC' | 'DL' | 'DR' | 'DM' | 'MC' | 'AML' | 'AMR' | 'AMC' | 'ST';
export type Formation = '433dm' | '4231' | '442';
export type Source = 'demo' | 'manual';
export interface RoleFit { roleId: string; roleName: string; score: number | null; coverage: number; missing: string[]; strengths: string[] }
export interface PlayerRow { id: number; name: string; age: number; positions: Position[]; condition: number; form: number | null; injured: boolean; role: RoleFit | null }
export interface Player extends Omit<PlayerRow, 'role'> { foot: 'left' | 'right' | 'both'; morale: number; minutes: number; attributes: Record<string, number> }
export interface PlayerDetail { player: Player; roles: RoleFit[] }
export interface Slot { key: string; label: string; x: number; y: number; roleId: string; player: PlayerRow | null; fit: RoleFit | null }
export interface Insight { kind: 'info' | 'positive' | 'warning'; title: string; detail: string }
export interface Analysis { formation: Formation; slots: Slot[]; score: number | null; filled: number; available: number; insights: Insight[] }
export interface Dashboard { club: string; season: string; source: Source; capturedAt: string; playerCount: number; analysis: Analysis }
export interface PlayerPage { players: PlayerRow[]; total: number; offset: number }
export interface PlayerQuery { search?: string; position?: Position | null; offset?: number; limit?: number }
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
