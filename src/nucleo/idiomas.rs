//! Tabela de textos. Chaves sao variantes de enum: chave errada nao compila, e cada idioma
//! e um array do mesmo tamanho, entao faltar traducao tambem nao compila.

use std::sync::atomic::{AtomicUsize, Ordering};

/// Codigo e nome nativo, na ordem da lista da tela Sistema.
pub const DISPONIVEIS: &[(&str, &str)] = &[
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
    ("ko", "한국어"),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum T {
    Geral,
    Sistema,
    Sobre,
    EsconderApos,
    SegundosSemUso,
    Atalho,
    AtalhoDica,
    AtalhoOcupado,
    IniciarWindows,
    IniciarWindowsObs,
    Idioma,
    IdiomaObs,
    Versao,
    Descricao,
    Limitacao,
    MenuPausar,
    MenuSair,
    Pausado,
    SegundosCurto,
    MinutosCurto,
    Contador,
    ContadorPausado,
    ContadorEscondido,
    BuscarAtualizacao,
    Verificando,
    Atualizado,
    NovaVersao,
    Baixar,
    Baixando,
    Instalando,
    FalhaVerificar,
    FalhaConexao,
    FalhaBaixar,
    Ativo,
}

const N: usize = 34;

const PT: [&str; N] = [
    "Geral",
    "Sistema",
    "Sobre",
    "Esconder o cursor após",
    "segundos sem uso do mouse ou teclado",
    "Esconder o mouse na hora",
    "pressione a combinação",
    "combinação já usada por outro app",
    "Iniciar com o Windows",
    "Grava só no seu usuário, sem pedir administrador.",
    "Idioma",
    "Auto segue o idioma do Windows.",
    "versão",
    "Esconde o cursor depois de um tempo sem uso e o traz de volta no primeiro movimento.",
    "Limitação conhecida: jogos em tela cheia exclusiva desenham o próprio cursor e ignoram a troca de cursor do sistema. Isso é comportamento do Windows, não é falha do app.",
    "Pausar",
    "Fechar",
    "Pausado",
    "s",
    "min",
    "Escondendo em",
    "pausado",
    "cursor escondido",
    "Procurar atualizações",
    "verificando...",
    "você já está na versão mais recente",
    "versão {0} disponível",
    "Atualizar agora",
    "baixando {0}%",
    "instalando...",
    "não deu para verificar agora",
    "Falha na conexão",
    "não deu para baixar a atualização",
    "Ativo",
];

const EN: [&str; N] = [
    "General",
    "System",
    "About",
    "Hide the cursor after",
    "seconds without mouse or keyboard",
    "Hide the cursor now",
    "press the combination",
    "combination already used by another app",
    "Start with Windows",
    "Written to your user only, no administrator needed.",
    "Language",
    "Auto follows the Windows language.",
    "version",
    "Hides the cursor after a period without input and brings it back on the first movement.",
    "Known limitation: games in exclusive fullscreen draw their own cursor and ignore the system cursor swap. That is Windows behaviour, not an app fault.",
    "Pause",
    "Close",
    "Paused",
    "s",
    "min",
    "Hiding in",
    "paused",
    "cursor hidden",
    "Check for updates",
    "checking...",
    "you're on the latest version",
    "version {0} available",
    "Update now",
    "downloading {0}%",
    "installing...",
    "couldn't check right now",
    "Connection failed",
    "couldn't download the update",
    "Active",
];

const ES: [&str; N] = [
    "General",
    "Sistema",
    "Acerca de",
    "Ocultar el cursor tras",
    "segundos sin ratón ni teclado",
    "Ocultar el ratón al instante",
    "pulsa la combinación",
    "combinación ya usada por otra app",
    "Iniciar con Windows",
    "Se guarda solo en tu usuario, sin administrador.",
    "Idioma",
    "Auto sigue el idioma de Windows.",
    "versión",
    "Oculta el cursor tras un tiempo sin uso y lo devuelve al primer movimiento.",
    "Limitación conocida: los juegos en pantalla completa exclusiva dibujan su propio cursor e ignoran el cambio de cursor del sistema. Es comportamiento de Windows, no un fallo de la app.",
    "Pausar",
    "Cerrar",
    "En pausa",
    "s",
    "min",
    "Ocultando en",
    "en pausa",
    "cursor oculto",
    "Buscar actualizaciones",
    "comprobando...",
    "ya tienes la última versión",
    "versión {0} disponible",
    "Actualizar ahora",
    "descargando {0}%",
    "instalando...",
    "no se pudo comprobar ahora",
    "Error de conexión",
    "no se pudo descargar la actualización",
    "Activo",
];

const FR: [&str; N] = [
    "Général",
    "Système",
    "À propos",
    "Masquer le curseur après",
    "secondes sans souris ni clavier",
    "Masquer le curseur maintenant",
    "appuyez sur la combinaison",
    "combinaison déjà utilisée par une autre app",
    "Démarrer avec Windows",
    "Écrit uniquement pour votre utilisateur, sans administrateur.",
    "Langue",
    "Auto suit la langue de Windows.",
    "version",
    "Masque le curseur après un temps sans activité et le ramène au premier mouvement.",
    "Limite connue : les jeux en plein écran exclusif dessinent leur propre curseur et ignorent le changement de curseur du système. C'est le comportement de Windows, pas un défaut de l'app.",
    "Pause",
    "Fermer",
    "En pause",
    "s",
    "min",
    "Masquage dans",
    "en pause",
    "curseur masqué",
    "Rechercher des mises à jour",
    "vérification...",
    "vous avez déjà la dernière version",
    "version {0} disponible",
    "Mettre à jour",
    "téléchargement {0}%",
    "installation...",
    "impossible de vérifier maintenant",
    "Échec de connexion",
    "impossible de télécharger la mise à jour",
    "Actif",
];

const DE: [&str; N] = [
    "Allgemein",
    "System",
    "Über",
    "Zeiger ausblenden nach",
    "Sekunden ohne Maus oder Tastatur",
    "Maus sofort ausblenden",
    "Tastenkombination drücken",
    "Kombination bereits von einer anderen App belegt",
    "Mit Windows starten",
    "Nur für Ihr Benutzerkonto, ohne Administrator.",
    "Sprache",
    "Auto folgt der Windows-Sprache.",
    "Version",
    "Blendet den Zeiger nach einer Zeit ohne Eingabe aus und holt ihn bei der ersten Bewegung zurück.",
    "Bekannte Einschränkung: Spiele im exklusiven Vollbild zeichnen ihren eigenen Zeiger und ignorieren den Systemzeiger. Das ist Windows-Verhalten, kein Fehler der App.",
    "Pause",
    "Schließen",
    "Pausiert",
    "s",
    "min",
    "Ausblenden in",
    "pausiert",
    "Zeiger ausgeblendet",
    "Nach Updates suchen",
    "wird geprüft...",
    "Sie haben die neueste Version",
    "Version {0} verfügbar",
    "Jetzt aktualisieren",
    "Download {0}%",
    "wird installiert...",
    "Prüfung gerade nicht möglich",
    "Verbindung fehlgeschlagen",
    "Update konnte nicht geladen werden",
    "Aktiv",
];

const IT: [&str; N] = [
    "Generale",
    "Sistema",
    "Informazioni",
    "Nascondi il cursore dopo",
    "secondi senza mouse o tastiera",
    "Nascondi il mouse subito",
    "premi la combinazione",
    "combinazione già usata da un'altra app",
    "Avvia con Windows",
    "Scritto solo per il tuo utente, senza amministratore.",
    "Lingua",
    "Auto segue la lingua di Windows.",
    "versione",
    "Nasconde il cursore dopo un periodo di inattività e lo riporta al primo movimento.",
    "Limite noto: i giochi in schermo intero esclusivo disegnano il proprio cursore e ignorano il cambio di cursore di sistema. È il comportamento di Windows, non un difetto dell'app.",
    "Pausa",
    "Chiudi",
    "In pausa",
    "s",
    "min",
    "Nascondo tra",
    "in pausa",
    "cursore nascosto",
    "Cerca aggiornamenti",
    "controllo...",
    "hai già l'ultima versione",
    "versione {0} disponibile",
    "Aggiorna ora",
    "download {0}%",
    "installazione...",
    "impossibile controllare ora",
    "Connessione non riuscita",
    "impossibile scaricare l'aggiornamento",
    "Attivo",
];

const RU: [&str; N] = [
    "Общие",
    "Система",
    "О программе",
    "Скрывать курсор через",
    "секунд без мыши и клавиатуры",
    "Скрыть курсор сразу",
    "нажмите сочетание клавиш",
    "сочетание уже занято другим приложением",
    "Запускать вместе с Windows",
    "Записывается только для вашего пользователя, без прав администратора.",
    "Язык",
    "Авто следует языку Windows.",
    "версия",
    "Скрывает курсор после простоя и возвращает его при первом движении.",
    "Известное ограничение: игры в эксклюзивном полноэкранном режиме рисуют собственный курсор и игнорируют системный. Это поведение Windows, а не ошибка приложения.",
    "Пауза",
    "Закрыть",
    "Пауза",
    "с",
    "мин",
    "Скрытие через",
    "пауза",
    "курсор скрыт",
    "Проверить обновления",
    "проверка...",
    "у вас последняя версия",
    "доступна версия {0}",
    "Обновить",
    "загрузка {0}%",
    "установка...",
    "сейчас не удалось проверить",
    "Ошибка соединения",
    "не удалось скачать обновление",
    "Активен",
];

const JA: [&str; N] = [
    "一般",
    "システム",
    "このアプリについて",
    "カーソルを隠すまで",
    "マウスやキーボードの操作がないとき",
    "すぐにカーソルを隠す",
    "キーの組み合わせを押してください",
    "この組み合わせは他のアプリが使用中です",
    "Windows 起動時に開始",
    "現在のユーザーにのみ登録します。管理者権限は不要です。",
    "言語",
    "自動は Windows の言語に従います。",
    "バージョン",
    "操作がない状態がしばらく続くとカーソルを隠し、最初の動きで元に戻します。",
    "既知の制限: 排他的フルスクリーンのゲームは独自のカーソルを描画するため、システムカーソルの変更を無視します。これは Windows の仕様であり、アプリの不具合ではありません。",
    "一時停止",
    "閉じる",
    "一時停止中",
    "秒",
    "分",
    "隠すまで",
    "一時停止中",
    "カーソル非表示",
    "更新を確認",
    "確認中...",
    "最新バージョンです",
    "バージョン {0} が利用できます",
    "今すぐ更新",
    "ダウンロード中 {0}%",
    "インストール中...",
    "今は確認できませんでした",
    "接続に失敗しました",
    "更新をダウンロードできませんでした",
    "有効",
];

const ZH: [&str; N] = [
    "常规",
    "系统",
    "关于",
    "隐藏光标前等待",
    "无鼠标或键盘操作",
    "立即隐藏鼠标",
    "请按下组合键",
    "该组合键已被其他应用占用",
    "开机时启动",
    "仅写入当前用户，无需管理员权限。",
    "语言",
    "自动跟随 Windows 语言。",
    "版本",
    "在一段时间无操作后隐藏光标，并在第一次移动时恢复。",
    "已知限制：独占全屏游戏会绘制自己的光标并忽略系统光标替换。这是 Windows 的行为，不是本应用的缺陷。",
    "暂停",
    "关闭",
    "已暂停",
    "秒",
    "分",
    "隐藏倒计时",
    "已暂停",
    "光标已隐藏",
    "检查更新",
    "正在检查...",
    "已是最新版本",
    "有新版本 {0}",
    "立即更新",
    "下载中 {0}%",
    "正在安装...",
    "暂时无法检查",
    "连接失败",
    "无法下载更新",
    "已启用",
];

const KO: [&str; N] = [
    "일반",
    "시스템",
    "정보",
    "커서를 숨기기까지",
    "동안 마우스나 키보드 입력이 없을 때",
    "마우스 즉시 숨기기",
    "조합 키를 누르세요",
    "다른 앱이 이미 사용 중인 조합입니다",
    "Windows 시작 시 실행",
    "현재 사용자에게만 기록하며 관리자 권한이 필요 없습니다.",
    "언어",
    "자동은 Windows 언어를 따릅니다.",
    "버전",
    "입력이 없는 시간이 지나면 커서를 숨기고 첫 움직임에 다시 표시합니다.",
    "알려진 제한: 전체 화면 전용 모드의 게임은 자체 커서를 그리므로 시스템 커서 변경을 무시합니다. 이는 Windows 동작이며 앱의 결함이 아닙니다.",
    "일시 중지",
    "닫기",
    "일시 중지됨",
    "초",
    "분",
    "숨기기까지",
    "일시 중지됨",
    "커서 숨김",
    "업데이트 확인",
    "확인 중...",
    "최신 버전입니다",
    "버전 {0} 사용 가능",
    "지금 업데이트",
    "다운로드 중 {0}%",
    "설치 중...",
    "지금은 확인할 수 없습니다",
    "연결 실패",
    "업데이트를 내려받지 못했습니다",
    "활성",
];

const TABELAS: [(&str, &[&str; N]); 10] = [
    ("pt", &PT),
    ("en", &EN),
    ("es", &ES),
    ("fr", &FR),
    ("de", &DE),
    ("it", &IT),
    ("ru", &RU),
    ("ja", &JA),
    ("zh", &ZH),
    ("ko", &KO),
];

/// Indice em DISPONIVEIS escolhido pelo usuario (0 = auto) e indice efetivo em TABELAS.
static ESCOLHIDO: AtomicUsize = AtomicUsize::new(0);
static EFETIVO: AtomicUsize = AtomicUsize::new(1);

/// Define o idioma. Codigo desconhecido vira "auto"; idioma do sistema sem traducao cai no ingles.
pub fn definir(codigo: &str, do_sistema: &str) {
    let escolhido = DISPONIVEIS.iter().position(|(c, _)| *c == codigo).unwrap_or(0);
    let efetivo = if escolhido == 0 { do_sistema } else { DISPONIVEIS[escolhido].0 };
    let tabela = TABELAS.iter().position(|(c, _)| *c == efetivo).unwrap_or(1);
    ESCOLHIDO.store(escolhido, Ordering::Relaxed);
    EFETIVO.store(tabela, Ordering::Relaxed);
}

/// Codigo escolhido ("auto", "pt", ...), o que vai para o config.
pub fn codigo() -> &'static str {
    DISPONIVEIS[ESCOLHIDO.load(Ordering::Relaxed)].0
}

/// Codigo efetivamente em uso, depois de resolver o "auto".
pub fn efetivo() -> &'static str {
    TABELAS[EFETIVO.load(Ordering::Relaxed)].0
}

pub fn t(chave: T) -> &'static str {
    TABELAS[EFETIVO.load(Ordering::Relaxed)].1[chave as usize]
}

/// Texto com "{0}" trocado pelo valor.
pub fn tf(chave: T, valor: impl std::fmt::Display) -> String {
    t(chave).replace("{0}", &valor.to_string())
}

/// Nome para a lista; "Auto" ganha o idioma do sistema entre parenteses.
pub fn nome_de(codigo: &str, do_sistema: &str) -> String {
    let nome = DISPONIVEIS.iter().find(|(c, _)| *c == codigo).map_or(codigo, |(_, n)| *n);
    if codigo == "auto" { format!("{nome} ({})", do_sistema.to_uppercase()) } else { nome.to_owned() }
}

/// Separador decimal do idioma em uso, para o contador ("3,2 s" em pt, "3.2 s" em en).
pub fn decimal(valor: f64) -> String {
    let texto = format!("{valor:.1}");
    match efetivo() {
        "en" | "ja" | "zh" | "ko" => texto,
        _ => texto.replace('.', ","),
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn auto_resolve_pelo_sistema_e_cai_no_ingles() {
        definir("auto", "pt");
        assert_eq!(t(T::Geral), "Geral");
        definir("auto", "nl");
        assert_eq!(efetivo(), "en");
        definir("xx", "de");
        assert_eq!(codigo(), "auto");
        assert_eq!(efetivo(), "de");
        definir("ja", "pt");
        assert_eq!(codigo(), "ja");
        definir("pt", "en");
    }

    #[test]
    fn placeholders_existem_em_todos_os_idiomas() {
        for (_, tabela) in TABELAS {
            assert!(tabela[T::NovaVersao as usize].contains("{0}"));
            assert!(tabela[T::Baixando as usize].contains("{0}"));
        }
    }
}
