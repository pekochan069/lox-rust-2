use proc_macro::TokenStream;
use quote::quote;
use syn::{BinOp, parse_macro_input};

#[proc_macro]
pub fn binary_op(input: TokenStream) -> TokenStream {
    let op = parse_macro_input!(input as BinOp);

    // 코드 생성
    let expanded = quote! {
        {
            let Some(b) = self.pop_value() else {
                return InterpretResult::RuntimeError;
            };
            let Some(a) = self.pop_value() else {
                return InterpretResult::RuntimeError;
            };
            self.push_value(a #op b);
        }
    };

    TokenStream::from(expanded)
}
