import { Channel, invoke, isTauri } from '@tauri-apps/api/core';
import type { ChatEvent, ChatMessage, CodexStatus, Dashboard, Formation, PlayerDetail, PlayerPage, PlayerQuery } from './types';

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
    const all = (await fixture()).page.players as PlayerPage['players'];
    const matching = all.filter(p => p.name.toLocaleLowerCase('tr').includes((query.search ?? '').toLocaleLowerCase('tr')) && (!query.position || p.positions.includes(query.position)));
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
  async chat(prompt: string, history: ChatMessage[], model: string | null, onEvent: (event: ChatEvent) => void): Promise<string> {
    if (!native) throw new Error('Gerçek AI sohbeti için Mac uygulamasını aç.');
    const onEventChannel = new Channel<ChatEvent>();
    onEventChannel.onmessage = onEvent;
    return invoke('send_chat', { prompt, history, model, onEvent: onEventChannel });
  },
  async stopChat(): Promise<void> { if (native) return invoke('stop_chat'); },
  async ready(): Promise<void> { if (native) return invoke('frontend_ready'); },
};
