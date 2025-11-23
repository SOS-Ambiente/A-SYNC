import { execSync } from 'child_process';
import { platform } from 'os';
import readline from 'readline';

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout
});

function question(query) {
  return new Promise(resolve => rl.question(query, resolve));
}

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

async function main() {
  console.log('\n🚀 MSSCS Build System\n');
  console.log('═══════════════════════════════════════\n');

  const currentPlatform = platform() === 'win32' ? 'windows' : 'linux';
  
  // 1. Build desktop client for current platform
  console.log(`\n📦 Building Desktop Client for ${currentPlatform}...\n`);
  
  if (currentPlatform === 'windows') {
    exec('pnpm build', 'msscs_client');
    exec('pnpm tauri build --target x86_64-pc-windows-msvc', 'msscs_client');
  } else {
    exec('pnpm build', 'msscs_client');
    exec('pnpm tauri build --target x86_64-unknown-linux-gnu', 'msscs_client');
  }

  // 2. Ask about Android build
  const buildAndroid = await question('\n📱 Build Android APK? (y/n): ');
  
  if (buildAndroid.toLowerCase() === 'y') {
    console.log('\n📱 Building Android APK...\n');
    exec('pnpm build', 'msscs_mobile');
    exec('pnpm android:build:apk', 'msscs_mobile');
  }

  // 3. Build web version
  console.log('\n🌐 Building Web Version...\n');
  exec('node scripts/build-web.js');

  // 4. Ask about serving web
  const serveWeb = await question('\n🌐 Start web server? (y/n): ');
  
  if (serveWeb.toLowerCase() === 'y') {
    console.log('\n🌐 Starting web server...\n');
    console.log('Press Ctrl+C to stop the server\n');
    exec('node server.js', 'msscs_web');
  }

  console.log('\n✅ All builds complete!\n');
  console.log('═══════════════════════════════════════\n');
  console.log('📦 Desktop builds: msscs_client/src-tauri/target/release/');
  console.log('📱 Android APK: msscs_mobile/src-tauri/gen/android/app/build/outputs/apk/');
  console.log('🌐 Web files: msscs_web/public/');
  console.log('\nTo serve web: pnpm serve:web\n');
  
  rl.close();
}

main().catch(console.error);
