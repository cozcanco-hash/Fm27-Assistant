# FM27 Assistant

Can için özel, oyun öncesinde de kullanılabilen bir taktik çalışma alanı.
Aktif geliştirme **Rust + Tauri 2 + React + SQLite** üzerinde `desktop/` altında sürüyor.

## Rust sürümünde

- 24 kurgusal oyuncuyla hemen başlayan çalışma alanı.
- Kadro arama, mevki filtresi, oyuncu özellikleri ve açıklanabilir rol puanları.
- 4–3–3 DM, 4–2–3–1 ve 4–4–2 saha görünümü; aynı oyuncuyu iki kez seçmeyen ilk 11 analizi.
- Doğrulamalı özel JSON kadro aktarımı ve Mac'te kalıcı SQLite kaydı.
- Yerel Codex üzerinden ChatGPT girişi, model seçimi ve akış halinde AI sohbeti.

**Oyun sahibi olman gerekmiyor.** Örnek kadro tamamen kurgusal; gerçek FM27
verisi veya oyuna canlı bağlantı içermiyor. Bu ilk Rust sürümü, Java projesindeki
tüm özelliklerin taşındığı bir sürüm değildir. Gerçek hesap ve oyun testi ayrıca yapılacak.

- [Kurulum, geliştirme ve veri şeması](desktop/README.md)
- [Rust testleri ve Apple Silicon paketleri](https://github.com/cozcanco-hash/Fm27-Assistant/actions/workflows/rust-desktop.yml)
- [Kurgusal örnek kadro](desktop/fixtures/demo-squad.json)
- [Önceki Java önizlemesi ve doğrulaması](JAVA_PREVIEW.md)

```bash
cd desktop
npm ci
npm run desktop:dev
```

Geliştirme için Rust, Node.js 24 ve Xcode komut satırı araçları gerekir.
Derlenmiş Mac uygulamasında Rust, Node veya Java kurulumu gerekmez.
ChatGPT özellikleri için resmi Codex CLI ve kullanıcının kendi hesabıyla giriş gerekir.

## Kaynak geçmişi

Java referansı [JelmerBouma1985/fm-ai-assistent](https://github.com/JelmerBouma1985/fm-ai-assistent)
projesinin `a06ac845acfd6c0eccbd61e3a633b3518584e457` commitinden aktarılmıştır.
Orijinal dokümantasyon [README_UPSTREAM.md](README_UPSTREAM.md) dosyasında korunur.
Rust çalışma alanı bu depoda yeni uygulanmıştır. Depo özel tutulur; upstream kod
üzerinde yeni bir lisans hakkı beyan edilmez.
