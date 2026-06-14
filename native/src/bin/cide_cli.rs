//! Cide CLI — 直接运行 Rust 后端编译器/VM 的命令行调试工具
//!
//! 用法:
//!   cargo run --bin cide_cli -- compile <file.c>
//!   cargo run --bin cide_cli -- run    <file.c> [-i input.txt]
//!   cargo run --bin cide_cli -- step   <file.c> [-i input.txt]
//!   cargo run --bin cide_cli -- unified <file.c> [-i input.txt]

use std::env;
use std::fs;
use std::io::{self, Read, Write};

use cide_native::flutter_bridge;
use cide_native::session::CodeFile;

fn print_usage() {
    eprintln!("Cide CLI — C 语言教学 IDE 后端调试工具");
    eprintln!();
    eprintln!("用法:");
    eprintln!("  cide_cli compile <file.c>           编译并显示诊断信息");
    eprintln!("  cide_cli run    <file.c> [-i <in>] [-- <arg>...]  编译并全速运行（-- 后参数传给 main）");
    eprintln!("  cide_cli step   <file.c> [-i <in>]  交互式单步调试");
    eprintln!("  cide_cli unified <file.c> [-i <in>] 统一模式（时间旅行）执行并摘要");
    eprintln!("  cide_cli export <file1.c> [file2.c ...] -o <out.json> [--builtin-libc]  预编译为字节码产物");
    eprintln!();
    eprintln!("特殊文件名:");
    eprintln!("  -          从标准输入读取源代码（如 echo '...' | cide_cli run -）");
    eprintln!();
    eprintln!("选项:");
    eprintln!("  -i <file>   从文件读取标准输入（多行输入）");
    eprintln!("  -o <file>   指定输出文件（仅 export 命令需要）");
    eprintln!("  --builtin-libc  库模式导出（export 命令）：不混入已有 Bytecode Libc 符号");
}

fn read_source(path: &str) -> String {
    if path == "-" {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf).unwrap_or_else(|e| {
            eprintln!("错误: 无法读取标准输入: {}", e);
            std::process::exit(1);
        });
        buf
    } else {
        fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("错误: 无法读取文件 '{}': {}", path, e);
            std::process::exit(1);
        })
    }
}

fn read_input_file(path: &str) -> Vec<String> {
    fs::read_to_string(path)
        .unwrap_or_else(|e| {
            eprintln!("错误: 无法读取输入文件 '{}': {}", path, e);
            std::process::exit(1);
        })
        .lines()
        .map(|s| s.to_string())
        .collect()
}

fn compile_file(path: &str, source: &str) -> bool {
    flutter_bridge::reset_session();
    let filename = if path == "-" {
        "main.c".to_string()
    } else {
        path.to_string()
    };
    let result = flutter_bridge::compile_multi(vec![CodeFile {
        filename,
        source: source.to_string(),
    }]);

    if !result.diagnostics.is_empty() {
        println!("=== 诊断信息 ===");
        for d in &result.diagnostics {
            let severity = match d.severity {
                0 => "错误",
                1 => "警告",
                2 => "提示",
                _ => "信息",
            };
            println!("[{}] {}:{}  {} (E{})", severity, d.line, d.column, d.message, d.error_code);
            if !d.fix_suggestion.is_empty() {
                println!("    建议: {}", d.fix_suggestion);
            }
        }
    }

    if !result.success {
        eprintln!("\n编译失败。");
        false
    } else {
        println!("\n编译成功。");
        if !result.algorithm_matches.is_empty() {
            println!("检测到算法:");
            for m in &result.algorithm_matches {
                println!("  • {} (置信度: {}%)", m.display_name, m.confidence);
            }
        }
        true
    }
}

fn cmd_compile(path: &str) {
    let source = read_source(path);
    compile_file(path, &source);
}

fn cmd_run(path: &str, input_lines: Vec<String>, argv: Vec<String>) {
    let source = read_source(path);
    if !compile_file(path, &source) {
        std::process::exit(1);
    }

    // 注入输入
    for line in input_lines {
        flutter_bridge::provide_input_line(line);
    }

    flutter_bridge::set_argv(argv);

    let result = flutter_bridge::run_code();
    println!("\n=== 运行输出 ===");
    println!("{}", result.output);
    if !result.success {
        if let Some(err) = result.error {
            eprintln!("运行错误: {}", err);
        }
        std::process::exit(1);
    }
    if result.waiting_input {
        println!("[程序等待输入，但输入已耗尽]");
    }
}

fn cmd_step(path: &str, input_lines: Vec<String>) {
    let source = read_source(path);
    if !compile_file(path, &source) {
        std::process::exit(1);
    }

    // 注入输入
    for line in &input_lines {
        flutter_bridge::provide_input_line(line.clone());
    }

    println!("=== 交互式单步调试 ===");
    println!("命令: [Enter]=下一步, p=打印变量, o=打印输出, q=退出, r=运行到结束");
    println!();

    let mut step_count = 0;
    loop {
        let line = flutter_bridge::get_current_line();
        let source_line = source.lines().nth((line.saturating_sub(1)) as usize).unwrap_or("").trim();

        print!("步 {:4} | 行 {:3}: {}  > ", step_count, line, source_line);
        io::stdout().flush().unwrap();

        let mut buf = String::new();
        if io::stdin().read_line(&mut buf).is_err() {
            break;
        }
        let cmd = buf.trim();

        match cmd {
            "q" | "quit" => {
                println!("退出调试。");
                break;
            }
            "p" | "print" => {
                let vars = flutter_bridge::get_variables();
                if vars.is_empty() {
                    println!("  (无局部变量)");
                } else {
                    for v in &vars {
                        println!("  {}: {:?} = {}", v.name, v.ty, v.value);
                    }
                }
                continue;
            }
            "o" | "output" => {
                let out = flutter_bridge::get_output();
                if out.is_empty() {
                    println!("  (无输出)");
                } else {
                    println!("{}", out);
                }
                continue;
            }
            "r" | "run" => {
                let result = flutter_bridge::run_code();
                println!("\n=== 最终输出 ===");
                println!("{}", result.output);
                if !result.success {
                    if let Some(err) = result.error {
                        eprintln!("运行错误: {}", err);
                    }
                }
                break;
            }
            "" => {
                // 下一步
            }
            _ => {
                println!("未知命令: {}", cmd);
                continue;
            }
        }

        let result = flutter_bridge::step_next();
        step_count += 1;

        use cide_native::session::StepStatus;
        match result.status {
            StepStatus::Paused => {}
            StepStatus::WaitingInput => {
                println!("  [等待输入...]");
            }
            StepStatus::Finished => {
                println!("\n程序执行完毕。");
                println!("\n=== 最终输出 ===");
                println!("{}", flutter_bridge::get_output());
                break;
            }
            StepStatus::Trap => {
                eprintln!("\n运行错误 (trap)。");
                println!("\n=== 当前输出 ===");
                println!("{}", flutter_bridge::get_output());
                break;
            }
        }
    }
}

fn cmd_export(source_paths: &[String], output_path: &str, is_builtin_libc: bool) {
    use cide_native::engine::compile_pipeline::run_multi_file_pipeline;
    use cide_native::session::{CompileUnit, Session};

    let mut units = Vec::new();
    for path in source_paths {
        let source = read_source(path);
        units.push(CompileUnit { filename: path.clone(), source });
    }

    // BytecodeGen 需要 main 函数，添加一个空 stub，导出后过滤掉
    units.push(CompileUnit {
        filename: "__export_main_stub.c".to_string(),
        source: "int main() { return 0; }".to_string(),
    });

    let mut session = Session::default();
    if let Err(e) = run_multi_file_pipeline(&mut session, units, is_builtin_libc) {
        eprintln!("编译失败: {}", e);
        let diags: Vec<String> = session
            .compile
            .diagnostics
            .iter()
            .map(|d| format!("{}:{}: {} (E{})", d.filename, d.line, d.message, d.error_code))
            .collect();
        if !diags.is_empty() {
            eprintln!("诊断:\n{}", diags.join("\n"));
        }
        std::process::exit(1);
    }

    #[derive(serde::Serialize)]
    struct BytecodeLibcExport {
        version: u32,
        code_len: usize,
        code: Vec<cide_native::vm::instruction::Instruction>,
        func_table: std::collections::HashMap<String, cide_native::session::FuncMeta>,
        func_index: std::collections::HashMap<String, i32>,
        globals_init_32: Vec<(u32, i32)>,
        globals_init_64: Vec<(u32, u64)>,
        string_data: Vec<(u32, String)>,
        f64_constants: Vec<f64>,
        i64_constants: Vec<i64>,
        globals_size: u32,
    }

    // 过滤掉 main stub 相关的产物
    let mut code = session.compile.bytecode.clone();
    let mut func_table = session.compile.func_table.clone();
    let mut func_index = session.compile.func_index.clone();
    func_table.remove("main");
    func_index.remove("main");

    // --builtin-libc 模式：移除 BytecodeGen 预注册的旧 Bytecode Libc 函数。
    // 这些函数只在 func_index 中有条目（用于用户代码调用固定索引），
    // 但没有 func_table 条目（不是当前源码实际定义的）。
    if is_builtin_libc {
        use cide_native::vm::bytecode_libc_index::BYTECODE_LIBC_ALL_FUNCS;
        let old_names: Vec<String> = func_index
            .keys()
            .filter(|name| BYTECODE_LIBC_ALL_FUNCS.contains(&name.as_str()) && !func_table.contains_key(name.as_str()))
            .cloned()
            .collect();
        for name in old_names {
            func_index.remove(&name);
        }
    }

    // 移除 BytecodeGen 生成的入口 wrapper（Jump + Call main + Ret）
    // code[0] 是 Jump 到 wrapper_ip，wrapper_ip 位置是 Call main 和 Ret
    let wrapper_ip = if !code.is_empty() && code[0].op == cide_native::vm::opcode::OpCode::Jump {
        code[0].operand as usize
    } else {
        code.len()
    };
    // 将入口 Jump 替换为 Nop（Bytecode Libc 作为库，不需要入口 Jump）
    if !code.is_empty() {
        code[0] = cide_native::vm::instruction::Instruction::new(
            cide_native::vm::opcode::OpCode::Nop,
            0,
            cide_native::vm::instruction::SourceLoc::default(),
        );
    }
    // 截断掉 wrapper 部分（Call main + Ret）
    code.truncate(wrapper_ip);

    // 计算全局变量使用的最大偏移
    let globals_size = session
        .compile
        .globals_init
        .iter()
        .map(|(offset, _)| *offset)
        .chain(session.compile.globals_init_64.iter().map(|(offset, _)| *offset))
        .max()
        .unwrap_or(0);

    let export = BytecodeLibcExport {
        version: 1,
        code_len: code.len(),
        code,
        func_table,
        func_index,
        globals_init_32: session.compile.globals_init.clone(),
        globals_init_64: session.compile.globals_init_64.clone(),
        string_data: session.compile.string_data.clone(),
        f64_constants: session.compile.f64_constants.clone(),
        i64_constants: session.compile.i64_constants.clone(),
        globals_size: globals_size + 4, // 预留一点余量
    };

    let json = serde_json::to_string_pretty(&export).unwrap_or_else(|e| {
        eprintln!("序列化失败: {}", e);
        std::process::exit(1);
    });

    fs::write(output_path, json).unwrap_or_else(|e| {
        eprintln!("写入输出文件失败 '{}': {}", output_path, e);
        std::process::exit(1);
    });

    println!("预编译完成: {}", output_path);
    println!("  代码长度: {} 条指令", export.code_len);
    println!("  函数数量: {}", export.func_index.len());
    println!("  全局变量大小: {} bytes", export.globals_size);
}

fn cmd_unified(path: &str, input_lines: Vec<String>) {
    let source = read_source(path);

    // 使用底层 API 进行统一模式执行
    use cide_native::engine::compile_pipeline::{run_multi_file_pipeline, setup_vm};
    use cide_native::engine::session_ops::{inject_preset_files, reset_runtime_for_step};
    use cide_native::session::{CompileUnit, Session};
    use cide_native::unified::engine::UnifiedEngine;
    use cide_native::vm::core::CideVM;

    let mut session = Session::default();
    session.compile.compile_units.push(CompileUnit {
        filename: path.to_string(),
        source: source.clone(),
    });

    let units = session.compile.compile_units.clone();
    if run_multi_file_pipeline(&mut session, units, false).is_err() {
        eprintln!("编译失败。");
        std::process::exit(1);
    }

    let mut engine = UnifiedEngine::new();
    engine.reset();

    let mut vm = CideVM::default();
    reset_runtime_for_step(&mut session);
    setup_vm(&mut vm, &session);
    inject_preset_files(&mut vm, &mut session);
    session.runtime.running = true;

    // 保存初始检查点
    engine.checkpoints.save(0, &mut vm, &session);
    session.vm = Some(vm);

    // 注入输入 (通过 flutter_bridge 的全局 session 不行，因为这里用的是本地 session)
    // 我们直接把输入放到 session.runtime.input_lines 中
    session.runtime.input_lines = input_lines;
    session.runtime.input_index = 0;

    println!("=== 统一模式执行（时间旅行引擎）===");

    let mut total_steps = 0;
    let mut trapped = false;
    let mut trap_msg = None;

    loop {
        let mut vm = session.vm.take().unwrap_or_default();
        let result = engine.run_batch(&mut vm, &mut session, 100);
        session.vm = Some(vm);

        match result {
            Ok(batch) => {
                total_steps += batch.payloads.len() as i32;
                if batch.finished {
                    break;
                }
                if batch.trapped {
                    trapped = true;
                    trap_msg = batch.trap_message;
                    break;
                }
                if batch.waiting_input && session.runtime.input_lines.is_empty() {
                    println!("[程序等待输入，但输入已耗尽]");
                    break;
                }
            }
            Err(e) => {
                trapped = true;
                trap_msg = Some(e);
                break;
            }
        }

        if total_steps % 500 == 0 {
            print!("\r  已执行 {} 步...", total_steps);
            io::stdout().flush().unwrap();
        }
    }

    println!("\r  共执行 {} 步", total_steps);

    // 输出摘要
    println!("\n=== 执行摘要 ===");
    println!("总步数: {}", total_steps);
    if trapped {
        println!("状态: 异常终止");
        if let Some(msg) = trap_msg {
            println!("错误: {}", msg);
        }
    } else {
        println!("状态: 正常结束");
    }

    println!("\n=== 最终输出 ===");
    println!("{}", session.runtime.output());

    // 打印最后几步的变量
    if !engine.frame_cache.is_empty() {
        let last = &engine.frame_cache[engine.frame_cache.len().saturating_sub(1)];
        println!("\n=== 最后一步变量 (行 {}) ===", last.code_line);
        for v in &last.local_vars {
            println!("  {}: {} = {}", v.name, v.ty_name, v.value);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        print_usage();
        std::process::exit(1);
    }

    let cmd = &args[1];
    let file_path = &args[2];

    // 解析 -i 选项与 -- 后的命令行参数
    let mut input_lines = Vec::new();
    let mut argv = vec![file_path.clone()];
    let mut i = 3;
    let mut passthrough = false;
    while i < args.len() {
        if passthrough {
            argv.push(args[i].clone());
            i += 1;
        } else if args[i] == "-i" && i + 1 < args.len() {
            input_lines = read_input_file(&args[i + 1]);
            i += 2;
        } else if args[i] == "--" {
            passthrough = true;
            i += 1;
        } else {
            // 未识别的位置参数视为传给 main 的 argv
            argv.push(args[i].clone());
            i += 1;
        }
    }

    match cmd.as_str() {
        "compile" => cmd_compile(file_path),
        "run" => cmd_run(file_path, input_lines, argv),
        "step" => cmd_step(file_path, input_lines),
        "unified" => cmd_unified(file_path, input_lines),
        "export" => {
            // export 命令需要至少一个源文件和 -o 选项
            let mut output_path = String::new();
            let mut is_builtin_libc = false;
            let mut source_paths = vec![file_path.clone()];
            let mut i = 3;
            while i < args.len() {
                if args[i] == "-o" && i + 1 < args.len() {
                    output_path = args[i + 1].clone();
                    i += 2;
                } else if args[i] == "--builtin-libc" {
                    is_builtin_libc = true;
                    i += 1;
                } else {
                    source_paths.push(args[i].clone());
                    i += 1;
                }
            }
            if output_path.is_empty() {
                eprintln!("错误: export 命令需要 -o <输出文件> 选项");
                print_usage();
                std::process::exit(1);
            }
            cmd_export(&source_paths, &output_path, is_builtin_libc);
        }
        _ => {
            eprintln!("未知命令: {}", cmd);
            print_usage();
            std::process::exit(1);
        }
    }
}
