import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { resolve } from "node:path";

const pastaApp = resolve(fileURLToPath(new URL("..", import.meta.url)));
const [html, codigo] = await Promise.all([
  readFile(resolve(pastaApp, "index.html"), "utf8"),
  readFile(resolve(pastaApp, "src/main.ts"), "utf8"),
]);

const seletores = new Set(
  [...codigo.matchAll(/document\.querySelector(?:<[^>]+>)?\(\s*["']#([\w-]+)/g)]
    .map((correspondencia) => correspondencia[1]),
);
const ausentes = [...seletores].filter(
  (id) => !new RegExp(`\\bid=["']${id.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}['"]`).test(html),
);

if (ausentes.length) {
  console.error(`Seletores sem elemento correspondente em index.html: ${ausentes.join(", ")}`);
  process.exitCode = 1;
} else {
  console.log(`Integridade da interface conferida: ${seletores.size} IDs encontrados.`);
}
