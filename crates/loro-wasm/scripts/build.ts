import * as path from "https://deno.land/std@0.105.0/path/mod.ts";
import { gunzip, gzip } from "https://deno.land/x/compress@v0.4.5/mod.ts";
const __dirname = path.dirname(path.fromFileUrl(import.meta.url));

// deno run -A build.ts debug
// deno run -A build.ts release
// deno run -A build.ts release web
// deno run -A build.ts release nodejs
let profile = "dev";
let profileDir = "debug";
if (Deno.args[0] == "release") {
  profile = "release";
  profileDir = "release";
}
const TARGETS = ["bundler", "nodejs", "web"];
const startTime = performance.now();
const LoroWasmDir = path.resolve(__dirname, "..");

console.log(LoroWasmDir);
async function build() {
  await cargoBuild();
  const target = Deno.args[1];
  if (target != null) {
    if (!TARGETS.includes(target)) {
      throw new Error(`Invalid target ${target}`);
    }

    buildTarget(target);
    return;
  }

  await Promise.all(
    TARGETS.map((target) => {
      return buildTarget(target);
    }),
  );

  if (profile !== "dev") {
    await Promise.all(
      TARGETS.map(async (target) => {
        // --snip-rust-panicking-code --snip-rust-fmt-code
        // const snip = `wasm-snip ./${target}/loro_wasm_bg.wasm -o ./${target}/loro_wasm_bg.wasm`;
        // console.log(">", snip);
        // await Deno.run({ cmd: snip.split(" "), cwd: LoroWasmDir }).status();
        // const cmd = `wasm-opt -O4 ./${target}/loro_wasm_bg.wasm -o ./${target}/loro_wasm_bg.wasm`;
        // console.log(">", cmd);
        // await Deno.run({ cmd: cmd.split(" "), cwd: LoroWasmDir }).status();
      }),
    );
  }

  console.log("\n");
  console.log(
    "✅",
    "Build complete in",
    (performance.now() - startTime) / 1000,
    "s",
  );

  if (profile === "release") {
    const wasm = await Deno.readFile(path.resolve(LoroWasmDir, "bundler", "loro_wasm_bg.wasm"));
    console.log("Wasm size: ", (wasm.length / 1024).toFixed(2), "KB");
    const gzipped = await gzip(wasm);
    console.log("Gzipped size: ", (gzipped.length / 1024).toFixed(2), "KB");
  }
}

async function cargoBuild() {
  const cmd = `cargo build --target wasm32-unknown-unknown --profile ${profile}`;
  console.log(cmd);
  const status = await Deno.run({
    cmd: cmd.split(" "),
    cwd: LoroWasmDir,
  }).status();
  if (!status.success) {
    console.log(
      "❌",
      "Build failed in",
      (performance.now() - startTime) / 1000,
      "s",
    );
    Deno.exit(status.code);
  }
}

async function buildTarget(target: string) {
  console.log("🏗️  Building target", `[${target}]`);
  const targetDirPath = path.resolve(LoroWasmDir, target);
  try {
    await Deno.remove(targetDirPath, { recursive: true });
    console.log("Clear directory " + targetDirPath);
  } catch (_e) {
    //
  }

  // TODO: polyfill FinalizationRegistry
  const cmd = `wasm-bindgen --weak-refs --target ${target} --out-dir ${target} ../../target/wasm32-unknown-unknown/${profileDir}/loro_wasm.wasm`;
  console.log(">", cmd);
  await Deno.run({ cmd: cmd.split(" "), cwd: LoroWasmDir }).status();
  console.log();

  if (target === "nodejs") {
    console.log("🔨  Patching nodejs target");
    const patch = await Deno.readTextFile(
      path.resolve(__dirname, "./nodejs_patch.js"),
    );
    const wasm = await Deno.readTextFile(
      path.resolve(targetDirPath, "loro_wasm.js"),
    );
    await Deno.writeTextFile(
      path.resolve(targetDirPath, "loro_wasm.js"),
      wasm + "\n" + patch,
    );
  }
  if (target === "bundler") {
    console.log("🔨  Patching bundler target");
    const patch = await Deno.readTextFile(
      path.resolve(__dirname, "./bundler_patch.js"),
    );
    await Deno.writeTextFile(
      path.resolve(targetDirPath, "loro_wasm.js"),
      patch,
    );
  }
}

build();
