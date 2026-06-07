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

fn print_usage() {
    eprintln!("Cide CLI — C 语言教学 IDE 后端调试工具");
    eprintln!();
    eprintln!("用法:");
    eprintln!("  cide_cli compile <file.c>           编译并显示诊断信息");
    eprintln!("  cide_cli run    <file.c> [-i <in>]  编译并全速运行");
    eprintln!("  cide_cli step   <file.c> [-i <in>]  交互式单步调试");
    eprintln!("  cide_cli unified <file.c> [-i <in>] 统一模式（时间旅行）执行并摘要");
    eprintln!();
    eprintln!("特殊文件名:");
    eprintln!("  -          从标准输入读取源代码（如 echo '...' | cide_cli run -）");
    eprintln!();
    eprintln!("选项:");
    eprintln!("  -i <file>   从文件读取标准输入（多行输入）");
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

fn compile_file(_path: &str, source: &str) -> bool {
    flutter_bridge::reset_session();
    let result = flutter_bridge::compile(source.to_string());

    if !result.diagnostics.is_empty() {
        println!("=== 诊断信息 ===");
        for d in &result.diagnostics {
            let severity = match d.severity {
                1 => "错误",
                2 => "警告",
                3 => "提示",
                _ => "信息",
            };
            println!(
                "[{}] {}:{}  {} (E{})",
                severity, d.line, d.column, d.message, d.error_code
            );
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

fn cmd_run(path: &str, input_lines: Vec<String>) {
    let source = read_source(path);
    if !compile_file(path, &source) {
        std::process::exit(1);
    }

    // 注入输入
    for line in input_lines {
        flutter_bridge::provide_input_line(line);
    }

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
        let source_line = source
            .lines()
            .nth((line.saturating_sub(1)) as usize)
            .unwrap_or("")
            .trim();

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

fn cmd_unified(path: &str, input_lines: Vec<String>) {
    let source = read_source(path);

    // 使用底层 API 进行统一模式执行
    use cide_native::engine::compile_pipeline::{run_multi_file_pipeline, setup_vm};
    use cide_native::engine::session_ops::{reset_runtime_for_step, inject_preset_files};
    use cide_native::session::{Session, CompileUnit};
    use cide_native::vm::vm::CideVM;
    use cide_native::unified::engine::UnifiedEngine;

    let mut session = Session::default();
    session.compile.compile_units.push(CompileUnit {
        filename: path.to_string(),
        source: source.clone(),
    });

    let units = session.compile.compile_units.clone();
    if run_multi_file_pipeline(&mut session, units).is_err() {
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

    // 解析 -i 选项
    let mut input_lines = Vec::new();
    let mut i = 3;
    while i < args.len() {
        if args[i] == "-i" && i + 1 < args.len() {
            input_lines = read_input_file(&args[i + 1]);
            i += 2;
        } else {
            i += 1;
        }
    }

    match cmd.as_str() {
        "compile" => cmd_compile(file_path),
        "run" => cmd_run(file_path, input_lines),
        "step" => cmd_step(file_path, input_lines),
        "unified" => cmd_unified(file_path, input_lines),
        _ => {
            eprintln!("未知命令: {}", cmd);
            print_usage();
            std::process::exit(1);
        }
    }
}
