using System.Net.Http;
using System.Text.Json;

namespace MouseHider;

/// <summary>
/// Consulta a API publica de releases do GitHub. ponytail: sem cliente de API, sem cache,
/// sem verificacao automatica em segundo plano — o usuario clica quando quiser.
/// </summary>
static class Atualizacao
{
    const string Api = "https://api.github.com/repos/asguth/mouse-hider/releases/latest";
    public const string Pagina = "https://github.com/asguth/mouse-hider/releases/latest";

    public enum Resultado { Atualizado, TemNova, Falhou }

    public static async Task<(Resultado Estado, string Versao)> Verificar(string versaoAtual)
    {
        try
        {
            using var http = new HttpClient { Timeout = TimeSpan.FromSeconds(10) };
            // A API do GitHub recusa requisicao sem User-Agent.
            http.DefaultRequestHeaders.UserAgent.ParseAdd("MouseHider/" + versaoAtual);

            using var doc = JsonDocument.Parse(await http.GetStringAsync(Api));
            var tag = doc.RootElement.GetProperty("tag_name").GetString() ?? "";
            var limpa = tag.TrimStart('v', 'V');

            if (!Version.TryParse(limpa, out var remota) || !Version.TryParse(versaoAtual, out var local))
            {
                Config.Log("versao ilegivel na consulta de atualizacao: tag=" + tag + " local=" + versaoAtual);
                return (Resultado.Falhou, "");
            }

            return (remota > local ? Resultado.TemNova : Resultado.Atualizado, limpa);
        }
        catch (Exception e)
        {
            Config.Log("falha ao verificar atualizacao: " + e.Message);
            return (Resultado.Falhou, "");
        }
    }
}
