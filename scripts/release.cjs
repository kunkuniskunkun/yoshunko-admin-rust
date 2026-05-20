// Release script — verifies versions, checks CHANGELOG, creates git tag
// Usage: node scripts/release.js [--dry-run]

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const ROOT = __dirname + '/..';
const DRY = process.argv.includes('--dry-run');
const CI = process.argv.includes('--ci');
const YELLOW = '\x1b[33m';
const GREEN = '\x1b[32m';
const RED = '\x1b[31m';
const RESET = '\x1b[0m';

function fail(msg) { console.error(`${RED}ERROR:${RESET} ${msg}`); process.exit(1); }
function ok(msg) { console.log(`${GREEN}✓${RESET} ${msg}`); }
function warn(msg) { console.log(`${YELLOW}⚠${RESET} ${msg}`); }

// 1. Read versions
const pkg = JSON.parse(fs.readFileSync(`${ROOT}/package.json`, 'utf8'));
const frontVer = pkg.version; // e.g. "0.713.0"

const tauriConfPath = `${ROOT}/src-tauri/tauri.conf.json`;
const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf8'));
const backVer = tauriConf.version; // e.g. "0.713.0"

console.log(`Frontend version: ${frontVer}`);
console.log(`Backend version:  ${backVer}`);

if (frontVer !== backVer) {
    fail(`Version mismatch: package.json=${frontVer} tauri.conf.json=${backVer}`);
}
ok('Versions match');

// 2. Build display version
const parts = frontVer.split('.');
if (parts.length < 2) fail(`Invalid version format: ${frontVer}`);
const displayVer = `V${parts[0]}.${parts[1].padStart(3, '0')}`;
console.log(`Display version:  ${displayVer}`);

if (!CI) {
    // 3. Check CHANGELOG
    const changelogPath = `${ROOT}/CHANGELOG.md`;
    if (!fs.existsSync(changelogPath)) {
        warn('CHANGELOG.md not found, skipping changelog check');
    } else {
        const changelog = fs.readFileSync(changelogPath, 'utf8');
        const verPatterns = [displayVer, `v${frontVer}`];
        const found = verPatterns.some(p => changelog.includes(p));
        if (found) {
            ok(`CHANGELOG.md contains ${displayVer}`);
        } else {
            warn(`CHANGELOG.md does not contain ${displayVer} — please update before releasing`);
        }
    }

    // 4. Check git status
    const status = execSync('git status --porcelain', { cwd: ROOT, encoding: 'utf8' });
    if (status.trim()) {
        warn('Uncommitted changes detected — commit them before release');
        console.log(status.trim());
    }
    const branch = execSync('git branch --show-current', { cwd: ROOT, encoding: 'utf8' }).trim();
    console.log(`Current branch:   ${branch}`);
}

// 4.5 Generate updater manifest (latest.json)
const nsisDir = `${ROOT}/src-tauri/target/release/bundle/nsis`;
const exeFiles = fs.readdirSync(nsisDir).filter(f => f.endsWith('.exe'));
if (exeFiles.length === 0) fail('No .exe found in NSIS bundle dir');
const exeName = exeFiles[0]; // e.g. "Yoshunko Admin_0.716.0_x64-setup.exe"
const sigPath = `${nsisDir}/${exeName}.sig`;
let signature = '';
if (fs.existsSync(sigPath)) {
  signature = fs.readFileSync(sigPath, 'utf8').trim();
} else {
  warn(`Signature file not found: ${sigPath} — updater verification will fail`);
}

const latestJson = {
  version: `${frontVer}`,
  notes: (() => {
    try {
      const changelog = fs.readFileSync(`${ROOT}/CHANGELOG.md`, 'utf8');
      const match = changelog.match(new RegExp(`## ${displayVer}[\\s\\S]*?(?=## V|$)`));
      return match ? match[0].trim() : `Release ${displayVer}`;
    } catch { return `Release ${displayVer}`; }
  })(),
  pub_date: new Date().toISOString(),
  platforms: {
    "windows-x86_64": {
      signature,
      url: `https://github.com/kunkuniskunkun/yoshunko-admin-rust/releases/download/v${frontVer}/${encodeURIComponent(exeName)}`
    }
  }
};

const latestPath = `${ROOT}/src-tauri/target/release/latest.json`;
fs.mkdirSync(path.dirname(latestPath), { recursive: true });
fs.writeFileSync(latestPath, JSON.stringify(latestJson, null, 2));
ok('latest.json generated');

if (CI) {
    console.log(`${GREEN}✓${RESET} latest.json generated for CI`);
    process.exit(0);
}

// 5. Tag
const tag = `v${frontVer}`;
const existing = (() => {
    try { execSync(`git rev-parse ${tag}`, { cwd: ROOT, encoding: 'utf8', stdio: 'pipe' }); return true; }
    catch { return false; }
})();

if (existing) {
    fail(`Tag ${tag} already exists`);
}

if (DRY) {
    console.log(`\n${YELLOW}[DRY RUN]${RESET} Would create tag: ${tag}`);
} else {
    console.log(`\nCreating tag: ${tag} ...`);
    execSync(`git tag -a ${tag} -m "Release ${displayVer}"`, { cwd: ROOT, stdio: 'inherit' });
    ok(`Tag ${tag} created`);
    console.log(`\nNext: ${YELLOW}git push origin ${tag}${RESET}`);
}
