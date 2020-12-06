use quote::quote;
use syn::{parenthesized, parse, parse2, parse_macro_input, DeriveInput, Ident};

struct ParseFnParams {
    parse_fn: Ident,
}

impl parse::Parse for ParseFnParams {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);
        let parse_fn = content.parse()?;

        Ok(ParseFnParams{ parse_fn })
    }
}

#[proc_macro_derive(ValueParser, attributes(parse_fn))]
pub fn derive_value_parser(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let parse_fn_attr = input.attrs
        .iter()
        .find(|a| a.path.segments[0].ident == "parse_fn")
        .expect("No parse function provided (provide with #[parse_fn(_)] attribute)");

    let parse_fn_params: ParseFnParams = parse2(parse_fn_attr.tokens.clone()).expect("Help");
    let parse_fn = parse_fn_params.parse_fn;

    let expanded = quote! {
        impl<'a> Parser<'a> for #name {
            type Output = EspType;
            type Flags = u32;

            fn code(&self) -> &str { "" }

            fn do_parse(&self, ctx: &'a mut dyn Context<'a, Flags = Self::Flags >) -> Option<Self::Output> {
                let (remaining, output) = #parse_fn(ctx.get_bytes()).expect("Parser error");
                ctx.set_bytes(remaining);
                Some(output)
            }
        }
    };

    let tokens = proc_macro::TokenStream::from(expanded);
    tokens
}
