import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import ts from 'typescript';

const source = readFileSync('src/lib/i18n.ts', 'utf8');
const syntax = ts.createSourceFile('i18n.ts', source, ts.ScriptTarget.Latest, true);
let dictionary;
function visit(node) {
  if (ts.isVariableDeclaration(node) && node.name.getText(syntax) === 'zh' && node.initializer && ts.isObjectLiteralExpression(node.initializer)) {
    dictionary = node.initializer;
  }
  ts.forEachChild(node, visit);
}
visit(syntax);
if (!dictionary) throw new Error('Chinese dictionary not found');

const keys = new Map();
const errors = [];
const placeholders = text => [...text.matchAll(/\{(\w+)\}/g)].map(match => match[1]).sort().join(',');
for (const property of dictionary.properties) {
  if (!ts.isPropertyAssignment(property) || !ts.isStringLiteral(property.name) || !ts.isStringLiteral(property.initializer)) continue;
  const key = property.name.text;
  if (keys.has(key)) errors.push(`Duplicate translation: ${key}`);
  keys.set(key, property.initializer.text);
  if (placeholders(key) !== placeholders(property.initializer.text)) errors.push(`Placeholder mismatch: ${key}`);
}

function filesIn(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap(entry =>
    entry.isDirectory() ? filesIn(join(directory, entry.name)) : /\.(?:svelte|ts)$/.test(entry.name) ? [join(directory, entry.name)] : []);
}
for (const file of filesIn('src')) {
  if (file.endsWith('i18n.ts')) continue;
  const text = readFileSync(file, 'utf8');
  for (const match of text.matchAll(/(?:\$t|\btr)\(\s*(['"])(.*?)\1/g)) {
    if (!keys.has(match[2])) errors.push(`Missing translation in ${file}: ${match[2]}`);
  }
}

if (errors.length) {
  for (const error of errors) console.error(error);
  process.exitCode = 1;
} else {
  console.log(`Verified ${keys.size} Chinese translations and their placeholders.`);
}
