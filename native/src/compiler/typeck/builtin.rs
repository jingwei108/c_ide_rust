use super::*;

impl TypeChecker {
    pub(crate) fn is_builtin_func(&self, name: &str) -> bool {
        crate::vm::host_func_id::is_builtin(name)
    }

    pub(crate) fn visit_call(&mut self, name: &str, args: &mut [Expr], loc: &SourceLoc) -> Type {
        match name {
            "malloc" => self.check_builtin_malloc(args, loc),
            "free" => self.check_builtin_free(args, loc),
            "print_int" | "__cide_output" | "__cide_step" => {
                self.check_builtin_print_int(args, loc, name)
            }
            "printf" => self.check_builtin_printf(args, loc),
            "scanf" => self.check_builtin_scanf(args, loc),
            "strlen" => self.check_builtin_strlen(args, loc),
            "strcpy" => self.check_builtin_strcpy(args, loc),
            "strcmp" => self.check_builtin_strcmp(args, loc),
            "strdup" => self.check_builtin_strdup(args, loc),
            "getchar" => self.check_builtin_getchar(args, loc),
            "putchar" => self.check_builtin_putchar(args, loc),
            "ungetc" => self.check_builtin_ungetc(args, loc),
            "rand" => self.check_builtin_rand(args, loc),
            "srand" => self.check_builtin_srand(args, loc),
            "memset" => self.check_builtin_memset(args, loc),
            "exit" => self.check_builtin_exit(args, loc),
            "strcat" => self.check_builtin_strcat(args, loc),
            "atoi" => self.check_builtin_atoi(args, loc),
            "fopen" => self.check_builtin_fopen(args, loc),
            "fread" => self.check_builtin_fread(args, loc),
            "fwrite" => self.check_builtin_fwrite(args, loc),
            "fclose" => self.check_builtin_fclose(args, loc),
            "feof" => self.check_builtin_feof(args, loc),
            "fgets" => self.check_builtin_fgets(args, loc),
            "fputs" => self.check_builtin_fputs(args, loc),
            "fprintf" => self.check_builtin_fprintf(args, loc),
            "realloc" => self.check_builtin_realloc(args, loc),
            "qsort" => {
                if self.funcs.contains_key(name) {
                    self.check_user_func(name, args, loc)
                } else {
                    self.check_builtin_qsort(args, loc)
                }
            }
            "puts" => self.check_builtin_puts(args, loc),
            "calloc" => self.check_builtin_calloc(args, loc),
            "bsearch" => self.check_builtin_bsearch(args, loc),
            "sprintf" => self.check_builtin_sprintf(args, loc),
            "snprintf" => self.check_builtin_snprintf(args, loc),
            "sscanf" => self.check_builtin_sscanf(args, loc),
            "fgetc" => self.check_builtin_fgetc(args, loc),
            "fputc" => self.check_builtin_fputc(args, loc),
            "fseek" => self.check_builtin_fseek(args, loc),
            "ftell" => self.check_builtin_ftell(args, loc),
            "rewind" => self.check_builtin_rewind(args, loc),
            "strncat" => self.check_builtin_strncat(args, loc),
            "strncmp" => self.check_builtin_strncmp(args, loc),
            "memcmp" => self.check_builtin_memcmp(args, loc),
            "strchr" => self.check_builtin_strchr(args, loc),
            "strrchr" => self.check_builtin_strrchr(args, loc),
            "strstr" => self.check_builtin_strstr(args, loc),
            "memchr" => self.check_builtin_memchr(args, loc),
            "atof" => self.check_builtin_atof(args, loc),
            "atol" => self.check_builtin_atol(args, loc),
            "tan" => self.check_builtin_tan(args, loc),
            "log10" => self.check_builtin_log10(args, loc),
            "fabs" => self.check_builtin_fabs(args, loc),
            "ceil" => self.check_builtin_ceil(args, loc),
            "floor" => self.check_builtin_floor(args, loc),
            "round" => self.check_builtin_round(args, loc),
            "fmod" => self.check_builtin_fmod(args, loc),
            // math.h functions are resolved through stub declarations in funcs
            _ => self.check_user_func(name, args, loc),
        }
    }

    // ---------- 内建函数检查器辅助方法 ----------

    fn builtin_check_count(&mut self, args: &[Expr], expected: usize, name: &str, loc: &SourceLoc) -> bool {
        if args.len() != expected {
            self.report_error(&format!("{} 需要{}个参数", name, expected), loc, ErrorCode::E3028_BuiltInArgCount);
            false
        } else {
            true
        }
    }

    fn builtin_check_pointer(&mut self, arg: &mut Expr, idx: usize, name: &str, loc: &SourceLoc) {
        let arg_type = self.resolve_expr_type(arg);
        if !arg_type.is_pointer() && !arg_type.is_array() {
            self.report_error(&format!("{} 的第 {} 个参数必须是指针或数组", name, idx + 1), loc, ErrorCode::E3029_BuiltInArgType);
        }
    }

    fn builtin_check_int(&mut self, arg: &mut Expr, idx: usize, name: &str, loc: &SourceLoc) {
        let expected = Type::int();
        let arg_type = self.resolve_expr_type(arg);
        if !self.check_assignable(&expected, &arg_type, loc) {
            self.report_error(&format!("{} 的第 {} 个参数必须是 int", name, idx + 1), loc, ErrorCode::E3029_BuiltInArgType);
        } else {
            insert_implicit_cast(arg, &expected);
        }
    }

    // ---------- 内建函数检查器 ----------

    fn check_builtin_malloc(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 1 {
            self.report_error("malloc 需要一个参数", loc, ErrorCode::E3024_MallocArgCount);
        } else {
            let expected = Type::int();
            let arg_type = self.resolve_expr_type(&mut args[0]);
            if !self.check_assignable(&expected, &arg_type, loc) {
                self.report_error("malloc 参数必须是 int", loc, ErrorCode::E3025_MallocArgType);
            } else {
                insert_implicit_cast(&mut args[0], &expected);
            }
        }
        Type::pointer_to(Type::void())
    }

    fn check_builtin_free(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 1, "free", loc) {
            self.builtin_check_pointer(&mut args[0], 0, "free", loc);
        }
        Type::void()
    }

    fn check_builtin_print_int(
        &mut self,
        args: &mut [Expr],
        loc: &SourceLoc,
        name: &str,
    ) -> Type {
        if args.len() != 1 {
            self.report_error(
                &format!("{} 需要一个参数", name),
                loc,
                ErrorCode::E3028_BuiltInArgCount,
            );
        } else {
            let expected = Type::int();
            let arg_type = self.resolve_expr_type(&mut args[0]);
            if !self.check_assignable(&expected, &arg_type, loc) {
                self.report_error(
                    &format!("{} 参数必须是 int", name),
                    loc,
                    ErrorCode::E3029_BuiltInArgType,
                );
            } else {
                insert_implicit_cast(&mut args[0], &expected);
            }
        }
        Type::void()
    }

    /// 解析 printf/scanf 格式字符串，返回非 %% 的格式说明符列表。
    /// 每个元素为 (spec_char, length_mod) 例如 ('d', "") 或 ('f', "l")
    fn parse_format_specs(fmt: &str) -> Vec<(char, String)> {
        let mut specs = Vec::new();
        let mut chars = fmt.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '%' {
                if let Some(&next) = chars.peek() {
                    if next == '%' {
                        chars.next(); // skip %%
                        continue;
                    }
                }
                // skip flags
                while let Some(&c) = chars.peek() {
                    if c == '-' || c == '+' || c == ' ' || c == '#' || c == '0' {
                        chars.next();
                    } else { break; }
                }
                // skip width
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() || c == '*' { chars.next(); } else { break; }
                }
                // skip precision
                if let Some(&'.') = chars.peek() {
                    chars.next();
                    while let Some(&c) = chars.peek() {
                        if c.is_ascii_digit() || c == '*' { chars.next(); } else { break; }
                    }
                }
                // length modifier
                let mut len_mod = String::new();
                if let Some(&c) = chars.peek() {
                    if c == 'l' || c == 'h' || c == 'L' || c == 'z' || c == 'j' || c == 't' {
                        chars.next();
                        len_mod.push(c);
                        if c == 'l' || c == 'h' {
                            if let Some(&c2) = chars.peek() {
                                if c2 == c {
                                    chars.next();
                                    len_mod.push(c2);
                                }
                            }
                        }
                    }
                }
                if let Some(&spec) = chars.peek() {
                    chars.next();
                    specs.push((spec, len_mod));
                }
            }
        }
        specs
    }

    fn check_printf_format(&mut self, fmt: &str, args: &mut [Expr], loc: &SourceLoc) {
        let specs = Self::parse_format_specs(fmt);
        if specs.len() != args.len() {
            self.report_error(
                &format!("printf 格式说明符数量（{}）与参数数量（{}）不匹配", specs.len(), args.len()),
                loc,
                ErrorCode::E3032_PrintfArgType,
            );
        }
        for (i, ((spec, len_mod), arg)) in specs.iter().zip(args.iter_mut()).enumerate() {
            let arg_type = self.resolve_expr_type(arg);
            let ok = match (*spec, len_mod.as_str()) {
                ('d' | 'i' | 'u' | 'x' | 'X' | 'o' | 'n', "") => {
                    matches!(arg_type.kind(), TypeKind::Int | TypeKind::Char)
                }
                ('d' | 'i' | 'u' | 'x' | 'X' | 'o', "l" | "ll") => {
                    matches!(arg_type.kind(), TypeKind::LongLong | TypeKind::Int)
                }
                ('f' | 'e' | 'g' | 'E' | 'G', "") => {
                    matches!(arg_type.kind(), TypeKind::Float | TypeKind::Double)
                }
                ('f' | 'e' | 'g' | 'E' | 'G', "l") | ('F', "") => {
                    matches!(arg_type.kind(), TypeKind::Double | TypeKind::Float)
                }
                ('c', "") => {
                    matches!(arg_type.kind(), TypeKind::Int | TypeKind::Char)
                }
                ('s', "") => {
                    arg_type.is_pointer() || arg_type.is_array()
                }
                ('p', "") => {
                    arg_type.is_pointer() || arg_type.is_array()
                }
                _ => true, // unknown spec, be permissive
            };
            if !ok {
                self.report_error(
                    &format!("printf 格式 '%{}' 与第 {} 个参数类型 '{}' 不匹配", spec, i + 2, arg_type),
                    loc,
                    ErrorCode::E3062_PrintfFormatMismatch,
                );
            }
        }
    }

    fn check_scanf_format(&mut self, fmt: &str, args: &mut [Expr], loc: &SourceLoc) {
        let specs = Self::parse_format_specs(fmt);
        if specs.len() != args.len() {
            self.report_error(
                &format!("scanf 格式说明符数量（{}）与参数数量（{}）不匹配", specs.len(), args.len()),
                loc,
                ErrorCode::E3035_ScanfArgType,
            );
        }
        for (i, ((spec, len_mod), arg)) in specs.iter().zip(args.iter_mut()).enumerate() {
            let arg_type = self.resolve_expr_type(arg);
            // scanf args must be pointers
            let pointee = if let Type::Pointer { pointee, .. } = &arg_type {
                Some((**pointee).clone())
            } else if let Type::Array { element, .. } = &arg_type {
                Some((**element).clone())
            } else {
                None
            };
            let ok = match (*spec, len_mod.as_str()) {
                ('d' | 'i' | 'u' | 'x' | 'X' | 'o' | 'n', "") => {
                    pointee.as_ref().is_some_and(|t| matches!(t.kind(), TypeKind::Int | TypeKind::Char))
                }
                ('d' | 'i' | 'u' | 'x' | 'X' | 'o', "l" | "ll") => {
                    pointee.as_ref().is_some_and(|t| matches!(t.kind(), TypeKind::LongLong | TypeKind::Int))
                }
                ('f' | 'e' | 'g' | 'E' | 'G', "") => {
                    pointee.as_ref().is_some_and(|t| matches!(t.kind(), TypeKind::Float | TypeKind::Int))
                }
                ('f' | 'e' | 'g' | 'E' | 'G', "l") | ('F', "") => {
                    pointee.as_ref().is_some_and(|t| matches!(t.kind(), TypeKind::Double | TypeKind::Float | TypeKind::Int))
                }
                ('c', "") => {
                    pointee.as_ref().is_some_and(|t| matches!(t.kind(), TypeKind::Char | TypeKind::Int))
                }
                ('s', "") => {
                    arg_type.is_pointer() || arg_type.is_array()
                }
                ('p', "") => {
                    arg_type.is_pointer() || arg_type.is_array()
                }
                _ => true,
            };
            if !ok {
                self.report_error(
                    &format!("scanf 格式 '%{}' 与第 {} 个参数类型 '{}' 不匹配", spec, i + 2, arg_type),
                    loc,
                    ErrorCode::E3063_ScanfFormatMismatch,
                );
            }
        }
    }

    fn check_builtin_printf(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.is_empty() {
            self.report_error(
                "printf 至少需要 1 个参数（格式字符串）",
                loc,
                ErrorCode::E3030_PrintfArgCount,
            );
        } else {
            let fmt_type = self.resolve_expr_type(&mut args[0]);
            if !fmt_type.is_pointer() && !fmt_type.is_array() {
                self.report_error(
                    "printf 的第一个参数必须是字符串",
                    loc,
                    ErrorCode::E3031_PrintfFirstArg,
                );
            }
            // 如果格式字符串是字面量，进行格式-参数类型匹配检查
            let fmt_str = if let Expr::StringLiteral { ref value, .. } = args[0] {
                Some(value.clone())
            } else {
                None
            };
            if let Some(fmt) = fmt_str {
                self.check_printf_format(&fmt, &mut args[1..], loc);
            } else {
                // 非字面量格式字符串，只做粗略检查
                for (i, arg) in args.iter_mut().enumerate().skip(1) {
                    let arg_type = self.resolve_expr_type(arg);
                    if !self.is_scalar(&arg_type) && !arg_type.is_pointer() && !arg_type.is_array() {
                        self.report_error(
                            &format!("printf 的第 {} 个参数必须是 int、float、char 或指针", i + 1),
                            loc,
                            ErrorCode::E3032_PrintfArgType,
                        );
                    }
                }
            }
        }
        Type::void()
    }

    fn check_builtin_scanf(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() < 2 {
            self.report_error(
                "scanf 至少需要 2 个参数（格式字符串和地址）",
                loc,
                ErrorCode::E3033_ScanfArgCount,
            );
        } else {
            let fmt_type = self.resolve_expr_type(&mut args[0]);
            if !fmt_type.is_pointer() && !fmt_type.is_array() {
                self.report_error(
                    "scanf 的第一个参数必须是字符串",
                    loc,
                    ErrorCode::E3034_ScanfFirstArg,
                );
            }
            let fmt_str = if let Expr::StringLiteral { ref value, .. } = args[0] {
                Some(value.clone())
            } else {
                None
            };
            if let Some(fmt) = fmt_str {
                self.check_scanf_format(&fmt, &mut args[1..], loc);
            } else {
                for (i, arg) in args.iter_mut().enumerate().skip(1) {
                    let arg_type = self.resolve_expr_type(arg);
                    if !arg_type.is_pointer() && !arg_type.is_array() {
                        self.report_error(
                            &format!("scanf 的第 {} 个参数必须是指针", i + 1),
                            loc,
                            ErrorCode::E3035_ScanfArgType,
                        );
                    }
                }
            }
        }
        Type::void()
    }

    fn check_builtin_strlen(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 1, "strlen", loc) {
            self.builtin_check_pointer(&mut args[0], 0, "strlen", loc);
        }
        Type::int()
    }

    fn check_builtin_strcpy(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 2, "strcpy", loc) {
            self.builtin_check_pointer(&mut args[0], 0, "strcpy", loc);
            self.builtin_check_pointer(&mut args[1], 1, "strcpy", loc);
        }
        Type::pointer_to(Type::char())
    }

    fn check_builtin_strdup(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 1, "strdup", loc) {
            self.builtin_check_pointer(&mut args[0], 0, "strdup", loc);
        }
        Type::pointer_to(Type::char())
    }

    fn check_builtin_strcmp(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 2, "strcmp", loc) {
            self.builtin_check_pointer(&mut args[0], 0, "strcmp", loc);
            self.builtin_check_pointer(&mut args[1], 1, "strcmp", loc);
        }
        Type::int()
    }

    fn check_builtin_getchar(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        self.builtin_check_count(args, 0, "getchar", loc);
        Type::int()
    }

    fn check_builtin_ungetc(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 2, "ungetc", loc) {
            self.builtin_check_int(&mut args[0], 0, "ungetc", loc);
            self.builtin_check_int(&mut args[1], 1, "ungetc", loc);
        }
        Type::int()
    }

    fn check_builtin_putchar(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 1, "putchar", loc) {
            self.builtin_check_int(&mut args[0], 0, "putchar", loc);
        }
        Type::void()
    }

    fn check_builtin_rand(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        self.builtin_check_count(args, 0, "rand", loc);
        Type::int()
    }

    fn check_builtin_srand(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 1, "srand", loc) {
            self.builtin_check_int(&mut args[0], 0, "srand", loc);
        }
        Type::void()
    }

    fn check_builtin_memset(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 3 {
            self.report_error("memset 需要三个参数", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            let ptr_type = self.resolve_expr_type(&mut args[0]);
            if !ptr_type.is_pointer() && !ptr_type.is_array() {
                self.report_error(
                    "memset 第一个参数必须是指针",
                    loc,
                    ErrorCode::E3029_BuiltInArgType,
                );
            }
            for i in 1..3 {
                let expected = Type::int();
                let t = self.resolve_expr_type(&mut args[i]);
                if !self.check_assignable(&expected, &t, loc) {
                    self.report_error(
                        &format!("memset 的第 {} 个参数必须是 int", i + 1),
                        loc,
                        ErrorCode::E3029_BuiltInArgType,
                    );
                } else {
                    insert_implicit_cast(&mut args[i], &expected);
                }
            }
        }
        Type::pointer_to(Type::void())
    }

    fn check_builtin_exit(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 1, "exit", loc) {
            self.builtin_check_int(&mut args[0], 0, "exit", loc);
        }
        Type::void()
    }

    fn check_builtin_strcat(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 2, "strcat", loc) {
            self.builtin_check_pointer(&mut args[0], 0, "strcat", loc);
            self.builtin_check_pointer(&mut args[1], 1, "strcat", loc);
        }
        Type::pointer_to(Type::char())
    }

    fn check_builtin_atoi(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 1, "atoi", loc) {
            self.builtin_check_pointer(&mut args[0], 0, "atoi", loc);
        }
        Type::int()
    }

    fn check_builtin_fprintf(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() < 2 {
            self.report_error(
                "fprintf 至少需要 2 个参数（文件指针和格式字符串）",
                loc,
                ErrorCode::E3030_PrintfArgCount,
            );
        } else {
            let stream_type = self.resolve_expr_type(&mut args[0]);
            if !stream_type.is_pointer() && !matches!(stream_type.kind(), TypeKind::Int) {
                self.report_error(
                    "fprintf 的第一个参数必须是文件指针或整数",
                    loc,
                    ErrorCode::E3029_BuiltInArgType,
                );
            }
            let fmt_type = self.resolve_expr_type(&mut args[1]);
            if !fmt_type.is_pointer() && !fmt_type.is_array() {
                self.report_error(
                    "fprintf 的第二个参数必须是字符串",
                    loc,
                    ErrorCode::E3031_PrintfFirstArg,
                );
            }
            for (i, arg) in args.iter_mut().enumerate().skip(2) {
                let arg_type = self.resolve_expr_type(arg);
                if !self.is_scalar(&arg_type) && !arg_type.is_pointer() && !arg_type.is_array() {
                    self.report_error(
                        &format!("fprintf 的第 {} 个参数必须是 int、float、char 或指针", i + 1),
                        loc,
                        ErrorCode::E3032_PrintfArgType,
                    );
                }
            }
        }
        Type::void()
    }

    fn check_builtin_realloc(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 2 {
            self.report_error("realloc 需要两个参数", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            let ptr_type = self.resolve_expr_type(&mut args[0]);
            if !ptr_type.is_pointer() && !matches!(ptr_type.kind(), TypeKind::Int) {
                self.report_error(
                    "realloc 第一个参数必须是指针",
                    loc,
                    ErrorCode::E3029_BuiltInArgType,
                );
            }
            let size_type = self.resolve_expr_type(&mut args[1]);
            if !self.check_assignable(&Type::int(), &size_type, loc) {
                self.report_error(
                    "realloc 第二个参数必须是 int",
                    loc,
                    ErrorCode::E3029_BuiltInArgType,
                );
            } else {
                insert_implicit_cast(&mut args[1], &Type::int());
            }
        }
        Type::pointer_to(Type::void())
    }

    fn check_builtin_qsort(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 4 {
            self.report_error("qsort 需要四个参数", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            let base_type = self.resolve_expr_type(&mut args[0]);
            if !base_type.is_pointer() && !base_type.is_array() {
                self.report_error(
                    "qsort 第一个参数必须是指针或数组",
                    loc,
                    ErrorCode::E3029_BuiltInArgType,
                );
            }
            for i in 1..3 {
                let t = self.resolve_expr_type(&mut args[i]);
                if !self.check_assignable(&Type::int(), &t, loc) {
                    self.report_error(
                        &format!("qsort 第 {} 个参数必须是 int", i + 1),
                        loc,
                        ErrorCode::E3029_BuiltInArgType,
                    );
                } else {
                    insert_implicit_cast(&mut args[i], &Type::int());
                }
            }
            let compar_type = self.resolve_expr_type(&mut args[3]);
            if !matches!(compar_type.kind(), TypeKind::Int) && !compar_type.is_pointer() {
                self.report_error(
                    "qsort 第四个参数必须是函数指针",
                    loc,
                    ErrorCode::E3029_BuiltInArgType,
                );
            }
        }
        Type::void()
    }

    fn check_builtin_puts(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 1, "puts", loc) {
            self.builtin_check_pointer(&mut args[0], 0, "puts", loc);
        }
        Type::int()
    }

    fn check_builtin_calloc(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 2 {
            self.report_error("calloc 需要两个参数", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            for i in 0..2 {
                self.builtin_check_int(&mut args[i], i, "calloc", loc);
            }
        }
        Type::pointer_to(Type::void())
    }

    fn check_builtin_bsearch(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 5 {
            self.report_error("bsearch 需要五个参数", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            let key_type = self.resolve_expr_type(&mut args[0]);
            if !key_type.is_pointer() && !key_type.is_array() {
                self.report_error("bsearch 第一个参数必须是指针或数组", loc, ErrorCode::E3029_BuiltInArgType);
            }
            let base_type = self.resolve_expr_type(&mut args[1]);
            if !base_type.is_pointer() && !base_type.is_array() {
                self.report_error("bsearch 第二个参数必须是指针或数组", loc, ErrorCode::E3029_BuiltInArgType);
            }
            for i in 2..4 {
                self.builtin_check_int(&mut args[i], i, "bsearch", loc);
            }
            let compar_type = self.resolve_expr_type(&mut args[4]);
            if !matches!(compar_type.kind(), TypeKind::Int) && !compar_type.is_pointer() {
                self.report_error("bsearch 第五个参数必须是函数指针", loc, ErrorCode::E3029_BuiltInArgType);
            }
        }
        Type::pointer_to(Type::void())
    }

    fn check_builtin_sprintf(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() < 2 {
            self.report_error("sprintf 至少需要两个参数", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            self.builtin_check_pointer(&mut args[0], 0, "sprintf", loc);
            self.builtin_check_pointer(&mut args[1], 1, "sprintf", loc);
        }
        Type::int()
    }

    fn check_builtin_snprintf(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() < 3 {
            self.report_error("snprintf 至少需要三个参数", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            self.builtin_check_pointer(&mut args[0], 0, "snprintf", loc);
            self.builtin_check_int(&mut args[1], 1, "snprintf", loc);
            self.builtin_check_pointer(&mut args[2], 2, "snprintf", loc);
        }
        Type::int()
    }

    fn check_builtin_sscanf(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() < 2 {
            self.report_error("sscanf 至少需要两个参数", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            self.builtin_check_pointer(&mut args[0], 0, "sscanf", loc);
            self.builtin_check_pointer(&mut args[1], 1, "sscanf", loc);
        }
        Type::int()
    }

    fn check_builtin_fopen(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 2 {
            self.report_error("fopen 需要两个参数（路径和模式）", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            for (i, arg) in args.iter_mut().enumerate() {
                let arg_type = self.resolve_expr_type(arg);
                if !arg_type.is_pointer() && !arg_type.is_array() {
                    self.report_error(
                        &format!("fopen 第 {} 个参数必须是字符串", i + 1),
                        loc,
                        ErrorCode::E3029_BuiltInArgType,
                    );
                }
            }
        }
        Type::pointer_to(Type::void())
    }

    fn check_builtin_fread(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 4 {
            self.report_error("fread 需要四个参数（缓冲区、元素大小、元素数量、文件指针）", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            let buf_type = self.resolve_expr_type(&mut args[0]);
            if !buf_type.is_pointer() && !buf_type.is_array() {
                self.report_error("fread 第一个参数必须是指针或数组", loc, ErrorCode::E3029_BuiltInArgType);
            }
            for i in 1..3 {
                let t = self.resolve_expr_type(&mut args[i]);
                if !self.check_assignable(&Type::int(), &t, loc) {
                    self.report_error(&format!("fread 第 {} 个参数必须是 int", i + 1), loc, ErrorCode::E3029_BuiltInArgType);
                } else {
                    insert_implicit_cast(&mut args[i], &Type::int());
                }
            }
            let stream_type = self.resolve_expr_type(&mut args[3]);
            if !stream_type.is_pointer() && !matches!(stream_type.kind(), TypeKind::Int) {
                self.report_error("fread 第四个参数必须是文件指针", loc, ErrorCode::E3029_BuiltInArgType);
            }
        }
        Type::int()
    }

    fn check_builtin_fwrite(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 4 {
            self.report_error("fwrite 需要四个参数（缓冲区、元素大小、元素数量、文件指针）", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            let buf_type = self.resolve_expr_type(&mut args[0]);
            if !buf_type.is_pointer() && !buf_type.is_array() {
                self.report_error("fwrite 第一个参数必须是指针或数组", loc, ErrorCode::E3029_BuiltInArgType);
            }
            for i in 1..3 {
                let t = self.resolve_expr_type(&mut args[i]);
                if !self.check_assignable(&Type::int(), &t, loc) {
                    self.report_error(&format!("fwrite 第 {} 个参数必须是 int", i + 1), loc, ErrorCode::E3029_BuiltInArgType);
                } else {
                    insert_implicit_cast(&mut args[i], &Type::int());
                }
            }
            let stream_type = self.resolve_expr_type(&mut args[3]);
            if !stream_type.is_pointer() && !matches!(stream_type.kind(), TypeKind::Int) {
                self.report_error("fwrite 第四个参数必须是文件指针", loc, ErrorCode::E3029_BuiltInArgType);
            }
        }
        Type::int()
    }

    fn check_builtin_fclose(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 1, "fclose", loc) {
            let stream_type = self.resolve_expr_type(&mut args[0]);
            if !stream_type.is_pointer() && !matches!(stream_type.kind(), TypeKind::Int) {
                self.report_error("fclose 参数必须是文件指针", loc, ErrorCode::E3029_BuiltInArgType);
            }
        }
        Type::int()
    }

    fn check_builtin_feof(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 1, "feof", loc) {
            let stream_type = self.resolve_expr_type(&mut args[0]);
            if !stream_type.is_pointer() && !matches!(stream_type.kind(), TypeKind::Int) {
                self.report_error("feof 参数必须是文件指针", loc, ErrorCode::E3029_BuiltInArgType);
            }
        }
        Type::int()
    }

    fn check_builtin_fgets(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 3, "fgets", loc) {
            let buf_type = self.resolve_expr_type(&mut args[0]);
            if !buf_type.is_pointer() && !buf_type.is_array() {
                self.report_error("fgets 第一个参数必须是指针或数组", loc, ErrorCode::E3029_BuiltInArgType);
            }
            let n_type = self.resolve_expr_type(&mut args[1]);
            if !self.check_assignable(&Type::int(), &n_type, loc) {
                self.report_error("fgets 第二个参数必须是 int", loc, ErrorCode::E3029_BuiltInArgType);
            } else {
                insert_implicit_cast(&mut args[1], &Type::int());
            }
            let stream_type = self.resolve_expr_type(&mut args[2]);
            if !stream_type.is_pointer() && !matches!(stream_type.kind(), TypeKind::Int) {
                self.report_error("fgets 第三个参数必须是文件指针", loc, ErrorCode::E3029_BuiltInArgType);
            }
        }
        Type::pointer_to(Type::char())
    }

    fn check_builtin_fputs(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 2, "fputs", loc) {
            let s_type = self.resolve_expr_type(&mut args[0]);
            if !s_type.is_pointer() && !s_type.is_array() {
                self.report_error("fputs 第一个参数必须是字符串", loc, ErrorCode::E3029_BuiltInArgType);
            }
            let stream_type = self.resolve_expr_type(&mut args[1]);
            if !stream_type.is_pointer() && !matches!(stream_type.kind(), TypeKind::Int) {
                self.report_error("fputs 第二个参数必须是文件指针", loc, ErrorCode::E3029_BuiltInArgType);
            }
        }
        Type::int()
    }

    fn check_builtin_fgetc(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 1, "fgetc", loc) {
            let stream_type = self.resolve_expr_type(&mut args[0]);
            if !stream_type.is_pointer() && !matches!(stream_type.kind(), TypeKind::Int) {
                self.report_error("fgetc 参数必须是文件指针", loc, ErrorCode::E3029_BuiltInArgType);
            }
        }
        Type::int()
    }

    fn check_builtin_fputc(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 2 {
            self.report_error("fputc 需要两个参数（字符和文件指针）", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            let c_type = self.resolve_expr_type(&mut args[0]);
            if !self.check_assignable(&Type::int(), &c_type, loc) {
                self.report_error("fputc 第一个参数必须是 int", loc, ErrorCode::E3029_BuiltInArgType);
            } else {
                insert_implicit_cast(&mut args[0], &Type::int());
            }
            let stream_type = self.resolve_expr_type(&mut args[1]);
            if !stream_type.is_pointer() && !matches!(stream_type.kind(), TypeKind::Int) {
                self.report_error("fputc 第二个参数必须是文件指针", loc, ErrorCode::E3029_BuiltInArgType);
            }
        }
        Type::int()
    }

    fn check_builtin_fseek(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 3 {
            self.report_error("fseek 需要三个参数（文件指针、偏移量、起始位置）", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            let stream_type = self.resolve_expr_type(&mut args[0]);
            if !stream_type.is_pointer() && !matches!(stream_type.kind(), TypeKind::Int) {
                self.report_error("fseek 第一个参数必须是文件指针", loc, ErrorCode::E3029_BuiltInArgType);
            }
            for i in 1..3 {
                let t = self.resolve_expr_type(&mut args[i]);
                if !self.check_assignable(&Type::int(), &t, loc) {
                    self.report_error(&format!("fseek 第 {} 个参数必须是 int", i + 1), loc, ErrorCode::E3029_BuiltInArgType);
                } else {
                    insert_implicit_cast(&mut args[i], &Type::int());
                }
            }
        }
        Type::int()
    }

    fn check_builtin_ftell(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 1, "ftell", loc) {
            let stream_type = self.resolve_expr_type(&mut args[0]);
            if !stream_type.is_pointer() && !matches!(stream_type.kind(), TypeKind::Int) {
                self.report_error("ftell 参数必须是文件指针", loc, ErrorCode::E3029_BuiltInArgType);
            }
        }
        Type::int()
    }

    fn check_builtin_rewind(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 1, "rewind", loc) {
            let stream_type = self.resolve_expr_type(&mut args[0]);
            if !stream_type.is_pointer() && !matches!(stream_type.kind(), TypeKind::Int) {
                self.report_error("rewind 参数必须是文件指针", loc, ErrorCode::E3029_BuiltInArgType);
            }
        }
        Type::void()
    }

    fn check_builtin_strncat(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 3 {
            self.report_error("strncat 需要三个参数", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            self.builtin_check_pointer(&mut args[0], 0, "strncat", loc);
            self.builtin_check_pointer(&mut args[1], 1, "strncat", loc);
            self.builtin_check_int(&mut args[2], 2, "strncat", loc);
        }
        Type::pointer_to(Type::char())
    }

    fn check_builtin_strncmp(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 3 {
            self.report_error("strncmp 需要三个参数", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            self.builtin_check_pointer(&mut args[0], 0, "strncmp", loc);
            self.builtin_check_pointer(&mut args[1], 1, "strncmp", loc);
            self.builtin_check_int(&mut args[2], 2, "strncmp", loc);
        }
        Type::int()
    }

    fn check_builtin_memcmp(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 3 {
            self.report_error("memcmp 需要三个参数", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            self.builtin_check_pointer(&mut args[0], 0, "memcmp", loc);
            self.builtin_check_pointer(&mut args[1], 1, "memcmp", loc);
            self.builtin_check_int(&mut args[2], 2, "memcmp", loc);
        }
        Type::int()
    }

    fn check_builtin_strchr(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 2 {
            self.report_error("strchr 需要两个参数", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            self.builtin_check_pointer(&mut args[0], 0, "strchr", loc);
            self.builtin_check_int(&mut args[1], 1, "strchr", loc);
        }
        Type::pointer_to(Type::char())
    }

    fn check_builtin_strrchr(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 2 {
            self.report_error("strrchr 需要两个参数", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            self.builtin_check_pointer(&mut args[0], 0, "strrchr", loc);
            self.builtin_check_int(&mut args[1], 1, "strrchr", loc);
        }
        Type::pointer_to(Type::char())
    }

    fn check_builtin_strstr(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 2 {
            self.report_error("strstr 需要两个参数", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            self.builtin_check_pointer(&mut args[0], 0, "strstr", loc);
            self.builtin_check_pointer(&mut args[1], 1, "strstr", loc);
        }
        Type::pointer_to(Type::char())
    }

    fn check_builtin_memchr(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 3 {
            self.report_error("memchr 需要三个参数", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            self.builtin_check_pointer(&mut args[0], 0, "memchr", loc);
            self.builtin_check_int(&mut args[1], 1, "memchr", loc);
            self.builtin_check_int(&mut args[2], 2, "memchr", loc);
        }
        Type::pointer_to(Type::void())
    }

    fn check_builtin_atof(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 1, "atof", loc) {
            self.builtin_check_pointer(&mut args[0], 0, "atof", loc);
        }
        Type::double()
    }
    fn check_builtin_atol(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 1, "atol", loc) {
            self.builtin_check_pointer(&mut args[0], 0, "atol", loc);
        }
        Type::long_long()
    }

    fn check_builtin_tan(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 1, "tan", loc) {
            let t = self.resolve_expr_type(&mut args[0]);
            if !self.check_assignable(&Type::double(), &t, loc) {
                self.report_error("tan 参数必须是 double", loc, ErrorCode::E3029_BuiltInArgType);
            } else {
                insert_implicit_cast(&mut args[0], &Type::double());
            }
        }
        Type::double()
    }

    fn check_builtin_log10(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 1, "log10", loc) {
            let t = self.resolve_expr_type(&mut args[0]);
            if !self.check_assignable(&Type::double(), &t, loc) {
                self.report_error("log10 参数必须是 double", loc, ErrorCode::E3029_BuiltInArgType);
            } else {
                insert_implicit_cast(&mut args[0], &Type::double());
            }
        }
        Type::double()
    }

    fn check_builtin_fabs(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 1, "fabs", loc) {
            let t = self.resolve_expr_type(&mut args[0]);
            if !self.check_assignable(&Type::double(), &t, loc) {
                self.report_error("fabs 参数必须是 double", loc, ErrorCode::E3029_BuiltInArgType);
            } else {
                insert_implicit_cast(&mut args[0], &Type::double());
            }
        }
        Type::double()
    }

    fn check_builtin_ceil(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 1, "ceil", loc) {
            let t = self.resolve_expr_type(&mut args[0]);
            if !self.check_assignable(&Type::double(), &t, loc) {
                self.report_error("ceil 参数必须是 double", loc, ErrorCode::E3029_BuiltInArgType);
            } else {
                insert_implicit_cast(&mut args[0], &Type::double());
            }
        }
        Type::double()
    }

    fn check_builtin_floor(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 1, "floor", loc) {
            let t = self.resolve_expr_type(&mut args[0]);
            if !self.check_assignable(&Type::double(), &t, loc) {
                self.report_error("floor 参数必须是 double", loc, ErrorCode::E3029_BuiltInArgType);
            } else {
                insert_implicit_cast(&mut args[0], &Type::double());
            }
        }
        Type::double()
    }

    fn check_builtin_round(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 1, "round", loc) {
            let t = self.resolve_expr_type(&mut args[0]);
            if !self.check_assignable(&Type::double(), &t, loc) {
                self.report_error("round 参数必须是 double", loc, ErrorCode::E3029_BuiltInArgType);
            } else {
                insert_implicit_cast(&mut args[0], &Type::double());
            }
        }
        Type::double()
    }

    fn check_builtin_fmod(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 2 {
            self.report_error("fmod 需要两个参数", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            for i in 0..2 {
                let t = self.resolve_expr_type(&mut args[i]);
                if !self.check_assignable(&Type::double(), &t, loc) {
                    self.report_error(&format!("fmod 第 {} 个参数必须是 double", i + 1), loc, ErrorCode::E3029_BuiltInArgType);
                } else {
                    insert_implicit_cast(&mut args[i], &Type::double());
                }
            }
        }
        Type::double()
    }
}
