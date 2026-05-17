use cide_native::engine::compile_pipeline::{run_compile_pipeline, setup_vm};
use cide_native::session::Session;
use cide_native::vm::vm::CideVM;

fn make_session(source: &str) -> Session {
    let mut session = Session::default();
    session.compile.compile_units.push(cide_native::session::CompileUnit {
        filename: "main.c".to_string(),
        source: source.to_string(),
    });
    let mut full_source = source.to_string();
    if !full_source.ends_with('\n') {
        full_source.push('\n');
    }
    run_compile_pipeline(&mut session, &full_source).expect("compile failed");
    session
}

fn setup_vm_for_session(session: &mut Session) -> CideVM {
    let mut vm = CideVM::new();
    setup_vm(&mut vm, session);
    vm
}

#[test]
fn test_snapshot_roundtrip() {
    let source = r#"
#include <stdio.h>
int main() {
    int a = 1;
    int b = 2;
    int c = a + b;
    printf("%d", c);
    return 0;
}
"#;
    let mut session = make_session(source);
    let mut vm = setup_vm_for_session(&mut session);

    // 执行若干步
    for _ in 0..5 {
        let _ = vm.step(&mut session);
    }

    let step_count_before = vm.get_executed_steps();
    let stack_before = vm.get_stack().to_vec();
    let call_stack_before = vm.get_call_stack().to_vec();
    let current_line_before = vm.get_current_line();

    // 创建快照
    let snap = vm.snapshot(&session);

    // 继续执行若干步，改变状态
    for _ in 0..10 {
        let _ = vm.step(&mut session);
    }
    assert_ne!(vm.get_executed_steps(), step_count_before);

    // 从快照恢复
    vm.restore(&snap, &mut session);

    // 验证状态完全一致
    assert_eq!(vm.get_executed_steps(), step_count_before);
    assert_eq!(vm.get_stack(), stack_before.as_slice());
    assert_eq!(vm.get_call_stack(), call_stack_before.as_slice());
    assert_eq!(vm.get_current_line(), current_line_before);
}

#[test]
fn test_snapshot_memory_preserved() {
    let source = r#"
int main() {
    int arr[3] = {10, 20, 30};
    arr[1] = 99;
    return 0;
}
"#;
    let mut session = make_session(source);
    let mut vm = setup_vm_for_session(&mut session);

    // 执行到 main 内部
    for _ in 0..20 {
        let _ = vm.step(&mut session);
    }

    let snap = vm.snapshot(&session);

    // 继续执行，修改内存
    for _ in 0..10 {
        let _ = vm.step(&mut session);
    }

    // 恢复
    vm.restore(&snap, &mut session);

    // 读取内存验证
    let _mem = vm.memory_ref();
    // 全局区起始 0x1000，arr 应该在全局区（因为是局部变量但放在栈上……
    // 这个测试主要验证 memory 的 copy_from_slice 工作正常，
    // 具体地址取决于 bytecode gen，这里只比较整段内存
    assert_eq!(vm.memory_ref(), snap.memory.as_slice());
}

#[test]
fn test_heatmap_collection() {
    let source = r#"
int main() {
    int sum = 0;
    for (int i = 0; i < 5; i++) {
        sum = sum + i;
    }
    return sum;
}
"#;
    let mut session = make_session(source);
    let mut vm = setup_vm_for_session(&mut session);

    // 执行完整程序
    loop {
        match vm.step(&mut session) {
            cide_native::vm::vm::StepResult::Finished => break,
            cide_native::vm::vm::StepResult::Trap => panic!("trap: {}", vm.get_error()),
            _ => {}
        }
    }

    let heatmap = &session.runtime.heatmap;
    let max_count = heatmap.max_count();
    assert!(max_count > 0, "heatmap should have recorded executions");

    // for 循环体应该被多次执行
    let loop_line = 4; // sum = sum + i;
    let loop_count = heatmap.line_counts.get(&loop_line).copied().unwrap_or(0);
    assert!(
        loop_count >= 5,
        "loop body line {} should execute at least 5 times, got {}",
        loop_line,
        loop_count
    );
}
