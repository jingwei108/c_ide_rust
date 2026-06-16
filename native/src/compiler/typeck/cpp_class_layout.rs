use super::*;

impl TypeChecker {
    /// Pass 1.5: Register classes and compute layouts (fields, methods, vtables, sizes).
    pub(crate) fn register_class_layouts(&mut self, program: &mut ProgramNode) {
        for c in &program.classes {
            self.register_single_class_layout(&c.name, c);
        }

        // Second pass: compute has_resource for all registered classes (including builtins)
        let class_names: Vec<String> = self.classes.keys().cloned().collect();
        for name in class_names {
            let has_resource = self.compute_class_has_resource(&name);
            if let Some(sym) = self.classes.get_mut(&name) {
                sym.has_resource = has_resource;
            }
        }

        // Write vtables back to program.classes for BytecodeGen
        for c in &mut program.classes {
            if let Some(sym) = self.classes.get(&c.name) {
                c.vtable = sym.vtable.clone();
            }
        }
    }

    /// Check whether a type contains resources (pointers, references, or class fields
    /// that themselves contain resources).
    fn type_contains_resource(&self, ty: &Type) -> bool {
        match ty {
            Type::Pointer { .. } | Type::Reference { .. } | Type::RValueRef { .. } => true,
            Type::Class { name, .. } => self.compute_class_has_resource(name),
            Type::Array { element, .. } => self.type_contains_resource(element),
            _ => false,
        }
    }

    /// Compute whether a class (by name) contains resource fields.
    fn compute_class_has_resource(&self, name: &str) -> bool {
        let sym = match self.classes.get(name) {
            Some(s) => s,
            None => return false,
        };
        // Check base class
        if let Some(ref base_name) = sym.base {
            if self.compute_class_has_resource(base_name) {
                return true;
            }
        }
        for (ty, _, _) in &sym.fields {
            if self.type_contains_resource(ty) {
                return true;
            }
        }
        false
    }

    pub(crate) fn register_single_class_layout(&mut self, name: &str, c: &ClassDecl) {
        if self.classes.contains_key(name) || self.structs.contains_key(name) {
            self.report_error(&format!("类 '{}' 重复定义", name), &c.loc, ErrorCode::E3002_StructRedeclared);
            return;
        }
        // Check base class exists
        if let Some(ref base_name) = c.base {
            if !self.classes.contains_key(base_name) && !self.structs.contains_key(base_name) {
                self.report_error(
                    &format!("基类 '{}' 未定义", base_name),
                    &c.loc,
                    ErrorCode::E4021_BaseClassNotFound,
                );
            }
        }
        let mut fields: Vec<(Type, String, AccessSpec)> = Vec::new();
        let mut static_fields: Vec<(Type, String, AccessSpec)> = Vec::new();
        let mut methods: HashMap<String, Vec<MethodSig>> = HashMap::new();
        let mut vtable_entries: Vec<(String, Type)> = Vec::new();

        // Names declared by this class. Base-class methods with the same name are
        // hidden (C++ name hiding), so we do not inherit them as overloads.
        let mut declared_method_names: HashSet<String> = HashSet::new();
        for member in &c.members {
            match member {
                ClassMember::Method { name, .. } => {
                    declared_method_names.insert(name.clone());
                }
                ClassMember::Constructor { params, .. } => {
                    let key = if params.is_empty() {
                        format!("__ctor__{}", name)
                    } else {
                        format!("__ctor__{}__{}", name, params.len())
                    };
                    declared_method_names.insert(key);
                }
                ClassMember::Destructor { .. } => {
                    declared_method_names.insert(format!("__dtor__{}", name));
                }
                _ => {}
            }
        }

        // Inherit base fields
        if let Some(ref base_name) = c.base {
            if let Some(base_sym) = self.classes.get(base_name) {
                for (fty, fname, faccess) in &base_sym.fields {
                    fields.push((fty.clone(), fname.clone(), *faccess));
                }
                for (mname, msigs) in &base_sym.methods {
                    let hidden = declared_method_names.contains(mname);
                    for msig in msigs {
                        // C++ name hiding: derived declarations hide inherited overloads,
                        // but virtual functions still occupy a vtable slot so that overrides
                        // can be dispatched through a base pointer.
                        if !hidden {
                            methods.entry(mname.clone()).or_default().push(msig.clone());
                        }
                        if msig.is_virtual && !vtable_entries.iter().any(|(n, _)| n == mname) {
                            let func_ty = Type::Function {
                                return_type: Box::new(msig.ret.clone()),
                                param_types: msig.param_types.clone(),
                                is_const: false,
                            };
                            vtable_entries.push((mname.clone(), func_ty));
                        }
                    }
                }
            }
        }

        // Class members already have access set by parser
        for member in &c.members {
            match member {
                ClassMember::Field {
                    name: field_name,
                    ty,
                    access,
                    is_static,
                } => {
                    if *is_static {
                        static_fields.push((ty.clone(), field_name.clone(), *access));
                    } else {
                        fields.push((ty.clone(), field_name.clone(), *access));
                    }
                }
                ClassMember::Method {
                    name: method_name,
                    ret,
                    params,
                    is_virtual,
                    access,
                    is_static,
                    is_const,
                    ..
                } => {
                    let acc = *access;
                    let param_types: Vec<Type> = params.iter().map(|p| p.ty.clone()).collect();
                    // A method that overrides a base virtual function is itself virtual.
                    let overrides_virtual = vtable_entries.iter().any(|(n, _)| n == method_name);
                    let sig = MethodSig {
                        ret: ret.clone(),
                        param_types: param_types.clone(),
                        is_virtual: *is_virtual || overrides_virtual,
                        is_static: *is_static,
                        is_explicit: false,
                        is_const: *is_const,
                        access: acc,
                    };
                    methods.entry(method_name.clone()).or_default().push(sig.clone());
                    if sig.is_virtual {
                        let func_ty = Type::Function {
                            return_type: Box::new(ret.clone()),
                            param_types,
                            is_const: false,
                        };
                        // Override check: if base has same virtual method, replace
                        if let Some(pos) = vtable_entries.iter().position(|(n, _)| n == method_name) {
                            vtable_entries[pos] = (method_name.clone(), func_ty);
                        } else {
                            vtable_entries.push((method_name.clone(), func_ty));
                        }
                    }
                }
                ClassMember::Constructor {
                    params, access, is_explicit, ..
                } => {
                    let acc = *access;
                    let is_exp = *is_explicit;
                    let param_types: Vec<Type> = params.iter().map(|p| p.ty.clone()).collect();
                    let sig = MethodSig {
                        ret: Type::void(),
                        param_types: param_types.clone(),
                        is_virtual: false,
                        is_static: false,
                        is_explicit: is_exp,
                        is_const: false,
                        access: acc,
                    };
                    let ctor_key = if param_types.is_empty() {
                        format!("__ctor__{}", name)
                    } else {
                        format!("__ctor__{}__{}", name, param_types.len())
                    };
                    methods.entry(ctor_key).or_default().push(sig);
                }
                ClassMember::Destructor { access, .. } => {
                    let acc = *access;
                    let sig = MethodSig {
                        ret: Type::void(),
                        param_types: vec![Type::Pointer {
                            pointee: Box::new(Type::Class {
                                name: name.to_string(),
                                is_const: false,
                            }),
                            is_const: false,
                        }],
                        is_virtual: false,
                        is_static: false,
                        is_explicit: false,
                        is_const: false,
                        access: acc,
                    };
                    methods.entry(format!("__dtor__{}", name)).or_default().push(sig);
                }
                ClassMember::NestedStruct { decl, .. } => {
                    let sym = StructSymbol {
                        fields: decl.fields.iter().map(|f| (f.ty.clone(), f.name.clone())).collect(),
                    };
                    self.structs.insert(decl.name.clone(), sym);
                }
                ClassMember::NestedClass { decl, .. } => {
                    self.register_single_class_layout(&decl.name, decl);
                }
            }
        }

        let vtable = if vtable_entries.is_empty() {
            None
        } else {
            Some(VTable { entries: vtable_entries })
        };

        // Insert with size 0 first so compute_type_size can resolve recursive class types
        let sym = ClassSymbol {
            fields: fields.clone(),
            static_fields: static_fields.clone(),
            methods,
            base: c.base.clone(),
            vtable,
            size: 0,
            has_resource: false,
        };
        self.classes.insert(name.to_string(), sym);
        // Now compute actual size (static fields are NOT part of instance)
        let total_field_size: i32 = fields.iter().map(|(ty, _, _)| self.compute_type_size(ty)).sum();
        // Compute has_resource immediately so implicit move ctor generation works
        // for class template instantiations created during Pass 3.
        let has_resource = self.compute_class_has_resource(name);
        // TODO(#D08): 刚 insert 的 key，理论上必存在；可考虑返回错误而非 unwrap。
        #[allow(clippy::unwrap_used)]
        let class_sym = self.classes.get_mut(name).unwrap();
        class_sym.size = total_field_size;
        class_sym.has_resource = has_resource;

        // Register mangled function symbols for methods, constructors, and destructor
        // so that constructor-style initialization and explicit calls can resolve them.
        for member in &c.members {
            match member {
                ClassMember::Method {
                    name: method_name,
                    ret,
                    params,
                    is_const,
                    is_static,
                    ..
                } => {
                    let overload_count = class_sym.methods.get(method_name).map(|v| v.len()).unwrap_or(1);
                    let mangled = if overload_count <= 1 {
                        format!("{}__{}", name, method_name)
                    } else {
                        format!("{}__{}__{}", name, method_name, params.len())
                    };
                    if self.funcs.contains_key(&mangled) || self.static_func_sigs.contains_key(&mangled) {
                        continue;
                    }
                    let param_types: Vec<Type> = if *is_static {
                        params.iter().map(|p| p.ty.clone()).collect()
                    } else {
                        std::iter::once(Type::Pointer {
                            pointee: Box::new(Type::Class {
                                name: name.to_string(),
                                is_const: *is_const,
                            }),
                            is_const: *is_const,
                        })
                        .chain(params.iter().map(|p| p.ty.clone()))
                        .collect()
                    };
                    self.funcs.insert(
                        mangled,
                        FuncSymbol {
                            return_type: ret.clone(),
                            param_types,
                        },
                    );
                }
                ClassMember::Constructor { params, .. } => {
                    let mangled = if params.is_empty() {
                        format!("__ctor__{}", name)
                    } else {
                        format!("__ctor__{}__{}", name, params.len())
                    };
                    if self.funcs.contains_key(&mangled) {
                        continue;
                    }
                    let param_types: Vec<Type> = std::iter::once(Type::Pointer {
                        pointee: Box::new(Type::Class {
                            name: name.to_string(),
                            is_const: false,
                        }),
                        is_const: false,
                    })
                    .chain(params.iter().map(|p| p.ty.clone()))
                    .collect();
                    self.funcs.insert(
                        mangled,
                        FuncSymbol {
                            return_type: Type::void(),
                            param_types,
                        },
                    );
                }
                ClassMember::Destructor { .. } => {
                    let mangled = format!("__dtor__{}", name);
                    if self.funcs.contains_key(&mangled) {
                        continue;
                    }
                    let param_types = vec![Type::Pointer {
                        pointee: Box::new(Type::Class {
                            name: name.to_string(),
                            is_const: false,
                        }),
                        is_const: false,
                    }];
                    self.funcs.insert(
                        mangled,
                        FuncSymbol {
                            return_type: Type::void(),
                            param_types,
                        },
                    );
                }
                _ => {}
            }
        }

        // Register an implicit default constructor for classes that do not declare one.
        // This allows `new Derived()` and `Derived d;` to resolve even when the class
        // has no user-defined default constructor.
        let default_ctor_name = format!("__ctor__{}", name);
        if class_sym.methods.get(&default_ctor_name).map(|v| v.is_empty()).unwrap_or(true) {
            class_sym.methods.entry(default_ctor_name.clone()).or_default().push(MethodSig {
                ret: Type::void(),
                param_types: vec![],
                is_virtual: false,
                is_static: false,
                is_explicit: false,
                is_const: false,
                access: AccessSpec::Public,
            });
            self.funcs.entry(default_ctor_name).or_insert_with(|| FuncSymbol {
                return_type: Type::void(),
                param_types: vec![Type::Pointer {
                    pointee: Box::new(Type::Class {
                        name: name.to_string(),
                        is_const: false,
                    }),
                    is_const: false,
                }],
            });
        }

        // Register builtin container layouts so TypeChecker knows their sizes.
        // In library mode we are compiling the builtin container implementations themselves,
        // so their ClassDecls are already present in program.classes; do not re-register them.
        if self.is_library_mode {
            return;
        }
        for (cpp_name, cide_name) in crate::compiler::cpp_frontend::builtin_layout::builtin_class_mappings() {
            if let Some(layout) = crate::compiler::cpp_frontend::builtin_layout::builtin_class_layout(cide_name) {
                for name in [cpp_name, cide_name] {
                    if self.classes.contains_key(name) {
                        continue;
                    }
                    let fields: Vec<(Type, String, AccessSpec)> = layout
                        .fields
                        .iter()
                        .map(|(n, t)| (t.clone(), n.clone(), AccessSpec::Public))
                        .collect();
                    let mut methods: HashMap<String, Vec<MethodSig>> = HashMap::new();
                    for m in &layout.methods {
                        methods.entry(m.name.clone()).or_default().push(MethodSig {
                            ret: m.ret.clone(),
                            param_types: m.params.clone(),
                            is_virtual: m.is_virtual,
                            is_static: false,
                            is_explicit: false,
                            is_const: false,
                            access: AccessSpec::Public,
                        });
                    }
                    self.classes.insert(
                        name.to_string(),
                        ClassSymbol {
                            fields,
                            static_fields: vec![],
                            methods,
                            base: None,
                            vtable: None,
                            size: layout.size,
                            has_resource: false,
                        },
                    );
                }
            }
        }
    }
}
