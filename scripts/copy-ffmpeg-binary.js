const execa = require("execa");
const fs = require("fs");
const pathToFfmpeg = require("ffmpeg-static");

async function main() {
  const rustTargetInfo = JSON.parse(
    (
      await execa("rustc", ["-Z", "unstable-options", "--print", "target-spec-json"], {
        env: {
          RUSTC_BOOTSTRAP: 1
        }
      })
    ).stdout
  );
  const platformPostfix = rustTargetInfo["llvm-target"];
  fs.mkdirSync("src-tauri/binaries", { recursive: true });
  fs.copyFileSync(pathToFfmpeg, `src-tauri/binaries/ffmpeg-${platformPostfix}`);
}

main().catch(e => {
  throw e;
});
