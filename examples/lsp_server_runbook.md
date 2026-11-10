# Runbook Operasional `lsp_server` (LSP Nusacoin)

Dokumen ini mencatat cara menjalankan dan merestart server LSP
(`examples/lsp_server.rs`) di VPS produksi, supaya operator mana pun
dapat menjalankannya tanpa bergantung pada ingatan satu orang.

Satu binary melayani **dua instance** yang berjalan berdampingan di
satu VPS. Keduanya node LDK mandiri dengan direktori data, seed, dan
port yang terpisah — jangan pernah tertukar.

| | Testnet | Mainnet |
|---|---|---|
| Direktori | `/root/lsp-test` | `/root/lsp-mainnet` |
| Listen P2P | `0.0.0.0:9738` | `0.0.0.0:9739` |
| Electrum | `127.0.0.1:60009` | `127.0.0.1:50009` |
| Seed | `/root/lsp-test/seed.bin` | `/root/lsp-mainnet/seed.bin` |
| File stop | `/tmp/stop-lsp` (bawaan) | `/tmp/stop-lsp-mainnet` (via `LSP_STOP_FILE`) |
| Alias bawaan | `NUX-LSP-testnet` | `NUX-LSP-mainnet` |
| node_id diawali | `02b69d63…` | `02a3c442…` |

Seed dan isi direktori data adalah **aset yang tidak tergantikan**:
jangan dipindah, jangan dihapus, dan selalu ada backup terverifikasi
sebelum operasi berisiko.

## Perintah start

### Testnet

Instance testnet berjalan hanya dengan empat variabel env; sisanya
memakai bawaan kode (jaringan testnet, token, alias).

```sh
cd /root/lsp-test
rm -f /tmp/stop-lsp
LSP_SEED_PATH=/root/lsp-test/seed.bin LSP_STORAGE=/root/lsp-test LSP_LISTEN=0.0.0.0:9738 LSP_ELECTRUM=127.0.0.1:60009 nohup ./lsp_server >> server.log 2>&1 &
```

### Mainnet

Struktur sama; nilai rahasia (`LSP_TOKEN`, `LSP_PROMISE_SECRET`, dan
keluarga `LSP_PAY_*` bila mode bayar aktif) diisi dari catatan operator
— **jangan tulis nilai rahasia ke dokumen ini**.

```sh
cd /root/lsp-mainnet
rm -f /tmp/stop-lsp-mainnet
LSP_NETWORK=mainnet LSP_LISTEN=0.0.0.0:9739 LSP_ELECTRUM=127.0.0.1:50009 LSP_STORAGE=/root/lsp-mainnet LSP_SEED_PATH=/root/lsp-mainnet/seed.bin LSP_STOP_FILE=/tmp/stop-lsp-mainnet LSP_ANNOUNCE_ADDR=163.245.193.102:9739 LSP_TOKEN=<dari catatan operator> LSP_PROMISE_SECRET=<dari catatan operator> nohup ./lsp_server >> server.log 2>&1 &
```

Mode bayar (TopUP) menambahkan `LSP_PAY_FILE`, `LSP_PAY_MAX_NUSAN`
(wajib bila mode bayar aktif), dan `LSP_PAY_ONLY_NODE` sesuai
konfigurasi yang sedang berlaku di instance.

### Aturan perintah start

- `rm -f <file stop>` **wajib** sebelum start: bila file sinyal stop
  masih ada (sisa penghentian sebelumnya), server baru akan langsung
  mati sendiri.
- **Jangan** sertakan variabel sekali-pakai dalam perintah start
  tetap — terutama `LSP_PEER_OPEN` dan `LSP_PEER_OPEN_ANNOUNCED`.
  Keduanya adalah perintah satu kali (buka channel saat start); bila
  terbawa ke start berikutnya, server akan mencoba membuka channel
  lagi.
- `LSP_MAX_FUNDING_NUSAN` tidak perlu diset: tanpa env ini batas
  funding channel masuk adalah tanpa plafon (lihat bagian
  Verifikasi). Isi hanya bila operator sengaja memasang plafon.

## Menghentikan server

Dua cara yang sah, pilih satu:

1. Graceful: `touch <file stop instance>` — server berhenti sendiri
   dengan bersih. Ingat file ini harus dihapus sebelum start ulang.
2. Sinyal: `kill <PID>` setelah PID diverifikasi (lihat prosedur
   restart).

Jangan menghentikan proses hanya berdasar nomor PID dari ingatan:
selalu cocokkan PID → port → direktori kerja terlebih dahulu.

## Prosedur restart / tukar binary (pola terverifikasi)

1. **Identifikasi** proses instance yang dituju:
   `ss -ltnp | grep <port>` → catat PID; pastikan
   `readlink /proc/<PID>/cwd` menunjuk direktori instance yang benar
   dan `readlink /proc/<PID>/exe` menunjuk binary yang akan ditukar.
2. **Unduh & verifikasi** binary baru di samping yang lama sebagai
   `lsp_server.new`; cocokkan ukuran dan `sha256sum` dengan nilai
   yang diumumkan untuk build tersebut. Jangan lanjut bila beda.
3. **Gerbang gema**: jalankan
   `echo "stop <PID> di <direktori>, tukar binary, start ulang"` dan
   baca kembali teksnya — pastikan instance dan PID yang tertulis
   memang yang dimaksud sebelum langkah destruktif.
4. **Stop** proses (salah satu cara di atas), tunggu sampai port
   kosong: `ss -ltnp | grep <port>` tidak menghasilkan apa-apa.
5. **Tukar** dengan backup bernama tanggal:
   `mv lsp_server lsp_server.bak-<yyyymmdd> && mv lsp_server.new lsp_server && chmod +x lsp_server`
6. **Start** dengan perintah start instance di atas — persis env
   tetapnya, tanpa variabel sekali-pakai.
7. **Verifikasi log**: `tail -15 server.log` (lihat bagian berikut).

Selama jendela restart, dompet ELNUX jaringan terkait tidak bisa
mengirim/menerima Lightning (LSP adalah peer-nya); aplikasi tetap
terbuka, saldo dan riwayat aman, channel dan dana tidak tersentuh
oleh pergantian binary.

## Verifikasi sesudah start

Baris-baris berikut harus muncul di `server.log` instance:

- `[lsp] network=<testnet|mainnet> electrum=<...> listen=<...> storage=<direktori instance>`
- `[lsp] memakai seed file: <path seed instance>`
- `[lsp] batas funding channel masuk: 18446744073709551615 nusan`
  (nilai `u64::MAX` = tanpa plafon; angka lain = plafon dari
  `LSP_MAX_FUNDING_NUSAN`)
- `[lsp] node_id=<node_id instance>` — harus **tetap sama** dengan
  sebelum restart. Node_id berubah berarti seed/direktori salah:
  berhenti dan periksa sebelum melayani apa pun.

## Referensi

- Seluruh variabel `LSP_*` beserta bawaan dan artinya didokumentasikan
  di komentar kepala `examples/lsp_server.rs` — sumber kebenaran untuk
  perilaku binary.
- Binary rilis diambil dari release `lsp-server-release` repo ini.
  Bila GitHub Actions repo nonaktif, binary dibangun lokal dari branch
  `nux-spike` (`cargo build --release --example lsp_server`, lalu
  `strip`) dan diunggah sebagai aset release; umumkan ukuran +
  sha256-nya bersama build.
