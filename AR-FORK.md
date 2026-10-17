# WordCraft AR — وورد كرافت العربي

نسخة مستقلة (fork) من [WordCraft](https://github.com/storytold/wordcraft) تضيف الطباعة العربية
وأدوات «كتابات الخدمة» العسكرية كجزء مدمج (built-in) من البرنامج. هويتها منفصلة عن الأصل كي لا يمحو
تحديثٌ للأصل ما بُني هنا، ويمكن تثبيت النسختين جنبًا إلى جنب:

| | WordCraft (الأصل) | WordCraft AR (هذه النسخة) |
|---|---|---|
| اسم النافذة | WordCraft | WordCraft AR |
| معرّف التطبيق | ai.storyteller.wordcraft | ai.storyteller.wordcraft-ar |
| مجلد الإعدادات | %APPDATA%\WordCraft | %APPDATA%\WordCraft-AR |
| فرع git | main | ar-main (الأصل محفوظ كـ remote باسم upstream) |

الثوابت كلها في `crates/control-key/src/lib.rs` (`PRODUCT`, `PRODUCT_AR`, `APP_ID`, `SETTINGS_FOLDER`).

## ما أُضيف على الأصل

### 1. ترقيم عربي للقوائم — `crates/doc/src/arabic_numbers.rs`, `section.rs`, `numbering.rs`
- **أبجد هوز بأسلوب كتابات الخدمة**: أ، ب، جـ، د، هـ، و، ز، ح، ط، ي، ك، ل… (جـ وهـ بمدّة واحدة).
- **الأبجدية**: أ، ب، ت، ث… — **الأرقام العربية الهندية**: ١، ٢، ٣ — **الترتيب بالكلمات**: أولاً،
  ثانياً، … حادي عشر… ثلاثون.
- تُحفظ وتُقرأ بصيغ OOXML القياسية (`arabicAbjad`, `arabicAlpha`, `hindiNumbers`, `ordinalText` مع لغة
  عربية) فتفتحها Word كما هي.
- قوالب جاهزة في معرض الترقيم (Home › Paragraph): «١. ٢. ٣.»، «أ. ب. جـ.»، وفي القوائم متعددة
  المستويات «١. أ. أولاً. (١).» — دورة كتابات الخدمة التسعة مستويات (`ListKind::Service`).

### 2. ضبط بالكشيدة — `crates/layout/src/kashida.rs`, `para.rs`, `display.rs`
- المحاذاة الموزّعة (Ctrl+Shift+J) وقيم Word `lowKashida` / `mediumKashida` / `highKashida` تملأ السطر
  بالكشيدة داخل الكلمات العربية بدل توسيع المسافات فقط.
- مدّة واحدة في كل كلمة، قبل الحرف الأخير غالبًا (أسلوب كتابات الخدمة)، ولا تُكسر لام-ألف ولا تُمطّ
  الحروف غير القابلة للوصل. تُرسم بحرف التطويل من خط النص نفسه.
- متاحة في لائحة الفقرة (Paragraph › Alignment): Distribute / Low / Medium / High Kashida.

### 3. تبويب «Service» (كتابات الخدمة) في الشريط — `crates/engine/src/cmd/service.rs`
| الأمر | ما يفعله |
|---|---|
| `service.renumber` | يقرأ العلامات المكتوبة (١. / أ. / أولاً. / (١) …) ويعيد ترقيمها بالدورة ١. → أ. → أولاً. → (١). → (أ). → (أولاً). مع مسافات بادئة معلّقة بحيث يبدأ كل مستوى تحت نقطة المستوى الأعلى؛ أول مستوى موجود في المستند يصير «١.» (فصلٌ مقتطَع يُرقَّم من جديد). |
| `service.applyList` | نفس القراءة لكن تحوّل العلامات إلى قائمة متعددة المستويات حقيقية (`ListKind::Service`). |
| `service.classification` | درجة الكتمان (عام / محدود / سري / سري وشخصي / سري للغاية) في وسط الرأس والتذييل، مع سطر الصفحات. |
| `service.pageNumbers` | «( رقم - الأخيرة )» بحقول `PAGE \* HindiArabic` و `SECTIONPAGES \* HindiArabic`. |
| `service.kashida` | يضبط المحاذاة: off / low / medium / high / distribute. |
| `service.finalDot` | يختم كل بند مرقّم بنقطة. |
| `service.external` | يرسل المستند إلى محرك **ServiceWord-AR** (Python) ويعيده منسقًا: قياس بالخطوط الحقيقية، توحيد أعمدة الترقيم بالحبر، لام-ألف مشتقة لـ«أولاً»، خط أرقام مضمّن، وتحويل PDF للطباعة. يبحث عن `cli.py` في `%WORDCRAFT_AR_ENGINE%` ثم مجلد `servicewordar` بجانب البرنامج ثم مجلد الإعدادات. |

الأوامر كلها تعمل من الشريط، ومن سطر الأوامر (`wordcraft-cli run --cmd service.renumber`), ومن MCP.

## التحديث الهجين من الأصل (بإشراف)
```
scripts/sync-upstream.sh            # يجلب upstream/main ويدمجه في ar-main ويشغّل الاختبارات
```
السياسة: الملفات التي تملكها هذه النسخة (`scripts/ar-owned.txt`) تُحسم التعارضات فيها لصالح النسخة
العربية ثم تُراجع يدويًا؛ ما عداها يُؤخذ من الأصل. لا يُدفع (push) أي شيء إلى الأصل. بعد كل دمج:
`cargo test -p wordcraft-doc -p wordcraft-layout -p wordcraft-engine -p wordcraft-ui-egui`.

## البناء على Windows
1. ثبّت Rust من https://rustup.rs (يثبّت أدوات MSVC عند الطلب).
2. شغّل `build-windows.bat` — الناتج `dist\WordCraft AR.exe` (أول بناء ١٠–٢٥ دقيقة).
3. ضع مجلد `servicewordar` (محرك Python) بجانب الملف التنفيذي كي يعمل زر «Format with ServiceWord-AR».

على لينكس/ماك: `cargo build --release -p wordcraft`.

## ما بعد (خارطة الطريق العربية)
- واجهة عربية كاملة (`crates/ui-egui/src/i18n/ar.tsv`، ~٢٢٠٠ عبارة) مع ترتيب بصري للكلمات في egui.
- نقل حسابات ServiceWord-AR (توحيد الأعمدة بالحبر، لام-ألف المشتقة) إلى Rust داخل `layout`.
- تدقيق إملائي عربي.
