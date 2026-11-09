// Same reason as `matrix_client`'s own `#![recursion_limit]` —
// `Timeline::send`'s future type nests deep enough to blow the default
// query-depth limit, verified locally 2026-11-06/07.
#![recursion_limit = "256"]

pub mod api;
mod frb_generated;
