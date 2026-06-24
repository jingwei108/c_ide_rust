use cide_ast::*;

use super::super::{AccessSpec, MethodSig, TypeChecker};

impl TypeChecker {
    /// 将类外方法定义（`void Foo::bar() { ... }`、`Foo::Foo() { ... }`）合并到对应
    /// ClassDecl 的成员声明中，避免产生重复的函数符号，同时让方法体能访问类字段。
    pub(crate) fn merge_out_of_line_method_definitions(&mut self, program: &mut ProgramNode) {
        let mut merged_indices = std::collections::HashSet::new();
        for (i, f) in program.funcs.iter().enumerate() {
            if f.body.is_none() {
                continue;
            }

            // 普通方法: ClassName__methodName(...)
            if let Some((class_name, method_name)) = f.name.split_once("__") {
                if !class_name.is_empty() && !method_name.is_empty() && !method_name.starts_with("ctor__") {
                    if let Some(c) = program.classes.iter_mut().find(|c| c.name == class_name) {
                        for member in &mut c.members {
                            if let ClassMember::Method { name, body, .. } = member {
                                if name == method_name && body.is_none() {
                                    *body = f.body.clone();
                                    merged_indices.insert(i);
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            // 构造函数: __ctor__ClassName 或 __ctor__ClassName__N
            if f.name.starts_with("__ctor__") {
                let rest = &f.name["__ctor__".len()..];
                let class_name = rest.split("__").next().unwrap_or(rest);
                if let Some(c) = program.classes.iter_mut().find(|c| c.name == class_name) {
                    for member in &mut c.members {
                        if let ClassMember::Constructor { body, .. } = member {
                            if body.is_none() {
                                *body = f.body.clone();
                                merged_indices.insert(i);
                                break;
                            }
                        }
                    }
                }
            }
        }

        program.funcs = program
            .funcs
            .drain(..)
            .enumerate()
            .filter(|(i, _)| !merged_indices.contains(i))
            .map(|(_, f)| f)
            .collect();
    }

    pub(crate) fn get_class_field_type_with_access(
        &self,
        class_name: &str,
        field_name: &str,
    ) -> (Option<Type>, Option<AccessSpec>) {
        let sym = match self.classes.get(class_name) {
            Some(s) => s,
            None => return (None, None),
        };
        for (fty, fname, faccess) in &sym.fields {
            if fname == field_name {
                return (Some(fty.clone()), Some(*faccess));
            }
        }
        for (fty, fname, faccess) in &sym.static_fields {
            if fname == field_name {
                return (Some(fty.clone()), Some(*faccess));
            }
        }
        (None, None)
    }

    pub(crate) fn find_class_method_sigs(&self, class_name: &str, method_name: &str) -> Option<Vec<MethodSig>> {
        let sym = self.classes.get(class_name)?;
        sym.methods.get(method_name).cloned()
    }

    /// Resolve a non-constructor method overload from the given class.
    /// Returns the matching signature and the mangled function name to call.
    pub(crate) fn resolve_method_overload(
        &self,
        class_name: &str,
        method_name: &str,
        arg_types: &[Type],
    ) -> Option<(MethodSig, String)> {
        let sigs = self.find_class_method_sigs(class_name, method_name)?;
        let mut best: Option<(MethodSig, usize)> = None;
        for sig in &sigs {
            if sig.param_types.len() != arg_types.len() {
                continue;
            }
            let mut score = 0usize;
            let mut ok = true;
            for (param, arg) in sig.param_types.iter().zip(arg_types.iter()) {
                let s = self.overload_match_score(param, arg);
                if s == 0 {
                    ok = false;
                    break;
                }
                score += s;
            }
            if !ok {
                continue;
            }
            match &best {
                None => best = Some((sig.clone(), score)),
                Some((_, cur)) if score > *cur => best = Some((sig.clone(), score)),
                _ => {}
            }
        }
        best.map(|(sig, _)| {
            let mangled = if sigs.len() <= 1 {
                format!("{}__{}", class_name, method_name)
            } else {
                format!("{}__{}__{}", class_name, method_name, sig.param_types.len())
            };
            (sig, mangled)
        })
    }

    /// Non-reporting compatibility score for overload resolution.
    fn overload_match_score(&self, param: &Type, arg: &Type) -> usize {
        if param == arg {
            return 3;
        }
        // Reference binding
        if let Type::Reference { base, .. } = param {
            if base.as_ref() == arg {
                return 3;
            }
            if base.kind() == arg.kind()
                && matches!(
                    base.kind(),
                    TypeKind::Int | TypeKind::Char | TypeKind::Float | TypeKind::Double | TypeKind::LongLong
                )
            {
                return 2;
            }
            if let Type::Pointer { pointee: pb, .. } = base.as_ref() {
                if let Type::Pointer { pointee: pa, .. } = arg {
                    if pb == pa || matches!(pb.as_ref(), Type::Void { .. }) {
                        return 2;
                    }
                }
            }
            return 0;
        }
        // Numeric promotion / conversion
        if matches!(
            param.kind(),
            TypeKind::Int | TypeKind::Char | TypeKind::Float | TypeKind::Double | TypeKind::LongLong
        ) && matches!(
            arg.kind(),
            TypeKind::Int | TypeKind::Char | TypeKind::Float | TypeKind::Double | TypeKind::LongLong
        ) {
            return 2;
        }
        // Pointer / array compatibility
        if param.is_pointer() && (arg.is_pointer() || arg.is_array()) {
            return 2;
        }
        // Class type match
        if param.is_class() && arg.is_class() && param.name() == arg.name() {
            return 3;
        }
        // RValue reference binding to class
        if let Type::RValueRef { base } = param {
            if arg.is_class() && base.as_ref() == arg {
                return 2;
            }
        }
        0
    }
}
