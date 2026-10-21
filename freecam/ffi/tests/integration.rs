#![allow(clippy::unwrap_used, clippy::expect_used)]

mod api;
mod services;
mod support;

#[global_allocator]
static GLOBAL: support::counting_alloc::CountingAlloc = support::counting_alloc::CountingAlloc::system();
