use cide_native::vm::vm::CideVM;

#[test]
fn test_write_cstring_basic() {
    let mut vm = CideVM::default();
    vm.write_cstring(0, "hello");
    assert_eq!(&vm.memory_ref()[..6], b"hello\0");
}

#[test]
fn test_write_cstring_at_offset() {
    let mut vm = CideVM::default();
    vm.write_cstring(10, "world");
    assert_eq!(&vm.memory_ref()[10..16], b"world\0");
    // 之前的区域不应被修改
    assert_eq!(vm.memory_ref()[0..10].iter().sum::<u8>(), 0);
}

#[test]
fn test_write_cstring_exact_fit() {
    let mut vm = CideVM::default();
    let addr = 100;
    vm.write_cstring(addr as u32, "hello");
    // "hello" is 5 bytes + null = 6
    let mem = vm.memory_ref();
    assert_eq!(&mem[addr..addr + 6], b"hello\0");
}

#[test]
fn test_write_cstring_boundary_rejected() {
    let mut vm = CideVM::default();
    let addr = (vm.get_memory_size() - 2) as usize; // 只剩 2 字节空间
    let before = vm.memory_ref()[addr..].to_vec();
    vm.write_cstring(addr as u32, "hello"); // 5+1=6 字节，超出边界
                                            // 不应写入
    assert_eq!(vm.memory_ref()[addr..], before);
}

#[test]
fn test_write_cstring_offset_boundary() {
    let mut vm = CideVM::default();
    let addr = (vm.get_memory_size() - 3) as usize;
    let before = vm.memory_ref()[addr..].to_vec();
    // addr + 6 > MEM_SIZE, should not write
    vm.write_cstring(addr as u32, "hello");
    assert_eq!(vm.memory_ref()[addr..], before);
}

#[test]
fn test_write_cstring_empty_string() {
    let mut vm = CideVM::default();
    vm.write_cstring(0, "");
    // empty string writes just '\0' at addr 0
    assert_eq!(vm.memory_ref()[0], 0);
}

#[test]
fn test_write_cstring_chinese() {
    let mut vm = CideVM::default();
    let s = "你好";
    vm.write_cstring(0, s);
    let written = &vm.memory_ref()[..s.len() + 1];
    assert_eq!(&written[..s.len()], s.as_bytes());
    assert_eq!(written[s.len()], 0);
}
