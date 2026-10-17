<div dir="rtl" align="right">

<h1 align="center">WordCraft AR — وورد كرافت العربي</h1>

<p align="center">
  <b>طباعة عربية وأدوات «كتابات الخدمة» العسكرية مدمجة في WordCraft — معالج النصوص مفتوح المصدر المكتوب بلغة Rust.</b>
</p>

<p align="center">
  <a href="README.md"><b>🇬🇧 Read this in English</b></a>
</p>

<p align="center">
  <img alt="Fork of storytold/wordcraft" src="https://img.shields.io/badge/fork%20of-storytold%2Fwordcraft-2b47b5?style=flat-square&logo=github&logoColor=white">
  <img alt="Written in Rust" src="https://img.shields.io/badge/written%20in-Rust-2b47b5?style=flat-square&logo=rust&logoColor=white">
  <img alt="Runs on Windows, macOS, Linux" src="https://img.shields.io/badge/runs%20on-Windows%20%C2%B7%20macOS%20%C2%B7%20Linux-3b5bdb?style=flat-square">
  <img alt="License: MIT OR Apache-2.0" src="https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-2b47b5?style=flat-square">
</p>

> [!IMPORTANT]
> **هذه نسخة مشتقّة (fork).** WordCraft AR مبنيّ على
> [WordCraft](https://github.com/storytold/wordcraft) من فريق ArtCraft ومساهمي WordCraft — وهو إعادة
> بناء مفتوحة المصدر لمايكروسوفت وورد بلغة Rust. كل ما يجعله معالج نصوص (المحرك، التخطيط، قراءة وكتابة
> `.docx`، الشريط، تعقّب التغييرات، المراجع، دمج المراسلات…) هو عملهم، ويبقى تحت رخصتهم وبأسمائهم.
> الملف التعريفي الأصلي محفوظ كما هو في [`docs/README-upstream.md`](docs/README-upstream.md).
> تضيف هذه النسخة ما يحتاجه الكاتب العربي — وكتّاب الدوائر العسكرية والحكومية العراقية خصوصًا — كل يوم،
> ولا يُدفع شيء إلى المستودع الأصلي إلا عبر قنواته.

<br>

<p align="center">
  <img src="docs/images/ar/service-tab.png" alt="WordCraft AR وتبويب Service مفتوح فوق مستند كتابات خدمة عربي: دورة الترقيم (١. ثم أ. ثم أولاً.) مطبّقة، ومجموعة الكشيدة بدرجاتها، وزر التنسيق بمحرك ServiceWord-AR" width="100%">
  <br><sub><b>تبويب الخدمة</b> (كتابات الخدمة): دورة الترقيم، ختم درجة الكتمان، ترقيم الصفحات بالأرقام الهندية، الضبط بالكشيدة، والتسليم إلى محرك ServiceWord-AR.</sub>
</p>

## ما تضيفه هذه النسخة

| المجال | ما تحصل عليه |
|---|---|
| **ترقيم عربي** | حروف الأبجد (أ، ب، جـ، د، هـ…)، الأرقام الهندية (١، ٢، ٣)، والترتيب اللفظي (أولاً، ثانياً… حتى الثلاثين) كصيغ قوائم Word حقيقية تُحفظ في `.docx` قياسي يفتحه Microsoft Word دون تغيير. معرض الترقيم يعرضها بمعاينة حيّة. |
| **دورة كتابات الخدمة** | نقرة واحدة تعيد ترقيم المستند كله (أو التحديد) بدورة كتابات الخدمة العراقية ذات التسعة مستويات — `١.` ← `أ.` ← `أولاً.` ← `(١)` ← … — بقراءة العلامات التي كتبتها يدويًا أو قوائم Word الموجودة أصلًا، مع مسافات معلّقة لكل مستوى. |
| **الضبط بالكشيدة** | كشيدة منخفضة ومتوسطة ومرتفعة وتوزيع كمحاذاة للفقرة، تُرسم من خط المستند نفسه (مدّة واحدة في الكلمة قبل توسيع المسافات)، وتُقرأ وتُكتب بصيغ Word: `lowKashida` / `mediumKashida` / `highKashida` / `distribute`. |
| **درجة الكتمان وأرقام الصفحات** | «محدود»، «سري»… مختومة في الرأس والتذييل، وأرقام صفحات `( ١ - ٥ )` بالأرقام الهندية كحقول `PAGE \* HindiArabic`. |
| **افتراضيات عربية** | المستند الفارغ الجديد من اليمين إلى اليسار بخط Times New Roman حجم ١٤ ولغة النص المركّب العربية؛ والأرقام تظهر هندية بجانب النص العربي (خيار Word: *Numeral: Context*). كلاهما خيار قابل للإطفاء. |
| **واجهة عربية** | ترجمة كاملة للواجهة (أكثر من ٢٦٠٠ عبارة)، خط واجهة عربي مضمّن (Noto Sans Arabic)، وترتيب صحيح من اليمين إلى اليسار للعناوين التي تخلط العربية بأرقام أو أسماء لاتينية. الواجهة تتبع لغة النظام وتُبدَّل في أي وقت من *File › Options*. |
| **مساعدة مصوّرة** | كل زر في تبويبي الخدمة والمراسلات يعرض عند التحويم صورة مصغّرة مرسومة لما يفعله وجملة شرح — بلغة الواجهة، وبالعربية أيضًا حين تكون الواجهة بلغة أخرى. |
| **محرك ServiceWord-AR** | محرك Python الذي يقيس الخطوط الحقيقية ويوحّد العلامات ويكتب الكشيدة داخل النص يأتي بجانب البرنامج (`servicewordar/`) وعلى بعد زر واحد: *Format with ServiceWord-AR*، مع نسخة الطباعة PDF أو بدونها. |
| **هوية مستقلة** | يُثبَّت WordCraft AR بجانب WordCraft: اسم نافذة خاص، معرّف تطبيق (`ai.storyteller.wordcraft-ar`)، مجلد إعدادات (`%APPDATA%\WordCraft-AR`)، ورمز ترقية خاص بالمثبّت. |

<table>
<tr>
<td width="50%" valign="top"><img src="docs/images/ar/new-document.png" alt="مستند جديد في WordCraft AR بواجهة إنجليزية: النص العربي يبدأ من الهامش الأيمن بخط Times New Roman حجم ١٤، والرقم 120 يظهر هنديًا ١٢٠، وزر الفقرة من اليمين إلى اليسار مفعّل" width="100%"><p align="center"><sub><b>المستند الجديد</b> عربي من أول ضغطة: من اليمين إلى اليسار، Times New Roman ١٤، أرقام هندية — والواجهة بأي لغة تفضّلها.</sub></p></td>
<td width="50%" valign="top"><img src="docs/images/ar/visual-help.png" alt="أربعة تلميحات في WordCraft AR: إعادة ترقيم الدورة، كشيدة منخفضة، تمييز حقول الدمج، والملصقات؛ كل واحدة بصورة مصغّرة مرسومة لما يفعله الزر وجملة إنجليزية والجملة نفسها بالعربية" width="100%"><p align="center"><sub><b>مساعدة مصوّرة.</b> حوّم فوق أي زر في الخدمة أو المراسلات لترى صورة مصغّرة وشرحًا وترجمته العربية.</sub></p></td>
</tr>
<tr>
<td colspan="2"><img src="docs/images/ar/arabic-interface.png" alt="WordCraft AR بالواجهة العربية: تبويبات الشريط ملف، الرئيسية، إدراج…، ومجموعات تبويب الرئيسية الحافظة، الخط، الفقرة، الأنماط، وشريط الحالة يقرأ الصفحة 1 من 2 و220 كلمة" width="100%"><p align="center"><sub><b>الواجهة العربية</b> على بعد خيار واحد. العناوين التي تخلط العربية بالأرقام تُقرأ صحيحة («الصفحة 1 من 2»، «220 كلمة»).</sub></p></td>
</tr>
</table>

كل ما يفعله WordCraft الأصلي ما زال هنا: الشريط، الأنماط، الجداول، تعقّب التغييرات، التعليقات،
المراجع، دمج المراسلات، تصدير PDF، أداة سطر الأوامر، وخادم MCP. انظر
[الملف التعريفي الأصلي](docs/README-upstream.md) للجولة الكاملة.

## التثبيت

### Windows (الحالة الشائعة)

نزّل `WordCraft-AR-<version>-windows-x64.msi` من صفحة [الإصدارات](../../releases) وشغّله. المثبّت
مكتفٍ بذاته (لا يحتاج Visual C++ Redistributable ولا إنترنت)، فيمكن نقله على فلاش إلى جهاز غير
متصل. يُثبَّت بجانب WordCraft إن وُجد، ويضيف إدخالًا في قائمة ابدأ واختصارًا على سطح المكتب، ويضع
محرك ServiceWord-AR بجانب البرنامج.

زر *Format with ServiceWord-AR* وحده يحتاج Python 3 على الجهاز (بقية البرنامج لا تحتاجه). على جهاز
غير متصل: ثبّت Python من مثبّته ثم شغّل `servicewordar\install_offline.bat` الذي يثبّت مكتبات
المحرك من حزم wheels المرفقة.

### macOS وLinux

ابنِ من المصدر (أدناه)؛ سكربتات التغليف الأصلية في `packaging/` تعمل كما هي عدا اسم المنتج.

## البناء من المصدر

```
git clone https://github.com/<you>/wordcraft-ar.git
cd wordcraft-ar
cargo build --release -p wordcraft -p wordcraft-cli
```

يلزم Rust مستقر (١٫٩٥ فأحدث)، وعلى Windows أدوات MSVC Build Tools (مثبّت rustup يعرضها). على
Windows يقوم `build-windows.bat` بما سبق ويجمع البرنامج والأداة والمحرك في `dist\`. البناء الأول
يستغرق ١٠–٢٥ دقيقة، وما بعده تدريجي.

اختبارات السلوك العربي: `cargo test -p wordcraft-doc -p wordcraft-layout -p wordcraft-engine -p wordcraft-ui-egui`.

## تبويب الخدمة

| الزر | الأمر | ما يفعله |
|---|---|---|
| Renumber Cycle | `service.renumber` | يقرأ العلامات المكتوبة أو قوائم Word ويعيد ترقيم كل بند بالدورة مستوى بعد مستوى مع مسافات معلّقة. *Selection Only* يحصره بالفقرات المحددة. |
| Service List | `service.applyList` | يحوّل العلامات المكتوبة إلى قائمة Word متعددة المستويات ترقّم نفسها. |
| Final Dot | `service.finalDot` | يختم كل بند بنقطة. |
| Classification | `service.classification` | يختم درجة الكتمان في الرأس والتذييل (`محدود` افتراضيًا). |
| Page Number | `service.pageNumbers` | يكتب `( الصفحة - المجموع )` بالأرقام الهندية أسفل كل صفحة. |
| Justify · Low · Medium · High · Distribute | `service.kashida` | درجات الكشيدة والتوزيع للفقرات الحالية. |
| Format with ServiceWord-AR · With PDF | `service.external` | يسلّم المستند إلى محرك Python ويفتح النتيجة؛ واختياريًا نسخة الطباعة PDF المختومة. |

الدورة والكشيدة وصيغ الترقيم مشروحة بالتفصيل في [`AR-FORK.md`](AR-FORK.md).

## للوكلاء والسكربتات

كل زر أمر، فتبويب الخدمة كله قابل للبرمجة من سطر الأوامر وعبر MCP تمامًا كما في WordCraft الأصلي:

<div dir="ltr">

```
wordcraft-cli run --file letter.docx \
  --cmd service.renumber \
  --cmd 'service.classification={"value":"محدود"}' \
  --cmd 'service.kashida={"value":"low"}' \
  --save letter-formatted.docx
wordcraft-cli convert letter-formatted.docx letter-formatted.pdf
```

</div>

`wordcraft-cli commands | grep service.` يسردها مع معاملاتها.

## مواكبة الأصل

WordCraft الأصلي يتطور بسرعة، وهذه النسخة مصممة لتتبعه لا لتبتعد عنه. الفرع `ar-main` يحمل النسخة
العربية، و`upstream` هو المستودع الأصلي.

<div dir="ltr">

```
scripts/sync-upstream.sh      # يجلب upstream/main ويدمجه في ar-main ويشغّل الاختبارات العربية
```

</div>

التعارضات في الملفات التي تملكها هذه النسخة (`scripts/ar-owned.txt`) تُحسم لصالحها وتُراجع يدويًا؛
ما عداها يُؤخذ من الأصل.

## خارطة الطريق

- شريط من اليمين إلى اليسار (الواجهة مترجمة، لكن egui يرصّ العناصر من اليسار إلى اليمين).
- نقل حسابات ServiceWord-AR (توحيد العلامات بالحبر، لام-ألف المشتقة) من Python إلى تخطيط Rust.
- تدقيق إملائي عربي.
- مراجعة الكتالوج العربي من مستخدمين عرب أكثر — كل عبارة سطر واحد في `crates/ui-egui/src/i18n/ar.tsv`.

## الرخصة والشكر

WordCraft AR مرخّص، مثل WordCraft، بـ **رخصة MIT أو رخصة Apache 2.0 حسب اختيارك**
([LICENSE-MIT](LICENSE-MIT)، [LICENSE-APACHE](LICENSE-APACHE)).

- **[WordCraft](https://github.com/storytold/wordcraft)** — حقوق فريق ArtCraft ومساهمي WordCraft.
  معالج النصوص الذي بُنيت عليه هذه النسخة؛ شكرًا لأنكم جعلتموه مفتوحًا. اسم ArtCraft وشعاراته
  علامات تجارية لهم ولا تستعملها هذه النسخة.
- **WordCraft AR** — الطباعة العربية، تبويب الخدمة، الواجهة العربية والمساعدة المصوّرة: يونس (العراق)
  مع Claude (Anthropic) شريكًا في البرمجة.
- **ServiceWord-AR** — محرك Python لتنسيق كتابات الخدمة العراقية، للمؤلف نفسه.
- الخطوط: Noto Sans Arabic وNoto Naskh Arabic (Google، رخصة SIL Open Font 1.1) إلى جانب خطوط الأصل؛
  كل ملف مرفق مذكور برخصته في [`ATTRIBUTION.md`](ATTRIBUTION.md).

إن أفادتك هذه النسخة فضع نجمة على [المشروع الأصلي](https://github.com/storytold/wordcraft) أيضًا —
ما كان لشيء من هذا أن يوجد لولاه.

</div>
