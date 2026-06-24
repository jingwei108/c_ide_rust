use super::*;

impl TypeChecker {
    pub(crate) fn resolve_literal(&mut self, ty: &mut Type) -> Type {
        if ty.is_unsigned() {
            ty.clone()
        } else {
            Type::int()
        }
    }

    #[allow(clippy::unused_self)]
    pub(crate) fn resolve_float_literal(&mut self) -> Type {
        Type::float()
    }

    #[allow(clippy::unused_self)]
    pub(crate) fn resolve_long_literal(&mut self) -> Type {
        Type::long_long()
    }

    pub(crate) fn resolve_string_literal(&mut self, value: &str, ty: &mut Type) -> Type {
        let array_size = value.len() as i32 + 1;
        *ty = Type::Array {
            element: Box::new(Type::char()),
            array_size,
            dims: vec![array_size],
            is_const: false,
            is_vla: false,
            vla_dims: vec![],
        };
        ty.clone()
    }

    pub(crate) fn resolve_init_list(&mut self, elements: &mut [InitElement], ty: &mut Type) -> Type {
        for elem in elements.iter_mut() {
            self.resolve_expr_type(&mut elem.value);
        }
        *ty = Type::void();
        ty.clone()
    }
}
