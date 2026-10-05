"""Página local para testar navegação, download e APIs de permissão do Canoa."""

from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from time import sleep
from urllib.parse import urlparse


PAGINA = """<!doctype html>
<html lang="pt-BR"><meta charset="utf-8"><title>Teste local do Canoa</title>
<style>body{font:16px system-ui;max-width:720px;margin:40px auto;padding:0 20px}
button,a{display:inline-block;margin:8px;padding:8px} output{font-weight:bold}</style>
<h1>Teste local do Canoa</h1>
<p>Contador: <output id="contador">0</output></p>
<p>Rede: <output id="rede">Aguardando</output></p>
<p>Permissão de localização: <output id="permissao">Não solicitada</output></p>
<p>Endereço da página: <output id="endereco-atual"></output></p>
<a id="download" href="/arquivo.txt">Baixar arquivo de teste</a>
<a id="download-lento" href="/arquivo-lento.txt">Baixar arquivo lento</a>
<button id="fragmento">Mudar fragmento</button>
<button id="rota">Mudar rota sem recarregar</button>
<button id="localizacao">Solicitar localização</button>
<script>
let numero = 0;
setInterval(() => { document.querySelector('#contador').textContent = String(++numero); }, 500);
function mostrarEndereco() { document.querySelector('#endereco-atual').textContent = location.href; }
mostrarEndereco();
addEventListener('hashchange', mostrarEndereco);
addEventListener('popstate', mostrarEndereco);
document.querySelector('#fragmento').onclick = () => { location.hash = 'teste-' + Date.now(); };
document.querySelector('#rota').onclick = () => {
  history.pushState({}, '', '/?rota=teste-' + Date.now());
  mostrarEndereco();
};
fetch('/api').then(r => r.text()).then(texto => {
  document.querySelector('#rede').textContent = texto;
}).catch(erro => { document.querySelector('#rede').textContent = String(erro); });
document.querySelector('#localizacao').onclick = () => {
  const saida = document.querySelector('#permissao');
  saida.textContent = 'Solicitada';
  navigator.geolocation.getCurrentPosition(
    () => { saida.textContent = 'Permitida'; },
    erro => { saida.textContent = 'Negada ou indisponível (' + erro.code + ')'; },
    {timeout: 5000}
  );
};
</script></html>""".encode("utf-8")


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        rota = urlparse(self.path).path
        if rota == "/arquivo-lento.txt":
            bloco = b"Canoa teste lento.\n" + b"." * (65536 - 19)
            self.send_response(200)
            self.send_header("Content-Type", "text/plain; charset=utf-8")
            self.send_header("Content-Length", str(len(bloco) * 16))
            self.send_header("Content-Disposition", 'attachment; filename="canoa-teste-lento.txt"')
            self.send_header("Cache-Control", "no-store")
            self.end_headers()
            try:
                for _ in range(16):
                    self.wfile.write(bloco)
                    self.wfile.flush()
                    sleep(0.5)
            except (BrokenPipeError, ConnectionResetError):
                print("CANOA_DOWNLOAD_LENTO_INTERROMPIDO", flush=True)
            return
        if rota == "/":
            corpo, tipo = PAGINA, "text/html; charset=utf-8"
        elif rota == "/api":
            corpo, tipo = b"Resposta local recebida", "text/plain; charset=utf-8"
        elif rota == "/arquivo.txt":
            corpo, tipo = b"Canoa: arquivo de teste local.\n", "text/plain; charset=utf-8"
        else:
            self.send_error(404)
            return
        self.send_response(200)
        self.send_header("Content-Type", tipo)
        self.send_header("Content-Length", str(len(corpo)))
        self.send_header("Cache-Control", "no-store")
        if rota == "/arquivo.txt":
            self.send_header(
                "Content-Disposition",
                'attachment; filename="canoa-teste-compatibilidade.txt"',
            )
        self.end_headers()
        self.wfile.write(corpo)


if __name__ == "__main__":
    servidor = ThreadingHTTPServer(("127.0.0.1", 8765), Handler)
    print("CANOA_TESTE_LOCAL http://127.0.0.1:8765/", flush=True)
    servidor.serve_forever()
