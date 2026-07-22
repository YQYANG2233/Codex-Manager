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

test("development mode uses a safe local update dialog demo", () => {
  assert.match(
    checkerSource,
    /IS_UPDATE_DIALOG_DEMO = process\.env\.NODE_ENV === "development"/,
  );
  assert.match(checkerSource, /latestVersion: "9\.9\.9-test"/);
  assert.match(
    checkerSource,
    /if \(IS_UPDATE_DIALOG_DEMO\) \{[\s\S]*未执行实际更新[\s\S]*return;/,
  );
  assert.match(
    bootstrapSource,
    /appSettings\.updateAutoCheck \|\| process\.env\.NODE_ENV === "development"/,
  );
});

test("automatic updater starts only after desktop settings are ready and enabled", () => {
  assert.match(
    bootstrapSource,
    /!isInitializing[\s\S]*isDesktopRuntime[\s\S]*appSettings\.updateAutoCheck[\s\S]*<AutomaticUpdateChecker/,
  );
});

test("basic settings exposes the persisted automatic update toggle", () => {
  assert.match(settingsCardSource, /checked=\{snapshot\.updateAutoCheck\}/);
  assert.match(
    settingsCardSource,
    /updateSettings\.mutate\(\{ updateAutoCheck: value \}\)/,
  );
  assert.match(settingsCardSource, /每 7 小时检查一次/);
});
