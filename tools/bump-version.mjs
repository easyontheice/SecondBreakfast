#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";

function fail(message) {
  console.error(message);
  process.exit(1);
}

const nextVersion = process.argv[2];
if (!nextVersion) {
  fail("Usage: node tools/bump-version.mjs <version>");
}

if (!/^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/.test(nextVersion)) {
  fail(`Invalid semver version: ${nextVersion}`);
}

const root = process.cwd();
const packageJsonPath = path.join(root, "package.json");
const cargoTomlPath = path.join(root, "src-tauri", "Cargo.toml");
const tauriConfPath = path.join(root, "src-tauri", "tauri.conf.json");

const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, "utf8"));
packageJson.version = nextVersion;
fs.writeFileSync(packageJsonPath, `${JSON.stringify(packageJson, null, 2)}\n`);

let cargoToml = fs.readFileSync(cargoTomlPath, "utf8");
const packageVersionPattern = /(\[package\][\s\S]*?^version\s*=\s*")[^"]+("\s*$)/m;
if (!packageVersionPattern.test(cargoToml)) {
  fail("Could not find [package] version in src-tauri/Cargo.toml");
}
cargoToml = cargoToml.replace(packageVersionPattern, `$1${nextVersion}$2`);
fs.writeFileSync(cargoTomlPath, cargoToml);

const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, "utf8"));
tauriConf.version = nextVersion;
fs.writeFileSync(tauriConfPath, `${JSON.stringify(tauriConf, null, 2)}\n`);

console.log(`Bumped version to ${nextVersion}`);
console.log("Updated files:");
console.log("- package.json");
console.log("- src-tauri/Cargo.toml");
console.log("- src-tauri/tauri.conf.json");
