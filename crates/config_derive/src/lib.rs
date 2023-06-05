
use proc_macro::TokenStream;
use quote::quote;
use syn::{self, Data, parse_quote};
use syn::DeriveInput;

fn is_Value(ty: &syn::Type) -> bool {
    if let syn::Type::Path(syn::TypePath { ref path, .. }) = ty {
        // 这里我们取segments的最后一节来判断是不是`Option<T>`，这样如果用户写的是`std:option:Option<T>`我们也能识别出最后的`Option<T>`
        if let Some(seg) = path.segments.last() {
            if seg.ident == "Value" {
                return true;
            } else {
                return false;
            }
        }
        else {
            return false;
        }
    } else {
        return false;
    }
}

#[proc_macro_derive(ConfigDerive)]
pub fn derive_config(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let id = ast.ident;

    let Data::Struct(s) = ast.data else{
        panic!("Config derive macro must use in struct");
    };

    let from_clause: Vec<_> = s.fields.iter().map(|v| {
        let ident = v.ident.as_ref().unwrap();
        let ident_string = ident.to_string();
        quote! {
            #ident: v[#ident_string].clone().into()
        }
    }).collect();

	let mut field_ast = quote!();


    // let aa: Attribute = parse_quote! {#[derive(ConfigDerive)]};
    // eprintln!("{:#?}", &id);
    // eprintln!("{:#?}", &s);
    for field in s.fields.iter(){
        let (field_id, field_attrs, field_ty) = (field.ident.as_ref().unwrap(), &field.attrs, &field.ty);
        // let is_value = field_attrs.contains(&parse_quote! {#[derive(ConfigDerive)]});
        let is_Value = is_Value(field_ty);
        // assert_eq!(field_id.type_id(),TypeId::of::<String>());
        // eprintln!("{:#?}", is_Value);

        let field_name_literal = field_id.to_string();
        if !is_Value {
            field_ast.extend( quote! {
                #field_id: CONFIGL.get(#field_name_literal)
                                    .map_or(#field_ty::new(),|v|v.clone().unwrap().into()),
                // #field_id: #field_ty::default(),
            })
        }else {
            field_ast.extend( quote! {
                #field_id: CONFIGL.get(#field_name_literal)
                                    .unwrap().clone().unwrap(),
                // #field_id: #field_ty::default(),
            })
        }
    }

	quote! {
        impl From<Value> for #id {
            fn from(v: Value) -> Self {
                Self {
                    #(#from_clause),*
                }
            }
        }
        impl #id {
            fn new() -> Self {
                Self {
					#field_ast
                }
            }
        }
    }.into()
}
