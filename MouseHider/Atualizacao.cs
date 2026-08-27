using System.Net.Http;
using System.Security.Cryptography;
using System.Text.Json;

namespace MouseHider;

/// <summary>
/// Consulta a API publica de releases do GitHub e, quando ha versao nova, baixa e roda o
/// instalador. ponytail: sem servico de update, sem verificacao em segundo plano, sem
/// delta — o usuario clica quando quiser e o instalador que ja existe faz o trabalho.
/// </summary>
static class Atualizacao
{
    const string Api = "https://api.github.com/repos/asguth/mouse-hider/releases/latest";
    const string Asset = "MouseHiderSetup.exe";
    public const string Pagina = "https://github.com/asguth/mouse-hider/releases/latest";

    public enum Resultado { Atualizado, TemNova, SemConexao, Falhou }

    /// <summary>Versao nova, com o instalador e o hash publicados no proprio release.</summary>
    public sealed record Novidade(string Versao, string? Url, string? Sha256);

    static HttpClient Cliente(string versaoAtual)
    {
        var http = new HttpClient { Timeout = TimeSpan.FromSeconds(10) };
        // A API do GitHub recusa requisicao sem User-Agent.
        http.DefaultRequestHeaders.UserAgent.ParseAdd("MouseHider/" + versaoAtual);
        return http;
    }

    public static async Task<(Resultado Estado, Novidade? Nova)> Verificar(string versaoAtual)
    {
        try
        {
            using var http = Cliente(versaoAtual);
            using var doc = JsonDocument.Parse(await http.GetStringAsync(Api));
            var raiz = doc.RootElement;

            var tag = raiz.GetProperty("tag_name").GetString() ?? "";
            var limpa = tag.TrimStart('v', 'V');

            if (!Version.TryParse(limpa, out var remota) || !Version.TryParse(versaoAtual, out var local))
            {
                Config.Log("versao ilegivel na consulta de atualizacao: tag=" + tag + " local=" + versaoAtual);
                return (Resultado.Falhou, null);
            }

            if (remota <= local) return (Resultado.Atualizado, null);

            // Url nula = release sem o asset esperado. Nao e motivo para dizer que falhou:
            // ha versao nova, so nao da para instalar sozinho, entao a tela cai na pagina.
            var (url, sha) = Instalador(raiz);
            if (url is null) Config.Log("release " + tag + " sem o asset " + Asset);
            return (Resultado.TemNova, new Novidade(limpa, url, sha));
        }
        // Sem rede e uma coisa (aviso vermelho "Falha na conexao"); resposta estranha do
        // GitHub e outra. O timeout do HttpClient chega como TaskCanceledException.
        catch (Exception e) when (e is HttpRequestException or TaskCanceledException)
        {
            Config.Log("sem conexao ao verificar atualizacao: " + e.Message);
            return (Resultado.SemConexao, null);
        }
        catch (Exception e)
        {
            Config.Log("falha ao verificar atualizacao: " + e.Message);
            return (Resultado.Falhou, null);
        }
    }

    /// <summary>O asset tem nome fixo — e ele que sustenta o link permanente do README.</summary>
    static (string? Url, string? Sha256) Instalador(JsonElement release)
    {
        if (!release.TryGetProperty("assets", out var assets)) return (null, null);

        foreach (var asset in assets.EnumerateArray())
        {
            if (!asset.TryGetProperty("name", out var nome) || nome.GetString() != Asset) continue;
            var url = asset.TryGetProperty("browser_download_url", out var u) ? u.GetString() : null;
            if (url is null) continue;

            // "sha256:<hex>". Campo recente da API: quando nao vier, baixa sem conferir.
            var digest = asset.TryGetProperty("digest", out var d) ? d.GetString() : null;
            const string prefixo = "sha256:";
            var sha = digest?.StartsWith(prefixo, StringComparison.OrdinalIgnoreCase) == true
                ? digest[prefixo.Length..]
                : null;
            return (url, sha);
        }

        return (null, null);
    }

    /// <summary>
    /// Baixa o instalador para a pasta temporaria e confere o sha256 do release. Devolve o
    /// caminho, ou null se algo falhar. ponytail: o arquivo fica no temp — o processo sai
    /// logo em seguida para o instalador poder substituir os arquivos, entao nao sobra
    /// ninguem para apagar; o Windows limpa o temp sozinho.
    /// </summary>
    public static async Task<string?> Baixar(Novidade nova, string versaoAtual, IProgress<int> progresso)
    {
        if (nova.Url is null) return null;
        var destino = Path.Combine(Path.GetTempPath(), "MouseHiderSetup-" + nova.Versao + ".exe");

        try
        {
            using var http = Cliente(versaoAtual);
            http.Timeout = TimeSpan.FromMinutes(10); // o instalador passa de 45 MB

            using var resposta = await http.GetAsync(nova.Url, HttpCompletionOption.ResponseHeadersRead);
            resposta.EnsureSuccessStatusCode();
            var total = resposta.Content.Headers.ContentLength ?? 0;

            await using (var origem = await resposta.Content.ReadAsStreamAsync())
            await using (var arquivo = File.Create(destino))
            {
                var buffer = new byte[81920];
                long lidos = 0;
                var ultimo = -1;
                int n;
                while ((n = await origem.ReadAsync(buffer)) > 0)
                {
                    await arquivo.WriteAsync(buffer.AsMemory(0, n));
                    lidos += n;
                    if (total <= 0) continue;
                    // So avisa quando a porcentagem muda: sao ~600 blocos por download.
                    var pct = (int)(lidos * 100 / total);
                    if (pct == ultimo) continue;
                    ultimo = pct;
                    progresso.Report(pct);
                }
            }

            // O hash vem do mesmo release que deu a URL, entao isto nao protege contra um
            // GitHub comprometido — protege contra download truncado, proxy que devolve
            // pagina de erro com 200, e cache envenenado no caminho.
            if (nova.Sha256 is { Length: > 0 } esperado && !Confere(destino, esperado))
            {
                Config.Log("sha256 do instalador nao bate com o release — download descartado");
                Apagar(destino);
                return null;
            }

            Config.Log("instalador da " + nova.Versao + " baixado em " + destino);
            return destino;
        }
        catch (Exception e)
        {
            Config.Log("falha ao baixar atualizacao: " + e.Message);
            Apagar(destino);
            return null;
        }
    }

    static bool Confere(string caminho, string esperado)
    {
        using var fs = File.OpenRead(caminho);
        return string.Equals(Convert.ToHexString(SHA256.HashData(fs)), esperado, StringComparison.OrdinalIgnoreCase);
    }

    static void Apagar(string caminho)
    {
        try { if (File.Exists(caminho)) File.Delete(caminho); }
        catch (Exception e) { Config.Log("falha ao apagar " + caminho + ": " + e.Message); }
    }

    /// <summary>
    /// Roda o instalador em silencio. O setup fecha este processo pelo Restart Manager e
    /// reabre o app no fim (o [Run] do .iss nao tem mais skipifsilent) — sem isso um app
    /// de bandeja simplesmente sumiria depois de atualizar.
    /// </summary>
    public static bool Instalar(string caminho)
    {
        try
        {
            using var _ = System.Diagnostics.Process.Start(
                new System.Diagnostics.ProcessStartInfo(caminho) { UseShellExecute = true, Arguments = "/SILENT" });
            return true;
        }
        catch (Exception e)
        {
            Config.Log("falha ao rodar o instalador: " + e.Message);
            return false;
        }
    }
}
