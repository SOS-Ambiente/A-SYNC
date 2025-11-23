import { execSync } from 'child_process';
import { copyFileSync, mkdirSync, existsSync, readdirSync, statSync } from 'fs';
import { join, dirname } from 'path';

function exec(cmd, cwd = '.') {
  console.log(`\n🔨 Running: ${cmd}`);
  try {
    execSync(cmd, { stdio: 'inherit', cwd });
    return true;
  } catch (error) {
    console.error(`❌ Failed: ${cmd}`);
    return false;
  }
}

function copyDir(src, dest) {
  if (!existsSync(dest)) {
    mkdirSync(dest, { recursive: true });
  }
  
  const entries = readdirSync(src, { withFileTypes: true });
  
  for (const entry of entries) {
    const srcPath = join(src, entry.name);
    const destPath = join(dest, entry.name);
    
    if (entry.isDirectory()) {
      copyDir(srcPath, destPath);
    } else {
      copyFileSync(srcPath, destPath);
    }
  }
}

console.log('\n🌐 Building Web Version...\n');

// Build client Vue app
console.log('📦 Building Vue app...');
exec('pnpm build', 'msscs_client');

// Copy dist to msscs_web
console.log('📋 Copying built files to web server...');
const distPath = 'msscs_client/dist';
const webPath = 'msscs_web/public';

if (existsSync(distPath)) {
  copyDir(distPath, webPath);
  console.log('✅ Files copied successfully!');
} else {
  console.error('❌ Build directory not found!');
  process.exit(1);
}

console.log('\n✅ Web version ready! Run: pnpm serve:web');
