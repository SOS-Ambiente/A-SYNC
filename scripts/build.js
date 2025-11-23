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
  const args = process.argv.slice(2);
  const targetPlatform = args[0] || (platform() === 'win32' ? 'windows' : 'linux');

  console.log(`\n🚀 Building MSSCS Client for ${targetPlatform}...\n`);

  // Build client
  if (targetPlatform === 'linux') {
    exec('pnpm build', 'msscs_client');
    exec('pnpm tauri build --target x86_64-unknown-linux-gnu', 'msscs_client');
  } else if (targetPlatform === 'windows') {
    exec('pnpm build', 'msscs_client');
    exec('pnpm tauri build --target x86_64-pc-windows-msvc', 'msscs_client');
  }

  console.log('\n✅ Build complete!');
  rl.close();
}

main().catch(console.error);
