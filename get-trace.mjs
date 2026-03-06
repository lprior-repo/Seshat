import extractZip from './node_modules/extract-zip/index.js';
import fs from 'fs';
import path from 'path';

async function main() {
  try {
    const tracePath = '/tmp/seshat-playwright/test-results/contracts-DOC-contracts-ba-522ba-node-removes-incident-edges-e2e-smoke/trace.zip';
    const targetDir = '/tmp/extracted-trace';
    
    if (fs.existsSync(targetDir)) {
      fs.rmSync(targetDir, { recursive: true, force: true });
    }
    
    await extractZip(tracePath, { dir: targetDir });
    
    const traceFiles = fs.readdirSync(targetDir).filter(f => f.endsWith('.trace'));
    if (traceFiles.length > 0) {
      const traceContent = fs.readFileSync(path.join(targetDir, traceFiles[0]), 'utf-8');
      const lines = traceContent.split('\n').filter(Boolean);
      
      const actions = lines.map(l => JSON.parse(l)).filter(o => o.type === 'action');
      console.log("Actions taken:");
      actions.forEach(a => {
        if (a.metadata && a.metadata.apiName) {
           console.log(a.metadata.apiName, a.metadata.params);
           if (a.metadata.error) console.log("ERROR:", a.metadata.error.message);
        }
      });
    }
  } catch (err) {
    console.error(err);
  }
}
main();
