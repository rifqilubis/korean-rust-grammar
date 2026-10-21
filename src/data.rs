//! Bank materi tata bahasa Korea.
//!
//! 12 topik tersusun dari tingkat dasar sampai lanjutan. Semua contoh memakai
//! bentuk sopan informal (해요체) agar konsisten dan langsung bisa dipakai.
//!
//! Data bersifat `static`, jadi `kg list` maupun `kg show 1` tidak melakukan
//! alokasi untuk memuat materi.

use crate::model::{e, q, r, Block, Level, Topic};

/// Seluruh topik, sudah terurut berdasarkan nomor.
pub static TOPICS: &[Topic] = &[
    /* ------------------------------------------------------------ 1. 한글 */
    Topic {
        id: "hangeul",
        no: 1,
        title: "한글",
        subtitle: "Huruf Korea: dasar membaca",
        level: Level::Dasar,
        icon: "한",
        intro: "Huruf Korea dibaca per suku kata: konsonan awal + vokal + konsonan akhir (받침).",
        blocks: &[
            Block::Heading("Konsonan dasar (자음)"),
            Block::Text(
                "Ada 14 konsonan dasar. Bentuk yang lebih tegas berarti bunyinya dihembuskan \
                 (aspirasi); bentuk rangkap berarti tegang.",
            ),
            Block::Rules(&[
                r("ㄱ g", "ㅋ k (hembus)", "ㄲ kk (tegang)"),
                r("ㄷ d", "ㅌ t (hembus)", "ㄸ tt (tegang)"),
                r("ㅂ b", "ㅍ p (hembus)", "ㅃ pp (tegang)"),
                r("ㅅ s", "ㅆ ss (tegang)", "ㅈ j / ㅉ jj / ㅊ ch"),
                r("ㅁ m", "ㄴ n", "ㄹ r/l / ㅎ h"),
                r("ㅇ (awal)", "tidak berbunyi", "hanya tempat vokal"),
            ]),
            Block::Heading("Vokal dasar (모음)"),
            Block::Rules(&[
                r("ㅏ a", "ㅑ ya", "ㅓ eo (seperti 'o' ringan)"),
                r("ㅕ yeo", "ㅗ o", "ㅛ yo"),
                r("ㅜ u", "ㅠ yu", "ㅡ eu (mulut melebar)"),
                r("ㅣ i", "ㅐ e", "ㅔ e (hampir sama)"),
                r("ㅘ wa", "ㅝ wo", "ㅢ ui"),
            ]),
            Block::Formula("suku kata = 초성 + 중성 (+ 종성) — mis. ㅎ + ㅏ + ㄴ = 한"),
            Block::Examples(&[
                e("한글", "hangeul", "huruf Korea"),
                e("한국어", "hangugeo", "bahasa Korea"),
                e("사람", "saram", "orang"),
                e("물", "mul", "air"),
            ]),
            Block::Note(
                "Bila suku kata berakhiran konsonan dan suku kata berikutnya dimulai vokal, \
                 konsonan itu pindah bunyi: 옷이 dibaca [오시] osi.",
            ),
        ],
        quiz: &[
            q(
                "Suku kata 한 terdiri dari konsonan dan vokal apa?",
                ["ㅎ + ㅏ + ㄴ", "ㅎ + ㅏ", "ㄱ + ㅏ + ㄴ", "ㅎ + ㅓ + ㄴ"],
                0,
                "한 = ㅎ (초성) + ㅏ (중성) + ㄴ (종성).",
            ),
            q(
                "Bunyi konsonan akhir (받침) pada 밥 adalah …",
                ["p", "b", "m", "n"],
                0,
                "받침 ㅂ pada akhir suku kata dilafalkan [p] tanpa dilepas.",
            ),
            q(
                "Manakah pasangan aspirasi (bunyi dihembuskan) yang benar?",
                ["ㄱ → ㅋ", "ㄱ → ㄲ", "ㄷ → ㄸ", "ㅂ → ㅃ"],
                0,
                "ㅋ adalah versi berhembus dari ㄱ; ㄲ adalah versi tegang.",
            ),
            q(
                "'한국어' dibaca …",
                ["hangugeo", "hanguksa", "hangeul", "hangug"],
                0,
                "연음: 국의 받침 ㄱ pindah menjadi awal suku kata 어 → hangugeo.",
            ),
        ],
    },
    /* -------------------------------------------------------- 2. 어순 */
    Topic {
        id: "urutan",
        no: 2,
        title: "어순",
        subtitle: "Urutan kalimat: SOV",
        level: Level::Dasar,
        icon: "어",
        intro: "Bahasa Korea menaruh kata kerja di AKHIR kalimat: Subjek + Objek + Predikat.",
        blocks: &[
            Block::Formula("S + O + V  (저는 + 밥을 + 먹어요)"),
            Block::Text(
                "Partikel penanda (은/는, 이/가, 을/를) membuat urutan S-O bisa ditukar tanpa \
                 mengubah arti. Kata kerja selalu di akhir, dan bentuk sopan berakhiran -아요/-어요.",
            ),
            Block::Examples(&[
                e("저는 밥을 먹어요.", "jeoneun babeul meogeoyo.", "Saya makan nasi."),
                e("민수는 한국어를 배워요.", "minsuneun hangugeoreul baewoyo.", "Minsu belajar bahasa Korea."),
                e("저는 커피를 마셔요.", "jeoneun keopireul masyeoyo.", "Saya minum kopi."),
                e("동생은 영화를 봐요.", "dongsaengeun yeonghwareul bwayo.", "Adik menonton film."),
            ]),
            Block::Note(
                "Keterangan waktu/ tempat biasanya di depan: 저는 아침에 빵을 먹어요 \
                 (Saya pagi-pagi makan roti).",
            ),
        ],
        quiz: &[
            q(
                "Urutan baku kalimat bahasa Korea adalah …",
                ["Subjek + Objek + Predikat", "Subjek + Predikat + Objek", "Predikat + Subjek + Objek", "Objek + Predikat + Subjek"],
                0,
                "Kata kerja (predikat) selalu berada di akhir kalimat.",
            ),
            q(
                "Susun: (먹어요 / 저는 / 밥을)",
                ["저는 밥을 먹어요", "저는 먹어요 밥을", "밥을 저는 먹어요", "먹어요 저는 밥을"],
                0,
                "S + O + V: 저는 밥을 먹어요.",
            ),
            q(
                "Kata mana yang merupakan predikat pada '동생은 영화를 봐요'?",
                ["봐요", "동생은", "영화를", "영화"],
                0,
                "봐요 (menonton) berada di akhir = predikat.",
            ),
        ],
    },
    /* ------------------------------------------------------ 3. 은/는 */
    Topic {
        id: "eun-neun",
        no: 3,
        title: "은/는",
        subtitle: "Partikel topik kalimat",
        level: Level::Dasar,
        icon: "은",
        intro: "은/는 menandai topik: 'mengenai hal itu …'. Dipilih menurut ada/tidaknya 받침.",
        blocks: &[
            Block::Formula("받침 ada → 은   |   받침 tidak ada → 는"),
            Block::Rules(&[
                r("저 + 는", "jeoneun", "saya (topik)"),
                r("학생 + 은", "haksaengeun", "murid (topik)"),
                r("물 + 은", "mureun", "air (topik)"),
                r("나 + 는", "naneun", "aku (topik)"),
            ]),
            Block::Text(
                "은/는 dipakai untuk memperkenalkan topik, memberi kontras, atau menyatakan \
                 hal umum. Bandingkan dengan 이/가 yang menekankan pelaku.",
            ),
            Block::Examples(&[
                e("저는 학생이에요.", "jeoneun haksaengieyo.", "Saya murid."),
                e("물은 있어요.", "mureun isseoyo.", "Airnya ada."),
                e("어머니는 요리를 해요.", "eomeonineun yorireul haeyo.", "Ibu memasak."),
                e("커피는 저는 마셔요.", "keopineun jeoneun masyeoyo.", "Kalau kopi, saya minum."),
            ]),
            Block::Note(
                "Jangan pakai 은/는 dua kali berturut-turut untuk topik dan subjek yang sama; \
                 cukup satu penanda per frasa.",
            ),
        ],
        quiz: &[
            q(
                "Pasangan yang benar untuk '책 (buku)' adalah …",
                ["책은", "책는", "책이", "책을"],
                0,
                "책 berakhiran 받침 ㄱ, jadi memakai 은.",
            ),
            q(
                "Lengkapi: 저___ 한국 사람이에요.",
                ["는", "은", "이", "을"],
                0,
                "저 tidak punya 받침 → 는.",
            ),
            q(
                "Fungsi utama 은/는 adalah …",
                ["menandai topik / kontras", "menandai objek", "menandai pelaku baru", "menyatakan tempat"],
                0,
                "은/는 menandai topik pembicaraan atau kontras, bukan pelaku baru.",
            ),
        ],
    },
    /* -------------------------------------------------------- 4. 이/가 */
    Topic {
        id: "i-ga",
        no: 4,
        title: "이/가",
        subtitle: "Partikel subjek pelaku",
        level: Level::Dasar,
        icon: "이",
        intro: "이/가 menandai pelaku/subjek, terutama informasi baru atau jawaban atas 'siapa/apa'.",
        blocks: &[
            Block::Formula("받침 ada → 이   |   받침 tidak ada → 가"),
            Block::Rules(&[
                r("비 + 가", "biga", "hujan (subjek)"),
                r("친구 + 가", "chinguga", "teman (subjek)"),
                r("사람 + 이", "sarami", "orang (subjek)"),
                r("책 + 이", "chaegi", "buku (subjek)"),
            ]),
            Block::Text(
                "이/가 fokus pada pelaku: 'siapa yang melakukan' atau 'apa yang terjadi'. \
                 Bandingkan: 누가 왔어요? 민수가 왔어요.",
            ),
            Block::Examples(&[
                e("누가 왔어요? 민수가 왔어요.", "nuga wasseoyo? minsuga wasseoyo.", "Siapa yang datang? Minsu yang datang."),
                e("비가 와요.", "biga wayo.", "Hujan turun."),
                e("친구가 많아요.", "chinguga manayo.", "Temannya banyak."),
                e("저는 머리가 아파요.", "jeoneun meoriga apayo.", "Saya sakit kepala."),
            ]),
            Block::Note(
                "Pada kalimat '저는 머리가 아파요', 머리가 adalah subjek dari 아프다, \
                 sedangkan 저는 adalah topik.",
            ),
        ],
        quiz: &[
            q(
                "Lengkapi: 비___ 와요.",
                ["가", "이", "는", "를"],
                0,
                "비 tidak punya 받침 → 가.",
            ),
            q(
                "Partikel yang tepat untuk 책 adalah …",
                ["이", "가", "는", "를"],
                0,
                "책 berakhiran 받침 → 이.",
            ),
            q(
                "Pertanyaan '누가 왔어요?' paling wajar dijawab dengan …",
                ["민수가 왔어요", "민수는 왔어요", "민수를 왔어요", "민수에 왔어요"],
                0,
                "Jawaban 'siapa' memakai 이/가 karena menekankan pelaku.",
            ),
        ],
    },
    /* -------------------------------------------------------- 5. 을/를 */
    Topic {
        id: "eul-reul",
        no: 5,
        title: "을/를",
        subtitle: "Partikel objek",
        level: Level::Dasar,
        icon: "을",
        intro: "을/를 menandai objek langsung dari kata kerja transitif.",
        blocks: &[
            Block::Formula("받침 ada → 을   |   받침 tidak ada → 를"),
            Block::Rules(&[
                r("밥 + 을", "babeul", "nasi (objek)"),
                r("커피 + 를", "keopireul", "kopi (objek)"),
                r("책 + 을", "chaegeul", "buku (objek)"),
                r("영화 + 를", "yeonghwareul", "film (objek)"),
            ]),
            Block::Text(
                "Kata kerja seperti 먹다 (makan), 마시다 (minum), 보다 (menonton), 배우다 \
                 (belajar) memerlukan objek berpartikel 을/를.",
            ),
            Block::Examples(&[
                e("저는 빵을 먹어요.", "jeoneun ppangeul meogeoyo.", "Saya makan roti."),
                e("물을 마셔요.", "mureul masyeoyo.", "Saya minum air."),
                e("한국어를 배워요.", "hangugeoreul baewoyo.", "Saya belajar bahasa Korea."),
                e("음악을 들어요.", "eumageul deureoyo.", "Saya mendengarkan musik."),
            ]),
            Block::Note(
                "Pada percakapan santai partikel objek sering dihilangkan: 밥 먹어 \
                 (makan nasi). Dalam bentuk sopan tetap dipakai.",
            ),
        ],
        quiz: &[
            q(
                "Lengkapi: 커피___ 마셔요.",
                ["를", "을", "가", "은"],
                0,
                "커피 berakhir vokal → 를.",
            ),
            q(
                "Partikel objek untuk 영화 adalah …",
                ["를", "을", "이", "는"],
                0,
                "영화 berakhir vokal → 를.",
            ),
            q(
                "Kalimat mana yang benar?",
                ["저는 책을 읽어요", "저는 책이 읽어요", "저는 책은 읽어요", "저는 책에 읽어요"],
                0,
                "읽다 (membaca) membutuhkan objek dengan 을/를.",
            ),
        ],
    },
    /* ----------------------------------------------------- 6. 이에요/예요 */
    Topic {
        id: "ieyo",
        no: 6,
        title: "이에요/예요",
        subtitle: "Kopula 'adalah'",
        level: Level::Dasar,
        icon: "예",
        intro: "이에요/예요 berarti 'adalah'. Bentuknya mengikuti ada/tidaknya 받침 pada kata sebelumnya.",
        blocks: &[
            Block::Formula("받침 ada → 이에요   |   받침 tidak ada → 예요"),
            Block::Rules(&[
                r("학생 + 이에요", "haksaengieyo", "adalah murid"),
                r("의사 + 예요", "uisayeyo", "adalah dokter"),
                r("물 + 이에요", "murieyo", "adalah air"),
                r("커피 + 예요", "keopiyeyo", "adalah kopi"),
            ]),
            Block::Text(
                "Pertanyaan dibentuk dengan mengubah intonasi naik atau menambahkan \
                 -이에요?/-예요?. Untuk negasi gunakan 아니에요.",
            ),
            Block::Examples(&[
                e("저는 학생이에요.", "jeoneun haksaengieyo.", "Saya murid."),
                e("이것은 책이에요.", "igeoseun chaegieyo.", "Ini buku."),
                e("그분은 의사예요.", "geubuneun uisayeyo.", "Beliau dokter."),
                e("저는 학생이 아니에요.", "jeoneun haksaengi anieyo.", "Saya bukan murid."),
            ]),
            Block::Note("아니에요 selalu tetap, tidak berubah menjadi '아니예요'."),
        ],
        quiz: &[
            q(
                "Lengkapi: 저는 의사___.",
                ["예요", "이에요", "이예요", "에요"],
                0,
                "의사 berakhir vokal → 예요.",
            ),
            q(
                "Lengkapi: 이것은 책___.",
                ["이에요", "예요", "있어요", "해요"],
                0,
                "책 berakhiran 받침 ㄱ → 이에요.",
            ),
            q(
                "Bentuk negasi yang benar adalah …",
                ["아니에요", "아니예요", "안이에요", "아녜요"],
                0,
                "아니에요 adalah bentuk baku untuk 'bukan'.",
            ),
        ],
    },
    /* ------------------------------------------------------- 7. 있어요 */
    Topic {
        id: "isseoyo",
        no: 7,
        title: "있어요/없어요",
        subtitle: "Ada / tidak ada, punya / tidak punya",
        level: Level::Dasar,
        icon: "있",
        intro: "있어요 menyatakan keberadaan atau kepemilikan; 없어요 adalah kebalikannya.",
        blocks: &[
            Block::Formula("N이/가 있어요 (ada)  |  N이/가 없어요 (tidak ada)"),
            Block::Rules(&[
                r("물이 있어요", "muri isseoyo", "ada air"),
                r("시간이 없어요", "sigani eopseoyo", "tidak ada waktu"),
                r("돈이 있어요", "doni isseoyo", "punya uang"),
                r("친구가 없어요", "chinguga eopseoyo", "tidak punya teman"),
            ]),
            Block::Text(
                "Karena menyatakan keberadaan, subjeknya memakai 이/가, bukan 은/는. \
                 있어요 juga berarti 'berada di': 저는 집에 있어요.",
            ),
            Block::Examples(&[
                e("물이 있어요.", "muri isseoyo.", "Ada air."),
                e("시간이 없어요.", "sigani eopseoyo.", "Tidak ada waktu."),
                e("저는 집에 있어요.", "jeoneun jibe isseoyo.", "Saya ada di rumah."),
                e("한국 친구가 있어요.", "hanguk chinguga isseoyo.", "Saya punya teman Korea."),
            ]),
            Block::Note("Jangan memakai 을/를 dengan 있어요/없어요."),
        ],
        quiz: &[
            q(
                "Arti '시간이 없어요' adalah …",
                ["Tidak ada waktu", "Ada waktu", "Waktunya cepat", "Saya punya waktu"],
                0,
                "없어요 = tidak ada.",
            ),
            q(
                "Partikel yang benar: 물___ 있어요.",
                ["이", "을", "는", "에"],
                0,
                "Keberadaan memakai 이/가 → 물이 있어요.",
            ),
            q(
                "Bagaimana mengatakan 'Saya ada di rumah'?",
                ["저는 집에 있어요", "저는 집을 있어요", "저는 집이 있어요", "저는 집에 없어요"],
                0,
                "Tempat memakai 에 + 있어요.",
            ),
        ],
    },
    /* ------------------------------------------------------ 8. 아요/어요 */
    Topic {
        id: "ayo-eoyo",
        no: 8,
        title: "아요/어요",
        subtitle: "Kata kerja bentuk sopan",
        level: Level::Dasar,
        icon: "요",
        intro: "Bentuk sopan informal (해요체) dibentuk dari akar kata kerja + 아요/어요.",
        blocks: &[
            Block::Formula("vokal akhir ㅏ/ㅗ → 아요   |   vokal lain → 어요   |   하다 → 해요"),
            Block::Rules(&[
                r("가다 → 가요", "gayo", "pergi"),
                r("먹다 → 먹어요", "meogeoyo", "makan"),
                r("보다 → 봐요", "bwayo", "melihat"),
                r("하다 → 해요", "haeyo", "melakukan"),
            ]),
            Block::Text(
                "Buang 다, lihat vokal terakhir batang kata. 가다 → 가 + 아요 → 가요 (vokal \
                 sama menyatu). 오다 → 와요. 마시다 → 마셔요.",
            ),
            Block::Examples(&[
                e("저는 학교에 가요.", "jeoneun hakgyoe gayo.", "Saya pergi ke sekolah."),
                e("밥을 먹어요.", "babeul meogeoyo.", "Saya makan nasi."),
                e("커피를 마셔요.", "keopireul masyeoyo.", "Saya minum kopi."),
                e("한국어를 공부해요.", "hangugeoreul gongbuhaeyo.", "Saya belajar bahasa Korea."),
            ]),
            Block::Note(
                "Kata kerja dengan akar berakhiran ㅡ (쓰다, 크다) mengikuti vokal suku \
                 sebelumnya: 쓰다 → 써요.",
            ),
        ],
        quiz: &[
            q(
                "Bentuk sopan dari 먹다 adalah …",
                ["먹어요", "먹아요", "먹요", "먹해요"],
                0,
                "Batang 먹 berakhir ㅓ → 먹 + 어요 = 먹어요.",
            ),
            q(
                "Bentuk sopan dari 하다 adalah …",
                ["해요", "하아요", "하요", "했요"],
                0,
                "하다 → 해요.",
            ),
            q(
                "Bentuk sopan dari 마시다 adalah …",
                ["마셔요", "마시어요", "마시아요", "마시요"],
                0,
                "마시 + 어요 menyatu menjadi 마셔요.",
            ),
            q(
                "Bentuk sopan dari 보다 adalah …",
                ["봐요", "보아요", "보요", "뵈요"],
                0,
                "보 + 아요 menyatu menjadi 봐요.",
            ),
        ],
    },
    /* --------------------------------------------------------- 9. 숫자 */
    Topic {
        id: "angka",
        no: 9,
        title: "숫자",
        subtitle: "Angka Sino-Korea dan asli Korea",
        level: Level::Menengah,
        icon: "수",
        intro: "Korea memakai dua sistem angka: Sino-Korea (dari Cina) dan angka asli Korea.",
        blocks: &[
            Block::Heading("Sino-Korea (일, 이, 삼 …)"),
            Block::Rules(&[
                r("1 일", "2 이", "3 삼"),
                r("4 사", "5 오", "6 육"),
                r("7 칠", "8 팔", "9 구"),
                r("10 십", "100 백", "1000 천"),
            ]),
            Block::Heading("Asli Korea (하나, 둘, 셋 …)"),
            Block::Rules(&[
                r("1 하나", "2 둘", "3 셋"),
                r("4 넷", "5 다섯", "6 여섯"),
                r("7 일곱", "8 여덟", "9 아홉"),
                r("10 열", "20 스물", "30 서른"),
            ]),
            Block::Text(
                "Sino-Korea dipakai untuk tanggal, uang, nomor telepon, dan menit. \
                 Angka asli Korea dipakai untuk jumlah benda, jam, dan usia.",
            ),
            Block::Examples(&[
                e("세 개", "se gae", "tiga buah (angka asli)"),
                e("이십 분", "isip bun", "20 menit (Sino)"),
                e("두 시", "du si", "jam dua (angka asli)"),
                e("만 원", "man won", "10.000 won (Sino)"),
            ]),
            Block::Note(
                "Sebelum satuan, angka asli berubah: 하나 → 한, 둘 → 두, 셋 → 세, 넷 → 네 \
                 (한 개, 두 명, 세 시).",
            ),
        ],
        quiz: &[
            q(
                "Angka Sino-Korea untuk 3 adalah …",
                ["삼", "셋", "세", "석"],
                0,
                "Sino-Korea: 3 = 삼. 셋 adalah angka asli Korea.",
            ),
            q(
                "Untuk menyebut jam dipakai …",
                ["angka asli Korea", "angka Sino-Korea", "keduanya bebas", "hanya Sino untuk jam"],
                0,
                "Jam memakai angka asli Korea (두 시), menit memakai Sino (이십 분).",
            ),
            q(
                "Bentuk yang benar untuk 'tiga buah' adalah …",
                ["세 개", "셋 개", "삼 개", "석 개"],
                0,
                "셋 berubah menjadi 세 sebelum satuan.",
            ),
        ],
    },
    /* -------------------------------------------------------- 10. lampau */
    Topic {
        id: "lampau",
        no: 10,
        title: "았어요/었어요",
        subtitle: "Masa lalu sopan",
        level: Level::Menengah,
        icon: "했",
        intro: "Masa lalu dibentuk dengan 았어요/었어요, dengan aturan vokal sama seperti 아요/어요.",
        blocks: &[
            Block::Formula("vokal ㅏ/ㅗ → 았어요  |  lainnya → 었어요  |  하다 → 했어요"),
            Block::Rules(&[
                r("가다 → 갔어요", "gasseoyo", "pergi (lampau)"),
                r("먹다 → 먹었어요", "meogeosseoyo", "makan (lampau)"),
                r("보다 → 봤어요", "bwasseoyo", "melihat (lampau)"),
                r("공부하다 → 공부했어요", "gongbuhaesseoyo", "belajar (lampau)"),
            ]),
            Block::Text(
                "Keterangan waktu seperti 어제 (kemarin), 지난주 (minggu lalu) sering \
                 dipakai bersama bentuk lampau. Bentuk lampau juga dipakai untuk keadaan \
                 yang sudah selesai.",
            ),
            Block::Examples(&[
                e("어제 영화를 봤어요.", "eoje yeonghwareul bwasseoyo.", "Kemarin saya menonton film."),
                e("아침에 빵을 먹었어요.", "achime ppangeul meogeosseoyo.", "Pagi tadi saya makan roti."),
                e("지난주에 부산에 갔어요.", "jinanjue busane gasseoyo.", "Minggu lalu saya pergi ke Busan."),
                e("한국어를 공부했어요.", "hangugeoreul gongbuhaesseoyo.", "Saya belajar bahasa Korea."),
            ]),
            Block::Note("Bentuk lampau dari 있다 adalah 있었어요; 없다 menjadi 없었어요."),
        ],
        quiz: &[
            q(
                "Bentuk lampau dari 먹다 adalah …",
                ["먹었어요", "먹았어요", "먹겠어요", "먹어요"],
                0,
                "Batang 먹 berakhir ㅓ → 먹 + 었어요.",
            ),
            q(
                "Bentuk lampau dari 공부하다 adalah …",
                ["공부했어요", "공부하았어요", "공부해요", "공부하겠어요"],
                0,
                "하다 → 했어요.",
            ),
            q(
                "Kalimat '어제 영화를 봤어요' berarti …",
                ["Kemarin saya menonton film", "Besok saya menonton film", "Saya sedang menonton film", "Saya tidak menonton film"],
                0,
                "봤어요 = bentuk lampau dari 보다.",
            ),
        ],
    },
    /* -------------------------------------------------------- 11. negasi */
    Topic {
        id: "negasi",
        no: 11,
        title: "안 / -지 않다",
        subtitle: "Menyatakan tidak / bukan",
        level: Level::Menengah,
        icon: "안",
        intro: "Ada dua cara umum menegasikan kata kerja: sisipkan 안 sebelum kata kerja, atau gunakan -지 않다.",
        blocks: &[
            Block::Formula("안 + kata kerja  ≡  batang + 지 않다"),
            Block::Rules(&[
                r("가요 → 안 가요", "an gayo", "tidak pergi"),
                r("먹어요 → 안 먹어요", "an meogeoyo", "tidak makan"),
                r("가요 → 가지 않아요", "gaji anayo", "tidak pergi"),
                r("해요 → 하지 않아요", "haji anayo", "tidak melakukan"),
            ]),
            Block::Text(
                "안 diletakkan tepat sebelum kata kerja (bukan sebelum partikel objek). \
                 Bentuk -지 않아요 lebih formal dan sering dipakai dalam tulisan.",
            ),
            Block::Examples(&[
                e("저는 고기를 안 먹어요.", "jeoneun gogireul an meogeoyo.", "Saya tidak makan daging."),
                e("오늘은 학교에 가지 않아요.", "oneureun hakgyoe gaji anayo.", "Hari ini saya tidak pergi ke sekolah."),
                e("커피를 마시지 않아요.", "keopireul masiji anayo.", "Saya tidak minum kopi."),
                e("그 영화는 재미없어요.", "geu yeonghwaneun jaemieopseoyo.", "Film itu tidak menarik."),
            ]),
            Block::Note(
                "Untuk 'tidak bisa', gunakan 못 (못 가요) atau -지 못하다. \
                 안 = tidak mau/tidak melakukan, 못 = tidak mampu.",
            ),
        ],
        quiz: &[
            q(
                "Bentuk negasi dari 먹어요 dengan 안 adalah …",
                ["안 먹어요", "먹어요 안", "안 먹다", "먹지 안아요"],
                0,
                "안 diletakkan sebelum kata kerja: 안 먹어요.",
            ),
            q(
                "Arti '가지 않아요' adalah …",
                ["tidak pergi", "pergi", "tidak bisa pergi", "sudah pergi"],
                0,
                "지 않아요 = negasi sopan dari 가다.",
            ),
            q(
                "Perbedaan 안 dan 못 adalah …",
                ["안 = tidak melakukan, 못 = tidak mampu", "keduanya sama", "못 = tidak mau", "안 = tidak mampu"],
                0,
                "못 menyatakan ketidakmampuan, 안 menyatakan penolakan/negasi biasa.",
            ),
        ],
    },
    /* ------------------------------------------------------ 12. penghubung */
    Topic {
        id: "penghubung",
        no: 12,
        title: "-고 / -서",
        subtitle: "Menghubungkan dua klausa",
        level: Level::Lanjutan,
        icon: "고",
        intro: "-고 menyambung dua kegiatan berurutan/sejajar; -아서/-어서 menyatakan sebab atau urutan.",
        blocks: &[
            Block::Formula("batang + 고 (dan, lalu)   |   batang + 아서/어서 (karena, lalu)"),
            Block::Rules(&[
                r("먹고", "meokgo", "makan lalu …"),
                r("가고", "gago", "pergi dan …"),
                r("비가 와서", "biga waseo", "karena hujan …"),
                r("피곤해서", "pigonhaeseo", "karena lelah …"),
            ]),
            Block::Text(
                "-고 hanya menambahkan informasi tanpa makna sebab. -아서/어서 menuntut \
                 hubungan sebab-akibat atau urutan yang erat, dan tidak dipakai bersama \
                 bentuk imperatif seperti -세요.",
            ),
            Block::Examples(&[
                e("저는 밥을 먹고 커피를 마셔요.", "jeoneun babeul meokgo keopireul masyeoyo.", "Saya makan lalu minum kopi."),
                e("비가 와서 집에 있었어요.", "biga waseo jibe isseosseoyo.", "Karena hujan, saya di rumah."),
                e("피곤해서 일찍 잤어요.", "pigonhaeseo iljjik jasseoyo.", "Karena lelah, saya tidur lebih awal."),
                e("민수는 학생이고 지수는 의사예요.", "minsuneun haksaengigo jisuneun uisayeyo.", "Minsu murid dan Jisu dokter."),
            ]),
            Block::Note(
                "-고 bisa dipakai dengan bentuk lampau di klausa pertama (-았고/-었고), \
                 sedangkan -아서/어서 tidak bisa didahului bentuk lampau.",
            ),
        ],
        quiz: &[
            q(
                "Penghubung yang menyatakan sebab adalah …",
                ["-아서/어서", "-고", "-지만", "-는데"],
                0,
                "-아서/어서 menyatakan sebab atau urutan erat.",
            ),
            q(
                "Lengkapi: 비가 ___ 집에 있었어요.",
                ["와서", "오고", "왔고", "오면"],
                0,
                "Sebab memakai -아서/어서: 비가 와서.",
            ),
            q(
                "Kalimat '밥을 먹고 커피를 마셔요' berarti …",
                ["Makan nasi lalu minum kopi", "Karena makan nasi, minum kopi", "Makan nasi atau kopi", "Tidak makan nasi"],
                0,
                "-고 menyatakan urutan sejajar: makan lalu minum.",
            ),
            q(
                "Mana yang TIDAK boleh memakai -아서/어서?",
                ["bentuk imperatif -세요", "bentuk lampau di klausa kedua", "bentuk sopan", "kalimat sebab"],
                0,
                "-아서/어서 tidak dipakai dengan -세요 (perintah).",
            ),
        ],
    },
];

/// Cari topik berdasarkan id (`eun-neun`) atau nomor (`3`).
pub fn find(key: &str) -> Option<&'static Topic> {
    let key = key.trim();
    if let Some(t) = TOPICS.iter().find(|t| t.id.eq_ignore_ascii_case(key)) {
        return Some(t);
    }
    if let Ok(no) = key.parse::<u32>() {
        return TOPICS.iter().find(|t| t.no == no);
    }
    None
}

/// Cari berdasarkan judul (한글) atau potongan teks.
pub fn find_by_title(key: &str) -> Option<&'static Topic> {
    TOPICS.iter().find(|t| t.title == key)
}

/// Filter topik berdasarkan tingkat.
pub fn by_level(level: Level) -> Vec<&'static Topic> {
    TOPICS.iter().filter(|t| t.level == level).collect()
}

/// Pencarian kata kunci pada seluruh isi materi.
pub fn search(query: &str) -> Vec<&'static Topic> {
    TOPICS.iter().filter(|t| t.matches(query)).collect()
}

/// Total soal di seluruh bank materi.
pub fn total_quiz() -> usize {
    TOPICS.iter().map(|t| t.quiz.len()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topics_are_unique_and_numbered() {
        let mut seen: Vec<&str> = Vec::new();
        for (i, t) in TOPICS.iter().enumerate() {
            assert_eq!(t.no as usize, i + 1, "nomor topik harus urut");
            assert!(!seen.contains(&t.id), "id duplikat: {}", t.id);
            seen.push(t.id);
        }
    }

    #[test]
    fn every_topic_has_content_and_quiz() {
        for t in TOPICS {
            assert!(!t.blocks.is_empty(), "{} tanpa blok", t.id);
            assert!(t.quiz.len() >= 3, "{} hanya {} soal", t.id, t.quiz.len());
            for item in t.quiz {
                assert!(item.answer < 4, "jawaban di luar rentang pada {}", t.id);
                assert!(!item.q.is_empty());
                assert!(!item.why.is_empty());
                for opt in item.options {
                    assert!(!opt.is_empty(), "opsi kosong pada {}", t.id);
                }
            }
        }
    }

    #[test]
    fn lookup_variants() {
        assert!(find("hangeul").is_some());
        assert!(find("3").is_some());
        assert_eq!(find("3").unwrap().id, "eun-neun");
        assert!(find("tidak-ada").is_none());
        assert_eq!(find_by_title("은/는").unwrap().no, 3);
    }

    #[test]
    fn levels_partition_topics() {
        let sum: usize = Level::ALL.iter().map(|l| by_level(*l).len()).sum();
        assert_eq!(sum, TOPICS.len());
    }

    #[test]
    fn search_hits_content() {
        let hits = search("hujan");
        assert!(hits.iter().any(|t| t.id == "i-ga"));
        assert!(search("").is_empty());
    }

    #[test]
    fn quiz_total_matches() {
        assert_eq!(
            total_quiz(),
            TOPICS.iter().map(|t| t.quiz.len()).sum::<usize>()
        );
        assert!(total_quiz() >= 36);
    }
}
