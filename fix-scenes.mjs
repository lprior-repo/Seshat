import fs from 'fs';
import path from 'path';
import crypto from 'crypto';

const scenesDir = path.resolve('diagram_tool', 'e2e', 'scenes');
const fixturesFile = path.resolve('diagram_tool', 'e2e', 'fixtures', 'rq-fixtures.ts');

const files = fs.readdirSync(scenesDir).filter(f => f.endsWith('.json'));

let fixturesContent = fs.readFileSync(fixturesFile, 'utf8');

for (const file of files) {
  const filePath = path.join(scenesDir, file);
  let content = fs.readFileSync(filePath, 'utf8');
  
  // Unlock all nodes
  content = content.replace(/"locked"\s*:\s*true/g, '"locked":false');
  
  fs.writeFileSync(filePath, content, 'utf8');
  
  const sceneName = file.replace('.json', '');
  const checksum = crypto.createHash('sha256').update(content, 'utf8').digest('hex');
  
  console.log(`Updated ${file}, new checksum: ${checksum}`);
  
  const regex = new RegExp(`(${sceneName}: \\{[\\s\\S]*?checksum:\\s*")[^"]+(")`);
  fixturesContent = fixturesContent.replace(regex, `$1${checksum}$2`);
}

fs.writeFileSync(fixturesFile, fixturesContent, 'utf8');
console.log('Updated rq-fixtures.ts');
