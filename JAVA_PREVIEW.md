# Önceki Java macOS önizlemesi

Can Özcan için özel macOS uyarlaması. Football Manager 27 taktik dosyalarını ve
ChatGPT hesabıyla çalışan yerel Codex asistanını Mac üzerinde kullanmayı hedefler.

**Durum: geliştirme önizlemesi.** Apple Silicon paketleme, macOS klasörleri ve
Codex keşfi eklendi. Mac üzerinde canlı kadro/oyuncu verisi okuma henüz uygulanmadı;
bu nedenle oyuncu formuna ve özelliklerine dayanan tam asistan henüz hazır değil.

## Başlangıç

- [Apple Silicon önizleme paketini indir](https://github.com/cozcanco-hash/Fm27-Assistant/actions/runs/42161326949/artifacts/13477226606) — 20 Kasım 2026'ya kadar saklanır.
- [macOS kurulum, derleme ve mevcut sınırlar](MACOS.md)
- [Apple Silicon derlemeleri ve test sonuçları](https://github.com/cozcanco-hash/Fm27-Assistant/actions/workflows/macos-preview.yml)
- [Orijinal proje dokümantasyonu](README_UPSTREAM.md)

Uygulama yerel bir `.app` başlatıcısı ve tarayıcı arayüzü kullanır. Java çalışma
ortamı pakete dahildir. Codex CLI ve ChatGPT hesabıyla giriş kullanıcının Mac'inde
kurulur; abonelik sınırları geçerlidir.

## Doğrulama — 6 Kasım 2026

Apple Silicon üzerinde tam derleme ve paket açılış kontrolü başarılı:
**403 test geçti**, yerel oyun/hesap/örnek dosya gerektiren **9 test atlandı**.
Test kapsamı eşikleri sağlandı; paketlenmiş uygulama kendi Java ortamıyla başladı,
sağlık kontrolü `UP`, ana sayfa yanıtı `HTTP 200` oldu.
[Başarılı çalıştırma ve raporlar](https://github.com/cozcanco-hash/Fm27-Assistant/actions/runs/42161326949).

Bu doğrulama, senin Mac'inde Finder'dan açılış, ChatGPT girişi veya gerçek FM27
kadrosuyla yapılmış bir test değildir. Paket imzasız bir geliştirici önizlemesidir.

## Kaynak

Temel proje: [JelmerBouma1985/fm-ai-assistent](https://github.com/JelmerBouma1985/fm-ai-assistent),
commit `a06ac845acfd6c0eccbd61e3a633b3518584e457`.
Bu depo özel tutulur; upstream kod için yeni bir lisans beyan edilmez.
