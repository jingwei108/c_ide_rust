use std::ffi::{c_char, CString};

fn test(name: &str, source: &str) {
    unsafe {
        let session = cide_native::capi::cide_session_create();
        let src = CString::new(source).unwrap();
        let ret = cide_native::capi::cide_compile(session, src.as_ptr() as *const c_char);
        if ret != 0 {
            let err = cide_native::capi::cide_get_compile_errors(session);
            let msg = if err.is_null() { "unknown".to_string() } else { std::ffi::CStr::from_ptr(err).to_string_lossy().to_string() };
            println!("{}: COMPILE ERROR: {}", name, msg);
        } else {
            cide_native::capi::cide_run(session);
            let out_len = cide_native::capi::cide_get_output_length(session);
            let mut buf = vec![0u8; out_len as usize + 1];
            cide_native::capi::cide_get_output(session, buf.as_mut_ptr() as *mut c_char, buf.len() as i32);
            println!("{}: OUTPUT: {:?}", name, String::from_utf8_lossy(&buf[..out_len as usize]));
        }
        cide_native::capi::cide_session_destroy(session);
    }
}

fn main() {
    test("null_char", "int main(){ char c='\0'; printf(\"%d\",c); return 0; }");
    test("for_assign", "int main(){ unsigned v=~0u; int i; for(i=1;(v=v>>1)>0;i++); printf(\"%d\",i); return 0; }");
    test("for_compound", "int main(){ unsigned x=0xFF; int b; for(b=0;x!=0;x&=x-1) ++b; printf(\"%d\",b); return 0; }");
    test("for_compound2", "int main(){ int a=5; a&=3; printf(\"%d\",a); return 0; }");
}
