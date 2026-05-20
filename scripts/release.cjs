// Release script — verifies versions, checks CHANGELOG, creates git tag
// Usage: node scripts/release.js [--dry-run]

const fs = require('fs');
const { execSync } = require('child_process');

const ROOT = __dirname + '/..';
const DRY = process.argv.includes('--dry-run');
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
