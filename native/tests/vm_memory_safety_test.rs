use cide_native::engine::compile_pipeline::write_string_to_vm_memory;

#[test]
fn test_write_string_to_vm_memory_basic() {
    let mut mem = vec![0u8; 256];
    let mem_ptr = mem.as_mut_ptr();
    write_string_to_vm_memory(mem_ptr, mem.len(), 0, "hello");
    assert_eq!(&mem[..6], b"hello\0");
}

#[test]
fn test_write_string_to_vm_memory_at_offset() {
    let mut mem = vec![0u8; 256];
    let mem_ptr = mem.as_mut_ptr();
    write_string_to_vm_memory(mem_ptr, mem.len(), 10, "world");
    assert_eq!(&mem[10..16], b"world\0");
    // 之前的区域不应被修改
    assert_eq!(mem[0..10].iter().sum::<u8>(), 0);
}

#[test]
fn test_write_string_to_vm_memory_exact_fit() {
    let mut mem = vec![0u8; 6];
    let mem_ptr = mem.as_mut_ptr();
    write_string_to_vm_memory(mem_ptr, mem.len(), 0, "hello");
    // "hello" is 5 bytes + null = 6, exactly fits
    assert_eq!(&mem[..6], b"hello\0");
}

#[test]
fn test_write_string_to_vm_memory_boundary_rejected() {
    let mut mem = vec![0u8; 5];
    let mem_ptr = mem.as_mut_ptr();
    write_string_to_vm_memory(mem_ptr, mem.len(), 0, "hello");
    // "hello" is 5 bytes + null = 6, but mem_size is 5, should not write
    assert_eq!(mem.iter().sum::<u8>(), 0);
}

#[test]
fn test_write_string_to_vm_memory_offset_boundary() {
    let mut mem = vec![0u8; 10];
    let mem_ptr = mem.as_mut_ptr();
    // addr=8, string="hello" (5+1=6 bytes), 8+6=14 > 10, should not write
    write_string_to_vm_memory(mem_ptr, mem.len(), 8, "hello");
    assert_eq!(mem.iter().sum::<u8>(), 0);
}

#[test]
fn test_write_string_to_vm_memory_empty_string() {
    let mut mem = vec![0xFFu8; 4];
    let mem_ptr = mem.as_mut_ptr();
    write_string_to_vm_memory(mem_ptr, mem.len(), 0, "");
    // empty string writes just '\0' at addr 0
    assert_eq!(mem[0], 0);
    assert_eq!(mem[1], 0xFF);
}

#[test]
fn test_write_string_to_vm_memory_chinese() {
    let mut mem = vec![0u8; 256];
    let mem_ptr = mem.as_mut_ptr();
    let s = "你好";
    write_string_to_vm_memory(mem_ptr, mem.len(), 0, s);
    let written = &mem[..s.len() + 1];
    assert_eq!(&written[..s.len()], s.as_bytes());
    assert_eq!(written[s.len()], 0);
}
