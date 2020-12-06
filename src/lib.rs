use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn dllmain(_: TokenStream, item: TokenStream) -> TokenStream {
    let func = syn::parse_macro_input!(item as syn::ItemFn);

    // Break the function down into its parts
    let syn::ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = func;

    let output = quote! {
        #[no_mangle]
        #[allow(non_snake_case, unused_variables)]
        extern "system" fn DllMain(dll_module: HINSTANCE, call_reason: DWORD, reserved: LPVOID) -> BOOL {
            const DLL_PROCESS_ATTACH: DWORD = 1;
            const DLL_PROCESS_DETACH: DWORD = 0;

            match call_reason {
                DLL_PROCESS_ATTACH => { #block },
                DLL_PROCESS_DETACH => (),
                _ => (),
            }
            minwindef::TRUE
        }
    };

    TokenStream::from(output)
}
