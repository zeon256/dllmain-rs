use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Error, FnArg, Ident, Item, LitStr, Pat, ReturnType, Token, Type};

#[derive(Clone, Copy, Eq, PartialEq)]
enum PanicPolicy {
    Abort,
    ReturnFalse,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum DllEvent {
    ProcessDetach,
    ProcessAttach,
    ThreadAttach,
    ThreadDetach,
}

impl DllEvent {
    fn from_ident(ident: &Ident) -> Result<Self, Error> {
        match ident.to_string().as_str() {
            "process_attach" => Ok(Self::ProcessAttach),
            "process_detach" => Ok(Self::ProcessDetach),
            "thread_attach" => Ok(Self::ThreadAttach),
            "thread_detach" => Ok(Self::ThreadDetach),
            _ => Err(Error::new_spanned(
                ident,
                "unknown event; expected one of: process_attach, process_detach, thread_attach, thread_detach",
            )),
        }
    }

    fn match_arm_tokens(self, reason_binding: Option<&Pat>, block: &syn::Block) -> TokenStream2 {
        let reason = match self {
            Self::ProcessDetach => quote! { DLL_PROCESS_DETACH },
            Self::ProcessAttach => quote! { DLL_PROCESS_ATTACH },
            Self::ThreadAttach => quote! { DLL_THREAD_ATTACH },
            Self::ThreadDetach => quote! { DLL_THREAD_DETACH },
        };

        let bind_reason = match reason_binding {
            Some(pattern) => quote! { let #pattern: u32 = call_reason; },
            None => quote! {},
        };

        quote! {
            #reason => {
                #bind_reason
                #block
            },
        }
    }
}

struct EntryArgs {
    events: Vec<DllEvent>,
    panic_policy: PanicPolicy,
}

impl Default for EntryArgs {
    fn default() -> Self {
        Self {
            events: vec![DllEvent::ProcessAttach],
            panic_policy: PanicPolicy::Abort,
        }
    }
}

impl Parse for EntryArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut args = EntryArgs::default();
        let mut seen_events = false;
        let mut seen_panic = false;

        while !input.is_empty() {
            let option: Ident = input.parse()?;

            if option == "events" {
                if seen_events {
                    return Err(Error::new_spanned(option, "duplicate option `events`"));
                }
                seen_events = true;

                let content;
                syn::parenthesized!(content in input);
                let parsed_events: Punctuated<Ident, Token![,]> =
                    content.parse_terminated(Ident::parse)?;

                if parsed_events.is_empty() {
                    return Err(Error::new_spanned(
                        option,
                        "`events(...)` must include at least one event",
                    ));
                }

                let mut events = Vec::with_capacity(parsed_events.len());
                for event_ident in parsed_events {
                    let event = DllEvent::from_ident(&event_ident)?;
                    if events.contains(&event) {
                        return Err(Error::new_spanned(
                            event_ident,
                            "duplicate event in `events(...)`",
                        ));
                    }
                    events.push(event);
                }
                args.events = events;
            } else if option == "panic" {
                if seen_panic {
                    return Err(Error::new_spanned(option, "duplicate option `panic`"));
                }
                seen_panic = true;

                input.parse::<Token![=]>()?;
                let value: LitStr = input.parse()?;

                args.panic_policy = match value.value().as_str() {
                    "abort" => PanicPolicy::Abort,
                    "return_false" => PanicPolicy::ReturnFalse,
                    _ => {
                        return Err(Error::new_spanned(
                            value,
                            "invalid panic policy; expected \"abort\" or \"return_false\"",
                        ))
                    }
                };
            } else {
                return Err(Error::new_spanned(
                    option,
                    "unknown option; expected `events(...)` or `panic = \"...\"`",
                ));
            }

            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
        }

        Ok(args)
    }
}

fn is_u32_type(ty: &Type) -> bool {
    match ty {
        Type::Path(path) => path.qself.is_none() && path.path.is_ident("u32"),
        _ => false,
    }
}

fn reason_pattern(sig: &syn::Signature) -> syn::Result<Option<&Pat>> {
    if sig.constness.is_some() {
        return Err(Error::new_spanned(
            sig.constness,
            "const functions are not supported by #[dllmain_rs::entry]",
        ));
    }

    if sig.asyncness.is_some() {
        return Err(Error::new_spanned(
            sig.asyncness,
            "async functions are not supported by #[dllmain_rs::entry]",
        ));
    }

    if sig.unsafety.is_some() {
        return Err(Error::new_spanned(
            sig.unsafety,
            "unsafe functions are not supported by #[dllmain_rs::entry]",
        ));
    }

    if let Some(abi) = &sig.abi {
        return Err(Error::new_spanned(
            abi,
            "explicit ABI is not supported; #[dllmain_rs::entry] generates DllMain ABI",
        ));
    }

    if let Some(variadic) = &sig.variadic {
        return Err(Error::new_spanned(
            variadic,
            "variadic functions are not supported by #[dllmain_rs::entry]",
        ));
    }

    if !sig.generics.params.is_empty() || sig.generics.where_clause.is_some() {
        return Err(Error::new_spanned(
            &sig.generics,
            "generic functions are not supported by #[dllmain_rs::entry]",
        ));
    }

    if !matches!(sig.output, ReturnType::Default) {
        return Err(Error::new_spanned(
            &sig.output,
            "function must return () for #[dllmain_rs::entry]",
        ));
    }

    match sig.inputs.len() {
        0 => Ok(None),
        1 => match sig.inputs.first() {
            Some(FnArg::Typed(arg)) => {
                if !is_u32_type(&arg.ty) {
                    return Err(Error::new_spanned(
                        &arg.ty,
                        "single argument must be `u32` (the DLL reason code)",
                    ));
                }
                Ok(Some(&arg.pat))
            }
            Some(FnArg::Receiver(receiver)) => Err(Error::new_spanned(
                receiver,
                "#[dllmain_rs::entry] expects a free function",
            )),
            None => Ok(None),
        },
        _ => Err(Error::new_spanned(
            &sig.inputs,
            "function must have signature `fn name()` or `fn name(reason: u32)`",
        )),
    }
}

#[proc_macro_attribute]
pub fn entry(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = match syn::parse::<EntryArgs>(attr) {
        Ok(args) => args,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };

    let parsed_item = match syn::parse::<Item>(item) {
        Ok(item) => item,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };

    let func = match parsed_item {
        Item::Fn(func) => func,
        other => {
            return TokenStream::from(
                Error::new_spanned(other, "#[dllmain_rs::entry] expects a free function")
                    .to_compile_error(),
            )
        }
    };

    let reason_binding = match reason_pattern(&func.sig) {
        Ok(binding) => binding,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };

    let block = &func.block;
    let match_arms: Vec<_> = args
        .events
        .iter()
        .copied()
        .map(|event| event.match_arm_tokens(reason_binding, block))
        .collect();

    let wrapped_body = quote! {
        match call_reason {
            #(#match_arms)*
            _ => {},
        }
        DLLMAIN_TRUE
    };

    let panic_policy = match args.panic_policy {
        PanicPolicy::Abort => quote! {
            match ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                #wrapped_body
            })) {
                Ok(value) => value,
                Err(_) => ::std::process::abort(),
            }
        },
        PanicPolicy::ReturnFalse => quote! {
            match ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                #wrapped_body
            })) {
                Ok(value) => value,
                Err(_) => DLLMAIN_FALSE,
            }
        },
    };

    let output = quote! {
        #[no_mangle]
        #[allow(non_snake_case, unused_variables)]
        extern "system" fn DllMain(
            _dll_module: *mut ::core::ffi::c_void,
            call_reason: u32,
            _reserved: *mut ::core::ffi::c_void,
        ) -> i32 {
            const DLL_PROCESS_DETACH: u32 = 0;
            const DLL_PROCESS_ATTACH: u32 = 1;
            const DLL_THREAD_ATTACH: u32 = 2;
            const DLL_THREAD_DETACH: u32 = 3;
            const DLLMAIN_TRUE: i32 = 1;
            const DLLMAIN_FALSE: i32 = 0;

            #panic_policy
        }
    };

    TokenStream::from(output)
}
