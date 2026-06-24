use super::*;

pub(crate) fn gen_literal(gen: &mut BytecodeGen, value: i32, loc: &SourceLoc) {
    gen.emit(OpCode::PushConst, value, loc);
}

pub(crate) fn gen_float_literal(gen: &mut BytecodeGen, value: f64, ty: &Type, loc: &SourceLoc) {
    if ty.kind() == TypeKind::Double {
        let idx = gen.push_f64_constant(value);
        gen.emit(OpCode::PushConstD, idx, loc);
    } else {
        let bits = (value as f32).to_bits() as i32;
        gen.emit(OpCode::PushConstF, bits, loc);
    }
}

pub(crate) fn gen_long_literal(gen: &mut BytecodeGen, value: i64, loc: &SourceLoc) {
    let idx = gen.push_i64_constant(value);
    gen.emit(OpCode::PushConstQ, idx, loc);
}

pub(crate) fn gen_string_literal(gen: &mut BytecodeGen, value: &str, loc: &SourceLoc) {
    let aligned = ((value.len() + 1) as u32 + 3) & !3;
    let addr = cide_runtime::GLOBAL_START + gen.next_global_offset as u32;
    let new_offset = gen.next_global_offset + aligned as i32;
    if new_offset as u32 + cide_runtime::GLOBAL_START > cide_runtime::MEM_SIZE / 16 {
        gen.report_error("字符串字面量过多，超出内存限制", loc);
        gen.emit(OpCode::PushConst, addr as i32, loc);
    } else {
        gen.string_data.push((addr, value.to_string()));
        gen.next_global_offset = new_offset;
        gen.emit(OpCode::PushConst, addr as i32, loc);
    }
}
