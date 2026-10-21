mod runtime;
mod support;
mod views;
mod widgets;

#[global_allocator]
static GLOBAL: support::counting_alloc::CountingAlloc = support::counting_alloc::CountingAlloc::system();
