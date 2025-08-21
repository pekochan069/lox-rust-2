use proc_macro::TokenStream;
use quote::quote;
use syn::{BinOp, parse_macro_input};

#[proc_macro]
pub fn binary_number_op(input: TokenStream) -> TokenStream {
    let op = parse_macro_input!(input as BinOp);

    // 코드 생성
    let expanded = quote! {
        {
            let (Some(b), Some(a)) = (self.pop_value(), self.pop_value()) else {
                return Err(self.runtime_error("Invalid access to stack."));
            };

            match (a, b) {
                (Value::Number { value: a_value }, Value::Number { value: b_value }) => self.push_value(Value::Number{ value: a_value #op b_value }),
                _ => {
                    return Err(self.runtime_error("Operands must be numbers."));
                },
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro]
pub fn binary_bool_op(input: TokenStream) -> TokenStream {
    let op = parse_macro_input!(input as BinOp);

    // 코드 생성
    let expanded = quote! {
        {
            let (Some(b), Some(a)) = (self.pop_value(), self.pop_value()) else {
                return Err(self.runtime_error("Invalid access to stack."));
            };

            match (a, b) {
                (Value::Number { value: a_value }, Value::Number { value: b_value }) => self.push_value(Value::Bool{ value: a_value #op b_value }),
                _ => {
                    return Err(self.runtime_error("Operands must be numbers."));
                },
            }
        }
    };

    TokenStream::from(expanded)
}
