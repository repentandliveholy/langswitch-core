use std::collections::HashSet;
use std::sync::OnceLock;

static EN_DICT: OnceLock<HashSet<&'static str>> = OnceLock::new();
static RU_DICT: OnceLock<HashSet<&'static str>> = OnceLock::new();
static FR_DICT: OnceLock<HashSet<&'static str>> = OnceLock::new();
static TR_DICT: OnceLock<HashSet<&'static str>> = OnceLock::new();

pub fn is_known_en_word(word_lower: &str) -> bool {
    let set = EN_DICT.get_or_init(|| {
        let words = [
            "a", "i", "an", "as", "at", "be", "by", "do", "go", "he", "hi", "if", "in", "is",
            "it", "me", "my", "no", "of", "oh", "ok", "on", "or", "so", "to", "up", "us", "we",
            "all", "and", "any", "are", "ask", "bad", "big", "box", "boy", "bug", "bus", "but",
            "buy", "can", "car", "cat", "cpu", "cry", "cut", "dad", "day", "did", "dog", "dry",
            "eat", "end", "eye", "far", "few", "fix", "fly", "for", "fun", "get", "god", "got",
            "guy", "had", "has", "her", "hey", "him", "his", "hit", "hot", "how", "ice", "its",
            "job", "key", "kid", "law", "lay", "let", "lie", "log", "lot", "low", "mac", "man",
            "map", "may", "men", "mom", "new", "not", "now", "off", "old", "one", "our", "out",
            "own", "pay", "per", "put", "ran", "raw", "red", "run", "sad", "saw", "say", "sea",
            "see", "set", "she", "sit", "six", "sky", "son", "sun", "tea", "ten", "the", "tie",
            "too", "top", "try", "two", "use", "van", "war", "was", "way", "who", "why", "win",
            "won", "yes", "yet", "you", "able", "also", "area", "army", "away", "baby", "back",
            "ball", "bank", "base", "bear", "beat", "been", "beer", "bell", "best", "bill",
            "bird", "blue", "boat", "body", "book", "born", "both", "burn", "busy", "call", "calm",
            "came", "camp", "card", "care", "case", "cash", "cast", "cell", "chat", "chip", "city",
            "club", "coal", "coat", "code", "cold", "come", "cook", "cool", "copy", "core",
            "cost", "crew", "crop", "dark", "data", "date", "dead", "deal", "dear", "deep", "desk",
            "dial", "diet", "disk", "does", "done", "door", "down", "draw", "drop", "drug", "dual",
            "duke", "dust", "duty", "each", "earn", "east", "easy", "edge", "else", "even", "ever",
            "evil", "exit", "face", "fact", "fail", "fair", "fall", "farm", "fast", "fate", "fear",
            "feed", "feel", "feet", "fell", "felt", "file", "fill", "film", "find", "fine", "fire",
            "firm", "fish", "five", "flag", "flat", "flow", "food", "foot", "ford", "form", "fort",
            "four", "free", "from", "fuel", "full", "fund", "gain", "game", "gate", "gave", "gear",
            "gene", "gift", "girl", "give", "glad", "goal", "goes", "gold", "golf", "gone", "good",
            "gray", "grew", "grey", "grow", "gulf", "hair", "half", "hall", "hand", "hang", "hard",
            "harm", "hate", "have", "head", "hear", "heat", "held", "hell", "help", "here", "hero",
            "high", "hill", "hire", "hold", "hole", "holy", "home", "hope", "host", "hour", "huge",
            "hung", "hunt", "hurt", "idea", "inch", "into", "iron", "item", "jack", "jane", "jean",
            "john", "join", "jump", "jury", "just", "keen", "keep", "kent", "kept", "kick", "kill",
            "kind", "king", "knee", "knew", "know", "lack", "lady", "laid", "lake", "land", "lane",
            "last", "late", "lead", "left", "less", "life", "lift", "like", "line", "link", "list",
            "live", "load", "loan", "lock", "logo", "long", "look", "lord", "lose", "loss", "lost",
            "love", "luck", "made", "mail", "main", "make", "male", "many", "mark", "mass", "matt",
            "meal", "mean", "meat", "meet", "menu", "mere", "mike", "mile", "milk", "mill", "mind",
            "mine", "miss", "mode", "mood", "moon", "more", "most", "move", "much", "must", "name",
            "navy", "near", "neck", "need", "news", "next", "nice", "nick", "nine", "none", "nose",
            "note", "okay", "once", "only", "onto", "open", "oral", "over", "pace", "pack", "page",
            "paid", "pain", "pair", "palm", "park", "part", "pass", "past", "path", "peak", "pick",
            "pink", "pipe", "plan", "play", "plot", "plug", "plus", "poll", "pool", "poor", "port",
            "post", "pull", "pure", "push", "race", "rail", "rain", "rank", "rare", "rate", "read",
            "real", "rear", "rely", "rent", "rest", "rice", "rich", "ride", "ring", "rise", "risk",
            "road", "rock", "role", "roll", "roof", "room", "root", "rose", "rule", "rush", "ruth",
            "safe", "said", "sake", "sale", "salt", "same", "sand", "save", "seat", "seed", "seek",
            "seem", "seen", "self", "sell", "send", "sent", "sept", "ship", "shop", "shot", "show",
            "shut", "sick", "side", "sign", "site", "size", "skin", "slip", "slow", "snow", "soft",
            "soil", "sold", "sole", "some", "song", "soon", "sort", "soul", "spot", "star", "stay",
            "step", "stop", "such", "suit", "sure", "take", "tale", "talk", "tall", "tank", "tape",
            "task", "team", "tech", "tell", "tend", "term", "test", "text", "than", "that", "them",
            "then", "they", "thin", "this", "thus", "till", "time", "tiny", "told", "toll", "tone",
            "tony", "took", "tool", "tour", "town", "tree", "trip", "true", "tune", "turn", "twin",
            "type", "unit", "upon", "used", "user", "vary", "vast", "very", "vice", "view", "vote",
            "wage", "wait", "wake", "walk", "wall", "want", "ward", "warm", "wash", "wave", "ways",
            "weak", "wear", "week", "well", "went", "were", "west", "what", "when", "whom", "wide",
            "wife", "wild", "will", "wind", "wine", "wing", "wire", "wise", "wish", "with", "wood",
            "word", "wore", "work", "yard", "yeah", "year", "your", "zero", "zone",
            "about", "above", "adapt", "after", "again", "agree", "allow", "alone", "along",
            "among", "apple", "apply", "array", "audio", "avoid", "aware", "basic", "begin",
            "below", "black", "block", "board", "boost", "brain", "brand", "break", "bring",
            "build", "built", "carry", "catch", "cause", "chain", "chair", "chart", "check",
            "child", "claim", "class", "clean", "clear", "click", "clock", "close", "cloud",
            "could", "count", "court", "cover", "craft", "crash", "cross", "daily", "dance",
            "debug", "delay", "depth", "doing", "doubt", "draft", "dream", "drink", "drive",
            "early", "earth", "empty", "enjoy", "enter", "entry", "equal", "error", "event",
            "every", "exact", "exist", "extra", "false", "fault", "field", "fight", "final",
            "first", "fixed", "flash", "floor", "focus", "force", "found", "frame", "fresh",
            "front", "fully", "funny", "given", "glass", "globe", "going", "grace", "grade",
            "grand", "grant", "great", "green", "group", "guard", "guess", "guide", "happy",
            "heart", "heavy", "hello", "house", "human", "ideal", "image", "index", "inner",
            "input", "issue", "joint", "judge", "known", "label", "large", "later", "laugh",
            "layer", "learn", "least", "leave", "legal", "level", "light", "limit", "links",
            "local", "logic", "lower", "lucky", "lunch", "magic", "major", "maker", "match",
            "maybe", "meant", "media", "metal", "might", "minor", "model", "money", "month",
            "mount", "mouse", "mouth", "movie", "music", "needs", "never", "night", "noise",
            "north", "noted", "novel", "occur", "ocean", "offer", "often", "order", "other",
            "paint", "panel", "paper", "party", "peace", "phase", "phone", "photo", "piece",
            "pilot", "place", "plain", "plane", "plant", "plate", "point", "power", "press",
            "price", "pride", "prime", "print", "prior", "prize", "proof", "proud", "prove",
            "queen", "quick", "quiet", "quite", "radio", "raise", "range", "rapid", "ratio",
            "reach", "ready", "refer", "right", "river", "rough", "round", "route", "royal",
            "scale", "scene", "scope", "score", "sense", "serve", "seven", "shall", "shape",
            "share", "sharp", "sheet", "shelf", "shell", "shift", "shirt", "shock", "shoot",
            "short", "shown", "sight", "since", "skill", "sleep", "slide", "small", "smart",
            "smile", "solid", "solve", "sorry", "sound", "south", "space", "spare", "speak",
            "speed", "spend", "spent", "split", "sport", "staff", "stage", "stand", "start",
            "state", "steam", "steel", "stick", "still", "stock", "stone", "stood", "store",
            "storm", "story", "strip", "stuck", "study", "stuff", "style", "sugar", "suite",
            "super", "sweet", "table", "taken", "taste", "teach", "thank", "their", "theme",
            "there", "these", "thick", "thing", "think", "third", "those", "three", "throw",
            "tight", "times", "tired", "title", "today", "topic", "total", "touch", "tough",
            "tower", "track", "trade", "train", "treat", "trend", "trial", "tried", "tries",
            "truck", "truly", "trust", "truth", "twice", "under", "union", "unity", "until",
            "upper", "upset", "urban", "usage", "usual", "valid", "value", "video", "virus",
            "visit", "vital", "voice", "waste", "watch", "water", "wheel", "where", "which",
            "while", "white", "whole", "whose", "woman", "women", "world", "worry", "worse",
            "worst", "worth", "would", "write", "wrong", "wrote", "yield", "young", "youth",
            "thanks", "please", "welcome", "awesome", "perfect", "language", "keyboard",
            "switch", "switcher", "automatic", "window", "windows", "linux", "macos", "system",
            "setting", "settings", "button", "screen", "folder", "memory", "project", "create",
            "update", "delete", "remove", "install", "website", "commit", "branch", "master",
            "server", "client", "country", "flags", "simple", "simplest", "superfast",
            // Developer / CLI vocabulary
            "git", "gh", "npm", "npx", "yarn", "pnpm", "bun", "cargo", "rustc", "rustup",
            "swift", "swiftc", "xcode", "clang", "gcc", "make", "cmake", "ninja", "brew",
            "sudo", "ssh", "scp", "curl", "wget", "grep", "rg", "find", "cat", "ls", "cd",
            "pwd", "mkdir", "rm", "cp", "mv", "chmod", "chown", "tar", "zip", "unzip",
            "docker", "kubectl", "helm", "terraform", "ansible", "python", "pip", "pytest",
            "node", "deno", "java", "kotlin", "scala", "ruby", "rails", "php", "perl",
            "lua", "zig", "nim", "elixir", "erlang", "haskell", "ocaml", "clojure", "dart",
            "flutter", "react", "vue", "svelte", "angular", "nextjs", "nuxt", "astro", "vite",
            "webpack", "rollup", "esbuild", "tailwind", "prisma", "graphql", "redis", "postgres",
            "mysql", "sqlite", "mongodb", "kafka", "nginx", "apache", "caddy", "ubuntu",
            "debian", "fedora", "arch", "alpine", "darwin", "ios", "android",
            "const", "var", "fn", "func", "function", "def", "struct", "enum",
            "trait", "impl", "interface", "typeof", "import", "export", "return",
            "async", "await", "promise", "future", "default",
            "continue", "loop", "elif", "throws", "finally", "pub", "priv", "private", "public",
            "protected", "static", "mut", "ref", "true", "null", "nil",
            "void", "int", "uint", "float", "double", "bool",
            "boolean", "char", "str", "string", "vec", "dict", "hashmap",
            "hashset", "btree", "option", "result", "arc", "rc", "mutex", "rwlock",
            "println", "printf", "format", "trace", "info", "warn", "panic", "assert",
            "sizeof", "alignof", "offsetof", "extern", "unsafe", "crate", "mod",
            "macro", "derive", "cfg", "bench", "deny", "todo",
            "github", "gitlab", "bitbucket", "vercel", "netlify", "cloudflare", "aws", "gcp",
            "azure", "heroku", "digitalocean", "supabase", "firebase", "stripe", "openai",
            "anthropic", "claude", "gemini", "chatgpt", "vscode", "cursor", "zed", "neovim",
            "vim", "emacs", "tmux", "zsh", "bash", "iterm", "alacritty", "kitty",
            "ghostty", "wezterm", "warp", "raycast", "alfred", "homebrew", "sparkle",
            "uniffi", "rusqlite", "tokio", "serde", "reqwest", "axum", "actix", "clap",
        ];
        words.into_iter().collect()
    });
    set.contains(word_lower)
}

pub fn is_known_ru_word(word_lower: &str) -> bool {
    let set = RU_DICT.get_or_init(|| {
        let words = [
            "и", "в", "не", "на", "я", "быть", "он", "с", "что", "а", "по", "это", "она", "этот",
            "к", "но", "они", "мы", "как", "из", "у", "который", "то", "за", "свой", "весь",
            "год", "от", "так", "о", "для", "ты", "же", "все", "тот", "мочь", "вы", "человек",
            "такой", "его", "сказать", "только", "или", "еще", "ещё", "бы", "себя", "один",
            "уже", "до", "время", "если", "сам", "когда", "другой", "вот", "говорить", "наш",
            "мой", "знать", "стать", "при", "чтобы", "дело", "жизнь", "кто", "первый", "очень",
            "два", "день", "ее", "её", "новый", "рука", "даже", "во", "со", "раз", "где", "там",
            "под", "можно", "ну", "какой", "после", "их", "работа", "без", "самый", "потом",
            "надо", "хотеть", "ли", "слово", "идти", "большой", "должен", "место", "иметь",
            "ничто", "тогда", "ведь", "сейчас", "тут", "лицо", "каждый", "друг", "нет", "да",
            "теперь", "ни", "глаз", "тоже", "видеть", "вопрос", "через", "дом", "сторона",
            "потому", "жить", "страна", "мир", "последний", "случай", "голова", "более", "делать",
            "что-то", "смотреть", "ребенок", "ребёнок", "просто", "конечно", "сила", "конец",
            "перед", "несколько", "вид", "система", "всегда", "работать", "между", "три",
            "понять", "пойти", "часть", "спросить", "город", "дать", "также", "никто", "понимать",
            "получить", "отношение", "сидеть", "оказаться", "взять", "прийти", "деньги", "вдруг",
            "любить", "люблю", "стол", "путь", "душа", "земля", "второй", "бог", "книга",
            "возможность", "решить", "начать", "отец", "час", "образ", "писать", "нога", "увидеть",
            "думать", "многие", "война", "снова", "ответить", "правда", "либо", "почти", "народ",
            "язык", "высокий", "машина", "закон", "уйти", "над", "мать", "сделать", "ночь",
            "кажется", "начало", "свет", "ждать", "голос", "пора", "остаться", "написать",
            "помочь", "общий", "вода", "уровень", "хороший", "хорошо", "плохо", "плохой",
            "привет", "здравствуйте", "здравствуй", "спасибо", "пожалуйста", "пока", "давай",
            "отлично", "супер", "круто", "понятно", "ясно", "ладно", "ок", "окей", "сегодня",
            "завтра", "вчера", "утром", "вечером", "ночью", "днем", "днём", "неделя", "месяц",
            "минута", "секунда", "быстро", "медленно", "нужно", "нельзя", "возможно", "точно",
            "наверное", "кстати", "вообще", "короче", "значит", "почему", "зачем",
            "куда", "откуда", "сколько", "столько", "никогда", "иногда", "часто", "редко",
            "вместе", "рядом", "далеко", "близко", "сразу", "скоро", "давно", "недавно",
            "дела", "проблема", "задача", "проект", "программа", "приложение", "сайт",
            "сервер", "клиент", "код", "файл", "папка", "окно", "экран", "кнопка", "меню",
            "настройки", "клавиатура", "раскладка", "переключатель", "флаг", "флаги", "русский",
            "английский", "французский", "турецкий", "пароль", "логин", "почта", "сообщение",
            "письмо", "звонок", "встреча", "хлеб", "молоко", "кофе", "чай", "обед", "ужин",
            "завтрак", "семья", "мама", "папа", "брат", "сестра", "сын", "дочь", "жена", "муж",
            "кот", "собака", "погода", "улица", "дорога", "магазин", "школа", "университет",
            "офис", "компания", "команда", "список", "ошибка", "исправить", "проверить",
            "отправить", "скачать", "открыть", "закрыть", "сохранить", "удалить", "добавить",
            "изменить", "создать", "запустить", "остановить", "работает", "делаешь", "делаете",
            "думаю", "знаю", "хочу", "могу", "буду", "будет", "будем", "будут", "был", "была",
            "было", "были", "есть", "здесь", "меня", "тебя", "него", "нее", "неё", "нас", "вас",
            "них", "мне", "тебе", "ему", "ей", "нам", "вам", "им", "мной", "тобой", "нами",
            "вами", "ими", "моя", "мое", "моё", "мои", "твоя", "твое", "твоё", "твои", "наша",
            "наше", "наши", "ваша", "ваше", "ваши", "этого", "этому", "этим", "этой", "этих",
            "того", "тому", "тем", "той", "тех", "какая", "какое", "какие", "какого", "такая",
            "такое", "такие", "клава",
            "спс", "пж", "плз", "кст", "норм", "имхо", "лол", "кек", "инфа", "комп", "ноут",
            "макос", "винда", "линукс", "телега", "телеграм", "слак", "зум", "гит", "пуш",
            "коммит", "репо", "ветка", "мерж", "билд", "релиз", "баг", "фича", "тест",
        ];
        words.into_iter().collect()
    });
    set.contains(word_lower)
}

pub fn is_known_fr_word(word_lower: &str) -> bool {
    let set = FR_DICT.get_or_init(|| {
        let words = [
            "bonjour", "salut", "merci", "été", "école", "déjà", "voilà", "ça", "où", "là", "élève", "écrire", "écouter", "général", "idée", "fenêtre", "première", "prêt", "bientôt", "frère", "mère", "père", "tête", "fête", "côté", "réponse", "début", "numéro", "musée", "café", "bonsoir", "oui", "non", "comment", "allez", "vous",
            "nous", "je", "tu", "il", "elle", "ils", "elles", "est", "sont", "suis", "avoir",
            "etre", "être", "faire", "dire", "aller", "voir", "savoir", "pouvoir", "falloir",
            "vouloir", "venir", "prendre", "arriver", "croire", "mettre", "passer", "devoir",
            "parler", "trouver", "donner", "comprendre", "aimer", "amour", "monde", "temps",
            "jour", "nuit", "maison", "travail", "toujours", "jamais", "aussi", "mais", "avec",
            "pour", "dans", "sur", "sous", "sans", "chez", "entre", "pendant", "avant", "apres",
            "après", "tres", "très", "bien", "plus", "moins", "tout", "tous", "toute", "toutes",
            "autre", "meme", "même", "grand", "petit", "nouveau", "premier", "dernier", "francais",
            "français", "france", "clavier", "langue", "ordinateur", "programme", "aujourd",
            "demain", "hier", "maintenant", "pourquoi", "parce", "quelque", "chose", "personne",
            "heure", "annee", "année", "ville", "pays", "famille", "enfant", "ami", "amis", "eau",
            "pain", "cafe", "café", "probleme", "problème", "solution", "question", "reponse",
            "réponse", "super", "genial", "génial", "daccord", "voila", "voilà", "pardon",
            "excusez", "beaucoup", "bonne", "bon", "soir", "matin", "midi", "semaine", "mois",
            "minute", "seconde", "vite", "lentement", "peut", "peux", "veux", "veut", "faut",
            "dois", "doit", "sais", "sait", "vois", "voit", "vas", "va", "vont", "allons",
            "fait", "fais", "font", "dit", "dis", "disent", "rien", "personne", "quelquun",
            "chaque", "plusieurs", "certain", "certains", "aucun", "aucune", "notre", "votre",
            "leur", "leurs", "mon", "ton", "son", "ma", "ta", "sa", "mes", "tes", "ses",
            "cette", "cet", "ces", "celui", "celle", "ceux", "celles", "qui", "que", "quoi",
            "dont", "ou", "où", "quand", "comme", "si", "car", "donc", "or", "ni", "puis",
            "ensuite", "alors", "encore", "deja", "déjà", "souvent", "parfois", "ici", "la",
            "là", "loin", "pres", "près", "ensemble", "seul", "seule", "vrai", "faux", "possible",
            "impossible", "facile", "difficile", "rapide", "beau", "belle", "joli", "jolie",
            "jeune", "vieux", "vieille", "fort", "forte", "heureux", "heureuse", "content",
        ];
        words.into_iter().collect()
    });
    set.contains(word_lower)
}

pub fn is_known_tr_word(word_lower: &str) -> bool {
    let set = TR_DICT.get_or_init(|| {
        let words = [
            "merhaba", "selam", "teşekkür", "teşekkürler", "tesekkur", "tesekkurler", "evet",
            "hayır", "hayir", "nasıl", "nasil", "nasılsın", "nasilsin", "iyi", "güzel", "guzel",
            "çok", "cok", "var", "yok", "ben", "sen", "biz", "siz", "onlar", "bu", "şu", "su",
            "ne", "kim", "nerede", "neden", "niçin", "nicin", "zaman", "gün", "gun", "gece",
            "sabah", "akşam", "aksam", "bugün", "bugun", "yarın", "yarin", "dün", "dun", "şimdi",
            "simdi", "sonra", "önce", "once", "her", "hiç", "hic", "hep", "daha", "kadar", "gibi",
            "için", "icin", "ile", "ama", "fakat", "ve", "veya", "lütfen", "lutfen", "tamam",
            "peki", "harika", "süper", "tabii", "belki", "doğru", "dogru", "yanlış", "yanlis",
            "kolay", "zor", "büyük", "buyuk", "küçük", "kucuk", "yeni", "eski", "uzun", "kısa",
            "kisa", "sevgi", "aşk", "ask", "seviyorum", "dünya", "dunya", "türkiye", "turkiye",
            "türkçe", "turkce", "dil", "klavye", "bilgisayar", "program", "iş", "ev", "okul",
            "araba", "yol", "şehir", "sehir", "ülke", "ulke", "insan", "arkadaş", "arkadas",
            "aile", "çocuk", "cocuk", "anne", "baba", "kardeş", "kardes", "ekmek", "çay", "cay",
            "kahve", "yemek", "para", "kitap", "soru", "cevap", "problem", "çözüm", "cozum",
            "yardım", "yardim", "başla", "basla", "bitir", "yapmak", "etmek", "olmak", "gelmek",
            "gitmek", "görmek", "gormek", "bilmek", "istemek", "sevmek", "konuşmak", "konusmak",
            "yazmak", "okumak", "anlamak", "çalışmak", "calismak", "gidiyorum", "geliyorum",
            "yapıyorum", "yapiyorum", "biliyorum", "istiyorum", "düşünüyorum", "dusunuyorum",
            "iyiyim", "sağol", "sagol", "hoşgeldin", "hosgeldin", "görüşürüz", "gorusuruz",
        ];
        words.into_iter().collect()
    });
    set.contains(word_lower)
}

pub fn english_morphology_bonus(word: &str) -> f64 {
    if word.len() < 4 {
        return 0.0;
    }
    let suffixes = [
        "tion", "sion", "ment", "ness", "able", "ible", "ling", "ight", "ough", "ould",
        "ious", "eous", "ting", "ding", "ning", "ring", "king", "ping", "ming",
        "ally", "ully", "less", "ship", "ward", "wise", "like", "free", "work", "ware",
        "ing", "est", "ers", "ied", "ies", "ful", "ous", "ive", "ize", "ise", "ent", "ant",
    ];
    for suf in suffixes {
        if word.ends_with(suf) {
            let stem_len = word.len() - suf.len();
            if stem_len >= 2 && has_en_vowel(&word[..stem_len]) {
                return 2.4;
            }
        }
    }
    if (word.ends_with("ed") || word.ends_with("ly") || word.ends_with("er") || word.ends_with("es"))
        && word.len() >= 5
        && has_en_vowel(&word[..word.len() - 2])
    {
        return 1.2;
    }
    0.0
}

pub fn russian_morphology_bonus(word: &str) -> f64 {
    let char_count = word.chars().count();
    if char_count < 4 {
        return 0.0;
    }
    let strong_suffixes = [
        "ость", "ение", "ание", "ться", "ется", "ются", "ский", "цкий", "ская",
        "ское", "ские", "овать", "евать", "ивать", "ывать", "тель", "льно", "льный",
        "льная", "льное", "льные", "ать", "ять", "ить", "еть", "уть", "ого", "его",
        "ому", "ему", "ыми", "ими", "ами", "ями", "ешь", "ишь", "ете", "ите", "уют",
        "ают", "яют", "али", "яли", "или", "ели", "ало", "яло", "ило", "ело",
    ];
    for suf in strong_suffixes {
        if word.ends_with(suf) && has_ru_vowel(word) {
            return 2.6;
        }
    }
    let medium_suffixes = [
        "ая", "яя", "ое", "ее", "ые", "ие", "ой", "ей", "ый", "ий", "ам", "ям", "ах",
        "ях", "ом", "ем", "ым", "им", "ть", "ти", "ся", "сь", "ла", "ли", "ло", "ет",
        "ит", "ут", "ют", "ат", "ят", "ка", "ки", "ку", "ке", "ко",
    ];
    if char_count >= 5 {
        for suf in medium_suffixes {
            if word.ends_with(suf) && has_ru_vowel(word) {
                return 1.3;
            }
        }
    }
    0.0
}

pub fn french_morphology_bonus(word: &str) -> f64 {
    let char_count = word.chars().count();
    if char_count < 4 {
        return 0.0;
    }
    let suffixes = [
        "tion", "ment", "ence", "ance", "ique", "able", "ible", "iste", "isme", "euse",
        "teur", "trice", "oire", "elle", "ette", "ille", "aise", "aient", "erait", "irait",
        "rons", "rez", "ront", "ais", "ait", "aux", "eux", "oir", "ons", "ent",
    ];
    for suf in suffixes {
        if word.ends_with(suf) && has_en_vowel(word) {
            return 2.3;
        }
    }
    0.0
}

pub fn turkish_morphology_bonus(word: &str) -> f64 {
    let char_count = word.chars().count();
    if char_count < 4 {
        return 0.0;
    }
    // Distinct Turkish chars (`ğ`, `ü`, `ş`, `ı`, `ö`, `ç`) are a very strong signal for Turkish!
    let tr_special = word
        .chars()
        .filter(|&c| matches!(c, 'ğ' | 'ü' | 'ş' | 'ı' | 'ö' | 'ç'))
        .count();
    let mut bonus = (tr_special as f64) * 3.0;

    let suffixes = [
        "ıyorum", "iyorum", "uyorum", "üyorum", "ıyorsun", "iyorsun", "lar", "ler",
        "lık", "lik", "luk", "lük", "mak", "mek", "yor", "miş", "mış", "muş", "müş",
        "dan", "den", "tan", "ten", "nın", "nin", "nun", "nün", "sız", "siz", "suz", "süz",
        "acak", "ecek", "dık", "dik", "duk", "dük",
    ];
    for suf in suffixes {
        if word.ends_with(suf) && has_tr_vowel(word) {
            bonus += 2.5;
            break;
        }
    }
    bonus
}

pub fn is_en_vowel(c: char) -> bool {
    matches!(
        c,
        'a' | 'e' | 'i' | 'o' | 'u' | 'y' | 'A' | 'E' | 'I' | 'O' | 'U' | 'Y'
            | 'é' | 'è' | 'ê' | 'ë' | 'à' | 'â' | 'ù' | 'û' | 'î' | 'ï' | 'ô' | 'œ'
    )
}

pub fn is_tr_vowel(c: char) -> bool {
    matches!(
        c,
        'a' | 'e' | 'ı' | 'i' | 'o' | 'ö' | 'u' | 'ü'
            | 'A' | 'E' | 'I' | 'İ' | 'O' | 'Ö' | 'U' | 'Ü'
    )
}

pub fn is_ru_vowel(c: char) -> bool {
    matches!(
        c,
        'а' | 'е' | 'ё' | 'и' | 'о' | 'у' | 'ы' | 'э' | 'ю' | 'я'
            | 'А' | 'Е' | 'Ё' | 'И' | 'О' | 'У' | 'Ы' | 'Э' | 'Ю' | 'Я'
    )
}

pub fn has_en_vowel(s: &str) -> bool {
    s.chars().any(is_en_vowel)
}

pub fn has_tr_vowel(s: &str) -> bool {
    s.chars().any(is_tr_vowel)
}

pub fn has_ru_vowel(s: &str) -> bool {
    s.chars().any(is_ru_vowel)
}

pub fn score_en_phonotactics(word_lower: &str) -> f64 {
    let chars: Vec<char> = word_lower.chars().collect();
    let n = chars.len();
    if n == 0 {
        return 0.0;
    }

    let mut score = 0.0;
    let mut embedded_punct = 0;
    for &c in &chars {
        if matches!(c, '[' | ']' | ';' | '\'' | ',' | '.' | '`' | '<' | '>' | '{' | '}' | ':' | '"') {
            embedded_punct += 1;
        } else if !c.is_ascii_alphabetic() {
            return -10.0;
        }
    }
    if embedded_punct > 0 {
        let valid_contraction = embedded_punct == 1
            && (word_lower.ends_with("'t")
                || word_lower.ends_with("'s")
                || word_lower.ends_with("'re")
                || word_lower.ends_with("'ve")
                || word_lower.ends_with("'ll")
                || word_lower.ends_with("'d")
                || word_lower.ends_with("'m"));
        if valid_contraction {
            score += 3.5;
        } else {
            score -= (embedded_punct as f64) * 4.5;
        }
    }

    let vowels = chars.iter().filter(|&&c| is_en_vowel(c)).count();
    if n >= 2 && vowels == 0 {
        score -= 4.5;
    } else if n >= 3 {
        let ratio = (vowels as f64) / (n as f64);
        if (0.20..=0.65).contains(&ratio) {
            score += 1.5;
        } else {
            score -= 1.2;
        }
    }

    let mut consec_consonants = 0;
    let mut max_consec_consonants = 0;
    for &c in &chars {
        if c.is_ascii_alphabetic() && !is_en_vowel(c) {
            consec_consonants += 1;
            if consec_consonants > max_consec_consonants {
                max_consec_consonants = consec_consonants;
            }
        } else {
            consec_consonants = 0;
        }
    }
    if max_consec_consonants >= 4 {
        score -= (max_consec_consonants as f64 - 2.5) * 2.2;
    }

    let common_en_bigrams = [
        "th", "he", "in", "er", "an", "re", "on", "at", "en", "nd", "ti", "es", "or", "te",
        "of", "ed", "is", "it", "al", "ar", "st", "to", "nt", "ng", "se", "ha", "as", "ou",
        "io", "le", "ve", "co", "me", "de", "hi", "ri", "ro", "ic", "ne", "ea", "ra", "ce",
        "li", "ch", "ll", "be", "ma", "si", "om", "ur", "ca", "el", "ta", "la", "ns", "ge",
        "ly", "ac", "pe", "ab", "sh", "wh", "oo", "ee", "ai", "ay", "ow", "ew", "ck", "qu",
    ];
    let impossible_en_bigrams = [
        "ghb", "bdtn", "rfr", "ltk", "cgc", "gj;", "fke", "qcn", "k.,", ",k.", "[jh", "jij",
        ";bp", "pym", "cnh", "hfn", "zps", "psr", "rjl", "afq", "gfg", "jry", "ryj", "rky",
        "qb", "qc", "qd", "qf", "qg", "qh", "qj", "qk", "ql", "qm", "qn", "qp", "qr", "qs",
        "qt", "qv", "qw", "qx", "qz", "vx", "zx", "xj", "xk", "xz", "jq", "jx", "jz", "kq",
        "kx", "kz", "pq", "px", "pz", "vq", "vz", "wq", "wx", "wz", "bd", "dt", "hb", "gf",
        "fk", "kj", "jk", "lj", "jl", "cf", "fc", "df", "fd", "fg", "hj", "jh",
    ];

    for bad in impossible_en_bigrams {
        if word_lower.contains(bad) {
            score -= 2.4;
        }
    }

    for win in chars.windows(2) {
        let mut buf = String::with_capacity(2);
        buf.push(win[0]);
        buf.push(win[1]);
        if common_en_bigrams.contains(&buf.as_str()) {
            score += 0.75;
        }
    }

    score + english_morphology_bonus(word_lower)
}

pub fn score_fr_phonotactics(word_lower: &str) -> f64 {
    let chars: Vec<char> = word_lower.chars().collect();
    let n = chars.len();
    if n == 0 {
        return 0.0;
    }

    for &c in &chars {
        if !c.is_ascii_alphabetic()
            && !matches!(c, 'é' | 'è' | 'ê' | 'ë' | 'à' | 'â' | 'ù' | 'û' | 'î' | 'ï' | 'ô' | 'ç' | 'œ' | '\'')
        {
            return -10.0;
        }
    }

    let mut score = 0.0;
    let vowels = chars.iter().filter(|&&c| is_en_vowel(c)).count();
    if n >= 2 && vowels == 0 {
        score -= 4.8;
    } else if n >= 3 {
        let ratio = (vowels as f64) / (n as f64);
        if (0.25..=0.70).contains(&ratio) {
            score += 1.5;
        } else {
            score -= 1.2;
        }
    }

    let common_fr_bigrams = [
        "es", "en", "le", "de", "re", "nt", "on", "er", "te", "el", "an", "se", "et", "la",
        "ai", "it", "me", "ou", "em", "ie", "ur", "us", "qu", "au", "is", "ne", "ar", "ti",
        "ce", "in", "ns", "ur", "co", "tr", "ch", "eu", "oi", "gn", "il", "ll", "ss", "mm",
    ];
    for win in chars.windows(2) {
        let mut buf = String::with_capacity(4);
        buf.push(win[0]);
        buf.push(win[1]);
        if common_fr_bigrams.contains(&buf.as_str()) {
            score += 0.8;
        }
    }

    score + french_morphology_bonus(word_lower)
}

pub fn score_tr_phonotactics(word_lower: &str) -> f64 {
    let chars: Vec<char> = word_lower.chars().collect();
    let n = chars.len();
    if n == 0 {
        return 0.0;
    }

    for &c in &chars {
        if !c.is_ascii_alphabetic() && !matches!(c, 'ğ' | 'ü' | 'ş' | 'ı' | 'ö' | 'ç') {
            return -10.0;
        }
    }

    let mut score = 0.0;
    let vowels = chars.iter().filter(|&&c| is_tr_vowel(c)).count();
    if n >= 2 && vowels == 0 {
        score -= 4.8;
    } else if n >= 3 {
        let ratio = (vowels as f64) / (n as f64);
        if (0.25..=0.68).contains(&ratio) {
            score += 1.6;
        } else {
            score -= 1.2;
        }
    }

    // Turkish never starts words with `ğ`
    if chars[0] == 'ğ' {
        score -= 6.0;
    }

    let common_tr_bigrams = [
        "ar", "er", "an", "en", "in", "ın", "un", "ün", "ла", "le", "la", "de", "da", "te",
        "ta", "ме", "ma", "me", "ка", "ka", "ke", "ли", "li", "lı", "lu", "lü", "ал", "el",
        "ир", "ir", "ır", "ur", "ür", "ор", "or", "йо", "yo", "ша", "şe", "şı", "şi", "га",
        "ğe", "ğı", "ği", "ча", "çe", "çı", "çi", "де", "di", "dı", "du", "dü",
    ];
    for win in chars.windows(2) {
        let mut buf = String::with_capacity(4);
        buf.push(win[0]);
        buf.push(win[1]);
        if common_tr_bigrams.contains(&buf.as_str()) {
            score += 0.75;
        }
    }

    score + turkish_morphology_bonus(word_lower)
}

pub fn score_ru_phonotactics(word_lower: &str) -> f64 {
    let chars: Vec<char> = word_lower.chars().collect();
    let n = chars.len();
    if n == 0 {
        return 0.0;
    }

    for &c in &chars {
        if !('\u{0400}'..='\u{04FF}').contains(&c) && c != '-' {
            return -10.0;
        }
    }

    let mut score = 0.0;

    if matches!(chars[0], 'ъ' | 'ь' | 'ы') {
        score -= 7.0;
    }

    let impossible_ru_pairs = [
        "ъъ", "ьь", "ъь", "ьъ", "ыы", "эы", "яы", "юы", "еы", "ёы", "аъ", "оъ", "уъ", "эъ",
        "яъ", "юъ", "иъ", "ыъ", "аы", "оы", "уы", "иы", "жы", "шы", "чя", "щя", "чю", "щю",
        "цэ", "щэ", "йй", "аь", "оь", "уь", "эь", "ыь", "иь", "яь", "юь", "еь", "ёь",
        "цщ", "щц", "щж", "жщ", "шщ", "щш", "фщ", "щф", "эщ", "фэ", "цф", "фц",
        "рудщ", "рудд", "цщкд", "еуые", "ыцша", "агтс", "куег", "ьукр", "ищто",
    ];
    for bad in impossible_ru_pairs {
        if word_lower.contains(bad) {
            score -= 3.5;
        }
    }

    let vowels = chars.iter().filter(|&&c| is_ru_vowel(c)).count();
    if n >= 2 && vowels == 0 {
        score -= 4.5;
    } else if n >= 3 {
        let ratio = (vowels as f64) / (n as f64);
        if (0.22..=0.68).contains(&ratio) {
            score += 1.6;
        } else {
            score -= 1.2;
        }
    }

    let mut consec_cons = 0;
    let mut max_consec_cons = 0;
    let mut consec_vow = 0;
    let mut max_consec_vow = 0;
    for &c in &chars {
        if is_ru_vowel(c) {
            consec_vow += 1;
            if consec_vow > max_consec_vow {
                max_consec_vow = consec_vow;
            }
            consec_cons = 0;
        } else {
            consec_cons += 1;
            if consec_cons > max_consec_cons {
                max_consec_cons = consec_cons;
            }
            consec_vow = 0;
        }
    }
    if max_consec_cons >= 5 {
        score -= (max_consec_cons as f64 - 3.0) * 2.2;
    }
    if max_consec_vow >= 3 {
        score -= (max_consec_vow as f64 - 2.0) * 2.0;
    }

    let common_ru_bigrams = [
        "ст", "но", "то", "на", "ен", "ов", "ни", "ра", "во", "ко", "ро", "пр", "по", "ре",
        "ли", "та", "ос", "ом", "не", "ер", "ор", "ет", "ва", "де", "ка", "ть", "ел",
        "ан", "ве", "ин", "ал", "ат", "ри", "ле", "го", "ск", "ам", "ас", "ог", "ак", "ся",
        "ит", "ля", "ви", "ти", "ем", "ме", "да", "за", "ди", "се", "ла", "ма", "ми", "мо",
        "му", "мы", "па", "пе", "пи", "при", "про", "ру", "са", "си", "со", "су",
        "те", "ту", "ты", "хо", "че", "чт", "эт",
    ];
    for win in chars.windows(2) {
        let mut buf = String::with_capacity(4);
        buf.push(win[0]);
        buf.push(win[1]);
        if common_ru_bigrams.contains(&buf.as_str()) {
            score += 0.8;
        }
    }

    score + russian_morphology_bonus(word_lower)
}

/// The whole built-in word list for a language (used for typo suggestions).
pub fn lexicon(lang: crate::Lang) -> &'static HashSet<&'static str> {
    use crate::Lang;
    match lang {
        Lang::En => {
            is_known_en_word("");
            EN_DICT.get().expect("initialised")
        }
        Lang::Ru => {
            is_known_ru_word("");
            RU_DICT.get().expect("initialised")
        }
        Lang::Fr => {
            is_known_fr_word("");
            FR_DICT.get().expect("initialised")
        }
        Lang::Tr => {
            is_known_tr_word("");
            TR_DICT.get().expect("initialised")
        }
    }
}
