use super::*;

impl TypeChecker {
    /// Pass 3.5: Check class method / constructor / destructor bodies.
    pub(crate) fn check_class_methods(&mut self, program: &mut ProgramNode) {
        let mut class_methods: Vec<FuncDecl> = Vec::new();
        for c in &program.classes {
            self.current_class = Some(c.name.clone());
            // Register class fields as pseudo-variables for method body checking
            let class_fields: Vec<(Type, String)> = if let Some(sym) = self.classes.get(&c.name) {
                sym.fields.iter().map(|(ty, name, _)| (ty.clone(), name.clone())).collect()
            } else {
                vec![]
            };
            for member in &c.members {
                match member {
                    ClassMember::Method {
                        name: method_name,
                        ret,
                        params,
                        body,
                        ..
                    } => {
                        if let Some(ref b) = body {
                            let mut func_decl = FuncDecl {
                                loc: c.loc,
                                return_type: ret.clone(),
                                name: format!("{}__{}", c.name, method_name),
                                params: std::iter::once(Param {
                                    name: "this".to_string(),
                                    ty: Type::Pointer {
                                        pointee: Box::new(Type::Class {
                                            name: c.name.clone(),
                                            is_const: false,
                                        }),
                                        is_const: false,
                                    },
                                    loc: c.loc,
                                })
                                .chain(params.iter().cloned())
                                .collect(),
                                body: Some(b.clone()),
                                is_static: false,
                                is_extern: false,
                                source_file: String::new(),
                            };
                            self.visit_func_decl_with_fields(&mut func_decl, &class_fields);
                            class_methods.push(func_decl);
                        }
                    }
                    ClassMember::Constructor { params, body, .. } => {
                        if let Some(ref b) = body {
                            let mut func_decl = FuncDecl {
                                loc: c.loc,
                                return_type: Type::void(),
                                name: format!("__ctor__{}", c.name),
                                params: std::iter::once(Param {
                                    name: "this".to_string(),
                                    ty: Type::Pointer {
                                        pointee: Box::new(Type::Class {
                                            name: c.name.clone(),
                                            is_const: false,
                                        }),
                                        is_const: false,
                                    },
                                    loc: c.loc,
                                })
                                .chain(params.iter().cloned())
                                .collect(),
                                body: Some(b.clone()),
                                is_static: false,
                                is_extern: false,
                                source_file: String::new(),
                            };
                            self.visit_func_decl_with_fields(&mut func_decl, &class_fields);
                            class_methods.push(func_decl);
                        }
                    }
                    ClassMember::Destructor { body, .. } => {
                        if let Some(ref b) = body {
                            let mut func_decl = FuncDecl {
                                loc: c.loc,
                                return_type: Type::void(),
                                name: format!("__dtor__{}", c.name),
                                params: vec![Param {
                                    name: "this".to_string(),
                                    ty: Type::Pointer {
                                        pointee: Box::new(Type::Class {
                                            name: c.name.clone(),
                                            is_const: false,
                                        }),
                                        is_const: false,
                                    },
                                    loc: c.loc,
                                }],
                                body: Some(b.clone()),
                                is_static: false,
                                is_extern: false,
                                source_file: String::new(),
                            };
                            self.visit_func_decl_with_fields(&mut func_decl, &class_fields);
                            class_methods.push(func_decl);
                        }
                    }
                    _ => {}
                }
            }
            self.current_class = None;
        }
        program.funcs.extend(class_methods);
    }

    /// 重载决议：从候选构造函数中选择最佳匹配。
    /// 当前为简化实现：返回第一个参数数量匹配的构造函数名称。
    /// TODO: 完善为基于类型相似度的优先级排序（移动构造 > 拷贝构造 > 普通构造）。
    #[allow(dead_code)]
    pub(crate) fn resolve_constructor_overload(
        &self,
        class_name: &str,
        arg_types: &[Type],
    ) -> Option<String> {
        let sym = self.classes.get(class_name)?;
        for (name, sig) in &sym.methods {
            if name.starts_with("__ctor__") {
                let user_params = sig.param_types.len().saturating_sub(1);
                if user_params == arg_types.len() {
                    return Some(name.clone());
                }
            }
        }
        None
    }
}
