extern crate proc_macro;

use proc_macro::{Ident, Span, TokenStream};
use std::collections::{HashMap, VecDeque};

use quote::quote;
use syn;
use syn::{Fields, FieldsNamed, File, GenericArgument, ItemStruct, Lit, Meta, MetaList, NestedMeta, PathArguments, Type};
use syn::visit::Visit;


// todo: generate enum with getters
// todo: return getter strings
// todo: return getter enum
// todo: pass through the original code, remove the macro attributes from it


#[derive(Debug)]
struct Struct {
    ident: String,
    fields: Vec<Field>,
    generate: bool,
    return_type: Option<String>,
    replacement_type: Option<String>,
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

#[derive(Debug)]
struct Field {
    ident: String,
    types: VecDeque<String>,
}

fn extract_types_deque(start_type: &Type) -> VecDeque<String> {
    let mut res = VecDeque::new();
    let mut rec_ty = VecDeque::new();
    rec_ty.push_front(start_type.clone());

    while let Some(ty) = rec_ty.pop_front() {
        match ty {
            Type::Path(p) => {
                for seg in p.path.segments {
                    res.push_back(seg.ident.to_string());
                    if !seg.arguments.is_empty() {
                        match seg.arguments {
                            PathArguments::AngleBracketed(ab_arg) => {
                                for arg in ab_arg.args.iter() {
                                    match arg {
                                        GenericArgument::Type(next_ty) => {
                                            rec_ty.push_back(next_ty.clone());
                                        }
                                        _ => panic!("only types as arguments are supported")
                                    }
                                }
                            }
                            _ => panic!("only angle-bracketed type segments are supported")
                        }
                    }
                }
            }
            _ => panic!("only type paths are supported")
        }
    }

    res
}

fn extract_fields(fields: &FieldsNamed) -> Vec<Field> {
    fields
        .named
        .iter()
        // .filter(|field| matches!(field.vis, Visibility::Public(_)))
        .map(|field| {
            let ident = field.ident.as_ref().unwrap().to_string();
            let types = extract_types_deque(&field.ty);
            Field { ident, types }
        })
        .collect()
}

#[derive(Default)]
struct StructVisitor {
    structs: HashMap<String, Struct>,
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

        let fields = match &i.fields {
            Fields::Named(named) => extract_fields(&named),
            _ => panic!("only named fields are supported")
        };

        self.structs.insert(
            ident.clone(),
            Struct {
                ident,
                fields,
                generate,
                return_type,
                replacement_type,
            });
    }
}

fn generate_getters(target: String, structs: &HashMap<String, Struct>) -> Vec<String> {
    let mut res = Vec::new();
    let mut to_visit = VecDeque::new();
    to_visit.push_back((target.clone(), structs.get(&target).unwrap()));

    while let Some((prefix, strct)) = to_visit.pop_front() {
        for field in strct.fields.iter() {
            let ty = field.types.back().unwrap();
            if structs.contains_key(ty) {
                to_visit.push_front((format!("{}.{}", prefix, field.ident), structs.get(ty).unwrap()));
            } else {
                res.push(format!("{}.{}", prefix, field.ident));
            }
        }
    }

    res
}

fn generate_impl(target: String, structs: &HashMap<String, Struct>) -> TokenStream {
    let trgt = syn::Ident::new(&target, proc_macro2::Span::mixed_site());
    let paths = generate_getters(target, structs);
    let res = quote!(
        impl StructDeepGetter for #trgt {
            fn deeper_structs() -> Vec<String> {
                let mut res = Vec::new();
                #(res.push(#paths.to_string());)*
                res
            }
        }
    );
    TokenStream::from(res)
}

#[proc_macro]
pub fn make_paths(input: TokenStream) -> TokenStream {
    let ast: File = syn::parse(input).unwrap();
    let mut state = StructVisitor::default();
    state.visit_file(&ast);

    let mut impls = Vec::new();
    for (ident, strct) in state.structs.iter() {
        if strct.generate {
            impls.push(generate_impl(ident.clone(), &state.structs));
        }
    }

    TokenStream::from_iter(impls)
}
