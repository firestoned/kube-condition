// Copyright (c) 2025 Erick Bourgeois, firestoned
// SPDX-License-Identifier: MIT

//! Procedural macro for deriving `StatusCondition` trait implementation.
//!
//! This crate provides the `#[derive(StatusCondition)]` macro for automatically
//! generating Kubernetes status condition mappings from error enums.

#![allow(clippy::needless_continue)] // From darling-generated code

use darling::{ast, FromDeriveInput, FromVariant};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(condition), supports(enum_any))]
struct ConditionOpts {
    ident: syn::Ident,
    data: ast::Data<ConditionVariant, ()>,
    /// Default condition type if not specified on variant (e.g., "Ready")
    #[darling(default)]
    default_type: Option<String>,
}

#[derive(Debug, FromVariant)]
#[darling(attributes(condition))]
struct ConditionVariant {
    ident: syn::Ident,
    /// The condition type (e.g., "Ready", "Synchronized", "Healthy")
    #[darling(default)]
    condition_type: Option<String>,
    /// The reason string for the condition (e.g., "`RndcFailed`", "`ZoneParseError`")
    #[darling(default)]
    reason: Option<String>,
    /// The status value - defaults to "False" for errors
    #[darling(default)]
    status: Option<String>,
    /// Severity level for observability (info, warning, error)
    #[darling(default)]
    severity: Option<String>,
    /// Whether this error is retryable
    #[darling(default)]
    retryable: Option<bool>,
    /// Suggested requeue delay in seconds for retryable errors
    #[darling(default)]
    requeue_secs: Option<u64>,
}

/// Derive macro for mapping error types to Kubernetes status conditions.
///
/// # Example
///
/// ```rust,ignore
/// use kube_condition::StatusCondition;
/// use thiserror::Error;
///
/// #[derive(Error, Debug, StatusCondition)]
/// #[condition(default_type = "Ready")]
/// pub enum ReconcileError {
///     #[error("RNDC command failed: {0}")]
///     #[condition(reason = "RndcFailed", severity = "error", retryable = true, requeue_secs = 30)]
///     RndcFailure(String),
///
///     #[error("Zone file parse error: {0}")]
///     #[condition(reason = "ZoneParseError", severity = "error")]
///     ZoneParse(String),
/// }
/// ```
///
/// # Panics
///
/// Panics if the macro is applied to a non-enum type.
#[proc_macro_derive(StatusCondition, attributes(condition))]
pub fn derive_status_condition(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let opts = match ConditionOpts::from_derive_input(&input) {
        Ok(opts) => opts,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    let enum_name = &opts.ident;
    let default_type = opts.default_type.unwrap_or_else(|| "Ready".to_string());

    let ast::Data::Enum(variants) = opts.data else {
        panic!("StatusCondition can only be derived for enums")
    };

    // Also parse the original syn data to get variant field information
    let syn_variants = match &input.data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => panic!("StatusCondition can only be derived for enums"),
    };

    // Helper function to generate the correct pattern for a variant
    let generate_pattern = |variant_ident: &syn::Ident, fields: &Fields| -> proc_macro2::TokenStream {
        match fields {
            Fields::Named(_) => quote! { Self::#variant_ident { .. } },
            Fields::Unnamed(_) => quote! { Self::#variant_ident(..) },
            Fields::Unit => quote! { Self::#variant_ident },
        }
    };

    // Generate match arms for to_condition()
    let condition_arms = variants.iter().zip(syn_variants.iter()).map(|(v, syn_v)| {
        let pattern = generate_pattern(&syn_v.ident, &syn_v.fields);
        let condition_type = v.condition_type.clone().unwrap_or_else(|| default_type.clone());
        let reason = v.reason.clone().unwrap_or_else(|| v.ident.to_string());
        let status = v.status.clone().unwrap_or_else(|| "False".to_string());

        quote! {
            #pattern => ::kube_condition::ConditionInfo {
                type_: #condition_type.to_string(),
                status: #status.to_string(),
                reason: #reason.to_string(),
                message: self.to_string(),
            }
        }
    });

    // Generate match arms for severity()
    let severity_arms = variants.iter().zip(syn_variants.iter()).map(|(v, syn_v)| {
        let pattern = generate_pattern(&syn_v.ident, &syn_v.fields);
        let severity = v.severity.clone().unwrap_or_else(|| "error".to_string());
        let severity_variant = match severity.as_str() {
            "info" => quote! { ::kube_condition::Severity::Info },
            "warning" => quote! { ::kube_condition::Severity::Warning },
            _ => quote! { ::kube_condition::Severity::Error },
        };

        quote! {
            #pattern => #severity_variant
        }
    });

    // Generate match arms for is_retryable()
    let retryable_arms = variants.iter().zip(syn_variants.iter()).map(|(v, syn_v)| {
        let pattern = generate_pattern(&syn_v.ident, &syn_v.fields);
        let retryable = v.retryable.unwrap_or(true); // Default to retryable

        quote! {
            #pattern => #retryable
        }
    });

    // Generate match arms for requeue_duration()
    let requeue_arms = variants.iter().zip(syn_variants.iter()).map(|(v, syn_v)| {
        let pattern = generate_pattern(&syn_v.ident, &syn_v.fields);
        let requeue = v.requeue_secs.unwrap_or(30); // DEFAULT_REQUEUE_SECONDS

        quote! {
            #pattern => ::std::time::Duration::from_secs(#requeue)
        }
    });

    let expanded = quote! {
        #[allow(unreachable_patterns)]
        impl ::kube_condition::StatusCondition for #enum_name {
            fn to_condition_info(&self) -> ::kube_condition::ConditionInfo {
                #[allow(unreachable_patterns)]
                match self {
                    #(#condition_arms),*
                }
            }

            fn severity(&self) -> ::kube_condition::Severity {
                #[allow(unreachable_patterns)]
                match self {
                    #(#severity_arms),*
                }
            }

            fn is_retryable(&self) -> bool {
                #[allow(unreachable_patterns)]
                match self {
                    #(#retryable_arms),*
                }
            }

            fn requeue_duration(&self) -> ::std::time::Duration {
                #[allow(unreachable_patterns)]
                match self {
                    #(#requeue_arms),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
