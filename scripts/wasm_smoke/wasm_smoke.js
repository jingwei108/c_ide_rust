// G2：wasm 出口冒烟（Node）。
// 验证：模块可实例化 + ABI 导出存在 + capi 全链路（建会话/编译/运行/取输出）
// 在 wasm32 下工作（出口 2 守护）。
// 字符串传递：无分配导出，使用 `__heap_base`（Rust wasm 固定导出）之后的
// 空闲线性内存——冒烟时堆未使用，无踩踏风险。
const fs = require("fs");
const path = require("path");

const wasmPath = process.argv[2]
  || path.join(__dirname, "../../native/target/wasm32-unknown-unknown/release/cide_native.wasm");
if (!fs.existsSync(wasmPath)) {
  console.error(`[wasm-smoke] 找不到 wasm 模块: ${wasmPath}`);
  process.exit(2);
}

(async () => {
  // wasm 产物含 wasm-bindgen 占位导入（js-sys 依赖链）。冒烟路径（capi 全链路）
  // 不触达这些回调，用 Proxy 桩提供"任何名字都是抛错函数"的导入对象即可实例化。
  const stubFn = () => { throw new Error("__wbindgen stub called（冒烟路径不应触达）"); };
  const bindgenStub = new Proxy({}, { get: () => stubFn });
  const imports = new Proxy(
    {},
    {
      // 任何导入模块/名字都返回抛错桩——冒烟路径不触达 wasm-bindgen 回调，
      // 只需让实例化通过（真实消费由 wasm-bindgen 胶水提供）
      get: (target, prop) => {
        if (prop === 'memory' || prop === '__wbindgen_externref_xform__') {
          return bindgenStub;
        }
        return bindgenStub;
      },
    }
  );
  const { instance } = await WebAssembly.instantiate(fs.readFileSync(wasmPath), imports);
  const ex = instance.exports;
  const ok = (cond, label) => {
    if (!cond) {
      console.error(`[wasm-smoke] FAIL ${label}`);
      process.exit(1);
    }
    console.log(`  PASS  ${label}`);
  };

  ok(typeof ex.cide_abi_version === "function", "cide_abi_version 导出存在");
  ok(ex.cide_abi_version() >= 1, `ABI 版本 ${ex.cide_abi_version()} >= 1`);

  ok(typeof ex.__heap_base === "object" || typeof ex.__heap_base === "number", "__heap_base 导出存在");
  const heapBase = Number(ex.__heap_base?.value ?? ex.__heap_base ?? 0);
  ok(heapBase > 0, `堆基址 0x${heapBase.toString(16)}`);

  // 布局：fname @ base，src @ base+64（均含 NUL，互不重叠）
  const base = heapBase + 16;
  const mem = () => new Uint8Array(ex.memory.buffer);
  const writeCStr = (ptr, s) => {
    const b = Buffer.from(s + "\0", "utf-8");
    mem().set(b, ptr);
    return ptr;
  };

  const session = ex.cide_session_create();
  ok(Number(session) !== 0, "cide_session_create 成功");

  const fnamePtr = writeCStr(base, "main.c");
  const srcPtr = writeCStr(base + 64, '#include <stdio.h>\nint main(){ printf("wasm-ok"); return 0; }\n');

  ok(ex.cide_compile_unit(session, fnamePtr, srcPtr) === 0, "cide_compile_unit 成功");
  ok(ex.cide_compile_all(session) === 0, "cide_compile_all 成功");
  const runRet = ex.cide_run(session);
  ok(runRet >= 0, `cide_run 执行（返回 ${runRet}）`);

  // E-P1-5 口径：读纯程序 stdout 通道（cide_get_program_output*），不含引擎附注
  ok(
    typeof ex.cide_get_program_output_length === "function"
      && typeof ex.cide_get_program_output === "function",
    "纯 stdout 通道导出存在（cide_get_program_output*）"
  );
  const outLen = ex.cide_get_program_output_length(session);
  ok(outLen === 7, `输出长度 ${outLen} == 7（"wasm-ok"）`);
  ex.cide_get_program_output(session, base + 4096, outLen + 1);
  const out = Buffer.from(mem().subarray(base + 4096, base + 4096 + outLen)).toString("utf-8");
  ok(out === "wasm-ok", `输出内容 ${JSON.stringify(out)} === "wasm-ok"`);

  console.log("[wasm-smoke] 全部通过");
})().catch((e) => {
  console.error("[wasm-smoke] 异常:", e);
  process.exit(1);
});
