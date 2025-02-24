use sway_types::Spanned;

use crate::{
    decl_engine::{DeclId, DeclRef},
    error::*,
    language::{ty, CallPath},
    semantic_analysis::TypeCheckContext,
    CompileResult, TypeBinding,
};

pub(crate) fn instantiate_constant_decl(
    ctx: TypeCheckContext,
    const_ref: DeclRef<DeclId<ty::TyConstantDeclaration>>,
    call_path_binding: TypeBinding<CallPath>,
) -> CompileResult<ty::TyExpression> {
    let const_decl = ctx.decl_engine.get_constant(const_ref.id());
    ok(
        ty::TyExpression {
            return_type: const_decl.type_ascription.type_id,
            span: call_path_binding.span(),
            expression: ty::TyExpressionVariant::VariableExpression {
                name: const_decl.call_path.suffix,
                span: call_path_binding.inner.suffix.span(),
                mutability: ty::VariableMutability::Immutable,
                call_path: Some(call_path_binding.inner.to_fullpath(ctx.namespace)),
            },
        },
        vec![],
        vec![],
    )
}
