//! Type checker for the Dahlia language
//!

use std::collections::HashMap;

use crate::{
    ast::{
        Expr, Literal,
        operator::{BinaryOp, UnaryOp},
        types::{Type, TypeId},
    },
    middle::typesubst::{TypeScheme, TypeSubst, UnifyError, bind_var, deref, ftv},
};

// Type environment mapping variable names -> type schemes
pub type TypeEnv = HashMap<String, TypeScheme>;

#[derive(Debug)]
pub enum TypeError {
    UnknownName(String),
    NotAFunction(Type),
    NotAPointer(Type),

    ReturnOutsideFunction,
    BreakOutsideLoop,

    // Unification errors
    Unify(UnifyError),
}

impl From<UnifyError> for TypeError {
    fn from(err: UnifyError) -> Self {
        TypeError::Unify(err)
    }
}

#[derive(Debug, Clone)]
pub struct Typecheck {
    subst: TypeSubst,
    env: TypeEnv,
    var_count: usize,
    curr_return_type: Option<Type>,
    loop_depth: usize,
}

impl Typecheck {
    pub fn check_expr(&mut self, expr: &Expr) -> Result<(), TypeError> {
        todo!()
    }

    // Inference
    // ----------------

    pub fn infer_literal(&mut self, lit: &Literal) -> Result<Type, TypeError> {
        match lit {
            Literal::Int(_) => Ok(Type::ISize),
            Literal::UInt(_) => Ok(Type::USize),
            Literal::Float(_) => Ok(Type::F64),
            Literal::Bool(_) => Ok(Type::Bool),
            Literal::String(_) => Ok(Type::Str),
            Literal::Char(_) => Ok(Type::Char),
            Literal::Array {
                elements,
                element_type,
            } => Ok(Type::Array {
                element_type: Box::new(self.infer_expr(&elements[0])?),
                size: elements.len(),
            }),
        }
    }

    pub fn infer_unary(&mut self, op: &UnaryOp, expr: &Expr) -> Result<Type, TypeError> {
        let expr_type = self.infer_expr(expr)?;

        match op {
            UnaryOp::Negate => {
                if expr_type.is_signed() {
                    Ok(expr_type)
                } else {
                    Err(TypeError::Unify(UnifyError::Mismatch {
                        expected: Type::I32,
                        found: expr_type,
                    }))
                }
            }
            UnaryOp::Not => {
                if expr_type == Type::Bool {
                    Ok(Type::Bool)
                } else {
                    Err(TypeError::Unify(UnifyError::Mismatch {
                        expected: Type::Bool,
                        found: expr_type,
                    }))
                }
            }
            UnaryOp::Deref => match expr_type {
                Type::Pointer(inner) => Ok(*inner),
                _ => Err(TypeError::NotAPointer(expr_type)),
            },
            UnaryOp::Ref => Ok(Type::Pointer(Box::new(expr_type))),
        }
    }

    pub fn infer_binary(
        &mut self,
        left: &Expr,
        op: &BinaryOp,
        right: &Expr,
    ) -> Result<Type, TypeError> {
        let left_type = self.infer_expr(left)?;
        let right_type = self.infer_expr(right)?;

        todo!()
    }

    pub fn infer_expr(&mut self, expr: &Expr) -> Result<Type, TypeError> {
        match expr {
            Expr::Literal(lit) => self.infer_literal(lit),
            Expr::Identifier(id) => self
                .lookup(&id.name)
                .ok_or(TypeError::UnknownName(id.name.clone())),
            Expr::Dotted(_name) => todo!("Field and enum access not implemented yet"),
            Expr::UnaryOp { op, expr } => self.infer_unary(op, expr),
            Expr::BinaryOp { left, op, right } => self.infer_binary(left, op, right),

            _ => todo!("Function calls, indexing, and pointer dereference not implemented yet"),
        }
    }

    // --------------------------------
}

impl Typecheck {
    pub fn new() -> Self {
        Typecheck {
            subst: TypeSubst::new(),
            env: HashMap::new(),
            var_count: 0,
            curr_return_type: None,
            loop_depth: 0,
        }
    }

    pub fn fresh(&mut self) -> Type {
        let id = TypeId(self.var_count);
        self.var_count += 1;
        Type::Var(id)
    }

    pub fn lookup(&mut self, name: &str) -> Option<Type> {
        // Lookup the type scheme for the variable name and instantiate it to get a concrete type
        self.env
            .get(name)
            .cloned()
            .map(|scheme| self.instantiate(&scheme))
    }

    pub fn instantiate(&mut self, scheme: &TypeScheme) -> Type {
        // If the type scheme has no bound variables, return the type as is
        if scheme.bound_vars.is_empty() {
            return scheme.ty.clone();
        }

        let mut map = HashMap::new();
        for var in &scheme.bound_vars {
            // Generate a fresh type variable for each bound variable in the scheme
            let fresh_tv = self.fresh();
            map.insert(var.clone(), fresh_tv);
        }

        self.subst.apply_mapping(&scheme.ty, &map)
    }

    pub fn generalize(&self, ty: Type) -> TypeScheme {
        // Find free variables in the type and environment
        let mut free_in_type = ftv(&ty);

        // Remove any type variables that are free in the environment from the set of free type variables in the type
        free_in_type.retain(|v| !self.subst.0.contains_key(v));

        let mut free_in_env = Vec::new();
        for scheme in self.env.values() {
            free_in_env.extend(ftv(&scheme.ty));
        }

        // Remove any type variables that are free in the environment from the set of free type variables in the type
        free_in_type.retain(|v| !free_in_env.contains(v));

        TypeScheme {
            bound_vars: free_in_type,
            ty,
        }
    }

    pub fn unify(&mut self, t1: &Type, t2: &Type) -> Result<(), UnifyError> {
        let t1 = deref(t1, &self.subst);
        let t2 = deref(t2, &self.subst);

        if t1 == t2 {
            return Ok(());
        }

        match (&t1, &t2) {
            // Primitive types must match exactly
            (t1, t2) if t1.is_primitive() && t2.is_primitive() => {
                if t1 != t2 {
                    return Err(UnifyError::Mismatch {
                        expected: t1.clone(),
                        found: t2.clone(),
                    });
                }
                Ok(())
            }

            // Type variables can be unified with any type, as long as they don't occur in the type
            (Type::Var(v), ty) => bind_var(v.clone(), ty, &mut self.subst),
            (ty, Type::Var(v)) => bind_var(v.clone(), ty, &mut self.subst),

            // Polymorphic types can be unified if their inner types can be unified
            (Type::Forall(_, inner1), Type::Forall(_, inner2)) => self.unify(inner1, inner2),

            // Pointer types can be unified if their inner types can be unified
            (Type::Pointer(p1), Type::Pointer(p2)) => self.unify(p1, p2),

            // Boxed types
            (
                Type::Array {
                    element_type: e1,
                    size: s1,
                },
                Type::Array {
                    element_type: e2,
                    size: s2,
                },
            ) => {
                if s1 != s2 {
                    return Err(UnifyError::ArityMismatch {
                        expected: *s1,
                        found: *s2,
                    });
                }
                self.unify(e1, e2)
            }

            (
                Type::Allocator {
                    name: name1,
                    size: s1,
                },
                Type::Allocator {
                    name: name2,
                    size: s2,
                },
            ) => {
                if name1 != name2 || s1 != s2 {
                    return Err(UnifyError::ArityMismatch {
                        expected: *s1,
                        found: *s2,
                    });
                }
                Ok(())
            }

            (
                Type::Function {
                    param_types: params1,
                    return_type: ret1,
                },
                Type::Function {
                    param_types: params2,
                    return_type: ret2,
                },
            ) => {
                if params1.len() != params2.len() {
                    return Err(UnifyError::ArityMismatch {
                        expected: params1.len(),
                        found: params2.len(),
                    });
                }
                for (p1, p2) in params1.iter().zip(params2.iter()) {
                    self.unify(p1, p2)?;
                }
                self.unify(ret1, ret2)
            }

            // If none of the above cases match, the types cannot be unified
            _ => Err(UnifyError::Mismatch {
                expected: t1,
                found: t2,
            }),
        }
    }
}
