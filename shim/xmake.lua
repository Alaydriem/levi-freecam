add_rules("mode.debug", "mode.release")

add_repositories("levimc-repo https://github.com/LiteLDev/xmake-repo.git")

add_requires("levilamina 26.51.6", {configs = {target_type = "client"}})
add_requires("levibuildscript 0.6.2")
add_requires("imgui v1.92.9+b-docking", {configs = {shared = false, win32 = true, dx11 = true}})

if not has_config("vs_runtime") then
    set_runtimes("MD")
end

target("alaydriem-freecam")
    add_rules("@levibuildscript/linkrule")
    add_rules("@levibuildscript/modpacker", {modName = "alaydriem-freecam", modVersion = "0.1.0"})
    add_packages("levilamina", "imgui")
    add_syslinks("d3d11", "d3d12", "dxgi", "user32", "kernel32", "ntdll", "userenv", "ws2_32", "dbghelp")
    add_defines("NOMINMAX", "UNICODE", "_AMD64_")
    set_exceptions("none")
    add_cxflags("/EHa", "/utf-8", "/W4", "/w44265", "/w44289", "/w44296", "/w45263", "/w44738", "/w45204")
    add_cxflags(
        "/EHs",
        "-Wno-microsoft-cast",
        "-Wno-invalid-offsetof",
        "-Wno-c++2b-extensions",
        "-Wno-microsoft-include",
        "-Wno-overloaded-virtual",
        "-Wno-ignored-qualifiers",
        "-Wno-missing-field-initializers",
        "-Wno-potentially-evaluated-expression",
        "-Wno-pragma-system-header-outside-header",
        {tools = {"clang_cl"}}
    )
    set_toolchains("clang-cl")
    set_kind("shared")
    set_languages("c++20")
    set_symbols("debug")
    add_headerfiles("src/**.h", "include/levi_freecam.h")
    add_files("src/**.cpp")
    add_includedirs("src", "include")
    add_defines("LL_PLAT_C")
    add_linkdirs("$(projectdir)/../target/release")
    add_links("freecam_ffi")
