use super::*;

impl TypeChecker {
    /// Pass 1.5: Register classes and compute layouts (fields, methods, vtables, sizes).
    pub(crate) fn register_class_layouts(&mut self, program: &mut ProgramNode) {
        for c in &program.classes {
            if self.classes.contains_key(&c.name) || self.structs.contains_key(&c.name) {
                self.report_error(
                    &format!("类 '{}' 重复定义", c.name),
                    &c.loc,
                    ErrorCode::E3002_StructRedeclared,
                );
                continue;
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
            let mut methods: HashMap<String, MethodSig> = HashMap::new();
            let mut vtable_entries: Vec<(String, Type)> = Vec::new();

            // Inherit base fields
            if let Some(ref base_name) = c.base {
                if let Some(base_sym) = self.classes.get(base_name) {
                    for (fty, fname, faccess) in &base_sym.fields {
                        fields.push((fty.clone(), fname.clone(), *faccess));
                    }
                    for (mname, msig) in &base_sym.methods {
                        methods.insert(mname.clone(), msig.clone());
                        if msig.is_virtual {
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

            // Class members already have access set by parser
            for member in &c.members {
                match member {
                    ClassMember::Field { name, ty, access } => {
                        fields.push((ty.clone(), name.clone(), *access));
                    }
                    ClassMember::Method {
                        name,
                        ret,
                        params,
                        is_virtual,
                        access,
                        ..
                    } => {
                        let acc = *access;
                        let param_types: Vec<Type> = params.iter().map(|p| p.ty.clone()).collect();
                        let sig = MethodSig {
                            ret: ret.clone(),
                            param_types: param_types.clone(),
                            is_virtual: *is_virtual,
                            is_static: false,
                            access: acc,
                        };
                        methods.insert(name.clone(), sig);
                        if *is_virtual {
                            let func_ty = Type::Function {
                                return_type: Box::new(ret.clone()),
                                param_types,
                                is_const: false,
                            };
                            // Override check: if base has same virtual method, replace
                            if let Some(pos) = vtable_entries.iter().position(|(n, _)| n == name) {
                                vtable_entries[pos] = (name.clone(), func_ty);
                            } else {
                                vtable_entries.push((name.clone(), func_ty));
                            }
                        }
                    }
                    ClassMember::Constructor { params, access, .. } => {
                        let acc = *access;
                        let param_types: Vec<Type> = params.iter().map(|p| p.ty.clone()).collect();
                        let sig = MethodSig {
                            ret: Type::void(),
                            param_types,
                            is_virtual: false,
                            is_static: false,
                            access: acc,
                        };
                        methods.insert(format!("__ctor__{}", c.name), sig);
                    }
                    ClassMember::Destructor { access, .. } => {
                        let acc = *access;
                        let sig = MethodSig {
                            ret: Type::void(),
                            param_types: vec![Type::Pointer {
                                pointee: Box::new(Type::Class {
                                    name: c.name.clone(),
                                    is_const: false,
                                }),
                                is_const: false,
                            }],
                            is_virtual: false,
                            is_static: false,
                            access: acc,
                        };
                        methods.insert(format!("__dtor__{}", c.name), sig);
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
                methods,
                base: c.base.clone(),
                vtable,
                size: 0,
            };
            self.classes.insert(c.name.clone(), sym);
            // Now compute actual size
            let total_field_size: i32 = fields.iter().map(|(ty, _, _)| self.compute_type_size(ty)).sum();
            self.classes.get_mut(&c.name).unwrap().size = total_field_size;
        }

        // Write vtables back to program.classes for BytecodeGen
        for c in &mut program.classes {
            if let Some(sym) = self.classes.get(&c.name) {
                c.vtable = sym.vtable.clone();
            }
        }

        // Register builtin container layouts so TypeChecker knows their sizes
        let container_mappings = [
            ("vector<int>", "cide_vec_int"),
            ("vector<float>", "cide_vec_float"),
            ("vector<char>", "cide_vec_char"),
            ("string", "cide_string"),
            ("list<int>", "cide_list_int"),
        ];
        for (cpp_name, cide_name) in &container_mappings {
            if let Some(layout) = crate::compiler::cpp_frontend::builtin_layout::builtin_class_layout(cide_name) {
                for name in [*cpp_name, *cide_name] {
                    if self.classes.contains_key(name) {
                        continue;
                    }
                    let fields: Vec<(Type, String, AccessSpec)> = layout
                        .fields
                        .iter()
                        .map(|(n, t)| (t.clone(), n.clone(), AccessSpec::Public))
                        .collect();
                    let methods: HashMap<String, MethodSig> = layout
                        .methods
                        .iter()
                        .map(|m| {
                            (
                                m.name.clone(),
                                MethodSig {
                                    ret: m.ret.clone(),
                                    param_types: m.params.clone(),
                                    is_virtual: m.is_virtual,
                                    is_static: false,
                                    access: AccessSpec::Public,
                                },
                            )
                        })
                        .collect();
                    self.classes.insert(
                        name.to_string(),
                        ClassSymbol {
                            fields,
                            methods,
                            base: None,
                            vtable: None,
                            size: layout.size,
                        },
                    );
                }
            }
        }
    }
}
