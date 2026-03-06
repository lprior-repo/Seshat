import extractZip from './node_modules/extract-zip/index.js';
import fs from 'fs';
import path from 'path';

async function main() {
  try {
    const targetDir = '/tmp/extracted-trace';
    
    const traceFiles = fs.readdirSync(targetDir).filter(f => f.endsWith('.trace'));
    if (traceFiles.length > 0) {
      const traceContent = fs.readFileSync(path.join(targetDir, traceFiles[0]), 'utf-8');
      const lines = traceContent.split('\n').filter(Boolean);
      
      const actions = lines.map(l => JSON.parse(l)).filter(o => o.type === 'action');
      console.log("Found " + actions.length + " actions. Details:");
      actions.forEach(a => {
        if (a.metadata && a.metadata.apiName) {
           console.log(a.metadata.apiName, a.metadata.params, a.metadata.error?.message);
        }
      });
      console.log("Last line error info if any:");
      const last = actions[actions.length-1];
      console.log(last?.metadata?.error);
    }
  } catch (err) {
    console.error(err);
  }
}
main();
