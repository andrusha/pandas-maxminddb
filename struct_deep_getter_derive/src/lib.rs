extern crate proc_macro;

use proc_macro::TokenStream;
use std::collections::HashMap;

use quote::quote;
use syn;
use syn::{File, ItemStruct, Lit, Meta, MetaList, NestedMeta};
use syn::visit::Visit;

trait StructDeepGetter {
    fn deeper_structs() -> Vec<String>;
}

#[derive(Debug)]
struct Struct {
    ident: String,
    generate: bool,
    return_type: Option<String>,
    replacement_type: Option<String>,
}

#[derive(Default)]
struct StructVisitor {
    structs: HashMap<String, Struct>,
}

fn get_nested_meta_params(meta: &MetaList) -> HashMap<String, String> {
    meta
        .nested
        .iter()
        .map(|nm| match nm {
            NestedMeta::Meta(m) => match m {
                Meta::NameValue(nameval) =>
                    match (nameval.path.get_ident(), &nameval.lit) {
                        (Some(ident), Lit::Str(lit)) => {
                            (ident.to_string(), lit.value())
                        }
                        _ => panic!("struct_deep_getter attributes are expected to be strings")
                    }
                _ => panic!("struct_deep_getter attributes should be named list")
            }
            NestedMeta::Lit(_) => panic!("struct_deep_getter attributes should be named list")
        })
        .collect()
}

fn get_meta(i: &ItemStruct) -> Option<MetaList> {
    for attr in &i.attrs {
        if let Ok(Meta::List(meta)) = attr.parse_meta() {
            if let Some(macro_name) = meta.path.get_ident() {
                if macro_name == "struct_deep_getter" {
                    return Some(meta);
                }
            }
        }
    }

    None
}

impl<'ast> Visit<'ast> for StructVisitor {
    fn visit_item_struct(&mut self, i: &'ast ItemStruct) {
        let ident = i.ident.to_string();
        let mut generate = false;
        let mut return_type = None;
        let mut replacement_type = None;

        if let Some(meta) = get_meta(i) {
            generate = true;

            let params = get_nested_meta_params(&meta);
            return_type = params.get("return_type").map(|v| v.clone());
            replacement_type = params.get("replacement_type").map(|v| v.clone());
        }

        self.structs.insert(
            ident.clone(),
            Struct {
                ident,
                generate,
                return_type,
                replacement_type,
            });
    }
}

#[proc_macro]
pub fn make_paths(input: TokenStream) -> TokenStream {
    let ast: File = syn::parse(input).unwrap();
    let mut state = StructVisitor::default();
    state.visit_file(&ast);


    let debug = state.structs.into_iter()
        .map(|(k, v)| format!("{} => {:?}", k, v))
        .collect::<Vec<String>>()
        .join("\n");

    let res = quote!(
      fn structs<'a>()->&'a str {
            #debug
        }
    );
    TokenStream::from(res)
}
