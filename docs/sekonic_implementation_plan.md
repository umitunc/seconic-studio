# SEKONIC YENİ NESİL OPTİK OKUMA & ÖLÇME-DEĞERLENDİRME SİSTEMİ
## Master Supervisor Uygulama Spesifikasyonu (AI Worker / Gemini Hydrogen Uyumlu)

> **Doküman Rolü:** Bu belge, `d:\Works\2026\seconic` çalışma alanındaki 5 Sekonic kurulum paketinin tersine mühendislik (reverse engineering) bulgularını, hedef sistem mimarisini, veri sözleşmelerini ve **WP-01'den WP-12'ye kadar atomik iş paketlerini** içeren **Master Supervisor Planıdır**. Daha hızlı/küçük kodlama modellerinin (örn. **Gemini Hydrogen**) hiçbir mimari detayı tahmin etmek zorunda kalmadan, halüsinasyon üretmeden ve her adımda deterministik testlerle doğrulanarak uçtan uca sistemi inşa edebilmesi için tasarlanmıştır.

---

## BÖLÜM 0: AI Uygulayıcı Ajan (Gemini Hydrogen) İçin Supervisor Çalışma Anayasası

Bu projeyi kodlayan AI ajanı (Worker), aşağıdaki **7 Kesin Kurala** kayıtsız şartsız uymak zorundadır:

1. **Sıfır Tahmin / Sıfır Halüsinasyon Kuralı:**
   - Asla USB `VID`/`PID`, `GUID`, protokol komutu, byte çerçevesi (`STX`/`ETX`) veya DLL fonksiyon imzası uydurma.
   - Tüm sabitleri **Bölüm 1 (Doğrulanmış Donanım ve İkili Referans El Kitabı)** tablosundan ve **WP-02**'de üretilen `docs/wire-spec.md` çıktısından al.
2. **Tek Seferde Tek İş Paketi (Atomic WP Execution):**
   - İş paketlerini sırasıyla (`WP-01` -> `WP-02` -> ... -> `WP-12`) uygula.
   - Mevcut İş Paketinin sonundaki **"Zorunlu Doğrulama Kapısı (Verification Gate)"** komutu çalıştırılıp `exit code 0` alınmadan **kesinlikle** bir sonraki İş Paketine geçme.
3. **Kalıcı İlerleme Defteri (`PROGRESS.md`):**
   - Kök dizinde `PROGRESS.md` dosyasını oluştur ve her WP tamamlandığında durumu (`[x]`), değiştirilen dosyaları ve test çıktısı özetini kaydet. Yeni bir oturum başladığında ilk iş `PROGRESS.md` dosyasını oku.
4. **Bellek Güvenliği ve Hata Yönetimi Kısıtları (Rust):**
   - Tüm saf Rust crate'lerinde (`sekonic-schema`, `sekonic-sim`, `sekonic-transport`, `sekonic-proto-omr`, `sekonic-proto-cis`, `sekonic-grader`, `sekonic-sync`) `#![forbid(unsafe_code)]` zorunludur.
   - Üretim kodunda (testler hariç) `.unwrap()`, `.expect()`, `panic!()` ve `todo!()` kullanımı **yasaktır**. Tüm hatalar `thiserror` ile tanımlanmış tip güvenli `Result<T, SekonicError>` döndürmelidir.
5. **Performans ve Kaynak Tüketimi Kısıtları:**
   - SR-6500 saatte **6.500 kağıt** (~550 ms / kağıt) okur. Okuma döngüsü (hot loop) içinde gereksiz heap tahsisi (`Vec::new()`, `String::from()` tekrarı) yapma; önceden ayrılmış yeniden kullanılabilir tamponlar (`bytes::BytesMut` veya sabit boyutlu diziler) kullan.
   - Boşta RAM tüketimi **< 15 MB**, aktif okumada **< 30 MB** olmalıdır.
6. **Temiz Oda (Clean-Room Interoperability) İlkesi:**
   - `d:\Works\2026\seconic\docs\reverse-engineering\*.zip` arşivlerindeki `.exe` ve `.dll` dosyaları yalnızca **protokol doğrulama referansı** içindir. Üretim uygulamasında hiçbir 32-bit eski DLL (`OMRAPI.dll`, `SkCommIo.dll`, `MSCOMM32.OCX`, `msvbvm50.dll`) paketlenmeyecek veya çağrılmayacaktır.
7. **Çapraz Derleme (Native + WASM) Disiplini:**
   - Çekirdek kütüphaneler (`sekonic-schema`, `sekonic-proto-omr`, `sekonic-proto-cis`, `sekonic-grader`) işletim sistemine özgü API içeremez; hem `x86_64/aarch64` (Windows & macOS) hem de `wasm32-unknown-unknown` (Web Tarayıcı) hedeflerinde hatasız derlenmelidir.

---

## BÖLÜM 1: Doğrulanmış Tersine Mühendislik Referans El Kitabı (Ground-Truth)

Aşağıdaki tüm bilgiler `d:\Works\2026\seconic` altındaki 5 ZIP paketinin PE başlıkları, `.rdata`/`.data` bölümleri, ikili `GUID` tabloları ve `.inf` sürücü dosyalarından doğrudan çıkarılmış **kesin gerçeklerdir (ground-truth)**.

### 1.1. Kaynak Arşivler ve Önemli İkili Dosya SHA-256 Tablosu

| Arşiv / Dosya Yolu | Boyut (Byte) | SHA-256 (İlk 16 Karakter) | Mimari | Rolü |
| :--- | ---: | :--- | :--- | :--- |
| `sr55dkurulum.zip` | 3,157,359 | - | Kaynak ZIP | SR-55D CIS tabanlı kurulum paketi |
| `sr1800kurulum.zip` | 7,503,944 | - | Kaynak ZIP | SR-1800 / SR-1800EX kurulum paketi |
| `sr3500kurulum.zip` | 7,489,324 | - | Kaynak ZIP | SR-3500 kurulum paketi |
| `sr6000kurulum.zip` | 7,467,825 | - | Kaynak ZIP | SR-6000 kurulum paketi |
| `sr6500kurulum.zip` | 7,467,825 | - | Kaynak ZIP | SR-6500 kurulum paketi |
| `*/SysWOW64 Dosyaları/OMRAPI.dll` | 122,880 | `e91cb0851c77ccd2` | x86 PE DLL | **Aile A Güncel Komut DLL'i (125 Export)** — `SkCommIo.dll` + Seri Port |
| `SR1800/OMRAPI.DLL` | 90,112 | `02c9244c8d024c5f` | x86 PE DLL | Aile A Eski Komut DLL'i (91 Export) — Doğrudan SetupAPI + Seri Port |
| `*/SysWOW64 Dosyaları/SkCommIo.dll` | 126,976 | `f7ed656556426228` | x86 PE DLL | **WinUSB Taşıma Katmanı (21 Export)** — `WINUSB.DLL` sarmalayıcı |
| `SR-55D/Middle/SkDvSr55.dll` | 188,416 | `68594b30831adab8` | x86 PE DLL | **Aile B CIS Sürücü DLL'i (98 Export)** — Kalibrasyon & Ham Görüntü |
| `SR-55D/Middle/SkRgOmr.dll` | 177,664 | `348a9ad1bbd6476e` | x86 PE DLL | **Aile B Yazılımsal OMR Motoru (22 Export)** — Görüntüden Baloncuk Okuma |
| `SR-55D/Middle/SkVsSr55.dll` | 176,128 | `0dfc481436146de7` | x86 PE DLL | Sanal `SkCommIo` Emülatörü (SR-55D'yi klasik OMR gibi gösterir) |
| `*/SekOku.exe` | 872,448 | (5 ZIP'te aynı) | x86 VB6 | Eski okuma arayüzü (`.fmt` / `.bldm` -> `.txt`). **Kaldırılacak.** |
| `*/Hazirlama106.exe` | 5,062,144 | `062393ef26791462` | x86 VB5 | Eski `.fmt` hazırlama aracı. **Kaldırılacak.** |

### 1.2. Doğrulanmış USB Kimlikleri (`VID` / `PID`) ve Windows `DeviceInterfaceGUID` Sabitleri

- **USB Vendor ID (Tüm Sekonic Cihazlar):** `0x0A41` (`SEKONIC CORPORATION`)
- **USB Product ID (`PID`) Haritası:**
  - `0x0002` -> `SEKONIC SR-2300 / SR-5500` (Aile A - Klasik OMR)
  - `0x1004` -> `SEKONIC SR-3500 / SR-6000 / SR-6500` (Aile A - Klasik OMR)
  - `0x1008` -> `SEKONIC SR-1800 / SR-1800EX` (Aile A - Klasik OMR)
  - `0x2000` -> `SEKONIC SR-55D / CIS Serisi` (Aile B - CIS Görüntü OMR)
  - `0x2002` -> `SEKONIC SR-45C/NE, SR-38C/NE / CIS Serisi` (Aile B)
  - `0x2003` -> `SEKONIC SMF / Hybrid Serisi` (Aile B)
  - `0x3005` -> `SEKONIC SC-1500 / Hybrid Serisi` (Aile B)
  - `0x5000` -> `SEKONIC SP-100 / Yeni Nesil Serisi` (Aile B)
- **`SkCommIo.dll` (`.rdata` RVA `0x17BA8..0x17BE8`) ve `.inf` İçinden Çıkarılan Kesin GUID Sabitleri:**
  - Cihaz Sınıfı GUID (`SkSr1800.inf` / `SkSr6500.inf` `ClassGUID`): `{a43d14b6-add8-11d3-99f0-005004d38a59}`
  - Eski WDM Arayüz GUID (`UsbSkOmr` / `OMRAPI.DLL` RVA `0xC120`): `{b637bf20-a911-4127-b57f-303fbf314a7e}`
  - WinUSB Arayüz GUID #1 (`SkCommIo.dll` RVA `0x17BC8` - SR Serisi WinUSB): `{c4fa33ee-2253-4394-8a32-389e13b03fd3}`
  - WinUSB Arayüz GUID #2 (`SkCommIo.dll` RVA `0x17BD8` - SR-55D / SMF WinUSB): `{dc78e193-e752-4b23-bc0b-4da9a52ec825}`
  - Windows WIA/Image GUID (`SkCommIo.dll` RVA `0x17BA8`): `{6bdd1fc6-810f-11d0-bec7-08002be2092f}`
- **`SkCommIo.dll` Yerleşik Sanal Test Boruları (Windows Named Pipes):**
  - Kontrol: `\\.\pipe\SekonicStubDevice\Ctrl`
  - Gönderim: `\\.\pipe\SekonicStubDevice\Send1`
  - Alım: `\\.\pipe\SekonicStubDevice\Recv1`

### 1.3. Aile A (`OMRAPI.dll`) Tel Protokolü Komut Sözlüğü (SR-1800EX, SR-3500, SR-6000, SR-6500)

- **Taşıma:** USB Bulk IN / Bulk OUT (`WinUSB` / `IOKit` / `WebUSB`) veya RS-232 Seri Port (`9600` / `19200` / `38400` / `57600` / `115200` baud, 8N1 veya 7E1 — WP-02'de `OMR_OpenDeviceSerial` içinden kesinleştirilecek).
- **Kontrol Byte'ları:**
  - `STX = 0x02`, `ETX = 0x03`, `EOT = 0x04`, `ENQ = 0x05`, `ACK = 0x06`, `NAK = 0x15`, `CR = 0x0D`.
- **Doğrulanmış Komut Şablonları (`OMRAPI.dll` `.rdata`):**
  - **Sistem & Kimlik:**
    - `IS` -> Başlangıç ayarlarına sıfırla (`OMR_InitialSetting`)
    - `MN` -> Makine model adını getir (`OMR_GetMachineName`, örn. `"SR-3500"`, `"SR-6500"`)
    - `DI` -> Donanım/Ünite bilgisini getir (`OMR_GetDeviceInfo`)
    - `FV%1d` -> Firmware versiyonu (`OMR_GetVersion`, ünite indeksine göre)
    - `ST` -> Cihaz durum ve hata bayraklarını sorgula (`OMR_GetStatus`)
    - `DS` -> Kağıt/kapak algılama sensörlerini sorgula (`OMR_GetDetectSensor`)
    - `CE` -> Mevcut hatayı temizle / iptal et (`OMR_CancelError`)
    - `SK` -> Kağıt eğiklik (skew) değerini al (`OMR_GetSkew`)
    - `TS` -> Kağıt kalınlık sensör verisi (`OMR_GetSheetThickness`)
    - `MI` -> Bakım / toplam besleme sayacı (`OMR_GetMaintenanceInfo`)
    - `BJ%3d` -> Buzzer sesli uyarı çal (`OMR_SetBuzzerConfig`)
  - **Kağıt Besleme, Hopper ve Çift Tepsi (Select Stacker) Kontrolü:**
    - `SF` -> Tek kağıt besle (`OMR_FeedSheet`)
    - `SR` -> Kağıdı geri çevir (`OMR_ReverseSheet`)
    - `ER%1d` -> Kağıdı istifleyiciye çıkar (`OMR_EjectSheet`: `0` = Ana İstifleyici / Main Stacker, `1` = Alt/Hatalı İstifleyici / Sub-Stacker!)
    - `HU%1d` -> Besleme tablasını (Hopper) kaldır/indir (`OMR_MoveHopper`)
    - `AR1,0` / `AR1,1` / `AR2,%1d` / `AR3` -> Otomatik besleme kipi (`OMR_SetSheetFeedMode`)
  - **Optik Okuma Ayarları & Ham Veri Okuma (EN KRİTİK KOMUTLAR):**
    - `NC1,%02d` -> Okunacak sütun sayısını ayarla (`OMR_SetNumberOfColumnsToRead`, örn. `48` sütun)
    - `RM1,%1d` -> Okuma yöntemini ayarla (`OMR_SetReadingMethod`)
    - `WB1,%d,%d` -> Ön / Arka (Duplex) okuma ünitesi aktivasyonu (`OMR_SetBackSensorUnit`)
    - `DT1,%02d,%03d` / `DT2` / `DT3` -> Yoğunluk eşiği (`OMR_SetDensityThreshold`)
    - `SH1,+%03d` / `SH1,-%03d` -> Yoğunluk kaydırma (`OMR_SetDensityLevelShift`)
    - `MD%01d` -> İkili işaret bit-maskesini getir (`OMR_GetMarks`, `0`=Ön yüz, `1`=Arka yüz)
    - `AD` / `AD%01d` -> **Ham 0–255 Analog Sensör Yoğunluk Matrisini Getir (`OMR_GetMarkAD` / `OMR_GetReadSensorAD`)** -> Silgi izi ve güven skoru analizinin temel veri kaynağı!
  - **Barkod Okuyucu Ünitesi (`BC*`) ve Kağıt Üstü Yazıcı / Endorser (`PR*`):**
    - `BC1,1,0` / `BC1,1,1` -> Barkod ünitesi kapat/aç (`OMR_SetBarcodeReaderUnit`)
    - `BC1,2,%03d,%03d` -> Barkod okuma alanı (`OMR_SetBarCodeArea`)
    - `BC1,4,%.1c,%01d` -> Barkod tipi ve check-digit (`OMR_SetBarCodeReadType`)
    - `BD1` / `BD2,%02d` -> Okunan barkod verisini çek (`OMR_GetBarcodeData`)
    - `PR1,1,%01d,%s` -> Kağıt üzerine basılacak metni ayarla (`OMR_SetPrintString`)
    - `PR1,2,%03d` -> Baskı pozisyonu (`OMR_SetPrintPosition`)
    - `PR1,3,%01d,%01d,%01d` -> Baskı açısı / yönü / sırası (`OMR_SetPrintAngle`, `OMR_SetPrintOrder`)
    - `PR1,5,%03d` / `PR1,6,%03d` -> Yazı boyutu ve aralığı (`OMR_SetPrintFontSize`, `OMR_SetPrintFontPitch`)
    - `PR1,8,0` / `PR1,8,1` -> Yazıcı ünitesini kapat/aç (`OMR_SetPrinterUnit`)

### 1.4. Aile B (`SkDvSr55.dll` & `SkRgOmr.dll`) CIS Tel Protokolü Komut Sözlüğü (SR-55D & Hybrid)

- **Taşıma:** `SkCommIo` üzerinden **Çift USB Bulk Pipe**:
  1. **Kontrol Borusu (`SendData` / `RecvData`):** `[STX] ... [ETX]` çerçeveli yapılandırma ve tarama başlatma komutları.
  2. **Yüksek Hızlı Görüntü Borusu (`RecvPipe`):** `SF1` komutundan sonra ham gri tonlamalı (8-bit grayscale) CIS tarama satırlarının aktığı Bulk IN endpoint'i.
- **Doğrulanmış `SR-55D` Komut Şablonları (`SkDvSr55.dll` `.rdata`):**
  - `IS%02d` -> Başlangıç ayarı (`SkCd_ReqInitSetting`)
  - `RS07,%c%c%c` -> Yazılımsal sıfırlama (`SkCd_ReqSoftwareReset`)
  - `FV%01d` -> Firmware versiyonu (`SkCd_GetFirmVer`)
  - `DS%01d` -> Sensör durumları (`SkCd_GetDetectionSensors`)
  - `BRw,%01d` / `BRr` -> Arka yüz (Duplex) okuma aç/kapat (`SkCd_SetBackSideReading`: `0`=Tek yüz, `1`=Çift yüz)
  - `SCw,%01d` / `SCr` -> Tarama kontrol tipi (`SkCd_SetScanControlType`)
  - `FLw,%04d,%04d,%03d,%03d` / `FLr` -> Form uzunluğu min/max sınırları (`SkCd_SetFormLength`)
  - `LCw,%01d,%01d` / `LCr,%01d` -> **CIS LED Dropout Işık Rengi (`SkCd_SetLightColor`)**: Kırmızı/Turuncu/Yeşil basılı optik formlarda form çizgilerini görünmez kılıp yalnızca kurşun kalem izini yakalamak için `Red`, `Green`, `Blue` veya `GrayS` seçimi!
  - `REw,%01d,%01d` / `REr` -> Çözünürlük ayarı (`SkCd_SetResolution`: `300` DPI / `600` DPI)
  - `HRr,%01d,%01d,%01d` / `HRw,%01d,%01d,%01d,0,%04d` -> CIS LED yanma süresi (`SkCd_SetCisLedOnTime`)
  - `SSw,%01d,%01d,%01d,%02d,` -> CIS Shading / Beyaz referans verisi (`SkCd_SetCisBaseData`)
  - `CS0` / `CS1,%04d` / `CS2` -> Kalibrasyon Başlat / Kalibrasyon Kağıdı Besle / Kalibrasyondan Çık (`SkCd_ReqCalibration*`)
  - `SF1` -> Form taramasını başlat (`SkCd_ReqScan_Start`) -> ardından görüntü pipe'ından ham piksel akışı okunur!
  - `ER%1.1d` -> Kağıdı dışarı at (`SkCd_ReqEjectSheet`)

### 1.5. Eski `.bldm` Şablon Formatı Dilbilgisi (Geriye Dönük İçe Aktarıcı İçin)

`SR1800/j-17.bldm` içinden doğrulanmış yapı:
- **1. Satır (Başlık):** `<ToplamSatir>=<ToplamSutun>=<TimingSutun veya Bosluk>=<Yon>=<CiftIsaretKarakteri>=<BosKarakteri>=`
  - Örnek: `64=48=04=D=*= =` -> 64 satır, 48 sütun, yön `D` (Dikey), çift işarette `*`, boşta `' '`.
- **2..N Satırları (Alan Tanımları):** `<AlanNo>=<BaslangicSatir>=<BaslangicSutun>=<BitisSutun/Uzunluk>=<KodlamaTipi>=<OkumaYonu>=<AlanAdi/Etiket>=<Secenekler>=`
  - Örnek: `01=01=01=48=K=D=A=X2=`

---

## BÖLÜM 2: Monorepo Dizin Ağacı ve Teknoloji Yığını

Gemini Hydrogen, kök dizin `d:\Works\2026\seconic` altında tam olarak aşağıdaki dizin yapısını kuracaktır (**mevcut 5 `.zip` dosyasına dokunulmayacak, silinmeyecektir**):

```text
d:\Works\2026\seconic\
├── Cargo.toml                        # Rust Workspace kök dosyası (Edition 2021, resolver = "2")
├── PROGRESS.md                       # AI Worker ilerleme takip tablosu
├── docs\
│   ├── sekonic_implementation_plan.md # Bu Master Supervisor Planı
│   ├── reverse-engineering\          # Kaynak Sekonic kurulum ZIP arşivleri
│   ├── mockups\                      # UI/UX referans ekran tasarımları
│   ├── wire-spec-family-a.md         # WP-02'de üretilen kesin byte/CRC/offset dokümanı
│   ├── wire-spec-family-b-cis.md     # WP-02'de üretilen SR-55D CIS byte/kalibrasyon dokümanı
│   └── integration-api-spec.md       # Ölçme-Değerlendirme WebSocket & REST protokol dokümanı
├── tools\
│   └── re_verify\                    # WP-01 & WP-02 Python statik analiz & disassembly betikleri
│       ├── verify_archives.py        # ZIP SHA-256 doğrulama ve geçici çıkarma aracı
│       ├── disasm_omrapi.py          # Capstone + pefile ile OMRAPI.dll fonksiyon/CRC çıkarıcı
│       └── disasm_sr55.py            # Capstone + pefile ile SkDvSr55.dll & SkRgOmr.dll çıkarıcı
├── crates\
│   ├── sekonic-schema\               # Dinamik JSON Form Şeması, Hata Tipleri, .bldm/.fmt dönüştürücü (WASM uyumlu)
│   │   ├── Cargo.toml
│   │   └── src\
│   │       ├── lib.rs
│   │       ├── error.rs
│   │       ├── form_schema.rs
│   │       ├── raw_capture.rs
│   │       └── legacy_bldm.rs
│   ├── sekonic-transport\            # USB Bulk (nusb / WebUSB) & Seri Port (tokio-serial / WebSerial) + Mock/Pipe
│   │   ├── Cargo.toml
│   │   └── src\
│   │       ├── lib.rs
│   │       ├── traits.rs
│   │       ├── usb_ids.rs
│   │       ├── native_usb.rs         # #[cfg(not(target_arch = "wasm32"))] -> nusb (WinUSB + macOS IOKit)
│   │       ├── native_serial.rs      # #[cfg(not(target_arch = "wasm32"))] -> tokio-serial
│   │       ├── wasm_usb.rs           # #[cfg(target_arch = "wasm32")] -> web-sys Navigator.usb
│   │       ├── wasm_serial.rs        # #[cfg(target_arch = "wasm32")] -> web-sys Navigator.serial
│   │       └── memory_channel.rs     # Simülatör ve birim testleri için bellek içi dubleks kanal
│   ├── sekonic-proto-omr\            # Aile A (SR-1800/3500/6000/6500) STX/ETX asenkron protokol motoru
│   │   ├── Cargo.toml
│   │   └── src\
│   │       ├── lib.rs
│   │       ├── codec.rs              # STX/ETX çerçeveleme, BCC/CRC hesaplama, ACK/NAK
│   │       ├── commands.rs           # IS, MN, ST, SF, ER, MD, AD, BC, PR komut üreticileri
│   │       ├── status_decoder.rs     # 60+ donanım hata/uyarı bayrağı çözümleyici
│   │       └── driver.rs             # Yüksek hızlı pipelined kağıt okuma döngüsü
│   ├── sekonic-proto-cis\            # Aile B (SR-55D & Hybrid) CIS protokol, kalibrasyon ve görüntü OMR motoru
│   │   ├── Cargo.toml
│   │   └── src\
│   │       ├── lib.rs
│   │       ├── commands.rs           # IS, BRw, FLw, LCw, REw, CS0/1/2, SF1 komutları
│   │       ├── calibration.rs        # Shading / beyaz referans düzeltme & önbellekleme
│   │       └── image_omr.rs          # Timing mark tespiti, deskew ve pikselden 0-255 matris çıkarma
│   ├── sekonic-grader\               # 0-255 Analog Silgi Ayıklama, Güven Skoru ve Canlı Yoklama/Kitapçık Motoru
│   │   ├── Cargo.toml
│   │   └── src\
│   │       ├── lib.rs
│   │       ├── analog_solver.rs      # Silgi vs Çift İşaret kontrast algoritması
│   │       ├── field_decoder.rs      # TC, Öğrenci No, Kitapçık, Çoklu Branş çözümleyici
│   │       └── roster_validator.rs   # Yoklama listesi kontrolü -> Anlık Main/Sub-Stacker kararı
│   ├── sekonic-sim\                  # Sanal Sekonic Donanım Emülatörü (Aile A + Aile B + Hata Senaryoları)
│   │   ├── Cargo.toml
│   │   └── src\
│   │       ├── lib.rs
│   │       ├── virtual_sr6500.rs
│   │       ├── virtual_sr55d.rs
│   │       └── paper_generator.rs    # Sentetik silgi izli, kaymış veya temiz sanal optik kağıt üretici
│   └── sekonic-sync\                 # SQLite WAL Çevrimdışı Kuyruk + Ölçme App WebSocket/REST İstemcisi (Native)
│       ├── Cargo.toml
│       └── src\
│           ├── lib.rs
│           ├── wal_queue.rs          # rusqlite (bundled) WAL modu sıfır veri kayıplı kuyruk
│           └── ws_client.rs          # tokio-tungstenite + reqwest otomatik yeniden bağlanan akış istemcisi
├── packages\
│   └── sekonic-web-sdk\              # WASM bağlayıcısı + TypeScript SDK (@kurum/sekonic-web-sdk)
│       ├── Cargo.toml                # wasm-bindgen sarmalayıcı crate
│       ├── package.json
│       ├── tsconfig.json
│       └── src\
│           ├── lib.rs
│           └── index.ts              # Otomatik Bridge (ws://127.0.0.1) vs Doğrudan WebUSB/WebSerial seçici
└── apps\
    └── sekonic-bridge\               # Tauri v2 Masaüstü & Sistem Tepsisi Uygulaması (Windows & macOS)
        ├── package.json
        ├── vite.config.ts
        ├── src\                      # uxiGL (https://uxigl.truncgil.com/) + SheetGL (https://sheetgl.truncgil.com/) + HTML5 Canvas Arayüzü
        └── src-tauri\
            ├── Cargo.toml
            ├── tauri.conf.json
            └── src\
                ├── main.rs
                ├── local_ws_bridge.rs # Web uygulamasının bağlandığı ws://127.0.0.1:17441 sunucusu
                └── scanner_manager.rs # Hotplug, aktif oturum ve SQLite senkronizasyon yöneticisi
```

---

## BÖLÜM 3: Kesin Veri Sözleşmeleri, Algoritmalar ve Protokol Şemaları

AI Worker (Hydrogen), modüller arası uyumsuzluk yaşamamak için aşağıdaki **birebir tip sözleşmelerini** kullanacaktır:

### 3.1. Taşıma Katmanı Trait Sözleşmesi (`crates/sekonic-transport/src/traits.rs`)

```rust
use async_trait::async_trait;
use sekonic_schema::error::SekonicError;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceFamily {
    /// SR-1800, SR-1800EX, SR-2300, SR-3500, SR-5500, SR-6000, SR-6500
    FamilyAOmrGrid,
    /// SR-55D ve CIS / Görüntü tabanlı Hybrid modeller
    FamilyBCisImage,
}

#[derive(Debug, Clone)]
pub struct DiscoveredScanner {
    pub id: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub model_hint: String,
    pub family: DeviceFamily,
    pub transport_kind: TransportKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportKind {
    UsbBulk,
    SerialPort,
    VirtualSimulator,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait ScannerTransport {
    /// Kontrol / komut borusundan ham byte gönderir.
    async fn write_control(&mut self, buf: &[u8], timeout: Duration) -> Result<usize, SekonicError>;

    /// Kontrol / yanıt borusundan ham byte okur.
    async fn read_control(&mut self, buf: &mut [u8], timeout: Duration) -> Result<usize, SekonicError>;

    /// Yalnızca Aile B (CIS) cihazlarda yüksek hızlı 2. Bulk IN görüntü borusundan veri okur.
    async fn read_image_pipe(&mut self, buf: &mut [u8], timeout: Duration) -> Result<usize, SekonicError>;

    /// Tamponlardaki eski/bozuk veriyi temizler ve USB borusunu abort/reset eder.
    async fn purge(&mut self) -> Result<(), SekonicError>;
}
```

### 3.2. Dinamik Optik Form JSON Şeması (`crates/sekonic-schema/src/form_schema.rs`)

`.fmt` dosyalarının yerini alan, Ölçme-Değerlendirme uygulamasından JSON olarak gelen evrensel şema:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OmrFormSchema {
    pub schema_version: u32,          // 1
    pub form_id: String,              // Örn: "LGS-2026-SAYISAL-A4"
    pub form_name: String,
    pub duplex: bool,                 // Ön + Arka yüz okuma
    pub dropout_color: DropoutColor,  // CIS cihazlar için: Red, Green, Blue, None
    pub grid: GridGeometry,
    pub threshold_policy: ThresholdPolicy,
    pub fields: Vec<FieldDefinition>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DropoutColor {
    None,
    Red,
    Green,
    Blue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridGeometry {
    pub front_rows: u16,              // Örn: 64 timing satırı
    pub front_cols: u16,              // Örn: 48 okuma sütunu
    pub back_rows: u16,               // Duplex değilse 0
    pub back_cols: u16,
    /// CIS görüntü modunda fiziksel form genişliği/yüksekliği (mm)
    pub width_mm: f32,                // Örn: 210.0
    pub height_mm: f32,               // Örn: 297.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdPolicy {
    /// Bir baloncuğun "boş değil, işaretlenmiş" sayılması için gereken minimum 0-255 koyuluk (varsayılan: 48)
    pub min_mark_density: u8,
    /// Güçlü/net bir kurşun kalem işareti eşiği (varsayılan: 115)
    pub strong_mark_density: u8,
    /// Silgi kurtarma oranı: (ikinci_en_koyu / birinci_en_koyu) <= erasure_ratio ise ikinci iz silgi sayılır! (varsayılan: 0.42)
    pub erasure_ratio: f32,
    /// Güven skoru bu değerin altındaysa operatör ekranında "Şüpheli / Onay Bekliyor" olarak işaretle (varsayılan: 0.65)
    pub review_confidence_floor: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDefinition {
    pub key: String,                  // Örn: "tc_kimlik", "ogrenci_no", "kitapcik", "matematik"
    pub field_type: FieldType,
    pub side: SheetSide,              // Front | Back
    pub required_for_valid_sheet: bool, // True ise ve geçersizse kağıt Sub-Stacker'a atılır!
    /// Her bir hane (sütun/satır) veya soru için baloncuk seçenekleri listesi
    pub items: Vec<BubbleGroup>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    StudentId,
    NationalId,
    BookletType,
    ClassCode,
    SubjectQuestions,
    CustomCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SheetSide {
    Front,
    Back,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BubbleGroup {
    pub index: u16,                   // Soru no (1..N) veya Hane no (1..11)
    pub options: Vec<BubbleCoord>,    // Örn: [{'label':'A', row:12, col:5}, {'label':'B', row:12, col:6}...]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BubbleCoord {
    pub label: String,                // "A", "B", "C", "D", "E" veya "0".."9" veya Harf
    pub row: u16,                     // 0-tabanlı satır indeksi
    pub col: u16,                     // 0-tabanlı sütun indeksi
}
```

### 3.3. Akıllı Analog Silgi ve Çift İşaret Çözümleme Algoritması (`crates/sekonic-grader/src/analog_solver.rs`)

Gemini Hydrogen, her `BubbleGroup` (örneğin 1 sorunun A, B, C, D, E şıkları) için tam olarak aşağıdaki matematiksel karar mantığını kodlayacaktır:

1. **Yerel Arka Plan Düzeltmesi (Paper Background Normalization):**
   - Kağıt üzerindeki tüm okunan hücrelerin alt %25'lik diliminin medyanı $B_{\text{sheet}}$ (kağıt beyazlık tabanı, genelde `5..18` arası) hesaplanır.
   - Her şıkkın düzeltilmiş net koyuluğu: $d_i = \max(0, \text{raw}_i - B_{\text{sheet}})$.
2. **Sıralama:**
   - Seçenekler $d_i$ değerine göre büyükten küçüğe sıralanır: en koyu şık $(L_1, v_1)$ ve ikinci en koyu şık $(L_2, v_2)$.
3. **Karar Kuralları (`BubbleDecision`):**
   - **Durum 1 — Boş (`Blank`):** Eğer $v_1 < T_{\text{min}}$ (`min_mark_density`) ise:
     - Sonuç: `Blank`, $\text{confidence} = \text{clamp}\left(1.0 - \frac{v_1}{T_{\text{min}}}, 0.5, 1.0\right)$.
   - **Durum 2 — Tek Net İşaret (`Marked`):** Eğer $v_1 \ge T_{\text{min}}$ ve $v_2 < T_{\text{min}}$ ise:
     - Sonuç: `Marked(L_1)`, $\text{confidence} = \text{clamp}\left(0.60 + 0.40 \cdot \frac{v_1 - T_{\text{min}}}{255 - T_{\text{min}}}, 0.60, 1.0\right)$.
   - **Durum 3 — Silgi İzi Kurtarma (`MarkedWithErasure`):** Eğer $v_1 \ge T_{\text{strong}}$ VE $v_2 \ge T_{\text{min}}$ VE $\frac{v_2}{v_1} \le R_{\text{erasure}}$ (`0.42`) ise:
     - Öğrenci $L_2$ şıkkını silmiş, $L_1$ şıkkını net biçimde kodlamıştır!
     - Sonuç: `Marked(L_1)` (`erasure_detected = true`, `erased_label = Some(L_2)`), $\text{confidence} = \text{clamp}\left(1.0 - \frac{v_2}{v_1}, 0.65, 0.95\right)$.
   - **Durum 4 — Gerçek Çift İşaret (`MultiMark`):** Diğer tüm $v_1 \ge T_{\text{min}}$ ve $v_2 \ge T_{\text{min}}$ durumlarında:
     - Sonuç: `MultiMark(vec![L_1, L_2, ...])`, $\text{confidence} = \text{clamp}\left(\frac{v_2}{v_1}, 0.50, 1.0\right)$.

### 3.4. Ölçme-Değerlendirme Uygulaması Doğrudan Entegrasyon Mesaj Sözleşmesi

Yerel `sekonic-bridge` WebSocket (`ws://127.0.0.1:17441/v1/stream`) ve Bulut Ölçme-Değerlendirme Sunucusu (`wss://<api-adresiniz>/v1/omr/stream`) birebir aynı zarf (envelope) yapısını kullanır:

```json
{
  "event": "sheet.scanned",
  "session_id": "ses_01J8XYZ...",
  "seq_no": 142,
  "timestamp_ms": 1763320000123,
  "scanner": {
    "model": "SR-6500",
    "serial_or_id": "VID_0A41_PID_1004_01",
    "stacker_routed": "main"
  },
  "barcode": "978000142",
  "validation": {
    "status": "valid",
    "matched_student_id": "20261045",
    "needs_operator_review": false,
    "warnings": ["q14_erasure_recovered"]
  },
  "decoded_fields": {
    "ogrenci_no": "20261045",
    "tc_kimlik": "12345678902",
    "kitapcik": "A"
  },
  "answers": {
    "turkce": "ABC*D EAB...",
    "matematik": "DCBA E..."
  },
  "answer_details": {
    "matematik": [
      { "q": 1, "state": "marked", "val": "D", "conf": 0.98, "densities": [8, 6, 11, 214] },
      { "q": 14, "state": "marked", "val": "C", "conf": 0.81, "erased": "A", "densities": [54, 7, 198, 9] }
    ]
  },
  "raw_matrix_b64": "AAcBCt4..."
}
```

> **Neden `raw_matrix_b64` Saklanıyor?** Kağıdın 48×64 ham 8-bit yoğunluk matrisi yalnızca **3 KB** (sıkıştırılmış ~600 byte) yer kaplar! Bu sayede sınavdan 3 gün sonra optik form şablonunda bir kayma/hata fark edilse bile **binlerce kağıdı tekrar tarayıcıdan geçirmeden** sunucuda 1 saniyede yeniden okuma (re-grade) yapılabilir ve ekranda gerçekçi optik kağıt ısı haritası çizilebilir!

---

## BÖLÜM 4: Atomik İş Paketleri (WP-01 .. WP-12 Supervisor Görev Kartları)

AI Worker (Gemini Hydrogen) her adımı sırasıyla uygulayacak ve doğrulama kapısını geçmeden ilerlemeyecektir.

---

### WP-01: Monorepo İskeleti, `PROGRESS.md` ve Kaynak ZIP Doğrulama Aracı
- **Hedef:** Rust Workspace iskeletini kurmak, `PROGRESS.md` dosyasını başlatmak ve 5 kurulum ZIP dosyasını çalışma alanını kirletmeden geçici analiz dizinine (`.cache/extracted_bins/`, `.gitignore` içinde) çıkaran Python doğrulama aracını yazmak.
- **Oluşturulacak Dosyalar:**
  - `.gitignore` (`.cache/`, `target/`, `node_modules/`, `dist/` hariç tutulacak)
  - `PROGRESS.md` (WP-01..WP-12 kontrol listesi)
  - `Cargo.toml` (Workspace tanımı ve ortak bağımlılıklar: `serde`, `serde_json`, `thiserror`, `bytes`, `tokio`, `tracing`)
  - `tools/re_verify/verify_archives.py`
  - Tüm 7 Rust crate için başlangıç `Cargo.toml` ve `#![forbid(unsafe_code)]` içeren derlenebilir boş `src/lib.rs` dosyaları.
- **Uygulama Talimatları:**
  1. `verify_archives.py` betiği `d:\Works\2026\seconic\docs\reverse-engineering\*.zip` dosyalarının varlığını kontrol etmeli, `.cache/extracted_bins/` altına açmalı ve Bölüm 1.1'deki SHA-256 özetlerini (`e91cb0851c77ccd2`, `f7ed656556426228`, `68594b30831adab8`, `348a9ad1bbd6476e`) doğrulamalıdır.
  2. `Cargo.toml` içinde `resolver = "2"` kullanılmalıdır.
- **Zorunlu Doğrulama Kapısı (Verification Gate):**
  ```powershell
  python tools/re_verify/verify_archives.py
  cargo check --workspace
  ```
  Her iki komut `exit code 0` dönmelidir.

---

### WP-02: Otomatik x86 Disassembly ile Byte/CRC/Offset Kesinleştirme (`re_verify`)
- **Hedef:** `OMRAPI.dll` (`122,880` byte) ve `SkDvSr55.dll` (`188,416` byte) içindeki paketleme, sağlama toplamı (BCC/CRC), seri port baudrate ve `AD`/`MD`/CIS piksel byte dizilimini statik disassembly ile %100 kesinleştirip dokümante etmek.
- **Oluşturulacak Dosyalar:**
  - `tools/re_verify/disasm_omrapi.py`
  - `tools/re_verify/disasm_sr55.py`
  - `docs/wire-spec-family-a.md`
  - `docs/wire-spec-family-b-cis.md`
- **Uygulama Talimatları:**
  1. Python ortamında `pefile` ve `capstone` (yoksa `pip install capstone pefile` ile kur) kullanarak `OMRAPI.dll` export tablosundan şu fonksiyonların x86 makine kodunu sök (disassemble et):
     - `OMR_OpenDeviceSerial` ve `OMR_OpenDeviceUSB` (DCB seri port ayarları: BaudRate, ByteSize, Parity, StopBits; USB pipe açılış sırası)
     - `_OMR_SendCommand`, `_OMR_SendCommandAndGetResponse`, `_OMR_ReceiveACKorNAK`, `_OMR_ReceiveData` (`0x02` STX, `0x03` ETX çerçevesi, sonrasında XOR/BCC/CRC byte'ı veya `CR` olup olmadığı!)
     - `OMR_GetMarks`, `OMR_GetMarkAD`, `OMR_GetStatus`, `OMR_EjectSheet` (`MD` ve `AD` yanıtlarının satır/sütun byte dizilimi ve ASCII-Hex vs Binary dönüşümü).
  2. `disasm_sr55.py` ile `SkDvSr55.dll` içindeki `SkCd_SendData`, `SkCd_RecvData`, `SkCd_ReqScan_Start`, `SkCd_ReqScan_RecvImage` ve `SkRgOmr.dll` içindeki `SkRg_ReqRecog` fonksiyonlarını sökerek CIS tarama satırı genişliğini, byte sırasını ve USB endpoint/pipe numaralarını çıkar.
  3. Elde edilen **kesin byte kurallarını** örnek hex paketleriyle `docs/wire-spec-family-a.md` ve `docs/wire-spec-family-b-cis.md` dosyalarına yaz.
- **Yasaklar:** Disassembly çıktısını görmeden sağlama toplamı (checksum) algoritmasını tahmin etme! Birebir disassembly kanıtını RVA adresiyle (`RVA 0x....`) dokümana yaz.
- **Zorunlu Doğrulama Kapısı:**
  ```powershell
  python tools/re_verify/disasm_omrapi.py
  python tools/re_verify/disasm_sr55.py
  Test-Path docs/wire-spec-family-a.md
  Test-Path docs/wire-spec-family-b-cis.md
  ```

---

### WP-03: Şema, Veri Yapıları ve Eski `.bldm` / `.fmt` Dönüştürücüsü (`sekonic-schema`)
- **Hedef:** Tüm sistemin ortak tiplerini, `SekonicError` hata hiyerarşisini, `OmrFormSchema` JSON şemasını ve eski `.bldm` / `.fmt` dosyalarını otomatik `OmrFormSchema`'ya çeviren modülü yazmak.
- **Oluşturulacak Dosyalar:**
  - `crates/sekonic-schema/src/{lib.rs, error.rs, form_schema.rs, raw_capture.rs, legacy_bldm.rs}`
- **Uygulama Talimatları:**
  1. Bölüm 3.2'deki `OmrFormSchema` yapılarını eksiksiz uygula ve şema doğrulama (`schema.validate()`) metodu ekle (satır/sütun sınır aşımı, mükerrer alan anahtarı kontrolü).
  2. `RawSheetCapture` tipini tanımla: `Matrix2D { front: Vec<u8>, back: Option<Vec<u8>>, rows: u16, cols: u16 }` (satır-öncelikli `row * cols + col` düz dizi, sıfır ekstra allocation!).
  3. `legacy_bldm.rs` içinde `.cache/extracted_bins/sr1800kurulum/SR1800/j-17.bldm`, `j-34.bldm`, `j-52.bldm`, `j-64.bldm` dosyalarını okuyup `OmrFormSchema` üreten ayrıştırıcıyı yaz ve gerçek `.bldm` dosyaları üzerinde birim testi ekle!
- **Zorunlu Doğrulama Kapısı:**
  ```powershell
  cargo test -p sekonic-schema
  cargo check -p sekonic-schema --target wasm32-unknown-unknown
  ```
  *(Not: `wasm32-unknown-unknown` hedefi kurulu değilse önce `rustup target add wasm32-unknown-unknown` çalıştır.)*

---

### WP-04: Taşıma Katmanı Soyutlaması (`sekonic-transport`)
- **Hedef:** Windows, macOS ve Web (WASM) üzerinde çalışan birleşik USB Bulk ve Seri Port taşıma katmanını kodlamak.
- **Oluşturulacak Dosyalar:**
  - `crates/sekonic-transport/src/{lib.rs, traits.rs, usb_ids.rs, native_usb.rs, native_serial.rs, wasm_usb.rs, wasm_serial.rs, memory_channel.rs}`
- **Uygulama Talimatları:**
  1. `usb_ids.rs` içine Bölüm 1.2'deki `VID_SEKONIC = 0x0A41`, tüm `PID_*` sabitlerini ve 4 `GUID` sabitini ekle; `classify_pid(pid: u16) -> Option<(DeviceFamily, &'static str)>` yardımcı fonksiyonunu yaz.
  2. Native hedeflerde (`#[cfg(not(target_arch = "wasm32"))]`):
     - `nusb` crate'i ile bağlı USB cihazlarını tara (`VID == 0x0A41`), interface claim et, Bulk OUT (kontrol yazma), Bulk IN #1 (kontrol okuma) ve varsa Bulk IN #2 (CIS görüntü okuma) endpoint adreslerini USB descriptor'dan dinamik bul (`WP-02` çıktısıyla uyumlu olarak).
     - `tokio-serial` ile RS-232 COM port bağlantısını uygula.
  3. WASM hedefinde (`#[cfg(target_arch = "wasm32")]`):
     - `web-sys` (`UsbDevice`, `SerialPort`) üzerinden `ScannerTransport` trait'ini uygula.
  4. `memory_channel.rs`: `tokio::sync::mpsc` / `DuplexStream` tabanlı `InMemoryTransport` çifti (`client_transport`, `device_transport`) oluştur — simülatör testleri bunun üzerinde koşturulacak!
- **Zorunlu Doğrulama Kapısı:**
  ```powershell
  cargo test -p sekonic-transport
  cargo check -p sekonic-transport --target wasm32-unknown-unknown
  ```

---

### WP-05: Sanal Sekonic Donanım Simülatörü (`sekonic-sim`)
- **Hedef:** Fiziksel cihaza bağlı kalmadan hem Aile A (`SR-6500` / `SR-3500` / `SR-1800EX`) hem de Aile B (`SR-55D` CIS) protokollerini birebir konuşan sanal donanım emülatörü ve sentetik optik kağıt üreticisi yazmak.
- **Oluşturulacak Dosyalar:**
  - `crates/sekonic-sim/src/{lib.rs, virtual_sr6500.rs, virtual_sr55d.rs, paper_generator.rs}`
- **Uygulama Talimatları:**
  1. `paper_generator.rs`: Verilen bir `OmrFormSchema` ve sahte öğrenci cevap anahtarı için:
     - Temiz kağıt (`Clean`),
     - Silgi izli kağıt (`WithErasures` — silinen şık `45..65` yoğunlukta, yeni şık `195..230` yoğunlukta, kağıt zemini `6..14` gürültülü),
     - Çift işaretli kağıt (`WithMultiMarks`),
     - Hatalı/Kayıtsız öğrenci numaralı kağıt (`InvalidStudentId`),
     - Fiziksel donanım hatası (`DoubleFeed`, `PaperJam`, `SkewError`, `HopperEmpty`)
     üreten deterministik (seed'li) üreteç yaz.
  2. `virtual_sr6500.rs`: `WP-02` dokümanındaki byte çerçevesine sadık kalarak `IS`, `MN`, `DI`, `ST`, `SF`, `MD`, `AD`, `ER0`/`ER1` (hangi kağıdın hangi stacker'a düştüğünü kaydeder!), `BC` ve `PR` komutlarına gerçek cihaz gibi yanıt veren asenkron görev yaz.
  3. `virtual_sr55d.rs`: `IS`, `LCw`, `REw`, `CS0/1/2` ve `SF1` komutlarını işleyip 2. görüntü borusundan sentetik 8-bit gri tonlamalı optik form görüntüsü (timing mark + baloncuklar dahil!) akıtan emülatör yaz.
- **Zorunlu Doğrulama Kapısı:**
  ```powershell
  cargo test -p sekonic-sim
  ```

---

### WP-06: Aile A Asenkron Protokol ve Pipelined Okuma Motoru (`sekonic-proto-omr`)
- **Hedef:** SR-1800EX, SR-3500, SR-6000 ve SR-6500 cihazlarını tam hızda (6.500 kağıt/saat) süren sıfır kopyalamalı (zero-copy) protokol sürücüsünü yazmak.
- **Oluşturulacak Dosyalar:**
  - `crates/sekonic-proto-omr/src/{lib.rs, codec.rs, commands.rs, status_decoder.rs, driver.rs}`
- **Uygulama Talimatları:**
  1. `codec.rs`: `WP-02`'de çıkarılan `STX`/`ETX`, BCC/CRC ve `ACK`/`NAK` çerçeveleme kodlayıcı/çözücüsünü önceden ayrılmış sabit tamponlarla (`BytesMut::with_capacity(4096)`) uygula.
  2. `status_decoder.rs`: `OMRAPI.dll` içindeki tüm donanım hata/uyarı durumlarını yapılandırılmış `HardwareStatus` / `HardwareFault` enum'larına çevir (kullanıcı dostu Türkçe + İngilizce açıklamalarıyla).
  3. `driver.rs`:
     - Cihaz başlatma sekansı: `IS` -> `MN` -> `DI` -> `NC1` -> `WB1` (duplex ise) -> `BC1` (barkod varsa).
     - Tekil ve Sürekli Okuma Döngüsü (`scan_batch_stream`): `SF` -> `AD0` (+ duplex ise `AD1`) + barkod varsa `BD1` -> `Grader` geri çağrısından gelen `StackerTarget` (`Main` veya `Sub`) kararına göre `ER0` veya `ER1` (+ endorser varsa `PR1,1` + `ER`) gönder!
     - `HopperEmpty` geldiğinde temiz şekilde oturumu duraklat; `DoubleFeed` veya `Jam` geldiğinde `CE` ve operatör kurtarma akışını tetikle.
  4. `sekonic-sim` (`VirtualSr6500`) üzerinde 500 ardışık sanal kağıdı (temiz + silgili + kayıtsız öğrencili) koşturup kayıtsız öğrencilerin %100'ünün `Sub-Stacker`'a yönlendirildiğini doğrulayan entegrasyon testi yaz!
- **Zorunlu Doğrulama Kapısı:**
  ```powershell
  cargo test -p sekonic-proto-omr
  cargo check -p sekonic-proto-omr --target wasm32-unknown-unknown
  ```

---

### WP-07: Aile B CIS Görüntü Protokolü ve Hızlı OMR Görüntü Motoru (`sekonic-proto-cis`)
- **Hedef:** SR-55D ve CIS tabanlı Hybrid modellerden ham tarama görüntüsünü alıp milisaniyeler içinde Aile A ile birebir aynı `RawSheetCapture::Matrix2D` (0–255 yoğunluk matrisi) çıktısına dönüştürmek.
- **Oluşturulacak Dosyalar:**
  - `crates/sekonic-proto-cis/src/{lib.rs, commands.rs, calibration.rs, image_omr.rs}`
- **Uygulama Talimatları:**
  1. `commands.rs`: `WP-02`'de doğrulanan `SkDvSr55` komutlarını (`IS`, `BRw`, `FLw`, `LCw` dropout rengi, `REw` 300 DPI, `HRw`, `SSw`, `CS0/1/2`, `SF1`, `ER`) uygula.
  2. `calibration.rs`: Shading (piksel bazlı siyah/beyaz kazanç normalizasyonu: $p_{\text{norm}} = \text{clamp}\left(255 \cdot \frac{p_{\text{raw}} - \text{dark}_x}{\text{white}_x - \text{dark}_x}, 0, 255\right)$) uygula ve kalibrasyon profilini JSON/binary olarak önbelleklenebilir yap.
  3. `image_omr.rs`:
     - Taranan 8-bit görüntüde sol/sağ kenardaki **Timing Marks (Zamanlama Çizgileri)** dizisini tespit et (böylece kağıt hızındaki ufak değişimler veya ±3° eğiklik / skew otomatik telafi edilir!).
     - Her `(row, col)` kesişim bölgesindeki baloncuk ROI (Region of Interest) kutusunun ters çevrilmiş ortalama koyuluğunu (`255 - luminance`, yani koyu kalem izi = yüksek değer `0..255`) hesaplayarak `RawSheetCapture::Matrix2D` üret!
  4. `VirtualSr55d` üzerinde eğik (±1.5°) ve silgi izli sanal görüntüyle uçtan uca birim/entegrasyon testi yaz.
- **Zorunlu Doğrulama Kapısı:**
  ```powershell
  cargo test -p sekonic-proto-cis
  cargo check -p sekonic-proto-cis --target wasm32-unknown-unknown
  ```

---

### WP-08: Akıllı Analog Değerlendirme ve Yoklama Doğrulama Motoru (`sekonic-grader`)
- **Hedef:** Hem Aile A'dan hem Aile B'den gelen `RawSheetCapture::Matrix2D` verisini `OmrFormSchema` ve sınav yoklama listesi (`ExamRoster`) ile milisaniyenin altında değerlendirip anlık yönlendirme ve puanlama çıktısı üretmek.
- **Oluşturulacak Dosyalar:**
  - `crates/sekonic-grader/src/{lib.rs, analog_solver.rs, field_decoder.rs, roster_validator.rs}`
- **Uygulama Talimatları:**
  1. `analog_solver.rs`: Bölüm 3.3'teki 4 durumlu (`Blank`, `Marked`, `MarkedWithErasure`, `MultiMark`) analog karar algoritmasını ve güven skoru hesabını birebir uygula.
  2. `field_decoder.rs`: Sayısal/Sözel kodlama alanlarını (`tc_kimlik`, `ogrenci_no`, `kitapcik`) ve çoklu branş soru dizilerini (`turkce`, `matematik`, `fen`, `sosyal` vb.) çöz. İsteğe bağlı cevap anahtarı verilmişse doğru/yanlış/boş ve **Net** hesabını anında çıkar.
  3. `roster_validator.rs`: `HashSet` / `HashMap` önbellekli öğrenci listesinde O(1) doğrulama yap:
     - Öğrenci numarası okunamadıysa / listede yoksa VEYA zorunlu `kitapcik` alanı boş/çift işaretliyse -> `StackerDecision::RouteToSubStacker(Reason)` döndür!
     - Mükerrer okuma (aynı öğrenci numarası aynı oturumda ikinci kez geçiyorsa) -> `DuplicateStudentSheet` uyarısı üret ve `Sub-Stacker`'a ayır!
  4. Kapsamlı test senaryoları: Silgi kurtarma başarısı, çift işaret yakalama, mükerrer öğrenci yakalama ve 10.000 kağıdın tek çekirdekte < 500 ms içinde değerlendirildiğini kanıtlayan hız testi!
- **Zorunlu Doğrulama Kapısı:**
  ```powershell
  cargo test -p sekonic-grader --release
  cargo check -p sekonic-grader --target wasm32-unknown-unknown
  ```

---

### WP-09: SQLite WAL Çevrimdışı Kuyruk ve Ölçme App Senkronizasyon İstemcisi (`sekonic-sync`)
- **Hedef:** İnternet kesintilerinde tek bir kağıdın bile kaybolmamasını sağlayan yerel SQLite WAL kuyruğunu ve Ölçme-Değerlendirme uygulamasıyla konuşan gerçek zamanlı WebSocket + REST Batch istemcisini yazmak.
- **Oluşturulacak Dosyalar:**
  - `crates/sekonic-sync/src/{lib.rs, wal_queue.rs, ws_client.rs}`
- **Uygulama Talimatları:**
  1. `wal_queue.rs`: `rusqlite` (`bundled` özelliği ile harici DLL gerektirmeden!) kullanarak `PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;` modunda tabloları kur:
     - `cached_exams` (çevrimdışı çalışabilmek için indirilen `OmrFormSchema` + öğrenci yoklama listesi)
     - `scanned_sheets_outbox` (`id`, `session_id`, `seq_no`, `payload_json`, `sync_status` [`pending` | `synced` | `failed`], `created_at_ms`). İdempotent `idempotency_key = "{session_id}:{seq_no}"` kullan, böylece ağ kopup tekrar bağlandığında aynı kağıt sunucuya iki kez yazılsa bile mükerrer kayıt oluşmaz!
  2. `ws_client.rs`:
     - Ölçme-Değerlendirme API'sine `Bearer <JWT>` ile bağlanır.
     - Canlı okuma sırasında her kağıdı önce yerel `wal_queue`'ya yazar (< 0.5 ms), ardından arka plan görevi WebSocket açıksa anında akıtır, kapalıysa üstel geri çekilme (exponential backoff: 1s, 2s, 5s, 15s) ile bağlantı geldiğinde `pending` kayıtları sırasıyla REST Batch veya WS üzerinden eritir.
  3. Yerel sahte (mock) WebSocket/HTTP sunucusu ayağa kaldırıp ağ kesintisi simülasyonu yapan (100 kağıt oku -> 50. kağıtta bağlantıyı kes -> kalan 50 kağıdı offline oku -> bağlantıyı aç -> 100 kağıdın eksiksiz ve sırasıyla sunucuya ulaştığını doğrula) entegrasyon testi yaz!
- **Zorunlu Doğrulama Kapısı:**
  ```powershell
  cargo test -p sekonic-sync
  ```

---

### WP-10: WebAssembly Derlemesi ve TypeScript Web SDK (`packages/sekonic-web-sdk`)
- **Hedef:** Ölçme-Değerlendirme web uygulamanızın içine doğrudan gömülebilecek, hem Chrome/Edge'de **sıfır kurulum WebUSB/WebSerial** ile çalışan hem de masaüstünde `sekonic-bridge` kuruluysa otomatik ona bağlanan hafif TypeScript + WASM SDK'sını üretmek.
- **Oluşturulacak Dosyalar:**
  - `packages/sekonic-web-sdk/Cargo.toml`
  - `packages/sekonic-web-sdk/src/lib.rs` (`wasm-bindgen` bağlamaları)
  - `packages/sekonic-web-sdk/src/index.ts` (`SekonicClient` sınıfı)
  - `packages/sekonic-web-sdk/package.json` & `tsconfig.json`
- **Uygulama Talimatları:**
  1. `SekonicClient.connect({ preferBridge: true })` çağrıldığında:
     - Önce `ws://127.0.0.1:17441/v1/stream` adresinde yerel `sekonic-bridge` çalışıyor mu diye 300 ms zaman aşımıyla kontrol et. Varsa tüm tarayıcılarda (Safari, Firefox, Chrome, Edge) köprü modunu kullan.
     - Yoksa ve tarayıcı `navigator.usb` veya `navigator.serial` destekliyorsa, `wasm-bindgen` ile derlenen Rust çekirdeğini (`sekonic-proto-omr` + `sekonic-proto-cis` + `sekonic-grader`) doğrudan tarayıcı sekmesinde çalıştır!
  2. Ek yardımcı fonksiyon: `renderSheetHeatmapCanvas(canvas, rawMatrixB64, formSchema, highlights)` — 3 KB'lık ham analog matristen HTML5 `<canvas>` üzerine gerçek zamanlı sanal optik form ve şüpheli baloncuk önizlemesi çizen hafif görselleştirici!
- **Zorunlu Doğrulama Kapısı:**
  ```powershell
  cargo check --manifest-path packages/sekonic-web-sdk/Cargo.toml --target wasm32-unknown-unknown
  ```
  *(Sistemde `wasm-pack` ve `npm` varsa ayrıca `wasm-pack build` ve `npm run typecheck` çalıştırılır.)*

---

### WP-11A: Tauri v2 Backend & Yerel WebSocket Köprüsü (`apps/sekonic-bridge/src-tauri`)
- **Hedef:** Windows (x64/ARM64) ve macOS (Apple Silicon/Intel) üzerinde < 25 MB RAM tüketen, hem sistem tepsisinde (System Tray) yerel donanım köprüsü olan hem de internet yokken SQLite WAL ile tek başına çalışan Tauri v2 Rust backend'ini kurmak.
- **Oluşturulacak Dosyalar:**
  - `apps/sekonic-bridge/src-tauri/{Cargo.toml, tauri.conf.json, src/main.rs, src/local_ws_bridge.rs, src/scanner_manager.rs, src/ipc_commands.rs}`
- **Uygulama Talimatları:**
  1. `local_ws_bridge.rs`: Yalnızca `127.0.0.1:17441` adresini dinleyen ve `Origin` başlığı doğrulaması (CORS / izinli domain listesi) yapan yerel WebSocket sunucusunu kur.
  2. `scanner_manager.rs`: USB tak-çıkar (hotplug) izleme, gerçek cihaz yoksa arayüzden tek tıkla **"Sanal SR-6500 / SR-55D Simülatörünü Bağla"** modunu açabilme.
  3. `ipc_commands.rs`: Frontend'e `start_scan`, `stop_scan`, `set_form_schema`, `override_sheet_answer`, `toggle_simulator_mode`, `get_offline_queue_stats` Tauri komutlarını aç ve okunan her kağıdı `sheet:scanned` Tauri event'i ile yayınla.
  4. Sistem Tepsisi (Tray Icon): Pencere kapatıldığında arka planda ~12 MB RAM ile sessizce çalışmaya devam etme ve tepsi menüsünden durum gösterme ("Sekonic SR-3500 Hazır — Senkronize").
- **Zorunlu Doğrulama Kapısı:**
  ```powershell
  cargo check --manifest-path apps/sekonic-bridge/src-tauri/Cargo.toml
  ```

---

### WP-11B: Frontend Uygulaması — `uxiGL` + `SheetGL` + `<canvas>` (`apps/sekonic-bridge/src`)
- **Hedef:** Görsel tasarım referansları (`docs/mockups/01_mission_control_cockpit.jpg`, `02_heatmap_erasure_review_modal.jpg`, `03_dynamic_form_designer.jpg`) ile birebir uyumlu, **uxiGL** (`https://uxigl.truncgil.com/`) tasarım sistemi ve **SheetGL** (`https://sheetgl.truncgil.com/`) WebGL/Canvas DataGrid'i üzerine kurulu, sıfır DOM şişmeli, klavye odaklı yüksek hızlı operatör arayüzünü geliştirmek.
- **Oluşturulacak Frontend Dosya Ağacı (`apps/sekonic-bridge/src/`):**
  ```text
  apps/sekonic-bridge/src/
  ├── main.tsx                         # React 19 + uxiGL ThemeProvider (Dark/Liquid Glass) giriş noktası
  ├── App.tsx                          # Üst Operasyon Barı + 3 Ana Görünüm (Kokpit / Form Tasarımcısı / Ayarlar)
  ├── stores/
  │   └── useScannerStore.ts           # Zustand/Context store: cihaz durumu, aktif sınav şeması, KPI sayaçları, satır tamponu
  ├── services/
  │   └── bridgeClient.ts              # Tauri IPC (masaüstü) veya @kurum/sekonic-web-sdk (web) soyutlama adaptörü
  ├── components/
  │   ├── layout/
  │   │   └── TopControlBar.tsx        # uxiGL Header: Cihaz durum rozeti (SR-6500 USB), Sınav seçici, Başlat/Durdur, Offline/Online senkron rozeti
  │   ├── cockpit/
  │   │   ├── DeviceTelemetryCard.tsx  # uxiGL Card + Switch: Hız (SPM), Hopper durumu, Dropout LED seçimi, "Sanal Simülatör Modu" anahtarı
  │   │   ├── KpiStrip.tsx             # 4'lü uxiGL özet kartı: Okunan, Ana Tepsi (Geçerli), Alt Tepsi (Red), Silgi Kurtarılan
  │   │   ├── LiveScanGrid.tsx         # SheetGL DataGrid: 100.000+ satırı 60 FPS listeleyen canlı okuma & veri önizleme tablosu
  │   │   ├── SheetCanvasPreview.tsx   # HTML5 <canvas>: Sol altta seçili kağıdın 0-255 analog yoğunluk ısı haritası minyatürü
  │   │   └── ExceptionAlertBar.tsx    # Alt sabit uyarı çubuğu: "18 kağıt Alt Tepsi'ye ayrıldı -> [İncele (F2)]"
  │   ├── review/
  │   │   ├── HeatmapReviewModal.tsx   # uxiGL Modal: Şüpheli/Silgili kağıt inceleme (Sol: Yakınlaştırılmış Canvas, Sağ: 0-255 Bar Grafik & Kısayollar)
  │   │   └── DensityBarChart.tsx      # Seçili soru için A/B/C/D/E 0-255 koyuluk çubuk grafiği + eşik çizgisi
  │   └── designer/
  │       ├── FormDesignerView.tsx     # Dinamik JSON Şema Tasarımcısı (Sıfır .fmt): 48x64 Canvas ızgara + sürükle-bırak alan seçimi
  │       └── FieldInspectorPanel.tsx  # uxiGL Input/Slider/Switch: Alan tipi, satır/sütun aralığı, min_mark_density & erasure_ratio kaydırıcıları
  ```
- **Uygulama Talimatları:**
  1. **Kurulum & Kütüphaneler:** `apps/sekonic-bridge` dizininde React + TypeScript + Vite projesi içinde **uxiGL** (`npx uxigl init` / NPM paketi) ve **SheetGL** (`https://sheetgl.truncgil.com/`) paketlerini kur.
  2. **Canlı Okuma & Veri Önizleme (`LiveScanGrid.tsx` — SheetGL):**
     - Sütunlar: `#Sıra`, `Öğrenci No`, `Ad Soyad`, `Kitapçık`, `Tepsi (Main/Sub)`, `Güven (%)`, `Durum` (`Tamam`, `Silgi Kurtarıldı`, `Çift İşaret`, `Kayıtsız`), `Cevap Özeti`.
     - `SheetGL` üzerinde satır içi düzenleme (`canEdit`), anlık filtreleme (ör. yalnızca `Silgi` veya `Alt Tepsi` olanları süz), toplu düzeltme (Batch CRUD) ve `Ctrl+Z` Undo/Redo aktif olmalıdır.
     - Bir satıra tıklandığında sol paneldeki `SheetCanvasPreview.tsx` anında güncellenir; çift tıklandığında veya `F2` basıldığında `HeatmapReviewModal.tsx` açılır.
  3. **0–255 Analog Isı Haritası (`SheetCanvasPreview.tsx` & `HeatmapReviewModal.tsx`):**
     - `raw_matrix_b64` çözülerek her `(row, col)` baloncuğu `<canvas>` üzerine çizilir:
       - Net işaret (`>= strong_mark_density`): **Parlak Yeşil dolu halka**
       - Silgi izi (`erasure_detected`): **Turuncu kesikli halka** (`Silinen: 52` vs `Seçilen: 210` etiketiyle)
       - Çift işaret / Belirsiz (`MultiMark`): **Kırmızı vurgulu halka**
     - Modal içinde klavye kısayolları zorunludur: `A/B/C/D/E` ile cevabı düzelt, `Space` veya `Enter` ile onayla ve sonraki şüpheli kağıda geç, `S` ile Alt Tepsiye işaretle, `Esc` ile kapat.
  4. **Dinamik Form Tasarımcısı (`FormDesignerView.tsx`):**
     - `.bldm` / `.fmt` içe aktarma butonu (`sekonic-schema` WASM/IPC dönüştürücüsünü çağırır) ve JSON Şema dışa aktarma / Ölçme App'e gönderme butonu içermelidir.
     - 48×64 ızgara üzerinde fareyle sürükleyerek yeni alan (`Öğrenci No`, `TC Kimlik`, `Kitapçık`, `Branş Soruları`) tanımlanabilmeli; sağdaki `FieldInspectorPanel` içinde `uxiGL Slider` ile `min_mark_density`, `erasure_ratio` ve `review_confidence_floor` canlı ayarlanabilmelidir.
- **Zorunlu Doğrulama Kapısı:**
  ```powershell
  npm --prefix apps/sekonic-bridge run typecheck
  npm --prefix apps/sekonic-bridge run build
  ```

---

### WP-12: Windows WinUSB Sürücü Kurulum Yardımcısı, macOS Paketleme ve Tam Sistem Stres Testi
- **Hedef:** Saha kurulumunu sıfır zahmete indirmek ve tüm sistemi uçtan uca stres testinden geçirmek.
- **Oluşturulacak Dosyalar:**
  - `apps/sekonic-bridge/drivers/sekonic-winusb.inf` (Tüm `VID_0A41` + `PID_0002, 1004, 1008, 2000, 2002, 2003, 3005, 5000` cihazlarını tek bir modern Microsoft WinUSB sürücüsünde birleştiren evrensel INF şablonu)
  - `tests/e2e_stress_test.rs`
  - `.github/workflows/build-release.yml` (Windows `.msi`/`.exe`, macOS Universal `.dmg` ve Web SDK `.tgz` derleme otomasyonu)
- **Uygulama Talimatları:**
  1. `e2e_stress_test.rs`:
     - Sanal `SR-6500` üzerinde **6.500 kağıt/saat hızında 2.000 ardışık kağıt** (içinde 150 silgi izli kağıt, 30 kayıtsız öğrenci, 15 çift işaretli kağıt ve okuma ortasında 5 saniyelik ağ kesintisi bulunan) tam boru hattı stres testi koştur.
     - Kabul kriterleri:
       - Silgi izli 150 kağıdın **%100'ü** iptal edilmeden doğru şıkla kurtarılmalı.
       - Kayıtsız 30 öğrencinin **%100'ü** `Sub-Stacker`'a ayrılmalı.
       - Ağ kesintisine rağmen 2.000 kağıdın **2.000'i de eksiksiz ve mükerrersiz** hedef sunucuya senkronize olmalı.
       - Toplam bellek artışı (memory leak kontrolü) sabit kalmalı!
- **Zorunlu Doğrulama Kapısı:**
  ```powershell
  cargo test --workspace --release
  ```

---

## BÖLÜM 5: Gemini Hydrogen İçin Kopyala-Yapıştır Başlatma Komutları (Supervisor Prompts)

Bu planı **Gemini Hydrogen** (veya herhangi bir hızlı kodlama modeli) ile çalıştırırken aşağıdaki hazır komutları doğrudan sohbete yapıştırabilirsiniz:

### 1. İlk Başlatma Komutu (Oturum Açılışında Yapıştırın)
```text
@[d:\Works\2026\seconic\docs\sekonic_implementation_plan.md] dosyasını baştan sona dikkatle oku. Sen bu projenin uygulayıcı mühendisisin ve bu doküman senin Master Supervisor Anayasandır.
BÖLÜM 0'daki tüm kurallara (%100 sıfır unsafe, üretim kodunda sıfır unwrap/expect, halüsinasyon yasağı, tek seferde tek WP kuralı) harfiyen uyacaksın.
Önce PROGRESS.md dosyasını kontrol et (yoksa oluştur) ve yalnızca WP-01 ile WP-02'yi eksiksiz uygula, zorunlu doğrulama kapısı komutlarını çalıştırıp sonuçlarını bana raporla. Doğrulama geçmeden WP-03'e geçme.
```

### 2. Sonraki Adımlara Geçiş Komutu (Her Onaydan Sonra Yapıştırın)
```text
@[d:\Works\2026\seconic\docs\sekonic_implementation_plan.md] ve @[d:\Works\2026\seconic\PROGRESS.md] dosyalarını oku.
Sıradaki tamamlanmamış İş Paketini (WP) BÖLÜM 0 kurallarına ve BÖLÜM 3 veri sözleşmelerine birebir sadık kalarak uygula.
İş paketinin sonundaki Zorunlu Doğrulama Kapısı (Verification Gate) komutlarını çalıştır, tüm testler yeşil (exit code 0) olduğunda PROGRESS.md dosyasını güncelle ve bana özet rapor ver.
```
