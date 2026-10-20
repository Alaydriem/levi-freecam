#pragma once

#include <windows.h>

#include <string>
#include <string_view>

namespace levi_freecam {

// the screenshot folder and file names: expands the configured folder, creates it, and names each file
// freecam-YYYYMMDD-HHMMSS.png, with -2, -3 ... when that name is taken
class ScreenshotPath {
public:
    // the UTF-8 folder from config with its environment variables expanded; empty when it cannot be converted
    static std::wstring expand(std::string_view utf8Folder);

    // creates every missing level of the folder; true when the folder exists afterwards
    static bool ensureFolder(std::wstring const& folder);

    // the file name for this time and attempt: attempt 1 has no suffix, attempt 2 ends in -2
    static std::wstring fileName(SYSTEMTIME const& time, int attempt);

    // the first index after the root of the path ("C:\", "\\server\share\"), where folder creation starts
    static size_t rootLength(std::wstring const& path);

    // UTF-16 to UTF-8; empty on a conversion failure
    static std::string utf8(std::wstring const& text);
};

} // namespace levi_freecam
