import { invoke, isTauri } from '@tauri-apps/api/core';
import type { Conversation, GameLinkStatus, CodexStatus, Dashboard, Formation, PlayerDetail, PlayerHistory, PlayerPage, PlayerQuery, Shortlist } from './types';

export const native = isTauri();
const fixture = () => import('../fixtures/preview-state.json');
let previewFormation: Formation = '433dm';

export const api = {
  async dashboard(): Promise<Dashboard> {
    if (native) return invoke('dashboard');
    return (await fixture()).dashboards.find(d => d.analysis.formation === previewFormation) as Dashboard;
  },
  async players(query: PlayerQuery = {}): Promise<PlayerPage> {
    if (native) return invoke('list_players', { query });
    const data = await fixture();
    const all = data.page.players as PlayerPage['players'];
    const key = (s: string) => s.replace(/[Iİı]/g, 'i').toLowerCase();
    const members = query.listId ? data.shortlists.find(l => l.id === query.listId)?.entries.map(e => e.playerId) ?? [] : null;
    // This read-only preview filters Rust-produced rows and role results. The
    // native app performs queries, validation, scoring and persistence in Rust.
    const matching = all.flatMap(row => {
      const detail = data.details.find(d => d.player.id === row.id) as PlayerDetail;
      const p = detail.player;
      const role = query.roleId ? detail.roles.find(r => r.roleId === query.roleId) : row.role;
      if (!key(p.name).includes(key(query.search ?? '')) || (query.position && !p.positions.includes(query.position))
        || (query.minAge != null && p.age < query.minAge) || (query.maxAge != null && p.age > query.maxAge)
        || (query.minCondition != null && p.condition < query.minCondition) || (query.minForm != null && (p.form == null || p.form < query.minForm))
        || (query.minCa != null && (p.currentAbility == null || p.currentAbility < query.minCa)) || (query.minPa != null && (p.potentialAbility == null || p.potentialAbility < query.minPa))
        || (query.foot && p.foot !== query.foot) || (query.availableOnly && p.injured) || (members && !members.includes(p.id))
        || (query.roleId && !role) || (query.minRoleScore != null && (role?.score == null || role.score < query.minRoleScore))
        || !(query.attributes ?? []).every(f => { const v = (f.hidden ? p.hiddenAttributes : p.attributes)?.[f.key]; return v != null && (f.min == null || v >= f.min) && (f.max == null || v <= f.max); })) return [];
      return [{ ...row, role: role ?? null }];
    });
    const sortValue = (p: PlayerPage['players'][number]) => query.sort === 'age' ? p.age : query.sort === 'ability' ? p.currentAbility : query.sort === 'potential' ? p.potentialAbility : query.sort === 'condition' ? p.condition : query.sort === 'form' ? p.form : query.sort === 'role' ? p.role?.score : key(p.name);
    matching.sort((a, b) => { const x = sortValue(a), y = sortValue(b); if (x == null) return y == null ? a.id - b.id : 1; if (y == null) return -1; return (x < y ? -1 : x > y ? 1 : 0) * (query.descending ? -1 : 1) || a.id - b.id; });
    const offset = query.offset ?? 0;
    return { players: matching.slice(offset, offset + (query.limit ?? 30)), total: matching.length, offset };
  },
  async player(id: number): Promise<PlayerDetail> {
    if (native) return invoke('get_player', { id });
    const detail = (await fixture()).details.find(d => d.player.id === id);
    if (!detail) throw new Error('Oyuncu bulunamadı.');
    return detail as PlayerDetail;
  },
  async formation(formation: Formation): Promise<Dashboard> {
    if (native) return invoke('set_formation', { formation });
    previewFormation = formation;
    return api.dashboard();
  },
  async shortlists(): Promise<Shortlist[]> {
    if (native) return invoke('list_shortlists');
    return (await fixture()).shortlists;
  },
  async createList(name: string): Promise<Shortlist[]> {
    if (!native) throw new Error('Listeleri düzenlemek için Mac uygulamasını aç.');
    return invoke('create_shortlist', { name });
  },
  async deleteEmptyList(id: number): Promise<Shortlist[]> {
    if (!native) throw new Error('Listeleri düzenlemek için Mac uygulamasını aç.');
    return invoke('delete_empty_shortlist', { id });
  },
  async shortlist(listId: number, playerId: number, selected: boolean): Promise<Shortlist[]> {
    if (!native) throw new Error('Listeleri düzenlemek için Mac uygulamasını aç.');
    return invoke('set_shortlisted', { listId, playerId, selected });
  },
  async history(id: number): Promise<PlayerHistory> {
    if (native) return invoke('player_history', { id });
    return (await fixture()).histories.find(h => h.id === id)?.history ?? { points: [], message: 'Bu oyuncu için kayıt bulunamadı.' };
  },
  async importRoster(content: string): Promise<Dashboard> {
    if (!native) throw new Error('Kadro içe aktarma Mac uygulamasında kullanılabilir.');
    return invoke('import_roster', { content });
  },
  async demo(): Promise<Dashboard> {
    if (native) return invoke('load_demo');
    return api.dashboard();
  },
  async codexStatus(): Promise<CodexStatus> {
    if (native) return invoke('codex_status');
    return { state: 'unavailable', message: 'AI bağlantısı Mac uygulamasında kullanılabilir.', plan: null, models: [] };
  },
  async login(): Promise<void> {
    if (!native) throw new Error('ChatGPT girişi için Mac uygulamasını aç.');
    return invoke('codex_login');
  },
  async conversation(): Promise<Conversation> {
    if (native) return invoke('conversation');
    return { scope: 'preview', club: 'KUTRA FC', source: 'demo', capturedAt: '', model: null, messages: [], busy: false, live: '', activeTurnId: null, models: [], error: null };
  },
  async chat(scope: string, prompt: string): Promise<Conversation> {
    if (!native) throw new Error('Gerçek AI sohbeti için Mac uygulamasını aç.');
    return invoke('send_chat', { request: { scope, prompt } });
  },
  async setChatModel(scope: string, model: string | null): Promise<Conversation> {
    if (!native) throw new Error('Model seçimi Mac uygulamasında kullanılabilir.');
    return invoke('set_chat_model', { scope, model });
  },
  async stopChat(turnId: number): Promise<void> { if (native) return invoke('stop_chat', { turnId }); },
  async gameLink(): Promise<GameLinkStatus> {
    if (native) return invoke('game_link_status');
    return { enabled: false, available: false, message: 'Oyun içi bağlantı Mac uygulamasında kullanılabilir.' };
  },
  async setGameLink(enabled: boolean): Promise<GameLinkStatus> {
    if (!native) throw new Error('Oyun içi bağlantı için Mac uygulamasını aç.');
    return invoke('set_game_link', { enabled });
  },
  async ready(): Promise<void> { if (native) return invoke('frontend_ready'); },
};
