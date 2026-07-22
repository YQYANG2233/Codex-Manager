import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import test from "node:test";

const appsRoot = path.resolve(import.meta.dirname, "..");
const checkerSource = await fs.readFile(
  path.join(
    appsRoot,
    "src",
    "components",
    "layout",
    "automatic-update-checker.tsx",
  ),
  "utf8",
);
const bootstrapSource = await fs.readFile(
  path.join(appsRoot, "src", "components", "layout", "app-bootstrap.tsx"),
  "utf8",
);
const settingsCardSource = await fs.readFile(
  path.join(
    appsRoot,
    "src",
    "app",
    "settings",
    "components",
    "general-basics-card.tsx",
  ),
  "utf8",
);
const settingsPageSource = await fs.readFile(
  path.join(appsRoot, "src", "app", "settings", "page.tsx"),
  "utf8",
);
const globalsSource = await fs.readFile(
  path.join(appsRoot, "src", "app", "globals.css"),
  "utf8",
);

test("automatic updater checks immediately and then every seven hours", () => {
  assert.match(
    checkerSource,
    /AUTO_UPDATE_CHECK_INTERVAL_MS = 7 \* 60 \* 60 \* 1_000/,
  );
  assert.match(
    checkerSource,
    /useEffect\(\(\) => \{[\s\S]*void runCheck\(\);[\s\S]*window\.setInterval\([\s\S]*AUTO_UPDATE_CHECK_INTERVAL_MS/,
  );
});

test("an available update restores and focuses the main window before opening the dialog", () => {
  assert.match(
    checkerSource,
    /if \(!summary\.hasUpdate\) \{[\s\S]*return;[\s\S]*await appClient\.showMainWindow\(\)\.catch\(\(\) => undefined\);[\s\S]*setDialogOpen\(true\)/,
  );
});

test("automatic updater starts only after desktop settings are ready and enabled", () => {
  assert.match(
    bootstrapSource,
    /!isInitializing[\s\S]*isDesktopRuntime[\s\S]*appSettings\.updateAutoCheck[\s\S]*<AutomaticUpdateChecker/,
  );
});

test("automatic updater has no development-only forced dialog path", () => {
  assert.doesNotMatch(checkerSource, /IS_UPDATE_DIALOG_DEMO|9\.9\.9-test/);
  assert.doesNotMatch(bootstrapSource, /NODE_ENV === "development"/);
  assert.match(checkerSource, /const summary = await checkForUpdate\(\)/);
  assert.match(checkerSource, /const summary = await appClient\.prepareUpdate\(\)/);
});

test("basic settings exposes the persisted automatic update toggle", () => {
  assert.match(settingsCardSource, /checked=\{snapshot\.updateAutoCheck\}/);
  assert.match(
    settingsCardSource,
    /updateSettings\.mutate\(\{ updateAutoCheck: value \}\)/,
  );
  assert.match(settingsCardSource, /每 7 小时检查一次/);
});

test("update dialogs use a compact hover treatment for the later button", () => {
  assert.match(
    checkerSource,
    /variant="outline"[\s\S]*className="update-dialog-later-button"[\s\S]*t\("稍后"\)/,
  );
  assert.match(
    settingsPageSource,
    /variant="outline"[\s\S]*className="update-dialog-later-button"[\s\S]*t\("稍后"\)/,
  );
  assert.match(
    globalsSource,
    /update-dialog-later-button:hover[\s\S]*box-shadow: none/,
  );
});
