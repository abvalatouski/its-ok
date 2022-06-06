use std::iter::{FromIterator, IntoIterator};
use syn::{self, fold::Fold};

pub struct ReplaceTryWithUnwrap {
    unchecked: bool,
    can_replace: bool,
}

impl ReplaceTryWithUnwrap {
    pub fn new(unchecked: bool) -> Self {
        Self {
            unchecked,
            can_replace: true,
        }
    }

    pub fn fold_statements<T, U>(&mut self, statements: T) -> U
    where
        T: IntoIterator<Item = syn::Stmt>,
        U: FromIterator<T::Item>,
    {
        statements
            .into_iter()
            .map(|statement| self.fold_stmt(statement))
            .collect()
    }
}

macro_rules! disallow_replacement {
    ($($fold_method:ident($node:ty)),*,) => {
        $(
            fn $fold_method(&mut self, node: $node) -> $node {
                if self.can_replace {
                    self.can_replace = false;
                    let node = syn::fold::$fold_method(self, node);
                    self.can_replace = true;
                    node
                } else {
                    node
                }
            }
        )*
    };
}

impl syn::fold::Fold for ReplaceTryWithUnwrap {
    fn fold_expr(&mut self, node: syn::Expr) -> syn::Expr {
        match node {
            syn::Expr::Try(syn::ExprTry {
                attrs,
                expr,
                question_token,
            }) if self.can_replace => syn::Expr::MethodCall(syn::ExprMethodCall {
                attrs,
                receiver: Box::new(syn::fold::fold_expr(self, *expr)),
                dot_token: syn::token::Dot {
                    spans: [question_token.span],
                },
                method: syn::Ident::new(
                    if self.unchecked {
                        "unwrap_unchecked"
                    } else {
                        "unwrap"
                    },
                    question_token.span,
                ),
                turbofish: None,
                paren_token: syn::token::Paren {
                    span: question_token.span,
                },
                args: syn::punctuated::Punctuated::new(),
            }),
            node => syn::fold::fold_expr(self, node),
        }
    }

    disallow_replacement! {
        fold_expr_closure(syn::ExprClosure),
        fold_item_fn(syn::ItemFn),
        fold_item_impl(syn::ItemImpl),
        fold_item_macro(syn::ItemMacro),
        fold_item_macro2(syn::ItemMacro2),
        fold_item_mod(syn::ItemMod),
        fold_item_trait(syn::ItemTrait),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use quote::quote;
    use syn;

    #[test]
    fn replace_question_marks_with_unwrap_calls() {
        let code = TokenStream::from(quote! {
            let mut buffer = Vec::new();
            buffer.write_all(b"bytes")?;
            x?.f()?.g();
        });
        let statements = syn::parse::Parser::parse2(syn::Block::parse_within, code).unwrap();

        let use_unwrap_unchecked = false;
        let mut replacer = ReplaceTryWithUnwrap::new(use_unwrap_unchecked);
        let statements = replacer.fold_statements::<_, Vec<_>>(statements);
        let code = TokenStream::from(quote! { #(#statements)* });

        let expected_code = TokenStream::from(quote! {
            let mut buffer = Vec::new();
            buffer.write_all(b"bytes").unwrap();
            x.unwrap().f().unwrap().g();
        });
        assert_eq!(expected_code.to_string(), code.to_string());
    }

    #[test]
    fn replacements_are_not_done_inside_disallowed_nodes() {
        let code = TokenStream::from(quote! {
            fn write(buffer: &mut Vec<u8>) -> io::Result<()> { buffer.write_all(b"bytes")?; Ok(()) }
            let mut buffer = Vec::new();
            buffer.write_all(b"bytes")?;
            (| | { buffer.write_all(b"bytes")?; })();
            buffer.write_all(b"bytes")?;
            (| | { buffer.write_all(b"bytes")?; })();
        });
        let statements = syn::parse::Parser::parse2(syn::Block::parse_within, code).unwrap();

        let use_unwrap_unchecked = true;
        let mut replacer = ReplaceTryWithUnwrap::new(use_unwrap_unchecked);
        let statements = replacer.fold_statements::<_, Vec<_>>(statements);
        let code = TokenStream::from(quote! { #(#statements)* });

        let expected_code = TokenStream::from(quote! {
            fn write(buffer: &mut Vec<u8>) -> io::Result<()> { buffer.write_all(b"bytes")?; Ok(()) }
            let mut buffer = Vec::new();
            buffer.write_all(b"bytes").unwrap_unchecked();
            (| | { buffer.write_all(b"bytes")?; })();
            buffer.write_all(b"bytes").unwrap_unchecked();
            (| | { buffer.write_all(b"bytes")?; })();
        });
        assert_eq!(expected_code.to_string(), code.to_string());
    }
}
