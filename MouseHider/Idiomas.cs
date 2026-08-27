using System.Globalization;

namespace MouseHider;

/// <summary>
/// Tabela de textos. ponytail: dicionario em memoria em vez de .resx — sao ~30 frases,
/// e assim traduzir e editar um arquivo, sem passar por ferramenta de recurso.
/// </summary>
static class Idiomas
{
    public static readonly (string Codigo, string Nome)[] Disponiveis =
    {
        ("auto", "Auto"),
        ("pt", "Português"),
        ("en", "English"),
        ("es", "Español"),
        ("fr", "Français"),
        ("de", "Deutsch"),
        ("it", "Italiano"),
        ("ru", "Русский"),
        ("ja", "日本語"),
        ("zh", "中文"),
        ("ko", "한국어")
    };

    public static string Codigo { get; private set; } = "auto";

    // Inicializado no construtor estatico, nao no campo: os dicionarios sao declarados abaixo e
    // um inicializador de campo aqui pegaria null (ordem de inicializacao e a de declaracao).
    static Dictionary<string, string> _atual = null!;

    static Idiomas() => Definir("auto");

    public static void Definir(string? codigo)
    {
        Codigo = Disponiveis.Any(d => d.Codigo == codigo) ? codigo! : "auto";
        var efetivo = Codigo == "auto" ? DoWindows() : Codigo;
        _atual = efetivo switch
        {
            "pt" => Pt,
            "es" => Es,
            "fr" => Fr,
            "de" => De,
            "it" => It,
            "ru" => Ru,
            "ja" => Ja,
            "zh" => Zh,
            "ko" => Ko,
            _ => En
        };
    }

    /// <summary>Nome para mostrar na lista; "Auto" ganha o idioma do Windows entre parenteses.</summary>
    public static string NomeDe(string codigo, string nome) =>
        codigo == "auto" ? nome + " (" + DoWindows().ToUpperInvariant() + ")" : nome;

    /// <summary>Idioma da interface do Windows; idioma sem traducao aqui cai no ingles.</summary>
    public static string DoWindows() => CultureInfo.InstalledUICulture.TwoLetterISOLanguageName.ToLowerInvariant();

    public static string T(string chave) =>
        _atual.TryGetValue(chave, out var v) ? v :
        En.TryGetValue(chave, out var p) ? p : chave;

    static readonly Dictionary<string, string> Pt = new()
    {
        ["geral"] = "Geral",
        ["sistema"] = "Sistema",
        ["sobre"] = "Sobre",
        ["esconder_apos"] = "Esconder o cursor após",
        ["segundos_sem_uso"] = "segundos sem uso do mouse ou teclado",
        ["atalho"] = "Esconder o mouse na hora",
        ["atalho_dica"] = "pressione a combinação",
        ["atalho_ocupado"] = "combinação já usada por outro app",
        ["iniciar_windows"] = "Iniciar com o Windows",
        ["iniciar_windows_obs"] = "Grava só no seu usuário, sem pedir administrador.",
        ["idioma"] = "Idioma",
        ["idioma_obs"] = "Auto segue o idioma do Windows.",
        ["versao"] = "versão",
        ["descricao"] = "Esconde o cursor depois de um tempo sem uso e o traz de volta no primeiro movimento.",
        ["limitacao"] = "Limitação conhecida: jogos em tela cheia exclusiva desenham o próprio cursor e ignoram a troca de cursor do sistema. Isso é comportamento do Windows, não é falha do app.",
        ["menu_pausar"] = "Pausar",
        ["menu_sair"] = "Fechar",
        ["pausado"] = "Pausado",
        ["segundos_curto"] = "s",
        ["minutos_curto"] = "min",
        ["contador"] = "Escondendo em",
        ["contador_pausado"] = "pausado",
        ["contador_escondido"] = "cursor escondido",
        ["buscar_atualizacao"] = "Procurar atualizações",
        ["verificando"] = "verificando...",
        ["atualizado"] = "você já está na versão mais recente",
        ["nova_versao"] = "versão {0} disponível",
        ["baixar"] = "Atualizar agora",
        ["baixando"] = "baixando {0}%",
        ["instalando"] = "instalando...",
        ["falha_verificar"] = "não deu para verificar agora",
        ["falha_conexao"] = "Falha na conexão",
        ["falha_baixar"] = "não deu para baixar a atualização"
    };

    static readonly Dictionary<string, string> En = new()
    {
        ["geral"] = "General",
        ["sistema"] = "System",
        ["sobre"] = "About",
        ["esconder_apos"] = "Hide the cursor after",
        ["segundos_sem_uso"] = "seconds without mouse or keyboard",
        ["atalho"] = "Hide the cursor now",
        ["atalho_dica"] = "press the combination",
        ["atalho_ocupado"] = "combination already used by another app",
        ["iniciar_windows"] = "Start with Windows",
        ["iniciar_windows_obs"] = "Written to your user only, no administrator needed.",
        ["idioma"] = "Language",
        ["idioma_obs"] = "Auto follows the Windows language.",
        ["versao"] = "version",
        ["descricao"] = "Hides the cursor after a period without input and brings it back on the first movement.",
        ["limitacao"] = "Known limitation: games in exclusive fullscreen draw their own cursor and ignore the system cursor swap. That is Windows behaviour, not an app fault.",
        ["menu_pausar"] = "Pause",
        ["menu_sair"] = "Close",
        ["pausado"] = "Paused",
        ["segundos_curto"] = "s",
        ["minutos_curto"] = "min",
        ["contador"] = "Hiding in",
        ["contador_pausado"] = "paused",
        ["contador_escondido"] = "cursor hidden",
        ["buscar_atualizacao"] = "Check for updates",
        ["verificando"] = "checking...",
        ["atualizado"] = "you're on the latest version",
        ["nova_versao"] = "version {0} available",
        ["baixar"] = "Update now",
        ["baixando"] = "downloading {0}%",
        ["instalando"] = "installing...",
        ["falha_verificar"] = "couldn't check right now",
        ["falha_conexao"] = "Connection failed",
        ["falha_baixar"] = "couldn't download the update"
    };

    static readonly Dictionary<string, string> Es = new()
    {
        ["geral"] = "General",
        ["sistema"] = "Sistema",
        ["sobre"] = "Acerca de",
        ["esconder_apos"] = "Ocultar el cursor tras",
        ["segundos_sem_uso"] = "segundos sin ratón ni teclado",
        ["atalho"] = "Ocultar el ratón al instante",
        ["atalho_dica"] = "pulsa la combinación",
        ["atalho_ocupado"] = "combinación ya usada por otra app",
        ["iniciar_windows"] = "Iniciar con Windows",
        ["iniciar_windows_obs"] = "Se guarda solo en tu usuario, sin administrador.",
        ["idioma"] = "Idioma",
        ["idioma_obs"] = "Auto sigue el idioma de Windows.",
        ["versao"] = "versión",
        ["descricao"] = "Oculta el cursor tras un tiempo sin uso y lo devuelve al primer movimiento.",
        ["limitacao"] = "Limitación conocida: los juegos en pantalla completa exclusiva dibujan su propio cursor e ignoran el cambio de cursor del sistema. Es comportamiento de Windows, no un fallo de la app.",
        ["menu_pausar"] = "Pausar",
        ["menu_sair"] = "Cerrar",
        ["pausado"] = "En pausa",
        ["segundos_curto"] = "s",
        ["minutos_curto"] = "min",
        ["contador"] = "Ocultando en",
        ["contador_pausado"] = "en pausa",
        ["contador_escondido"] = "cursor oculto",
        ["buscar_atualizacao"] = "Buscar actualizaciones",
        ["verificando"] = "comprobando...",
        ["atualizado"] = "ya tienes la última versión",
        ["nova_versao"] = "versión {0} disponible",
        ["baixar"] = "Actualizar ahora",
        ["baixando"] = "descargando {0}%",
        ["instalando"] = "instalando...",
        ["falha_verificar"] = "no se pudo comprobar ahora",
        ["falha_conexao"] = "Error de conexión",
        ["falha_baixar"] = "no se pudo descargar la actualización"
    };

    static readonly Dictionary<string, string> Fr = new()
    {
        ["geral"] = "Général",
        ["sistema"] = "Système",
        ["sobre"] = "À propos",
        ["esconder_apos"] = "Masquer le curseur après",
        ["segundos_sem_uso"] = "secondes sans souris ni clavier",
        ["atalho"] = "Masquer le curseur maintenant",
        ["atalho_dica"] = "appuyez sur la combinaison",
        ["atalho_ocupado"] = "combinaison déjà utilisée par une autre app",
        ["iniciar_windows"] = "Démarrer avec Windows",
        ["iniciar_windows_obs"] = "Écrit uniquement pour votre utilisateur, sans administrateur.",
        ["idioma"] = "Langue",
        ["idioma_obs"] = "Auto suit la langue de Windows.",
        ["versao"] = "version",
        ["descricao"] = "Masque le curseur après un temps sans activité et le ramène au premier mouvement.",
        ["limitacao"] = "Limite connue : les jeux en plein écran exclusif dessinent leur propre curseur et ignorent le changement de curseur du système. C'est le comportement de Windows, pas un défaut de l'app.",
        ["menu_pausar"] = "Pause",
        ["menu_sair"] = "Fermer",
        ["pausado"] = "En pause",
        ["segundos_curto"] = "s",
        ["minutos_curto"] = "min",
        ["contador"] = "Masquage dans",
        ["contador_pausado"] = "en pause",
        ["contador_escondido"] = "curseur masqué",
        ["buscar_atualizacao"] = "Rechercher des mises à jour",
        ["verificando"] = "vérification...",
        ["atualizado"] = "vous avez déjà la dernière version",
        ["nova_versao"] = "version {0} disponible",
        ["baixar"] = "Mettre à jour",
        ["baixando"] = "téléchargement {0}%",
        ["instalando"] = "installation...",
        ["falha_verificar"] = "impossible de vérifier maintenant",
        ["falha_conexao"] = "Échec de connexion",
        ["falha_baixar"] = "impossible de télécharger la mise à jour"
    };

    static readonly Dictionary<string, string> De = new()
    {
        ["geral"] = "Allgemein",
        ["sistema"] = "System",
        ["sobre"] = "Über",
        ["esconder_apos"] = "Zeiger ausblenden nach",
        ["segundos_sem_uso"] = "Sekunden ohne Maus oder Tastatur",
        ["atalho"] = "Maus sofort ausblenden",
        ["atalho_dica"] = "Tastenkombination drücken",
        ["atalho_ocupado"] = "Kombination bereits von einer anderen App belegt",
        ["iniciar_windows"] = "Mit Windows starten",
        ["iniciar_windows_obs"] = "Nur für Ihr Benutzerkonto, ohne Administrator.",
        ["idioma"] = "Sprache",
        ["idioma_obs"] = "Auto folgt der Windows-Sprache.",
        ["versao"] = "Version",
        ["descricao"] = "Blendet den Zeiger nach einer Zeit ohne Eingabe aus und holt ihn bei der ersten Bewegung zurück.",
        ["limitacao"] = "Bekannte Einschränkung: Spiele im exklusiven Vollbild zeichnen ihren eigenen Zeiger und ignorieren den Systemzeiger. Das ist Windows-Verhalten, kein Fehler der App.",
        ["menu_pausar"] = "Pause",
        ["menu_sair"] = "Schließen",
        ["pausado"] = "Pausiert",
        ["segundos_curto"] = "s",
        ["minutos_curto"] = "min",
        ["contador"] = "Ausblenden in",
        ["contador_pausado"] = "pausiert",
        ["contador_escondido"] = "Zeiger ausgeblendet",
        ["buscar_atualizacao"] = "Nach Updates suchen",
        ["verificando"] = "wird geprüft...",
        ["atualizado"] = "Sie haben die neueste Version",
        ["nova_versao"] = "Version {0} verfügbar",
        ["baixar"] = "Jetzt aktualisieren",
        ["baixando"] = "Download {0}%",
        ["instalando"] = "wird installiert...",
        ["falha_verificar"] = "Prüfung gerade nicht möglich",
        ["falha_conexao"] = "Verbindung fehlgeschlagen",
        ["falha_baixar"] = "Update konnte nicht geladen werden"
    };

    static readonly Dictionary<string, string> It = new()
    {
        ["geral"] = "Generale",
        ["sistema"] = "Sistema",
        ["sobre"] = "Informazioni",
        ["esconder_apos"] = "Nascondi il cursore dopo",
        ["segundos_sem_uso"] = "secondi senza mouse o tastiera",
        ["atalho"] = "Nascondi il mouse subito",
        ["atalho_dica"] = "premi la combinazione",
        ["atalho_ocupado"] = "combinazione già usata da un'altra app",
        ["iniciar_windows"] = "Avvia con Windows",
        ["iniciar_windows_obs"] = "Scritto solo per il tuo utente, senza amministratore.",
        ["idioma"] = "Lingua",
        ["idioma_obs"] = "Auto segue la lingua di Windows.",
        ["versao"] = "versione",
        ["descricao"] = "Nasconde il cursore dopo un periodo di inattività e lo riporta al primo movimento.",
        ["limitacao"] = "Limite noto: i giochi in schermo intero esclusivo disegnano il proprio cursore e ignorano il cambio di cursore di sistema. È il comportamento di Windows, non un difetto dell'app.",
        ["menu_pausar"] = "Pausa",
        ["menu_sair"] = "Chiudi",
        ["pausado"] = "In pausa",
        ["segundos_curto"] = "s",
        ["minutos_curto"] = "min",
        ["contador"] = "Nascondo tra",
        ["contador_pausado"] = "in pausa",
        ["contador_escondido"] = "cursore nascosto",
        ["buscar_atualizacao"] = "Cerca aggiornamenti",
        ["verificando"] = "controllo...",
        ["atualizado"] = "hai già l'ultima versione",
        ["nova_versao"] = "versione {0} disponibile",
        ["baixar"] = "Aggiorna ora",
        ["baixando"] = "download {0}%",
        ["instalando"] = "installazione...",
        ["falha_verificar"] = "impossibile controllare ora",
        ["falha_conexao"] = "Connessione non riuscita",
        ["falha_baixar"] = "impossibile scaricare l'aggiornamento"
    };

    static readonly Dictionary<string, string> Ru = new()
    {
        ["geral"] = "Общие",
        ["sistema"] = "Система",
        ["sobre"] = "О программе",
        ["esconder_apos"] = "Скрывать курсор через",
        ["segundos_sem_uso"] = "секунд без мыши и клавиатуры",
        ["atalho"] = "Скрыть курсор сразу",
        ["atalho_dica"] = "нажмите сочетание клавиш",
        ["atalho_ocupado"] = "сочетание уже занято другим приложением",
        ["iniciar_windows"] = "Запускать вместе с Windows",
        ["iniciar_windows_obs"] = "Записывается только для вашего пользователя, без прав администратора.",
        ["idioma"] = "Язык",
        ["idioma_obs"] = "Авто следует языку Windows.",
        ["versao"] = "версия",
        ["descricao"] = "Скрывает курсор после простоя и возвращает его при первом движении.",
        ["limitacao"] = "Известное ограничение: игры в эксклюзивном полноэкранном режиме рисуют собственный курсор и игнорируют системный. Это поведение Windows, а не ошибка приложения.",
        ["menu_pausar"] = "Пауза",
        ["menu_sair"] = "Закрыть",
        ["pausado"] = "Пауза",
        ["segundos_curto"] = "с",
        ["minutos_curto"] = "мин",
        ["contador"] = "Скрытие через",
        ["contador_pausado"] = "пауза",
        ["contador_escondido"] = "курсор скрыт",
        ["buscar_atualizacao"] = "Проверить обновления",
        ["verificando"] = "проверка...",
        ["atualizado"] = "у вас последняя версия",
        ["nova_versao"] = "доступна версия {0}",
        ["baixar"] = "Обновить",
        ["baixando"] = "загрузка {0}%",
        ["instalando"] = "установка...",
        ["falha_verificar"] = "сейчас не удалось проверить",
        ["falha_conexao"] = "Ошибка соединения",
        ["falha_baixar"] = "не удалось скачать обновление"
    };

    static readonly Dictionary<string, string> Ja = new()
    {
        ["geral"] = "一般",
        ["sistema"] = "システム",
        ["sobre"] = "このアプリについて",
        ["esconder_apos"] = "カーソルを隠すまで",
        ["segundos_sem_uso"] = "マウスやキーボードの操作がないとき",
        ["atalho"] = "すぐにカーソルを隠す",
        ["atalho_dica"] = "キーの組み合わせを押してください",
        ["atalho_ocupado"] = "この組み合わせは他のアプリが使用中です",
        ["iniciar_windows"] = "Windows 起動時に開始",
        ["iniciar_windows_obs"] = "現在のユーザーにのみ登録します。管理者権限は不要です。",
        ["idioma"] = "言語",
        ["idioma_obs"] = "自動は Windows の言語に従います。",
        ["versao"] = "バージョン",
        ["descricao"] = "操作がない状態がしばらく続くとカーソルを隠し、最初の動きで元に戻します。",
        ["limitacao"] = "既知の制限: 排他的フルスクリーンのゲームは独自のカーソルを描画するため、システムカーソルの変更を無視します。これは Windows の仕様であり、アプリの不具合ではありません。",
        ["menu_pausar"] = "一時停止",
        ["menu_sair"] = "閉じる",
        ["pausado"] = "一時停止中",
        ["segundos_curto"] = "秒",
        ["minutos_curto"] = "分",
        ["contador"] = "隠すまで",
        ["contador_pausado"] = "一時停止中",
        ["contador_escondido"] = "カーソル非表示",
        ["buscar_atualizacao"] = "更新を確認",
        ["verificando"] = "確認中...",
        ["atualizado"] = "最新バージョンです",
        ["nova_versao"] = "バージョン {0} が利用できます",
        ["baixar"] = "今すぐ更新",
        ["baixando"] = "ダウンロード中 {0}%",
        ["instalando"] = "インストール中...",
        ["falha_verificar"] = "今は確認できませんでした",
        ["falha_conexao"] = "接続に失敗しました",
        ["falha_baixar"] = "更新をダウンロードできませんでした"
    };

    static readonly Dictionary<string, string> Zh = new()
    {
        ["geral"] = "常规",
        ["sistema"] = "系统",
        ["sobre"] = "关于",
        ["esconder_apos"] = "隐藏光标前等待",
        ["segundos_sem_uso"] = "无鼠标或键盘操作",
        ["atalho"] = "立即隐藏鼠标",
        ["atalho_dica"] = "请按下组合键",
        ["atalho_ocupado"] = "该组合键已被其他应用占用",
        ["iniciar_windows"] = "开机时启动",
        ["iniciar_windows_obs"] = "仅写入当前用户，无需管理员权限。",
        ["idioma"] = "语言",
        ["idioma_obs"] = "自动跟随 Windows 语言。",
        ["versao"] = "版本",
        ["descricao"] = "在一段时间无操作后隐藏光标，并在第一次移动时恢复。",
        ["limitacao"] = "已知限制：独占全屏游戏会绘制自己的光标并忽略系统光标替换。这是 Windows 的行为，不是本应用的缺陷。",
        ["menu_pausar"] = "暂停",
        ["menu_sair"] = "关闭",
        ["pausado"] = "已暂停",
        ["segundos_curto"] = "秒",
        ["minutos_curto"] = "分",
        ["contador"] = "隐藏倒计时",
        ["contador_pausado"] = "已暂停",
        ["contador_escondido"] = "光标已隐藏",
        ["buscar_atualizacao"] = "检查更新",
        ["verificando"] = "正在检查...",
        ["atualizado"] = "已是最新版本",
        ["nova_versao"] = "有新版本 {0}",
        ["baixar"] = "立即更新",
        ["baixando"] = "下载中 {0}%",
        ["instalando"] = "正在安装...",
        ["falha_verificar"] = "暂时无法检查",
        ["falha_conexao"] = "连接失败",
        ["falha_baixar"] = "无法下载更新"
    };

    static readonly Dictionary<string, string> Ko = new()
    {
        ["geral"] = "일반",
        ["sistema"] = "시스템",
        ["sobre"] = "정보",
        ["esconder_apos"] = "커서를 숨기기까지",
        ["segundos_sem_uso"] = "동안 마우스나 키보드 입력이 없을 때",
        ["atalho"] = "마우스 즉시 숨기기",
        ["atalho_dica"] = "조합 키를 누르세요",
        ["atalho_ocupado"] = "다른 앱이 이미 사용 중인 조합입니다",
        ["iniciar_windows"] = "Windows 시작 시 실행",
        ["iniciar_windows_obs"] = "현재 사용자에게만 기록하며 관리자 권한이 필요 없습니다.",
        ["idioma"] = "언어",
        ["idioma_obs"] = "자동은 Windows 언어를 따릅니다.",
        ["versao"] = "버전",
        ["descricao"] = "입력이 없는 시간이 지나면 커서를 숨기고 첫 움직임에 다시 표시합니다.",
        ["limitacao"] = "알려진 제한: 전체 화면 전용 모드의 게임은 자체 커서를 그리므로 시스템 커서 변경을 무시합니다. 이는 Windows 동작이며 앱의 결함이 아닙니다.",
        ["menu_pausar"] = "일시 중지",
        ["menu_sair"] = "닫기",
        ["pausado"] = "일시 중지됨",
        ["segundos_curto"] = "초",
        ["minutos_curto"] = "분",
        ["contador"] = "숨기기까지",
        ["contador_pausado"] = "일시 중지됨",
        ["contador_escondido"] = "커서 숨김",
        ["buscar_atualizacao"] = "업데이트 확인",
        ["verificando"] = "확인 중...",
        ["atualizado"] = "최신 버전입니다",
        ["nova_versao"] = "버전 {0} 사용 가능",
        ["baixar"] = "지금 업데이트",
        ["baixando"] = "다운로드 중 {0}%",
        ["instalando"] = "설치 중...",
        ["falha_verificar"] = "지금은 확인할 수 없습니다",
        ["falha_conexao"] = "연결 실패",
        ["falha_baixar"] = "업데이트를 내려받지 못했습니다"
    };
}
