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
                        body: Some(ref b),
                        is_const,
                        is_static,
                        ..
                    } => {
                        self.current_method_is_const = *is_const && !*is_static;
                        let func_params: Vec<Param> = if *is_static {
                            params.to_vec()
                        } else {
                            std::iter::once(Param {
                                name: "this".to_string(),
                                ty: Type::Pointer {
                                    pointee: Box::new(Type::Class {
                                        name: c.name.clone(),
                                        is_const: *is_const,
                                    }),
                                    is_const: *is_const,
                                },
                                loc: c.loc,
                            })
                            .chain(params.iter().cloned())
                            .collect()
                        };
                        let user_param_types: Vec<Type> = params.iter().map(|p| p.ty.clone()).collect();
                        let mangled = self
                            .resolve_method_overload(&c.name, method_name, &user_param_types)
                            .map(|(_, m)| m)
                            .unwrap_or_else(|| format!("{}__{}", c.name, method_name));
                        let mut func_decl = FuncDecl {
                            loc: c.loc,
                            return_type: ret.clone(),
                            name: mangled,
                            params: func_params,
                            body: Some(b.clone()),
                            is_static: *is_static,
                            is_extern: false,
                            source_file: String::new(),
                        };
                        self.visit_func_decl_with_fields(&mut func_decl, &class_fields);
                        self.current_method_is_const = false;
                        class_methods.push(func_decl);
                    }
                    ClassMember::Constructor { params, body: Some(ref b), .. } => {
                        let ctor_name = self.resolve_constructor_overload(&c.name, params.len()).unwrap_or_else(|| {
                            if params.is_empty() {
                                format!("__ctor__{}", c.name)
                            } else {
                                format!("__ctor__{}__{}", c.name, params.len())
                            }
                        });
                        let mut func_decl = FuncDecl {
                            loc: c.loc,
                            return_type: Type::void(),
                            name: ctor_name,
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
                    ClassMember::Destructor { body: Some(ref b), .. } => {
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
                    _ => {}
                }
            }
            self.current_class = None;
        }
        program.funcs.extend(class_methods);
    }

    /// 重载决议：从候选构造函数中选择最佳匹配。
    /// 当前为简化实现：根据参数数量选择构造函数。默认（零参数）构造函数保持
    /// `__ctor__{Class}` 名称；带 N 个参数的构造函数编码为 `__ctor__{Class}__N`。
    /// TODO: 完善为基于类型相似度的优先级排序（移动构造 > 拷贝构造 > 普通构造）。
    pub(crate) fn resolve_constructor_overload(&self, class_name: &str, arg_count: usize) -> Option<String> {
        let sym = self.classes.get(class_name)?;
        let target = if arg_count == 0 {
            format!("__ctor__{}", class_name)
        } else {
            format!("__ctor__{}__{}", class_name, arg_count)
        };
        if sym.methods.get(&target).map(|v| !v.is_empty()).unwrap_or(false) {
            return Some(target);
        }
        // Fallback: scan methods for any ctor with matching user param count
        for (name, sigs) in &sym.methods {
            if name.starts_with("__ctor__") && !name.ends_with("__move") {
                for sig in sigs {
                    if sig.param_types.len() == arg_count {
                        return Some(name.clone());
                    }
                }
            }
        }
        None
    }

    /// Generate implicit move constructors for classes that contain resources
    /// (pointers, references, or class fields with resources) and do not already
    /// have an explicit move constructor.
    pub(crate) fn generate_implicit_move_ctors(&mut self, program: &mut ProgramNode) {
        let mut move_ctors: Vec<(String, FuncDecl)> = Vec::new();

        for (class_name, sym) in &self.classes {
            if !sym.has_resource {
                continue;
            }
            let move_ctor_name = format!("__ctor__{}__move", class_name);
            if self.funcs.contains_key(&move_ctor_name) {
                continue;
            }

            let body = Self::build_implicit_move_ctor_body(class_name, &sym.fields);
            let func_decl = FuncDecl {
                loc: SourceLoc { line: 0, column: 0 },
                return_type: Type::void(),
                name: move_ctor_name.clone(),
                params: vec![
                    Param {
                        name: "this".to_string(),
                        ty: Type::Pointer {
                            pointee: Box::new(Type::Class {
                                name: class_name.clone(),
                                is_const: false,
                            }),
                            is_const: false,
                        },
                        loc: SourceLoc { line: 0, column: 0 },
                    },
                    Param {
                        name: "other".to_string(),
                        ty: Type::RValueRef {
                            base: Box::new(Type::Class {
                                name: class_name.clone(),
                                is_const: false,
                            }),
                        },
                        loc: SourceLoc { line: 0, column: 0 },
                    },
                ],
                body: Some(body),
                is_static: false,
                is_extern: false,
                source_file: String::new(),
            };

            self.funcs.insert(
                move_ctor_name.clone(),
                FuncSymbol {
                    return_type: Type::void(),
                    param_types: func_decl.params.iter().map(|p| p.ty.clone()).collect(),
                },
            );
            move_ctors.push((class_name.clone(), func_decl));
        }

        // Register move ctor signatures in ClassSymbol.methods
        for (class_name, func) in &move_ctors {
            if let Some(sym) = self.classes.get_mut(class_name) {
                sym.methods.entry(func.name.clone()).or_default().push(MethodSig {
                    ret: Type::void(),
                    param_types: func.params.iter().map(|p| p.ty.clone()).collect(),
                    is_virtual: false,
                    is_static: false,
                    is_explicit: false,
                    is_const: false,
                    access: AccessSpec::Public,
                });
            }
        }

        program.funcs.extend(move_ctors.into_iter().map(|(_, f)| f));
    }

    fn build_implicit_move_ctor_body(class_name: &str, fields: &[(Type, String, AccessSpec)]) -> Stmt {
        let loc = SourceLoc { line: 0, column: 0 };
        let this_ty = Type::Pointer {
            pointee: Box::new(Type::Class {
                name: class_name.to_string(),
                is_const: false,
            }),
            is_const: false,
        };
        let other_ty = Type::RValueRef {
            base: Box::new(Type::Class {
                name: class_name.to_string(),
                is_const: false,
            }),
        };

        let mut stmts = Vec::new();
        for (fty, fname, _) in fields {
            // this->field = other.field;
            stmts.push(Stmt::Expr {
                expr: Expr::Assign {
                    left: Box::new(Expr::Member {
                        object: Box::new(Expr::This { loc, ty: this_ty.clone() }),
                        member: fname.clone(),
                        loc,
                        ty: fty.clone(),
                    }),
                    op: AssignOp::Assign,
                    right: Box::new(Expr::Member {
                        object: Box::new(Expr::Identifier {
                            name: "other".to_string(),
                            loc,
                            ty: other_ty.clone(),
                        }),
                        member: fname.clone(),
                        loc,
                        ty: fty.clone(),
                    }),
                    loc,
                    ty: fty.clone(),
                },
                loc,
            });

            // For pointer fields, null out the source to prevent double-free.
            if fty.is_pointer() {
                stmts.push(Stmt::Expr {
                    expr: Expr::Assign {
                        left: Box::new(Expr::Member {
                            object: Box::new(Expr::Identifier {
                                name: "other".to_string(),
                                loc,
                                ty: other_ty.clone(),
                            }),
                            member: fname.clone(),
                            loc,
                            ty: fty.clone(),
                        }),
                        op: AssignOp::Assign,
                        right: Box::new(Expr::Literal { value: 0, loc, ty: Type::int() }),
                        loc,
                        ty: fty.clone(),
                    },
                    loc,
                });
            }
        }

        Stmt::Block { stmts, loc }
    }
}
