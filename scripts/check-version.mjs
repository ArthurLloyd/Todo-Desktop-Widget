import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(fileURLToPath(new URL("..", import.meta.url)));
const readJson = (path) => JSON.parse(readFileSync(resolve(root, path), "utf8"));

const packageJson = readJson("package.json");
const packageLock = readJson("package-lock.json");
const tauriConfig = readJson("src-tauri/tauri.conf.json");
const cargoToml = readFileSync(resolve(root, "src-tauri/Cargo.toml"), "utf8");
const cargoVersion = cargoToml.match(
  /^\[package\][\s\S]*?^version\s*=\s*"([^"]+)"/m,
)?.[1];
const errors = [];

if (!/^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/.test(packageJson.version)) {
  errors.push(`package.json version is not valid SemVer: ${packageJson.version}`);
}
if (cargoVersion !== packageJson.version) {
  errors.push(`Cargo.toml version (${cargoVersion ?? "missing"}) does not match package.json (${packageJson.version})`);
}
if (packageLock.version !== packageJson.version || packageLock.packages?.[""]?.version !== packageJson.version) {
  errors.push("package-lock.json root version does not match package.json");
}
if (tauriConfig.version !== "../package.json") {
  errors.push('tauri.conf.json must use package.json as its version source ("../package.json")');
}
if (tauriConfig.productName !== "Todo Desktop Widget") {
  errors.push(`unexpected productName: ${tauriConfig.productName}`);
}

if (errors.length > 0) {
  for (const error of errors) console.error(`[version-check] ${error}`);
  process.exit(1);
}

console.log(`[version-check] version ${packageJson.version} is consistent`);
