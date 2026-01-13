const puppeteer = require('puppeteer');
const path = require('path');
const fs = require('fs');

const BASE_URL = 'http://localhost:5181';
const SCREENSHOTS_DIR = path.join(__dirname, '..', 'assets', 'screenshots');

const pages = [
  { name: 'dashboard', path: '/' },
  { name: 'dns', path: '/dns' },
  { name: 'security', path: '/security' },
  { name: 'ssl', path: '/ssl' },
  { name: 'cache', path: '/cache' },
  { name: 'performance', path: '/performance' },
  { name: 'workers', path: '/workers' },
  { name: 'r2', path: '/r2' },
  { name: 'd1', path: '/d1' },
  { name: 'stream', path: '/stream' },
  { name: 'analytics', path: '/analytics' },
  { name: 'rules', path: '/rules' },
  { name: 'settings', path: '/settings' },
];

async function takeScreenshots() {
  // Ensure screenshots directory exists
  if (!fs.existsSync(SCREENSHOTS_DIR)) {
    fs.mkdirSync(SCREENSHOTS_DIR, { recursive: true });
  }

  console.log('Launching browser...');
  const browser = await puppeteer.launch({
    headless: 'new',
    args: ['--no-sandbox', '--disable-setuid-sandbox'],
  });

  const page = await browser.newPage();
  await page.setViewport({ width: 1920, height: 1080 });

  for (const pageInfo of pages) {
    const url = `${BASE_URL}${pageInfo.path}`;
    const screenshotPath = path.join(SCREENSHOTS_DIR, `${pageInfo.name}.png`);

    console.log(`Capturing ${pageInfo.name}...`);

    try {
      await page.goto(url, { waitUntil: 'networkidle0', timeout: 30000 });
      await new Promise(r => setTimeout(r, 1500)); // Wait for animations
      await page.screenshot({ path: screenshotPath, fullPage: false });
      console.log(`  Saved: ${screenshotPath}`);
    } catch (error) {
      console.error(`  Error capturing ${pageInfo.name}: ${error.message}`);
    }
  }

  await browser.close();
  console.log('Done!');
}

takeScreenshots().catch(console.error);
