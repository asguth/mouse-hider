using System.Globalization;

namespace MouseHider;

/// <summary>
/// Tabela de textos. ponytail: dicionario em memoria em vez de .resx — sao ~20 frases,
/// e assim traduzir e editar um arquivo, sem passar por ferramenta de recurso.
/// </summary>
static class Idiomas
{
    public static readonly (string Codigo, string Nome)[] Disponiveis =
    {
        ("auto", "Auto"), ("pt", "Português"), ("en", "English"), ("es", "Español")
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
        _atual = efetivo switch { "pt" => Pt, "es" => Es, _ => En };
    }

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
        ["atalho"] = "Atalho para pausar e retomar",
        ["atalho_dica"] = "clique e pressione a combinação",
        ["atalho_ocupado"] = "combinação já usada por outro app",
        ["iniciar_windows"] = "Iniciar com o Windows",
        ["iniciar_windows_obs"] = "Grava só no seu usuário, sem pedir administrador.",
        ["idioma"] = "Idioma",
        ["idioma_obs"] = "Auto segue o idioma do Windows.",
        ["versao"] = "versão",
        ["descricao"] = "Esconde o cursor depois de um tempo sem uso e o traz de volta no primeiro movimento.",
        ["limitacao"] = "Limitação conhecida: jogos em tela cheia exclusiva desenham o próprio cursor e ignoram a troca de cursor do sistema. Isso é comportamento do Windows, não é falha do app.",
        ["menu_config"] = "Configurações",
        ["menu_pausar"] = "Pausar",
        ["menu_sair"] = "Sair",
        ["pausado"] = "Pausado",
        ["ativo"] = "Ativo",
        ["segundos_curto"] = "s",
        ["contador"] = "Escondendo em",
        ["contador_pausado"] = "pausado",
        ["contador_escondido"] = "cursor escondido",
        ["buscar_atualizacao"] = "Procurar atualizações",
        ["verificando"] = "verificando...",
        ["atualizado"] = "você já está na versão mais recente",
        ["nova_versao"] = "versão {0} disponível",
        ["baixar"] = "Baixar",
        ["falha_verificar"] = "não deu para verificar agora"
    };

    static readonly Dictionary<string, string> En = new()
    {
        ["geral"] = "General",
        ["sistema"] = "System",
        ["sobre"] = "About",
        ["esconder_apos"] = "Hide the cursor after",
        ["segundos_sem_uso"] = "seconds without mouse or keyboard",
        ["atalho"] = "Shortcut to pause and resume",
        ["atalho_dica"] = "click and press the combination",
        ["atalho_ocupado"] = "combination already used by another app",
        ["iniciar_windows"] = "Start with Windows",
        ["iniciar_windows_obs"] = "Written to your user only, no administrator needed.",
        ["idioma"] = "Language",
        ["idioma_obs"] = "Auto follows the Windows language.",
        ["versao"] = "version",
        ["descricao"] = "Hides the cursor after a period without input and brings it back on the first movement.",
        ["limitacao"] = "Known limitation: games in exclusive fullscreen draw their own cursor and ignore the system cursor swap. That is Windows behaviour, not an app fault.",
        ["menu_config"] = "Settings",
        ["menu_pausar"] = "Pause",
        ["menu_sair"] = "Exit",
        ["pausado"] = "Paused",
        ["ativo"] = "Active",
        ["segundos_curto"] = "s",
        ["minutos_curto"] = "min",
        ["contador"] = "Hiding in",
        ["contador_pausado"] = "paused",
        ["contador_escondido"] = "cursor hidden",
        ["buscar_atualizacao"] = "Check for updates",
        ["verificando"] = "checking...",
        ["atualizado"] = "you're on the latest version",
        ["nova_versao"] = "version {0} available",
        ["baixar"] = "Download",
        ["falha_verificar"] = "couldn't check right now"
    };

    static readonly Dictionary<string, string> Es = new()
    {
        ["geral"] = "General",
        ["sistema"] = "Sistema",
        ["sobre"] = "Acerca de",
        ["esconder_apos"] = "Ocultar el cursor tras",
        ["segundos_sem_uso"] = "segundos sin ratón ni teclado",
        ["atalho"] = "Atajo para pausar y reanudar",
        ["atalho_dica"] = "haz clic y pulsa la combinación",
        ["atalho_ocupado"] = "combinación ya usada por otra app",
        ["iniciar_windows"] = "Iniciar con Windows",
        ["iniciar_windows_obs"] = "Se guarda solo en tu usuario, sin administrador.",
        ["idioma"] = "Idioma",
        ["idioma_obs"] = "Auto sigue el idioma de Windows.",
        ["versao"] = "versión",
        ["descricao"] = "Oculta el cursor tras un tiempo sin uso y lo devuelve al primer movimiento.",
        ["limitacao"] = "Limitación conocida: los juegos en pantalla completa exclusiva dibujan su propio cursor e ignoran el cambio de cursor del sistema. Es comportamiento de Windows, no un fallo de la app.",
        ["menu_config"] = "Ajustes",
        ["menu_pausar"] = "Pausar",
        ["menu_sair"] = "Salir",
        ["pausado"] = "En pausa",
        ["ativo"] = "Activo",
        ["segundos_curto"] = "s",
        ["minutos_curto"] = "min",
        ["contador"] = "Ocultando en",
        ["contador_pausado"] = "en pausa",
        ["contador_escondido"] = "cursor oculto",
        ["buscar_atualizacao"] = "Buscar actualizaciones",
        ["verificando"] = "comprobando...",
        ["atualizado"] = "ya tienes la última versión",
        ["nova_versao"] = "versión {0} disponible",
        ["baixar"] = "Descargar",
        ["falha_verificar"] = "no se pudo comprobar ahora"
    };
}
