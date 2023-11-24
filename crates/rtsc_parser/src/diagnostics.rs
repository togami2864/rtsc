use miette::Diagnostic;
use thiserror::Error;

use crate::Span;

#[derive(Error, Debug, Diagnostic)]
#[error("unexpected token `{0}`")]
#[diagnostic()]
pub struct UnexpectedToken(pub char, #[label("unexpected token")] pub Span);

#[derive(Error, Debug, Diagnostic)]
#[error("invalid or unexpected token `{0}`")]
#[diagnostic()]
pub struct InvalidOrUnexpectedToken(pub char, #[label("invalid or unexpected token")] pub Span);

#[derive(Error, Debug, Diagnostic)]
#[error("unexpected number `{0}`")]
#[diagnostic()]
pub struct UnexpectedNumber(pub char, #[label("unexpected number")] pub Span);

#[derive(Error, Debug, Diagnostic)]
#[error("Legacy decimal escape is not permitted in strict mode")]
#[diagnostic()]
pub struct LegacyDecimalEscape(
    #[label("Legacy decimal escape is not permitted in strict mode")] pub Span,
);

#[derive(Error, Debug, Diagnostic)]
#[error("Legacy octal literals are not available")]
#[diagnostic()]
pub struct LegacyOctalLiteral(#[label("Legacy octal literals are not available")] pub Span);
