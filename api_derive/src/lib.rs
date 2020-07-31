#![forbid(unsafe_code)]
extern crate proc_macro;

use quote::{format_ident, quote};
use syn::{export::TokenStream, parse_macro_input};

#[proc_macro_attribute]
pub fn derive_db_fields(attr: TokenStream, item: TokenStream) -> TokenStream {
    let tokens = item.clone();
    let inputs = parse_macro_input![tokens as syn::ItemStruct];
    let struct_ident = &inputs.ident;
    let input = syn::parse_macro_input!(attr as syn::AttributeArgs);
    let mut with_extras = true;
    for nmeta in &input {
        if let syn::NestedMeta::Meta(syn::Meta::Path(thing)) = nmeta {
            if let Some(ident) = thing.get_ident() {
                if "DropExtra".eq(&ident.to_string()) {
                    with_extras = false;
                }
            }
        }
    }
    let mut found_struct = false;
    let mut new_struct: TokenStream = item
        .into_iter()
        .map(|r| {
            match r {
                proc_macro::TokenTree::Ident(ref ident) if ident.to_string() == "struct" => {
                    found_struct = true;
                    r
                }
                proc_macro::TokenTree::Group(ref group)
                    if group.delimiter() == proc_macro::Delimiter::Brace && found_struct =>
                {
                    let mut stream = proc_macro::TokenStream::new();
                    let flds_full: proc_macro::TokenStream = quote!(
                    #[serde(skip_serializing_if = "String::is_empty", default)]
                    #[validate(non_control_character, length(max = 254))]
                    pub _key: String,
                    // #[serde(skip_serializing_if = "String::is_empty", default)]
                    // #[validate(non_control_character)]
                    // pub _id: String,
                    #[serde(skip_serializing_if = "String::is_empty", default)]
                    #[validate(non_control_character)]
                    pub _rev: String,
                    #[serde(rename = "_oldRev", skip_serializing_if = "String::is_empty", default)]
                    #[validate(non_control_character)]
                    pub _old_rev: String,
                ).into();
                    let extras_fld: proc_macro::TokenStream = quote!(
                        #[serde(flatten)]
                        pub extra: std::collections::HashMap<String, serde_json::Value>,
                    )
                    .into();
                    stream.extend(flds_full);
                    if with_extras {
                        stream.extend(extras_fld);
                    }
                    stream.extend(group.stream());
                    proc_macro::TokenTree::Group(proc_macro::Group::new(
                        proc_macro::Delimiter::Brace,
                        stream,
                    ))
                }
                _ => r,
            }
        })
        .collect();
    let impl_fns: proc_macro::TokenStream = quote!(
        impl api_tools::DbFields for #struct_ident {
            fn _key(&self) -> String {
                self._key.clone()
            }
        }
    )
    .into();
    new_struct.extend(impl_fns);
    new_struct
}

#[proc_macro_derive(GetAll, attributes(author))]
pub fn derive_get_all(tokens: TokenStream) -> TokenStream {
    let inputs = parse_macro_input![tokens as syn::ItemStruct];
    let struct_ident = &inputs.ident;
    let data_name = struct_ident.to_string().to_lowercase();

    let author_field = &(*inputs
        .fields
        .iter()
        .filter(|field| {
            field.attrs.iter().any(|a| {
                if let Ok(mt) = a.parse_meta() {
                    return mt.path().is_ident("author");
                }
                false
            })
        })
        .collect::<Vec<&syn::Field>>()
        .first()
        .expect("A member of the struct must have the #[author] attribute."))
    .clone();

    let author_type_name = format!("{}", author_field.ident.clone().unwrap());
    let author_eq = format_ident!("{}_eq", author_type_name);
    let url_path = format!("/{}", data_name);
    let fn_name = format_ident!("list_{}", data_name);
    let coll_name = format!("{}s", data_name);

    let doc_comment = format!(
        "/// List all documents ot type {} for author {}.
        /// This handler can be mounted on {}.",
        struct_ident, author_type_name, url_path
    );
    let ts = quote!(
        #[doc = #doc_comment]
        pub async fn #fn_name(
            req: actix_web::HttpRequest,
            conn: actix_web::web::Data<arangoq::ArangoConnection>,
        ) -> actix_web::HttpResponse {
            log::debug!("{} entered", #coll_name);
            use actix_web::{ FromRequest, HttpResponse };
            use api_tools::web_query_ok;
            use futures::future::Either;

            let author = if let Ok(pth) = actix_web::web::Path::<String,>::extract(&req).await {
                pth.into_inner().clone()
            } else {
                String::new()
            };
            log::debug!("{} for author: {}", #coll_name, author);
            let coll = conn.context.collection_name(#coll_name);
            let query =
            if author.is_empty() {
                #struct_ident::query_builder(coll.as_str())
                    .read()
                    .limit(100)
                    .build()
            } else {
                #struct_ident::query_builder(coll.as_str())
                    .read()
                    .filter()
                    .#author_eq(&author)
                    .limit(100)
                    .build()
            };
            log::debug!("{} db query: {:?}", #coll_name, query);
            match web_query_ok::<#struct_ident>(query, &conn).await {
                Either::Left(ar) => {
                    if !ar.error {
                        let coll_res = serde_json::json!({"collection" : &ar.result});
                        HttpResponse::Ok().json(coll_res)
                    } else {
                        let msg = format!("Database Error:{} {}", ar.error_num, ar.error_message);
                        log::error!("{:#?} -> {}", &ar, msg);
                        HttpResponse::InternalServerError().json(Err::<(),_>(msg))
                    }
                },
                Either::Right(err) => err,
            }
        }
    );

    ts.into()
}

#[proc_macro_derive(Fetch, attributes(author))]
pub fn derive_fetch(tokens: TokenStream) -> TokenStream {
    let inputs = parse_macro_input![tokens as syn::ItemStruct];
    let struct_ident = &inputs.ident;
    let data_name = struct_ident.to_string().to_lowercase();

    let author_field = &(*inputs
        .fields
        .iter()
        .filter(|field| {
            field.attrs.iter().any(|a| {
                if let Ok(mt) = a.parse_meta() {
                    return mt.path().is_ident("author");
                }
                false
            })
        })
        .collect::<Vec<&syn::Field>>()
        .first()
        .expect("A member of the struct must have the #[author] attribute."))
    .clone();

    let author_type_name = format!("{}", author_field.ident.clone().unwrap());
    let author_eq = format_ident!("{}_eq", author_type_name);
    let url_path = format!("/{}", data_name);
    let fn_name = format_ident!("fetch_{}", data_name);
    let coll_name = format!("{}s", data_name);

    let doc_comment = format!(
        "/// Fetch a document of type {}.
        /// This handler can be mounted on {}.",
        struct_ident, url_path
    );
    let ts = quote!(
        #[doc = #doc_comment]
        // #[actix_web::get(#url_path)]
        pub async fn #fn_name(
            req: actix_web::HttpRequest,
            conn: actix_web::web::Data<arangoq::ArangoConnection>,
        ) -> actix_web::HttpResponse {
            use actix_web::{ FromRequest, HttpResponse };
            use api_tools::web_query_ok;
            use futures::future::Either;

            let r_key = req.match_info().get("key");
            match r_key {
                Some(key) => {
                    let author = if let Ok(pth) = actix_web::web::Path::<(String,String)>::extract(&req).await {
                        pth.0.clone()
                    } else {
                        String::new()
                    };

                    let coll = conn.context.collection_name(#coll_name);
                    let query = if author.is_empty() {
                        let coll = Collection::new(coll.as_str(), CollectionType::Document);
                        coll.get_by_key(key)
                    } else {
                        #struct_ident::query_builder(coll.as_str())
                            .read()
                            .filter()
                            .#author_eq(&author)
                            .filter()
                            ._key_eq(&key.to_owned())
                            .limit(1)
                            .build()
                    };
                    log::debug!("{} db query: {:?}", #coll_name, query);
                    match web_query_ok::<#struct_ident>(query, &conn).await {
                        Either::Left(ar) => {
                            if !ar.error {
                                if let Some(data) = ar.result.first() {
                                    HttpResponse::Ok().json(&data)
                                } else {
                                    HttpResponse::InternalServerError().json(Err::<(),_>("Empty db response."))
                                }
                            } else {
                                let msg = format!("Database Error:{} {}", ar.error_num, ar.error_message);
                                log::error!("{:#?} -> {}", &ar, msg);
                                HttpResponse::NotFound().json(Err::<(),_>(msg))
                            }
                        },
                        Either::Right(err) => err,
                    }
                },
                None => {
                    let path = req.path().to_string();
                    let msg = format!("Can not fetch document without key on path: {}", path);
                    HttpResponse::BadRequest().json(Err::<(),_>(msg))
                }
            }
        }
    );

    ts.into()
}

#[proc_macro_derive(Create, attributes(author))]
pub fn derive_create(tokens: TokenStream) -> TokenStream {
    let inputs = parse_macro_input![tokens as syn::ItemStruct];
    let struct_ident = &inputs.ident;
    let data_name = struct_ident.to_string().to_lowercase();
    let struct_name = struct_ident.to_string();

    let author_field = &(*inputs
        .fields
        .iter()
        .filter(|field| {
            field.attrs.iter().any(|a| {
                if let Ok(mt) = a.parse_meta() {
                    return mt.path().is_ident("author");
                }
                false
            })
        })
        .collect::<Vec<&syn::Field>>()
        .first()
        .expect("A member of the struct must have the #[author] attribute."))
    .clone();
    let author_prop = author_field.ident.clone().unwrap();
    let author_type_name = format!("{}", author_field.ident.clone().unwrap());

    let url_path = format!("/{}", data_name);
    let fn_name = format_ident!("create_{}", data_name);
    let coll_name = format!("{}s", data_name);

    let doc_comment = format!(
        "/// Create a new document of {} by posting to this handler.
        /// This handler can be mounted on {}.",
        struct_name, url_path
    );
    let ts = quote!(
        #[doc = #doc_comment]
        pub async fn #fn_name(
            data: actix_web::web::Json<#struct_ident>,
            req: actix_web::HttpRequest,
            conn: actix_web::web::Data<arangoq::ArangoConnection>,
            created_actor: actix_web::web::Data<actix::Addr<CreatedActor>>,
        ) -> actix_web::HttpResponse {
            log::debug!("{} entered", #coll_name);
            use actix_web::{ FromRequest, HttpResponse };
            use validator::Validate;
            use api_tools::web_query_ok;
            use futures::future::Either;

            match data.validate() {
                Ok(_) => (),
                Err(e) => {
                    let msg = format!("Failure during validation of {}: {}", #struct_name, serde_json::to_string(&e).unwrap());
                    log::warn!("{}", msg);
                    return HttpResponse::NotFound().json(Err::<(),_>(msg));
                }
            };
            let mut data = data.clone();

            let author = if let Ok(pth) = actix_web::web::Path::<String,>::extract(&req).await {
                let author = pth.into_inner();
                if !data.#author_prop.eq(&author) {
                    if !data.#author_prop.is_empty() {
                        let msg = format!("Invalid {} of {}: {:#?}, should be: {}", #author_type_name, #struct_name, data.#author_prop, author);
                        log::error!("{}", msg);
                        return HttpResponse::BadRequest().json(Err::<(),_>(msg));
                    } else {
                        data.#author_prop = author.clone();
                    }
                }
                author
            } else {
                String::new()
            };

            let coll = conn.context.collection_name(#coll_name);
            let query = #struct_ident::query_builder(coll.as_str())
                .create(&data)
                .build();

            log::debug!("{} db query: {:?}", #coll_name, query);
            match web_query_ok::<#struct_ident>(query, &conn).await {
                Either::Left(ar) => {
                    log::debug!("{} db response: {:#?}", #coll_name, &ar);
                    if !ar.error {
                        if let Some(data) = ar.result.first() {
                            created_actor.do_send(Created{data: data.clone()});
                            HttpResponse::Ok().json(&data)
                        } else {
                            HttpResponse::InternalServerError().json(Err::<(),_>("Empty db response."))
                        }
                    } else {
                        let msg = format!("Database Error:{} {}", ar.error_num, ar.error_message);
                        log::error!("{:#?} -> {}", &ar, msg);
                        HttpResponse::InternalServerError().json(Err::<(),_>(msg))
                    }
                },
                Either::Right(err) => err,
            }
        }
    );

    ts.into()
}

#[proc_macro_derive(Update, attributes(author))]
pub fn derive_update(tokens: TokenStream) -> TokenStream {
    let inputs = parse_macro_input![tokens as syn::ItemStruct];
    let struct_ident = &inputs.ident;
    let data_name = struct_ident.to_string().to_lowercase();

    let url_path = format!("/{}/{{key}}", data_name);
    let fn_name = format_ident!("update_{}", data_name);
    let coll_name = format!("{}s", data_name);
    let forbidden_char_msg =
        format!("Forbidden character found during validation of {}.", coll_name);

    let doc_comment = format!(
        "/// Update a document of {} by patch-ing to this handler.
        /// ```
        /// pub struct Person {{
        ///   name: &'static str,
        ///   age: u8,
        /// }}
        /// // setting age using {}:
        /// // patch().body(\"{{ \"age\": 66 }}\")
        /// 
        /// ```
        /// It may check _rev to deny accidentally changing a newer variant of the document.
        /// This handler can be mounted on {}.",
        struct_ident, fn_name, url_path
    );
    let ts = quote!(
        #[doc = #doc_comment]
        // #[actix_web::patch(#url_path)]
        pub async fn #fn_name(
            input: actix_web::web::Json<serde_json::Value>,
            req: actix_web::HttpRequest,
            conn: actix_web::web::Data<arangoq::ArangoConnection>,
        ) -> actix_web::HttpResponse {
            log::debug!("{} entered", #coll_name);
            use actix_web::{ FromRequest, HttpResponse };
            use validator::Validate;
            use api_tools::{web_query_ok};
            use futures::future::Either;

            use json_patch::merge;

            let r_key = req.match_info().get("key");
            match r_key {
                Some(key) => {
                    let data_res = serde_json::to_string(&*input);
                    log::debug!("Input({}): {:#?}", #coll_name, data_res);
                    if data_res.is_err() {
                        // this isn't supposed to happen, because we get a valid object only due to Json<UpdateParams> type
                        let msg = format!("Bad data in request body: {}", data_res.unwrap_err());
                        return HttpResponse::BadRequest().json(Err::<(),_>(msg));
                    }

                    let valid = validator::validate_non_control_character(data_res.unwrap());
                    if !valid {
                        let msg = #forbidden_char_msg;
                        log::warn!("{}", msg);
                        return HttpResponse::BadRequest().json(Err::<(),_>(msg));
                    }

                    let mut datamap = serde_json::to_value(#struct_ident::default()).unwrap();
                    merge(&mut datamap, &input.0);
                    log::debug!("{} -> {}", &input.0, datamap);
                    if let Err(err) = serde_json::from_value::<#struct_ident>(datamap.clone()) {
                        let msg = format!("{:?} line:{} column:{} classify?:{:?}, io?:{}, syntax?:{}, data?:{}, eof?:{}",
                        err, err.line(), err.column(), err.classify(), err.is_io(), err.is_syntax(), err.is_data(), err.is_eof());
                        log::error!("{} Error:{}", datamap, msg);
                        return HttpResponse::BadRequest().json(Err::<(),_>(msg));
                    }

                    let coll = conn.context.collection_name(#coll_name);
                    let collection = Collection::new(coll.as_str(), CollectionType::Document);
                    let query = collection.update(key, &*input);
                    log::debug!("update query: {:?}", query);
                    match web_query_ok::<#struct_ident>(query, &conn).await {
                        Either::Left(ar) => {
                            if !ar.error {
                                if let Some(data) = ar.result.first() {
                                    HttpResponse::Ok().json(&data)
                                } else {
                                    HttpResponse::InternalServerError().json(Err::<(),_>("Empty db response."))
                                }
                            } else {
                                let msg = format!("Database Error:{} {}", ar.error_num, ar.error_message);
                                log::error!("{:#?} -> {}", &ar, msg);
                                HttpResponse::InternalServerError().json(Err::<(),_>(msg))
                            }
                        },
                        Either::Right(err) => err,
                    }
                },
                None => HttpResponse::BadRequest()
                .json(Err::<(),_>("Can not update document without key.")),
            }
        }
    );

    ts.into()
}

#[proc_macro_derive(Replace, attributes(author))]
pub fn derive_replace(tokens: TokenStream) -> TokenStream {
    let inputs = parse_macro_input![tokens as syn::ItemStruct];
    let struct_ident = &inputs.ident;
    let data_name = struct_ident.to_string().to_lowercase();
    let struct_name = struct_ident.to_string();

    let author_field = &(*inputs
        .fields
        .iter()
        .filter(|field| {
            field.attrs.iter().any(|a| {
                if let Ok(mt) = a.parse_meta() {
                    return mt.path().is_ident("author");
                }
                false
            })
        })
        .collect::<Vec<&syn::Field>>()
        .first()
        .expect("A member of the struct must have the #[author] attribute."))
    .clone();
    let author_prop = author_field.ident.clone().unwrap();
    let author_type_name = format!("{}", author_field.ident.clone().unwrap());
    let author_eq = format_ident!("{}_eq", author_type_name);

    let url_path = format!("/{}", data_name);
    let fn_name = format_ident!("replace_{}", data_name);
    let coll_name = format!("{}s", data_name);

    let doc_comment = format!(
        "/// Replace a new document of {} by posting to this handler.
        /// It may check _rev to deny accidentally changing a newer variant of the document.
        /// This handler can be mounted on {}.",
        struct_name, url_path
    );
    let ts = quote!(
        #[doc = #doc_comment]
        pub async fn #fn_name(
            data: actix_web::web::Json<#struct_ident>,
            req: actix_web::HttpRequest,
            conn: actix_web::web::Data<arangoq::ArangoConnection>,
        ) -> actix_web::HttpResponse {
            log::debug!("{} entered", #coll_name);
            use actix_web::{ FromRequest, HttpResponse };
            use validator::Validate;
            use api_tools::web_query_ok;
            use futures::future::Either;

            let r_key = req.match_info().get("key");
            match r_key {
                Some(key) => {
                    match data.validate() {
                        Ok(_) => (),
                        Err(e) => {
                            let msg = format!("Failure during validation of {}: {}", #struct_name, serde_json::to_string(&e).unwrap());
                            log::warn!("{}", msg);
                            return HttpResponse::NotFound().json(Err::<(),_>(msg));
                        }
                    };
                    let mut data = data.clone();

                    let author = if let Ok(pth) = actix_web::web::Path::<(String,String)>::extract(&req).await {
                        let author = pth.0.clone();
                        if !data.#author_prop.eq(&author) {
                            if !data.#author_prop.is_empty() {
                                let msg = format!("Invalid {} of {}: {:#?}, should be: {}", #author_type_name, #struct_name, data.#author_prop, author);
                                log::error!("{}", msg);
                                return HttpResponse::BadRequest().json(Err::<(),_>(msg));
                            } else {
                                data.#author_prop = author.clone();
                            }
                        }
                        author
                    } else {
                        String::new()
                    };

                    let coll = conn.context.collection_name(#coll_name);
                    let query = if author.is_empty() {
                        #struct_ident::query_builder(coll.as_str())
                            .update()
                            .filter()
                            ._key_eq(&key.to_owned())
                            .replace_with(&data)
                            .build()
                    } else {
                        #struct_ident::query_builder(coll.as_str())
                            .update()
                            .filter()
                            .#author_eq(&author)
                            .and()
                            ._key_eq(&key.to_owned())
                            .replace_with(&data)
                            .build()
                    };

                    log::debug!("{} db query: {:?}", #coll_name, query);
                    match web_query_ok::<#struct_ident>(query, &conn).await {
                        Either::Left(ar) => {
                            log::debug!("{} db response: {:#?}", #coll_name, &ar);
                            if !ar.error {
                                if let Some(data) = ar.result.first() {
                                    // replaced_actor.do_send(Replaced{data: data.clone()});
                                    HttpResponse::Ok().json(&data)
                                } else {
                                    HttpResponse::InternalServerError().json(Err::<(),_>("Empty db response."))
                                }
                            } else {
                                let msg = format!("Database Error:{} {}", ar.error_num, ar.error_message);
                                log::error!("{:#?} -> {}", &ar, msg);
                                HttpResponse::InternalServerError().json(Err::<(),_>(msg))
                            }
                        },
                        Either::Right(err) => err,
                    }
                },
                None => {
                    let path = req.path().to_string();
                    let msg = format!("Can not fetch document without key on path: {}", path);
                    HttpResponse::BadRequest().json(Err::<(),_>(msg))
                }
            }
        }
    );

    ts.into()
}

#[proc_macro_derive(Delete, attributes(author))]
pub fn derive_delete(tokens: TokenStream) -> TokenStream {
    let inputs = parse_macro_input![tokens as syn::ItemStruct];
    let struct_ident = &inputs.ident;
    let data_name = struct_ident.to_string().to_lowercase();

    let author_field = &(*inputs
        .fields
        .iter()
        .filter(|field| {
            field.attrs.iter().any(|a| {
                if let Ok(mt) = a.parse_meta() {
                    return mt.path().is_ident("author");
                }
                false
            })
        })
        .collect::<Vec<&syn::Field>>()
        .first()
        .expect("A member of the struct must have the #[author] attribute."))
    .clone();

    let author_type_name = format!("{}", author_field.ident.clone().unwrap());
    let author_eq = format_ident!("{}_eq", author_type_name);

    let url_path = format!("/{}/{{key}}", data_name);
    let fn_name = format_ident!("delete_{}", data_name);
    let coll_name = format!("{}s", data_name);

    let doc_comment = format!(
        "/// Delete a document of type {}.
        /// It may check _rev to deny accidentally changing a newer variant of the document.
        /// This handler can be mounted on {}.",
        struct_ident, url_path
    );
    let ts = quote!(
        #[doc = #doc_comment]
        // #[actix_web::get(#url_path)]
        pub async fn #fn_name(
            req: actix_web::HttpRequest,
            conn: actix_web::web::Data<arangoq::ArangoConnection>,
        ) -> actix_web::HttpResponse {
            use actix_web::{ FromRequest, HttpResponse };
            use api_tools::web_query_ok;
            use futures::future::Either;


            let r_key = req.match_info().get("key");
            match r_key {
                Some(key) => {
                    let author = if let Ok(pth) = actix_web::web::Path::<(String,String)>::extract(&req).await {
                        pth.0.clone()
                    } else {
                        String::new()
                    };

                    let coll = conn.context.collection_name(#coll_name);
                    let query =
                    if author.is_empty() {
                        #struct_ident::query_builder(coll.as_str())
                        .delete()
                        .filter()
                        ._key_eq(&key.to_owned())
                        .build()
                    } else {
                        #struct_ident::query_builder(coll.as_str())
                        .delete()
                        .filter()
                        .#author_eq(&author)
                        .and()
                        ._key_eq(&key.to_owned())
                        .build()
                    };
                    log::debug!("{} delete query: {:?}", #coll_name, query);
                    match web_query_ok::<#struct_ident>(query, &conn).await {
                        Either::Left(ar) => {
                            if !ar.error {
                                if let Some(data) = ar.result.first() {
                                    HttpResponse::Ok().json(&data)
                                } else {
                                    HttpResponse::InternalServerError().json(Err::<(),_>("Empty db response."))
                                }
                            } else {
                                let msg = format!("Database Error:{} {}", ar.error_num, ar.error_message);
                                log::error!("{:#?} -> {}", &ar, msg);
                                HttpResponse::NotFound().json(Err::<(),_>(msg))
                            }
                        },
                        Either::Right(err) => err,
                    }
                },
                None => {
                    let path = req.path().to_string();
                    let msg = format!("Can not fetch document without key on path: {}", path);
                    HttpResponse::BadRequest().json(Err::<(),_>(msg))
                }
            }
        }
    );

    ts.into()
}
