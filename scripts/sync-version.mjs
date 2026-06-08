import { readFileSync, writeFileSync } from "node:fs";

const rawVersion = process.argv[2] ?? process.env.GITHUB_REF_NAME ?? "";
const version = rawVersion.replace(/^refs\/tags\//, "").replace(/^v/, "");

if (!/^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/.test(version)) {
  console.error(
    `Invalid version "${rawVersion}". Expected a semver tag like v1.2.3.`,
  );
  process.exit(1);
}

const writeJsonVersion = (path) => {
  const data = JSON.parse(readFileSync(path, "utf8"));
  if (data.version === version) {
    return;
  }

  data.version = version;
  writeFileSync(path, `${JSON.stringify(data, null, 2)}\n`);
};

writeJsonVersion("package.json");
writeJsonVersion("src-tauri/tauri.conf.json");

const cargoPath = "src-tauri/Cargo.toml";
const cargoToml = readFileSync(cargoPath, "utf8");
let cargoVersionFound = false;
const nextCargoToml = cargoToml.replace(
  /^version\s*=\s*"[^"]+"/m,
  () => {
    cargoVersionFound = true;
    return `version = "${version}"`;
  },
);

if (!cargoVersionFound) {
  console.error(`Could not find package version in ${cargoPath}.`);
  process.exit(1);
}

if (nextCargoToml !== cargoToml) {
  writeFileSync(cargoPath, nextCargoToml);
}

console.log(`Synced project version to ${version}.`);
