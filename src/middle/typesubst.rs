//! Type checker for the Dahlia language
//!

use std::collections::HashMap;

use crate::ast::types::{Type, TypeId};

/// Polymorphic type scheme: ∀a b c . τ
///
/// A type 'τ' that is universally quantified over type variables 'a', 'b', 'c'.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeScheme {
    pub bound_vars: Vec<TypeId>,
    pub ty: Type,
}

impl TypeScheme {
    /// Construct a new polymorphic type from a monomorphic type
    pub fn mono(mono_type: Type) -> Self {
        TypeScheme {
            bound_vars: Vec::new(),
            ty: mono_type,
        }
    }
}

#[derive(Debug)]
pub enum UnifyError {
    OccursCheck(TypeId, Type),
    Mismatch { expected: Type, found: Type },
    ArityMismatch { expected: usize, found: usize },
}

/// Type substitution mapping type variables -> types
#[derive(Debug, Clone, PartialEq)]
pub struct TypeSubst(pub HashMap<TypeId, Type>);

impl TypeSubst {
    pub fn new() -> Self {
        TypeSubst(HashMap::new())
    }

    // Apply a substitution mapping to a type, replacing any type variables with their corresponding types in the mapping
    pub fn apply_mapping(&self, ty: &Type, mapping: &HashMap<TypeId, Type>) -> Type {
        match ty {
            Type::Var(v) => mapping.get(v).cloned().unwrap_or(ty.clone()),
            Type::Pointer(inner) => Type::Pointer(Box::new(self.apply_mapping(inner, mapping))),
            Type::Function {
                param_types,
                return_type,
            } => Type::Function {
                param_types: param_types
                    .iter()
                    .map(|p| self.apply_mapping(p, mapping))
                    .collect(),
                return_type: Box::new(self.apply_mapping(return_type, mapping)),
            },
            Type::Forall(bound_vars, inner) => {
                Type::Forall(bound_vars.clone(), Box::new(*inner.clone()))
            }
            Type::Array { element_type, size } => Type::Array {
                element_type: Box::new(self.apply_mapping(element_type, mapping)),
                size: *size,
            },
            _ => ty.clone(),
        }
    }

    pub fn apply(&self, ty: &Type) -> Type {
        self.apply_mapping(ty, &self.0)
    }
}

// Dereference a type variable to its concrete type in a substitution environment.
//
// We look up a type variable in 'subst' and return the concrete type it maps to, if any.
// If the type is not a variable or is not found in the substitution, it returns the type itself.
pub fn deref(ty: &Type, subst: &TypeSubst) -> Type {
    match ty {
        Type::Var(v) => subst.0.get(v).cloned().unwrap_or(ty.clone()),
        Type::Forall(_ids, _inner) => ty.clone(),
        Type::Pointer(inner) => deref(inner, subst),

        _ => ty.clone(),
    }
}

// Find free type variables in a type T
pub fn ftv(ty: &Type) -> Vec<TypeId> {
    match ty {
        Type::Var(v) => vec![v.clone()],
        Type::Forall(bound_vars, inner) => {
            let mut free = ftv(inner);
            free.retain(|v| !bound_vars.contains(v));
            free
        }
        Type::Pointer(inner) => ftv(inner),
        Type::Array { element_type, .. } => ftv(element_type),
        Type::Function {
            param_types,
            return_type,
        } => {
            let mut free = param_types.iter().flat_map(|p| ftv(p)).collect::<Vec<_>>();
            free.extend(ftv(return_type));
            free
        }
        _ => Vec::new(),
    }
}

// Occurs check. Check if a type variable occurs recursively in a type.
pub fn occurs_check(var: &TypeId, ty: &Type, subst: &TypeSubst) -> bool {
    match ty {
        Type::Var(v) => v == var,
        Type::Forall(_id, inner) => occurs_check(var, inner, subst),
        Type::Pointer(inner) => occurs_check(var, inner, subst),
        Type::Allocator { name: _, size: _ } => false,
        Type::Array {
            element_type,
            size: _,
        } => occurs_check(var, element_type, subst),
        Type::Function {
            param_types,
            return_type,
        } => {
            param_types
                .iter()
                .any(|param_type| occurs_check(var, param_type, subst))
                || occurs_check(var, return_type, subst)
        }
        _ => false,
    }
}

pub fn bind_var(var: TypeId, ty: &Type, subst: &mut TypeSubst) -> Result<(), UnifyError> {
    if let Type::Var(other) = ty {
        if var == *other {
            return Ok(());
        }
    }

    // Occurs check: ensure that the type variable does not occur in the type
    if occurs_check(&var, ty, subst) {
        return Err(UnifyError::OccursCheck(var, ty.clone()));
    }

    // Bind the variable in the substitution environment
    subst.0.insert(var, ty.clone());
    Ok(())
}
