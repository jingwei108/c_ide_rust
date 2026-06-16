use super::*;

pub fn host_abs(vm: &mut CideVM, _session: &mut Session) {
    let n = vm.pop() as i32;
    vm.push(if n < 0 { n.wrapping_neg() as u64 } else { n as u64 });
}

// ========== math.h Host Functions ==========

pub fn host_sin(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::sin(x).to_bits());
}

pub fn host_cos(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::cos(x).to_bits());
}

pub fn host_sqrt(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::sqrt(x).to_bits());
}

pub fn host_pow(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    let y = f64::from_bits(vm.pop());
    vm.push(libm::pow(x, y).to_bits());
}

pub fn host_atan(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::atan(x).to_bits());
}

pub fn host_log(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::log(x).to_bits());
}

pub fn host_exp(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::exp(x).to_bits());
}

pub fn host_tan(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::tan(x).to_bits());
}

pub fn host_log10(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::log10(x).to_bits());
}

pub fn host_fabs(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::fabs(x).to_bits());
}

pub fn host_ceil(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::ceil(x).to_bits());
}

pub fn host_floor(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::floor(x).to_bits());
}

pub fn host_round(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::round(x).to_bits());
}

pub fn host_fmod(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    let y = f64::from_bits(vm.pop());
    vm.push(libm::fmod(x, y).to_bits());
}

pub fn host_asin(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::asin(x).to_bits());
}

pub fn host_acos(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::acos(x).to_bits());
}

pub fn host_atan2(vm: &mut CideVM, _session: &mut Session) {
    let y = f64::from_bits(vm.pop());
    let x = f64::from_bits(vm.pop());
    vm.push(libm::atan2(y, x).to_bits());
}

pub fn host_sinh(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::sinh(x).to_bits());
}

pub fn host_cosh(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::cosh(x).to_bits());
}

pub fn host_tanh(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::tanh(x).to_bits());
}

pub fn host_llabs(vm: &mut CideVM, _session: &mut Session) {
    let n = vm.pop() as i64;
    vm.push(if n < 0 { n.wrapping_neg() as u64 } else { n as u64 });
}
