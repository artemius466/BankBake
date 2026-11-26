import fs from "fs";
import path from "path";
import { execSync } from "child_process";

const rootDir = process.cwd();
const pkgJson = JSON.parse(
  fs.readFileSync(path.join(rootDir, "package.json"), "utf8")
);
const version = pkgJson.version || "1.0.0";
const releaseOutDir = path.join(rootDir, "release", `BankBake-v${version}`);
const zipOutPath = path.join(
  rootDir,
  "release",
  `BankBake-Portable-v${version}-win-x64.zip`
);

console.log(`=== Building BankBake v${version} Portable Release ===`);
execSync("pnpm tauri build --no-bundle", { stdio: "inherit" });

// Locate built bankbake.exe
const candidateExes = [
  path.join(rootDir, "src-tauri", "target", "release", "bankbake.exe"),
  process.env.CARGO_TARGET_DIR
    ? path.join(process.env.CARGO_TARGET_DIR, "release", "bankbake.exe")
    : "",
].filter(Boolean);

const builtExe = candidateExes.find((p) => fs.existsSync(p));
if (!builtExe) {
  console.error("Could not find compiled bankbake.exe in target/release.");
  process.exit(1);
}

console.log(`Found compiled executable: ${builtExe}`);

// Clean and assemble portable folder
fs.rmSync(releaseOutDir, { recursive: true, force: true });
fs.mkdirSync(releaseOutDir, { recursive: true });

// 1. Copy BankBake.exe
fs.copyFileSync(builtExe, path.join(releaseOutDir, "BankBake.exe"));

// 2. Copy stripped fmodstudio/ folder
const fmodSrc = path.join(rootDir, "fmodstudio");
if (fs.existsSync(fmodSrc)) {
  console.log("Copying bundled fmodstudio/ runtime...");
  fs.cpSync(fmodSrc, path.join(releaseOutDir, "fmodstudio"), { recursive: true });
}

// 3. Copy samples/ folder
const samplesSrc = path.join(rootDir, "samples");
if (fs.existsSync(samplesSrc)) {
  console.log("Copying samples/...");
  fs.cpSync(samplesSrc, path.join(releaseOutDir, "samples"), { recursive: true });
}

// 4. Create zip archive on Windows using PowerShell Compress-Archive
if (process.platform === "win32") {
  console.log(`Creating portable zip archive: ${zipOutPath}`);
  fs.rmSync(zipOutPath, { force: true });
  execSync(
    `powershell -NoProfile -Command "Compress-Archive -Path '${releaseOutDir}\\*' -DestinationPath '${zipOutPath}' -Force"`,
    { stdio: "inherit" }
  );
}

console.log("\n=== Portable Build Complete! ===");
console.log(`Portable folder: ${releaseOutDir}`);
console.log(`Portable zip:    ${zipOutPath}`);
