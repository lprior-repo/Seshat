import fs from 'fs';
import path from 'path';

async function main() {
  try {
    const targetDir = '/tmp/extracted-trace';
    
    const traceFiles = fs.readdirSync(targetDir).filter(f => f.endsWith('.trace'));
    if (traceFiles.length > 0) {
      const traceContent = fs.readFileSync(path.join(targetDir, traceFiles[0]), 'utf-8');
      const lines = traceContent.split('\n').filter(Boolean);
      
      const allObjects = lines.map(l => {
          try {
              return JSON.parse(l);
          } catch(e) { return null; }
      }).filter(Boolean);
      
      console.log("All object types:", new Set(allObjects.map(o => o.type)));
      
      const calls = allObjects.filter(o => o.type === 'call' || o.type === 'action' || o.method);
      console.log("Found " + calls.length + " calls. Details:");
      calls.forEach(a => {
        const method = a.method || (a.metadata && a.metadata.apiName) || "unknown";
        console.log(method, a.error ? "ERROR: " + a.error.message : (a.metadata?.error ? "ERROR: " + a.metadata.error.message : "OK"));
      });
    }
  } catch (err) {
    console.error(err);
  }
}
main();
