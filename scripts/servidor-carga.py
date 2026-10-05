"""Página local controlada para medir recursos de rede e atividade contínua.

Uso: python scripts/servidor-carga.py --porta 8766
Estatísticas: GET /__estatisticas; limpar contadores: POST /__zerar.
Escuta somente em 127.0.0.1 e não acessa serviços externos.
"""

import argparse
import json
import random
import struct
import threading
import zlib
from collections import defaultdict
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlparse


LARGURA, ALTURA = 800, 500


def bloco_png(tipo: bytes, dados: bytes) -> bytes:
    return (
        struct.pack(">I", len(dados))
        + tipo
        + dados
        + struct.pack(">I", zlib.crc32(tipo + dados) & 0xFFFFFFFF)
    )


def imagem_png() -> bytes:
    gerador = random.Random(28)
    pixels = b"".join(b"\0" + gerador.randbytes(LARGURA * 3) for _ in range(ALTURA))
    return (
        b"\x89PNG\r\n\x1a\n"
        + bloco_png(b"IHDR", struct.pack(">IIBBBBB", LARGURA, ALTURA, 8, 2, 0, 0, 0))
        + bloco_png(b"IDAT", zlib.compress(pixels, level=3))
        + bloco_png(b"IEND", b"")
    )


IMAGEM = imagem_png()
PAGINA = """<!doctype html><html lang="pt-BR"><meta charset="utf-8">
<link rel="icon" href="data:,">
<title>Canoa: carga controlada</title><link rel="stylesheet" href="/estilo.css">
<h1>Canoa: carga controlada</h1>
<p>Esta página local separa texto, estilo, imagens e atividade JavaScript.</p>
<p>Requisições periódicas: <output id="pulsos">0</output></p>
<button id="acao">Testar interação</button><output id="resultado"></output>
<section><h2>Imagem inicial</h2><img src="/imagem.png?i=1" alt="Imagem de teste 1" width="800" height="500"></section>
<section class="abaixo"><h2>Imagens abaixo da tela</h2>
<img loading="lazy" src="/imagem.png?i=2" alt="Imagem de teste 2" width="800" height="500">
<img loading="lazy" src="/imagem.png?i=3" alt="Imagem de teste 3" width="800" height="500"></section>
<script src="/script.js"></script></html>""".encode("utf-8")
ESTILO = b"""body{font:16px system-ui;max-width:840px;margin:30px auto;padding:0 18px;color:#173a3b}
img{display:block;max-width:100%;height:auto;margin:12px 0}button{padding:8px}
.abaixo{margin-top:600px}#resultado{margin-left:10px}
h1{animation:pulsar 3s ease-in-out infinite alternate}
@keyframes pulsar{to{opacity:.6}}"""
SCRIPT = b"""let pulsos=0;
setInterval(()=>{fetch('/pulso').then(()=>{
  document.querySelector('#pulsos').textContent=String(++pulsos);
}).catch(()=>{});},1000);
document.querySelector('#acao').addEventListener('click',()=>{
  document.querySelector('#resultado').textContent='Funciona';
});"""


class Servidor(ThreadingHTTPServer):
    def __init__(self, endereco):
        super().__init__(endereco, Handler)
        self.contadores = defaultdict(lambda: {"requisicoes": 0, "bytes": 0})
        self.trava = threading.Lock()


class Handler(BaseHTTPRequestHandler):
    def log_message(self, formato, *args):
        # Cada pulso geraria uma linha no terminal e alteraria o próprio teste.
        pass

    def responder(self, corpo: bytes, tipo: str, contabilizar: bool = True):
        self.send_response(200)
        self.send_header("Content-Type", tipo)
        self.send_header("Content-Length", str(len(corpo)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        try:
            self.wfile.write(corpo)
        except (BrokenPipeError, ConnectionResetError):
            return
        if contabilizar:
            rota = urlparse(self.path).path
            with self.server.trava:
                self.server.contadores[rota]["requisicoes"] += 1
                self.server.contadores[rota]["bytes"] += len(corpo)

    def do_GET(self):
        rota = urlparse(self.path).path
        if rota == "/__estatisticas":
            with self.server.trava:
                dados = dict(self.server.contadores)
            self.responder(json.dumps(dados, sort_keys=True).encode(), "application/json", False)
            return
        recursos = {
            "/": (PAGINA, "text/html; charset=utf-8"),
            "/estilo.css": (ESTILO, "text/css; charset=utf-8"),
            "/script.js": (SCRIPT, "text/javascript; charset=utf-8"),
            "/imagem.png": (IMAGEM, "image/png"),
            "/pulso": (b"ok", "text/plain; charset=utf-8"),
        }
        if rota not in recursos:
            self.send_error(404)
            return
        self.responder(*recursos[rota])

    def do_POST(self):
        if urlparse(self.path).path != "/__zerar":
            self.send_error(404)
            return
        with self.server.trava:
            self.server.contadores.clear()
        self.responder(b"ok", "text/plain; charset=utf-8", False)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--porta", type=int, default=8766)
    porta = parser.parse_args().porta
    with Servidor(("127.0.0.1", porta)) as servidor:
        print(f"CANOA_CARGA_LOCAL http://127.0.0.1:{servidor.server_port}/", flush=True)
        servidor.serve_forever()
