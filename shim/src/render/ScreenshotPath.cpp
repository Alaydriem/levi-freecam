#include "render/ScreenshotPath.h"

#include <cwchar>

namespace levi_freecam {

namespace {

constexpr wchar_t SEPARATOR          = L'\\';
constexpr wchar_t OTHER_SEPARATOR    = L'/';
constexpr size_t  DRIVE_ROOT         = 3;
constexpr wchar_t LONG_PREFIX[]      = L"\\\\?\\";
constexpr size_t  LONG_PREFIX_LENGTH = 4;
constexpr size_t  UNC_PREFIX_LENGTH  = 2;
constexpr size_t  NAME_CAPACITY      = 64;

bool isDriveRoot(std::wstring const& path, size_t at) {
    return path.size() >= at + DRIVE_ROOT && path[at + 1] == L':' && path[at + 2] == SEPARATOR;
}

// the index just after the next separator at or after `from`, or the path's end
size_t pastSeparator(std::wstring const& path, size_t from) {
    size_t next = path.find(SEPARATOR, from);
    return next == std::wstring::npos ? path.size() : next + 1;
}

bool isFolder(std::wstring const& path) {
    DWORD attributes = GetFileAttributesW(path.c_str());
    return attributes != INVALID_FILE_ATTRIBUTES && (attributes & FILE_ATTRIBUTE_DIRECTORY) != 0;
}

} // namespace

std::wstring ScreenshotPath::expand(std::string_view utf8Folder) {
    if (utf8Folder.empty()) return {};
    int size = MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, utf8Folder.data(), static_cast<int>(utf8Folder.size()), nullptr, 0);
    if (size <= 0) return {};
    std::wstring raw(static_cast<size_t>(size), L'\0');
    MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, utf8Folder.data(), static_cast<int>(utf8Folder.size()), raw.data(), size);
    DWORD needed = ExpandEnvironmentStringsW(raw.c_str(), nullptr, 0);
    if (needed == 0) return {};
    std::wstring expanded(needed, L'\0');
    DWORD written = ExpandEnvironmentStringsW(raw.c_str(), expanded.data(), needed);
    if (written == 0 || written > needed) return {};
    // the count includes the terminating null
    expanded.resize(written - 1);
    for (wchar_t& c : expanded) {
        if (c == OTHER_SEPARATOR) c = SEPARATOR;
    }
    while (expanded.size() > 1 && expanded.back() == SEPARATOR) expanded.pop_back();
    return expanded;
}

size_t ScreenshotPath::rootLength(std::wstring const& path) {
    if (path.compare(0, LONG_PREFIX_LENGTH, LONG_PREFIX) == 0) {
        return isDriveRoot(path, LONG_PREFIX_LENGTH) ? LONG_PREFIX_LENGTH + DRIVE_ROOT : LONG_PREFIX_LENGTH;
    }
    if (isDriveRoot(path, 0)) return DRIVE_ROOT;
    if (path.size() > UNC_PREFIX_LENGTH && path[0] == SEPARATOR && path[1] == SEPARATOR) {
        size_t share = pastSeparator(path, UNC_PREFIX_LENGTH);
        return pastSeparator(path, share);
    }
    return 0;
}

// a level that cannot be created is skipped: it may exist already and only deny the create (C:\Users); the
// final check decides
bool ScreenshotPath::ensureFolder(std::wstring const& folder) {
    if (folder.empty()) return false;
    if (isFolder(folder)) return true;
    size_t at = rootLength(folder);
    while (at < folder.size()) {
        size_t next = folder.find(SEPARATOR, at);
        if (next == std::wstring::npos) break;
        CreateDirectoryW(folder.substr(0, next).c_str(), nullptr);
        at = next + 1;
    }
    CreateDirectoryW(folder.c_str(), nullptr);
    return isFolder(folder);
}

std::wstring ScreenshotPath::fileName(SYSTEMTIME const& time, int attempt) {
    wchar_t name[NAME_CAPACITY]{};
    if (attempt <= 1) {
        swprintf(name, NAME_CAPACITY, L"freecam-%04u%02u%02u-%02u%02u%02u.png", time.wYear, time.wMonth, time.wDay, time.wHour, time.wMinute, time.wSecond);
    } else {
        swprintf(
            name,
            NAME_CAPACITY,
            L"freecam-%04u%02u%02u-%02u%02u%02u-%d.png",
            time.wYear,
            time.wMonth,
            time.wDay,
            time.wHour,
            time.wMinute,
            time.wSecond,
            attempt
        );
    }
    return name;
}

std::string ScreenshotPath::utf8(std::wstring const& text) {
    if (text.empty()) return {};
    int size = WideCharToMultiByte(CP_UTF8, 0, text.data(), static_cast<int>(text.size()), nullptr, 0, nullptr, nullptr);
    if (size <= 0) return {};
    std::string out(static_cast<size_t>(size), '\0');
    WideCharToMultiByte(CP_UTF8, 0, text.data(), static_cast<int>(text.size()), out.data(), size, nullptr, nullptr);
    return out;
}

} // namespace levi_freecam
