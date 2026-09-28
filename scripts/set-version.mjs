import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

export function normalizeVersion(input) {
  const version = input?.trim().replace(/^v/, '');
  if (!version || !/^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.test(version)) {
    throw new Error('Version must be a stable semantic version such as 0.2.0');
  }
  return version;
}

function replaceOnce(source, pattern, version, path) {
  let count = 0;
  const updated = source.replace(pattern, (_, before, after) => {
    count += 1;
    return `${before}${version}${after}`;
  });
  if (count !== 1) {
    throw new Error(`Expected exactly one version field in ${path}; found ${count}`);
  }
  return updated;
}

export function setVersion(root, input) {
  const version = normalizeVersion(input);
  const files = [
    {
      path: 'package.json',
      pattern: /("version"\s*:\s*")[^"]+("\s*)/g,
      validate: (source) => JSON.parse(source).version === version,
    },
    {
      path: 'src-tauri/tauri.conf.json',
      pattern: /("version"\s*:\s*")[^"]+("\s*)/g,
      validate: (source) => JSON.parse(source).version === version,
    },
    {
      path: 'src-tauri/Cargo.toml',
      pattern: /(^version\s*=\s*")[^"]+("\s*$)/gm,
    },
    {
      path: 'src-tauri/Cargo.lock',
      pattern: /(\[\[package\]\]\r?\nname = "sunshine-library-sync"\r?\nversion = ")[^"]+(")/g,
    },
  ];

  // Validate every file before writing any of them.
  const updates = files.map(({ path, pattern, validate }) => {
    const absolutePath = join(root, path);
    const updated = replaceOnce(readFileSync(absolutePath, 'utf8'), pattern, version, path);
    if (validate && !validate(updated)) {
      throw new Error(`Unable to verify updated version in ${path}`);
    }
    return { absolutePath, updated };
  });
  for (const { absolutePath, updated } of updates) {
    writeFileSync(absolutePath, updated);
  }
  return version;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
  console.log(setVersion(root, process.env.RELEASE_VERSION ?? process.argv[2]));
}
